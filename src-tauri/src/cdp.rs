use base64::{engine::general_purpose::STANDARD, Engine};
use serde_json::{json, Value};
use std::{fs, io::ErrorKind, net::TcpStream, path::Path, time::Duration};
use tungstenite::{client, stream::MaybeTlsStream, Message, WebSocket};

use crate::{
    codex::validate_target_ws_url, image_tools::resolve_imported_image, theme::Theme, AppError,
};

pub const BACKDROP_NODE_ID: &str = "codex-skin-workshop-backdrop";

const RENDERER_TEMPLATE: &str = r#"(() => {
const STYLE_ID='codex-skin-workshop-style';
const BG_ID='codex-skin-workshop-backdrop';
const CHROME_ID='codex-skin-workshop-chrome';
document.getElementById(STYLE_ID)?.remove();
document.getElementById(BG_ID)?.remove();
document.getElementById(CHROME_ID)?.remove();
const style=document.createElement('style');style.id=STYLE_ID;style.textContent=atob('__CSS__');document.documentElement.appendChild(style);
if('__IMAGE__'){const bg=document.createElement('div');bg.id=BG_ID;bg.setAttribute('aria-hidden','true');bg.style.backgroundImage=`url("data:__MIME__;base64,__IMAGE__")`;document.documentElement.prepend(bg);}
const chrome=document.createElement('div');chrome.id=CHROME_ID;chrome.setAttribute('aria-hidden','true');chrome.innerHTML=atob('__CHROME__');document.documentElement.appendChild(chrome);
})()"#;
const RESTORE_PAYLOAD: &str = "document.getElementById('codex-skin-workshop-style')?.remove();document.getElementById('codex-skin-workshop-backdrop')?.remove();document.getElementById('codex-skin-workshop-chrome')?.remove();";

