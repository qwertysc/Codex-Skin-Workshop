mod cdp;
mod codex;
mod image_tools;
mod theme;

use codex::{CodexInstallation, CodexState, DevtoolsTarget};
use image_tools::ImportedImage;
use serde::Serialize;
use std::{path::PathBuf, sync::MutexGuard, time::{Duration, Instant}};
use tauri::{Manager, State};
use theme::{Theme, ThemeStore};

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("{0}")]
    Validation(String),
    #[error("本机调试连接失败：{0}")]
    Cdp(String),
    #[error("文件操作失败：{0}")]
    Io(#[from] std::io::Error),
    #[error("主题文件格式错误：{0}")]
    Json(#[from] serde_json::Error),
    #[error("本机调试请求失败：{0}")]
    Http(#[from] reqwest::Error),
    #[error("图片处理失败：{0}")]
    Image(#[from] image::ImageError),
}
impl Serialize for AppError {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

struct StoreState(ThemeStore);

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct LaunchResult {
    port: u16,
    target: DevtoolsTarget,
}

fn managed<'a>(
    state: &'a State<'_, CodexState>,
) -> Result<MutexGuard<'a, Option<codex::ManagedCodex>>, AppError> {
    state
        .0
        .lock()
        .map_err(|_| AppError::Cdp("进程状态不可用".into()))
}

#[tauri::command]
fn list_themes(store: State<'_, StoreState>) -> Result<Vec<Theme>, AppError> {
    store.0.list()
}
#[tauri::command]
fn save_theme(store: State<'_, StoreState>, theme: Theme) -> Result<(), AppError> {
    store.0.save(&theme)
}
#[tauri::command]
fn delete_theme(store: State<'_, StoreState>, id: String) -> Result<(), AppError> {
    store.0.delete(&id)
}
#[tauri::command]
fn import_image(
    store: State<'_, StoreState>,
    source_path: PathBuf,
) -> Result<ImportedImage, AppError> {
    image_tools::import_image(&store.0, &source_path)
}
#[tauri::command]
fn import_theme_file(source_path: PathBuf) -> Result<Theme, AppError> {
    theme::read_theme_file(&source_path)
}
#[tauri::command]
fn export_theme_file(destination_path: PathBuf, theme: Theme) -> Result<(), AppError> {
    theme::export_theme_file(&destination_path, &theme)
}
#[tauri::command]
fn detect_codex() -> Vec<CodexInstallation> {
    codex::detect_codex()
}

#[tauri::command]
fn launch_codex(
    state: State<'_, CodexState>,
    executable: PathBuf,
) -> Result<LaunchResult, AppError> {
    let mut guard = managed(&state)?;
    if let Some(current) = guard.as_mut() {
        if current.child.try_wait()?.is_none() {
            return Err(AppError::Validation(
                "已有一个由皮肤工坊启动的 Codex 正在运行".into(),
            ));
        }
        *guard = None;
    }
    let child = codex::launch_managed(&executable)?;
    let port = child.port;
    *guard = Some(child);
    drop(guard);

    let deadline = Instant::now() + Duration::from_secs(15);
    let mut last = None;
    while Instant::now() < deadline {
        {
            let mut guard = managed(&state)?;
            let current = guard
                .as_mut()
                .ok_or_else(|| AppError::Cdp("Codex 启动状态已丢失".into()))?;
            if current.child.try_wait()?.is_some() {
                guard.take();
                return Err(AppError::Cdp(
                    "Codex 在调试窗口准备完成前已经退出".into(),
                ));
            }
        }
        match codex::fetch_targets(port) {
            Ok(targets) => {
                if let Some(target) = codex::select_preview_target(targets, port) {
                    return Ok(LaunchResult { port, target });
                }
                last = Some(AppError::Cdp(
                    "Codex 已打开，正在等待可应用主题的窗口".into(),
                ));
            }
            Err(error) => last = Some(error),
        }
        std::thread::sleep(Duration::from_millis(150));
    }
    if let Ok(mut guard) = managed(&state) {
        if let Some(mut process) = guard.take() {
            let _ = codex::stop_managed(&mut process);
        }
    }
    Err(last.unwrap_or_else(|| AppError::Cdp("Codex 调试窗口启动超时".into())))
}

#[tauri::command]
fn list_codex_targets(state: State<'_, CodexState>) -> Result<Vec<DevtoolsTarget>, AppError> {
    let mut guard = managed(&state)?;
    let current = guard
        .as_mut()
        .ok_or_else(|| AppError::Validation("当前没有由皮肤工坊启动的 Codex".into()))?;
    if current.child.try_wait()?.is_some() {
        guard.take();
        return Err(AppError::Validation("Codex 已退出，请重新应用主题".into()));
    }
    codex::fetch_targets(current.port)
}

fn trusted_target(
    state: &State<'_, CodexState>,
    target_id: &str,
) -> Result<(u16, DevtoolsTarget), AppError> {
    let mut guard = managed(state)?;
    let current = guard
        .as_mut()
        .ok_or_else(|| AppError::Validation("当前没有由皮肤工坊启动的 Codex".into()))?;
    if current.child.try_wait()?.is_some() {
        guard.take();
        return Err(AppError::Validation("Codex 已退出，请重新应用主题".into()));
    }
    let port = current.port;
    drop(guard);
    let target = codex::fetch_targets(port)?
        .into_iter()
        .find(|target| target.id == target_id)
        .ok_or_else(|| AppError::Validation("Codex 窗口已变化，请重新应用主题".into()))?;
    codex::validate_target_ws_url(&target.web_socket_debugger_url, port, &target.id)?;
    Ok((port, target))
}

#[tauri::command]
fn apply_theme(
    state: State<'_, CodexState>,
    store: State<'_, StoreState>,
    target_id: String,
    theme: Theme,
) -> Result<(), AppError> {
    theme.validate()?;
    let (port, target) = trusted_target(&state, &target_id)?;
    cdp::apply(
        &target.web_socket_debugger_url,
        port,
        &target.id,
        &theme,
        store.0.root(),
    )
}

#[tauri::command]
fn restore_codex(
    state: State<'_, CodexState>,
    target_id: Option<String>,
) -> Result<(), AppError> {
    let restore_result = match target_id {
        Some(id) => trusted_target(&state, &id)
            .and_then(|(port, target)| cdp::restore(&target.web_socket_debugger_url, port, &target.id)),
        None => Ok(()),
    };
    let stop_result = managed(&state).and_then(|mut guard| {
        if let Some(mut process) = guard.take() {
            codex::stop_managed(&mut process)
        } else {
            Ok(())
        }
    });
    match (restore_result, stop_result) {
        (Err(error), _) => Err(error),
        (Ok(()), result) => result,
    }
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let root = app.path().app_data_dir()?;
            app.manage(StoreState(ThemeStore::new(root)?));
            app.manage(CodexState(std::sync::Mutex::new(None)));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            list_themes,
            save_theme,
            delete_theme,
            import_image,
            import_theme_file,
            export_theme_file,
            detect_codex,
            launch_codex,
            list_codex_targets,
            apply_theme,
            restore_codex
        ])
        .run(tauri::generate_context!())
        .expect("Codex 皮肤工坊运行失败");
}
