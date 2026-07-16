use serde::{Deserialize, Serialize};
use std::{
    net::TcpListener,
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    sync::Mutex,
    time::Duration,
};
use url::Url;

use crate::AppError;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CodexInstallation {
    pub executable: PathBuf,
    pub display_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DevtoolsTarget {
    pub id: String,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub url: String,
    #[serde(rename = "type", default)]
    pub kind: String,
    #[serde(default)]
    pub web_socket_debugger_url: String,
}

pub struct ManagedCodex {
    pub child: Child,
    pub port: u16,
}
impl Drop for ManagedCodex {
    fn drop(&mut self) {
        if self.child.try_wait().ok().flatten().is_none() {
            let _ = self.child.kill();
            let _ = self.child.wait();
        }
    }
}
pub struct CodexState(pub Mutex<Option<ManagedCodex>>);

pub fn reserve_loopback_port() -> Result<(TcpListener, u16), AppError> {
    let listener = TcpListener::bind(("127.0.0.1", 0))?;
    let port = listener.local_addr()?.port();
    Ok((listener, port))
}

pub fn detect_codex() -> Vec<CodexInstallation> {
    candidate_paths()
        .into_iter()
        .filter(|p| p.is_file())
        .map(|executable| CodexInstallation {
            executable,
            display_name: "Codex".into(),
        })
        .collect()
}

#[cfg(target_os = "macos")]
fn candidate_paths() -> Vec<PathBuf> {
    let mut paths = vec![
        PathBuf::from("/Applications/Codex.app/Contents/MacOS/Codex"),
        PathBuf::from("/Applications/ChatGPT.app/Contents/MacOS/ChatGPT"),
    ];
    if let Some(home) = std::env::var_os("HOME") {
        let home = PathBuf::from(home);
        paths.push(home.join("Applications/ChatGPT.app/Contents/MacOS/ChatGPT"));
        paths.push(home.join("Applications/Codex.app/Contents/MacOS/Codex"));
    }
    paths
}
#[cfg(target_os = "windows")]
fn candidate_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();
    for base in [
        std::env::var_os("LOCALAPPDATA"),
        std::env::var_os("PROGRAMFILES"),
        std::env::var_os("PROGRAMFILES(X86)"),
    ]
    .into_iter()
    .flatten()
    {
        let base = PathBuf::from(base);
        paths.push(base.join("Codex").join("Codex.exe"));
        paths.push(base.join("OpenAI").join("Codex").join("ChatGPT.exe"));
    }
    if let Some(local) = std::env::var_os("LOCALAPPDATA") {
        let aliases = PathBuf::from(local).join("Microsoft").join("WindowsApps");
        paths.push(aliases.join("ChatGPT.exe"));
        paths.push(aliases.join("Codex.exe"));
    }
    paths
}
#[cfg(not(any(target_os = "macos", target_os = "windows")))]
fn candidate_paths() -> Vec<PathBuf> {
    Vec::new()
}

pub fn launch_managed(executable: &Path) -> Result<ManagedCodex, AppError> {
    if !detect_codex().iter().any(|c| c.executable == executable) {
        return Err(AppError::Validation("所选程序不是已检测到的 Codex".into()));
    }
    let (reservation, port) = reserve_loopback_port()?;
    drop(reservation);
    let child = Command::new(executable)
        .arg(format!("--remote-debugging-port={port}"))
        .arg("--remote-debugging-address=127.0.0.1")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;
    Ok(ManagedCodex { child, port })
}

pub fn stop_managed(managed: &mut ManagedCodex) -> Result<(), AppError> {
    if managed.child.try_wait()?.is_none() {
        managed.child.kill()?;
        managed.child.wait()?;
    }
    Ok(())
}

pub fn fetch_targets(port: u16) -> Result<Vec<DevtoolsTarget>, AppError> {
    let endpoint = format!("http://127.0.0.1:{port}/json/list");
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_millis(800))
        .redirect(reqwest::redirect::Policy::none())
        .no_proxy()
        .build()?;
    let response = client.get(endpoint).send()?;
    if !response.status().is_success() {
        return Err(AppError::Cdp(format!(
            "本机调试接口返回状态 {}",
            response.status()
        )));
    }
    let targets: Vec<DevtoolsTarget> = response.json()?;
    Ok(targets)
}

pub fn select_preview_target(
    targets: Vec<DevtoolsTarget>,
    port: u16,
) -> Option<DevtoolsTarget> {
    targets.into_iter().find(|target| {
        matches!(target.kind.as_str(), "page" | "webview" | "")
            && !target.id.is_empty()
            && validate_target_ws_url(&target.web_socket_debugger_url, port, &target.id).is_ok()
            && !target.url.starts_with("devtools://")
    })
}

pub fn validate_target_ws_url(
    raw: &str,
    expected_port: u16,
    expected_id: &str,
) -> Result<Url, AppError> {
    if expected_id.is_empty() || expected_id.contains('/') {
        return Err(AppError::Cdp("调试目标 ID 无效".into()));
    }
    let url = Url::parse(raw).map_err(|e| AppError::Cdp(e.to_string()))?;
    let exact_path = format!("/devtools/page/{expected_id}");
    if url.scheme() != "ws"
        || url.host_str() != Some("127.0.0.1")
        || url.port() != Some(expected_port)
        || url.path() != exact_path
        || url.query().is_some()
        || url.fragment().is_some()
        || !url.username().is_empty()
        || url.password().is_some()
    {
        return Err(AppError::Cdp("调试连接地址未通过安全校验".into()));
    }
    Ok(url)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loopback_port_is_reserved() {
        let (socket, port) = reserve_loopback_port().unwrap();
        assert_eq!(socket.local_addr().unwrap().ip().to_string(), "127.0.0.1");
        assert!(port > 0);
    }

    #[test]
    fn accepts_only_exact_ws_target() {
        assert!(validate_target_ws_url(
            "ws://127.0.0.1:3456/devtools/page/abc",
            3456,
            "abc"
        )
        .is_ok());
        for bad in [
            "ws://localhost:3456/devtools/page/abc",
            "ws://127.0.0.1:9999/devtools/page/abc",
            "wss://127.0.0.1:3456/devtools/page/abc",
            "ws://127.0.0.1:3456/devtools/page/other",
            "ws://127.0.0.1:3456/devtools/page/abc?q=1",
        ] {
            assert!(validate_target_ws_url(bad, 3456, "abc").is_err(), "{bad}");
        }
    }

    #[test]
    fn selects_page_and_skips_untrusted_targets() {
        let targets = vec![
            DevtoolsTarget {
                id: "worker".into(),
                title: String::new(),
                url: String::new(),
                kind: "service_worker".into(),
                web_socket_debugger_url:
                    "ws://127.0.0.1:3456/devtools/page/worker".into(),
            },
            DevtoolsTarget {
                id: "page".into(),
                title: "Codex".into(),
                url: "app://codex".into(),
                kind: "page".into(),
                web_socket_debugger_url: "ws://127.0.0.1:3456/devtools/page/page".into(),
            },
        ];
        assert_eq!(select_preview_target(targets, 3456).unwrap().id, "page");
    }
}