pub fn render_payload(theme: &Theme, store_root: &Path) -> Result<String, AppError> {
    theme.validate()?;
    let color = |key: &str, fallback: &str| {
        theme
            .colors
            .get(key)
            .cloned()
            .unwrap_or_else(|| fallback.into())
    };
    let vars = format!(
        "--csw-bg:{};--csw-panel:{};--csw-panel-alt:{};--csw-accent:{};--csw-accent-alt:{};--csw-secondary:{};--csw-highlight:{};--csw-text:{};--csw-muted:{};--csw-line:{};",
        color("background", "#f7f4f5"),
        color("panel", "#ffffff"),
        color("panel-alt", "#fff7f8"),
        color("accent", "#e25563"),
        color("accent-alt", "#f07a86"),
        color("secondary", "#f3a8af"),
        color("highlight", "#c93d4c"),
        color("text", "#2b2224"),
        color("muted", "#8a7a7d"),
        color("line", "#e5c9cd"),
    );
    let opacity = theme.opacity.unwrap_or(1.0);
    let blur = theme.blur_px.unwrap_or(0);
    let brightness = theme.brightness_pct.unwrap_or(100);
    let saturation = theme.saturation_pct.unwrap_or(100);
    let position_x = theme.image_position_x_pct.unwrap_or(50);
    let position_y = theme.image_position_y_pct.unwrap_or(50);
    let scale = theme.image_scale_pct.unwrap_or(100);
    let panel_opacity = theme.panel_opacity.unwrap_or(0.86);
    let panel_blur = theme.panel_blur_px.unwrap_or(12);
    let radius = theme.corner_radius_px.unwrap_or(18);
    let content_width = theme.content_max_width_px.unwrap_or(950);
    let particle_count = theme.particle_count.unwrap_or(8).min(24);

    let css = format!(
        r#"
:root{{{vars}}}
#{BACKDROP_NODE_ID}{{position:fixed;inset:-48px;z-index:0;background-size:{scale}% auto;background-repeat:no-repeat;background-position:{position_x}% {position_y}%;opacity:{opacity};filter:blur({blur}px) brightness({brightness}%) saturate({saturation}%);pointer-events:none}}
body{{background:var(--csw-bg)!important;color:var(--csw-text)!important}}
body>*:not(#{BACKDROP_NODE_ID}):not(#codex-skin-workshop-chrome){{position:relative;z-index:1}}
main.main-surface,main,[role="main"]{{background:color-mix(in srgb,var(--csw-panel) {panel_percent}%,transparent)!important;color:var(--csw-text)!important;backdrop-filter:blur({panel_blur}px) saturate(110%);border-radius:{radius}px!important;max-width:{content_width}px}}
aside.app-shell-left-panel,aside{{background:color-mix(in srgb,var(--csw-panel) {panel_percent}%,transparent)!important;border-color:var(--csw-line)!important;backdrop-filter:blur({panel_blur}px);border-radius:0 {radius}px {radius}px 0!important}}
.composer-surface-chrome,[data-testid*="composer"]{{background:color-mix(in srgb,var(--csw-panel-alt) {panel_percent}%,transparent)!important;border-color:var(--csw-line)!important;border-radius:{radius}px!important;backdrop-filter:blur({panel_blur}px) saturate(110%)}}
button[aria-pressed="true"],[data-state="active"]{{color:var(--csw-accent)!important}}
a{{color:var(--csw-highlight)!important}}
#codex-skin-workshop-chrome{{position:fixed;inset:0;z-index:2;pointer-events:none;color:var(--csw-text)}}
#codex-skin-workshop-chrome .csw-brand{{position:absolute;left:36px;top:24px;font-weight:700;color:var(--csw-accent)}}
#codex-skin-workshop-chrome .csw-brand small{{display:block;margin-top:4px;color:var(--csw-muted);font-size:11px;letter-spacing:.12em}}
#codex-skin-workshop-chrome .csw-status{{position:absolute;right:34px;top:28px;color:var(--csw-secondary);font:600 11px ui-monospace,monospace;letter-spacing:.14em}}
#codex-skin-workshop-chrome .csw-quote{{position:absolute;right:42px;bottom:34px;color:var(--csw-muted);font-style:italic;transform:rotate(-2deg)}}
#codex-skin-workshop-chrome .csw-orbit{{position:absolute;right:8%;top:16%;width:150px;height:150px;border:1px solid var(--csw-line);border-radius:50%;transform:rotate(18deg)}}
#codex-skin-workshop-chrome .csw-tagline{{position:absolute;left:36px;top:78px;max-width:360px;color:var(--csw-muted);font-size:12px}}
#codex-skin-workshop-chrome .csw-project{{position:absolute;left:36px;bottom:30px;color:var(--csw-highlight);font-size:12px;font-weight:650}}
#codex-skin-workshop-chrome .csw-particles i{{position:absolute;width:5px;height:5px;border-radius:50%;background:var(--csw-accent);opacity:.55;animation:csw-float 4.6s ease-in-out infinite alternate}}
@keyframes csw-float{{to{{transform:translateY(-18px);opacity:.9}}}}
"#,
        panel_percent = (panel_opacity * 100.0).round()
    );

    let mut chrome = String::new();
    if theme.show_brand.unwrap_or(true) {
        chrome.push_str(&format!(
            "<div class=\"csw-brand\">{}<small>{}</small></div>",
            html_escape(&theme.name),
            html_escape(&theme.subtitle)
        ));
    }
    if theme.show_status.unwrap_or(true) && !theme.status_text.is_empty() {
        chrome.push_str(&format!(
            "<div class=\"csw-status\">{}</div>",
            html_escape(&theme.status_text)
        ));
    }
    if theme.show_quote.unwrap_or(true) && !theme.quote.is_empty() {
        chrome.push_str(&format!(
            "<div class=\"csw-quote\">{}</div>",
            html_escape(&theme.quote)
        ));
    }
    if theme.show_orbit.unwrap_or(true) {
        chrome.push_str("<div class=\"csw-orbit\"></div>");
    }
    if !theme.tagline.is_empty() {
        chrome.push_str(&format!(
            "<div class=\"csw-tagline\">{}</div>",
            html_escape(&theme.tagline)
        ));
    }
    if !theme.project_label.is_empty() {
        chrome.push_str(&format!(
            "<div class=\"csw-project\">{}{} </div>",
            html_escape(&theme.project_prefix),
            html_escape(&theme.project_label)
        ));
    }
    if theme.show_particles.unwrap_or(true) && particle_count > 0 {
        chrome.push_str("<div class=\"csw-particles\">");
        for index in 0..particle_count {
            let left = 12 + (u16::from(index) * 37 % 78);
            let top = 18 + (u16::from(index) * 53 % 68);
            chrome.push_str(&format!("<i style=\"left:{left}%;top:{top}%;animation-delay:-{}ms\"></i>", u16::from(index) * 310));
        }
        chrome.push_str("</div>");
    }

    let (mime, image) = if let Some(relative) = &theme.background_image {
        let path = resolve_imported_image(store_root, relative)?;
        let bytes = fs::read(&path)?;
        if bytes.len() > 25 * 1024 * 1024 {
            return Err(AppError::Validation("导入的背景图片过大".into()));
        }
        let mime = match path
            .extension()
            .and_then(|v| v.to_str())
            .unwrap_or_default()
            .to_ascii_lowercase()
            .as_str()
        {
            "png" => "image/png",
            "jpg" | "jpeg" => "image/jpeg",
            "webp" => "image/webp",
            "gif" => "image/gif",
            _ => return Err(AppError::Validation("不支持的背景图片格式".into())),
        };
        (mime, STANDARD.encode(bytes))
    } else {
        ("image/png", String::new())
    };
    Ok(RENDERER_TEMPLATE
        .replace("__CSS__", &STANDARD.encode(css))
        .replace("__CHROME__", &STANDARD.encode(chrome))
        .replace("__MIME__", mime)
        .replace("__IMAGE__", &image))
}

fn html_escape(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

fn connect_exact(
    raw: &str,
    port: u16,
    id: &str,
) -> Result<WebSocket<MaybeTlsStream<TcpStream>>, AppError> {
    let url = validate_target_ws_url(raw, port, id)?;
    let address = format!("127.0.0.1:{port}");
    let tcp = TcpStream::connect_timeout(
        &address
            .parse()
            .map_err(|e: std::net::AddrParseError| AppError::Cdp(e.to_string()))?,
        Duration::from_secs(3),
    )?;
    tcp.set_read_timeout(Some(Duration::from_secs(5)))?;
    tcp.set_write_timeout(Some(Duration::from_secs(5)))?;
    let request = url
        .as_str()
        .into_client_request()
        .map_err(|e| AppError::Cdp(e.to_string()))?;
    let (socket, _) = client(request, MaybeTlsStream::Plain(tcp))
        .map_err(|e| AppError::Cdp(e.to_string()))?;
    Ok(socket)
}

use tungstenite::client::IntoClientRequest;
pub fn evaluate(raw: &str, port: u16, id: &str, expression: &str) -> Result<(), AppError> {
    let mut socket = connect_exact(raw, port, id)?;
    socket
        .send(Message::Text(
            json!({"id":1,"method":"Runtime.evaluate","params":{"expression":expression,"awaitPromise":true,"returnByValue":true}})
                .to_string(),
        ))
        .map_err(|e| AppError::Cdp(e.to_string()))?;
    loop {
        match socket.read() {
            Ok(Message::Text(text)) => {
                let reply: Value = serde_json::from_str(&text)?;
                if reply.get("id").and_then(Value::as_u64) == Some(1) {
                    if let Some(error) = reply.get("error") {
                        return Err(AppError::Cdp(error.to_string()));
                    }
                    if reply.pointer("/result/exceptionDetails").is_some() {
                        return Err(AppError::Cdp("主题渲染时发生异常".into()));
                    }
                    return Ok(());
                }
            }
            Ok(Message::Close(_)) => {
                return Err(AppError::Cdp("调试连接在返回结果前已关闭".into()))
            }
            Ok(_) => {}
            Err(tungstenite::Error::Io(e))
                if matches!(e.kind(), ErrorKind::WouldBlock | ErrorKind::TimedOut) =>
            {
                return Err(AppError::Cdp("等待 Codex 响应超时".into()))
            }
            Err(e) => return Err(AppError::Cdp(e.to_string())),
        }
    }
}

pub fn apply(
    raw: &str,
    port: u16,
    id: &str,
    theme: &Theme,
    root: &Path,
) -> Result<(), AppError> {
    evaluate(raw, port, id, &render_payload(theme, root)?)
}
pub fn restore(raw: &str, port: u16, id: &str) -> Result<(), AppError> {
    evaluate(raw, port, id, RESTORE_PAYLOAD)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    #[test]
    fn payload_contains_only_encoded_user_values() {
        let dir = tempfile::tempdir().unwrap();
        let theme = Theme {
            schema_version: 1,
            id: "x".into(),
            name: "测试".into(),
            subtitle: String::new(),
            tagline: String::new(),
            project_prefix: String::new(),
            project_label: String::new(),
            status_text: String::new(),
            quote: String::new(),
            colors: BTreeMap::from([("accent".into(), "#abcdef".into())]),
            background_image: None,
            opacity: Some(0.5),
            blur_px: Some(2),
            brightness_pct: Some(80),
            saturation_pct: Some(120),
            image_position_x_pct: Some(50),
            image_position_y_pct: Some(50),
            image_scale_pct: Some(100),
            panel_opacity: Some(0.8),
            panel_blur_px: Some(10),
            corner_radius_px: Some(18),
            content_max_width_px: Some(950),
            show_brand: Some(true),
            show_status: Some(true),
            show_quote: Some(true),
            show_orbit: Some(false),
            show_particles: Some(false),
            particle_count: Some(0),
        };
        let payload = render_payload(&theme, dir.path()).unwrap();
        assert!(payload.contains("codex-skin-workshop-style"));
        assert!(!payload.contains("#abcdef"));
        assert!(!payload.contains("--csw-accent"));
    }

    #[test]
    fn restore_is_fixed_and_scoped() {
        assert!(RESTORE_PAYLOAD.contains("codex-skin-workshop-style"));
        assert!(RESTORE_PAYLOAD.contains(BACKDROP_NODE_ID));
        assert!(!RESTORE_PAYLOAD.contains("querySelectorAll"));
    }
}
