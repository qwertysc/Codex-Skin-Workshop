use base64::{engine::general_purpose::STANDARD, Engine};
use serde_json::{json, Value};
use std::{fs, io::ErrorKind, net::TcpStream, path::Path, time::Duration};
use tungstenite::{client, stream::MaybeTlsStream, Message, WebSocket};
use crate::{codex::validate_target_ws_url, image_tools::resolve_imported_image, theme::Theme, AppError};

pub const STYLE_NODE_ID: &str = "codex-skin-workshop-style";
pub const BACKDROP_NODE_ID: &str = "codex-skin-workshop-backdrop";

// Trusted renderer owned by the application. Theme files contain values only, never CSS or JS.
const RENDERER_TEMPLATE: &str = r#"(() => {
const STYLE_ID = 'codex-skin-workshop-style';
const BG_ID = 'codex-skin-workshop-backdrop';
document.getElementById(STYLE_ID)?.remove(); document.getElementById(BG_ID)?.remove();
const style=document.createElement('style'); style.id=STYLE_ID; style.textContent=atob('__CSS__'); document.documentElement.appendChild(style);
if ('__IMAGE__') { const bg=document.createElement('div'); bg.id=BG_ID; bg.setAttribute('aria-hidden','true'); bg.style.backgroundImage=`url("data:__MIME__;base64,__IMAGE__")`; document.documentElement.prepend(bg); }
})()"#;
const RESTORE_PAYLOAD: &str = "document.getElementById('codex-skin-workshop-style')?.remove();document.getElementById('codex-skin-workshop-backdrop')?.remove();";

pub fn render_payload(theme: &Theme, store_root: &Path) -> Result<String, AppError> {
    theme.validate()?;
    let mut vars = String::new();
    for (key, value) in &theme.colors { vars.push_str(&format!("--csw-{key}:{value};")); }
    let opacity = theme.opacity.unwrap_or(1.0);
    let blur = theme.blur_px.unwrap_or(0);
    let brightness = theme.brightness_pct.unwrap_or(100);
    let saturation = theme.saturation_pct.unwrap_or(100);
    let css = format!(r#"
:root{{{vars}}}
#{BACKDROP_NODE_ID}{{position:fixed;inset:-32px;z-index:0;background-size:cover;background-position:center;opacity:{opacity};filter:blur({blur}px) brightness({brightness}%) saturate({saturation}%);pointer-events:none}}
body{{background:var(--csw-surface,#101319)!important;color:var(--csw-text,#f4f7f1)!important}}
body>*:not(#{BACKDROP_NODE_ID}){{position:relative;z-index:1}}
main.main-surface,main,[role="main"]{{background:color-mix(in srgb,var(--csw-surface,#101319) 78%,transparent)!important;color:var(--csw-text,#f4f7f1)!important}}
aside.app-shell-left-panel,aside{{background:color-mix(in srgb,var(--csw-surface,#101319) 88%,transparent)!important;border-color:color-mix(in srgb,var(--csw-accent,#b7ff5a) 28%,transparent)!important}}
.composer-surface-chrome,[data-testid*="composer"]{{background:color-mix(in srgb,var(--csw-surface,#101319) 90%,white 10%)!important;border-color:color-mix(in srgb,var(--csw-accent,#b7ff5a) 35%,transparent)!important}}
button[aria-pressed="true"],[data-state="active"]{{color:var(--csw-accent,#b7ff5a)!important}}
"# );
    let (mime, image) = if let Some(relative) = &theme.background_image {
        let path = resolve_imported_image(store_root, relative)?;
        let bytes = fs::read(&path)?;
        if bytes.len() > 25 * 1024 * 1024 { return Err(AppError::Validation("imported image is too large".into())); }
        let mime = match path.extension().and_then(|v| v.to_str()).unwrap_or_default().to_ascii_lowercase().as_str() {
            "png" => "image/png", "jpg" | "jpeg" => "image/jpeg", "webp" => "image/webp", "gif" => "image/gif",
            _ => return Err(AppError::Validation("unsupported imported image extension".into())),
        };
        (mime, STANDARD.encode(bytes))
    } else { ("image/png", String::new()) };
    Ok(RENDERER_TEMPLATE.replace("__CSS__", &STANDARD.encode(css)).replace("__MIME__", mime).replace("__IMAGE__", &image))
}

fn connect_exact(raw: &str, port: u16, id: &str) -> Result<WebSocket<MaybeTlsStream<TcpStream>>, AppError> {
    let url = validate_target_ws_url(raw, port, id)?;
    let address = format!("127.0.0.1:{port}");
    let tcp = TcpStream::connect_timeout(&address.parse().map_err(|e: std::net::AddrParseError| AppError::Cdp(e.to_string()))?, Duration::from_secs(3))?;
    tcp.set_read_timeout(Some(Duration::from_secs(5)))?;
    tcp.set_write_timeout(Some(Duration::from_secs(5)))?;
    let request = url.as_str().into_client_request().map_err(|e| AppError::Cdp(e.to_string()))?;
    let (socket, _) = client(request, MaybeTlsStream::Plain(tcp)).map_err(|e| AppError::Cdp(e.to_string()))?;
    Ok(socket)
}

use tungstenite::client::IntoClientRequest;
pub fn evaluate(raw: &str, port: u16, id: &str, expression: &str) -> Result<(), AppError> {
    let mut socket = connect_exact(raw, port, id)?;
    socket.send(Message::Text(json!({"id":1,"method":"Runtime.evaluate","params":{"expression":expression,"awaitPromise":true,"returnByValue":true}}).to_string().into())).map_err(|e| AppError::Cdp(e.to_string()))?;
    loop {
        match socket.read() {
            Ok(Message::Text(text)) => {
                let reply: Value = serde_json::from_str(&text)?;
                if reply.get("id").and_then(Value::as_u64) == Some(1) {
                    if let Some(error) = reply.get("error") { return Err(AppError::Cdp(error.to_string())); }
                    if reply.pointer("/result/exceptionDetails").is_some() { return Err(AppError::Cdp("renderer evaluation raised an exception".into())); }
                    return Ok(());
                }
            }
            Ok(Message::Close(_)) => return Err(AppError::Cdp("websocket closed before response".into())),
            Ok(_) => {}
            Err(tungstenite::Error::Io(e)) if matches!(e.kind(), ErrorKind::WouldBlock | ErrorKind::TimedOut) => return Err(AppError::Cdp("CDP response timed out".into())),
            Err(e) => return Err(AppError::Cdp(e.to_string())),
        }
    }
}

pub fn apply(raw: &str, port: u16, id: &str, theme: &Theme, root: &Path) -> Result<(), AppError> { evaluate(raw, port, id, &render_payload(theme, root)?) }
pub fn restore(raw: &str, port: u16, id: &str) -> Result<(), AppError> { evaluate(raw, port, id, RESTORE_PAYLOAD) }

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;
    #[test] fn payload_contains_only_encoded_user_values() {
        let dir=tempfile::tempdir().unwrap();
        let t=Theme{id:"x".into(),name:"X".into(),colors:BTreeMap::from([("accent".into(),"#abcdef".into())]),background_image:None,opacity:Some(.5),blur_px:Some(2),brightness_pct:Some(80),saturation_pct:Some(120)};
        let payload=render_payload(&t,dir.path()).unwrap();
        assert!(payload.contains(STYLE_NODE_ID)); assert!(!payload.contains("#abcdef")); assert!(!payload.contains("--csw-accent"));
    }
    #[test] fn restore_is_fixed_and_scoped() { assert!(RESTORE_PAYLOAD.contains(STYLE_NODE_ID)); assert!(RESTORE_PAYLOAD.contains(BACKDROP_NODE_ID)); assert!(!RESTORE_PAYLOAD.contains("querySelectorAll")); }
}
