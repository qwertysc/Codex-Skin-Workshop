mod cdp;
mod codex;
mod image_tools;
mod theme;

use codex::{CodexInstallation, CodexState, DevtoolsTarget};
use image_tools::ImportedImage;
use serde::Serialize;
use std::{path::PathBuf, sync::MutexGuard};
use tauri::{Manager, State};
use theme::{Theme, ThemeStore};

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("validation error: {0}")] Validation(String),
    #[error("CDP error: {0}")] Cdp(String),
    #[error(transparent)] Io(#[from] std::io::Error),
    #[error(transparent)] Json(#[from] serde_json::Error),
    #[error(transparent)] Http(#[from] reqwest::Error),
    #[error(transparent)] Image(#[from] image::ImageError),
}
impl Serialize for AppError {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> { serializer.serialize_str(&self.to_string()) }
}

struct StoreState(ThemeStore);

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct LaunchResult { port: u16, targets: Vec<DevtoolsTarget> }

fn managed<'a>(state: &'a State<'_, CodexState>) -> Result<MutexGuard<'a, Option<codex::ManagedCodex>>, AppError> {
    state.0.lock().map_err(|_| AppError::Cdp("managed process state is poisoned".into()))
}

#[tauri::command]
fn list_themes(store: State<'_, StoreState>) -> Result<Vec<Theme>, AppError> { store.0.list() }
#[tauri::command]
fn save_theme(store: State<'_, StoreState>, theme: Theme) -> Result<(), AppError> { store.0.save(&theme) }
#[tauri::command]
fn delete_theme(store: State<'_, StoreState>, id: String) -> Result<(), AppError> { store.0.delete(&id) }
#[tauri::command]
fn import_image(store: State<'_, StoreState>, source_path: PathBuf) -> Result<ImportedImage, AppError> { image_tools::import_image(&store.0, &source_path) }
#[tauri::command]
fn detect_codex() -> Vec<CodexInstallation> { codex::detect_codex() }

#[tauri::command]
fn launch_codex(state: State<'_, CodexState>, executable: PathBuf) -> Result<LaunchResult, AppError> {
    let mut guard = managed(&state)?;
    if let Some(current) = guard.as_mut() {
        if current.child.try_wait()?.is_none() { return Err(AppError::Validation("a managed Codex instance is already running".into())); }
        *guard = None;
    }
    let child = codex::launch_managed(&executable)?;
    let port = child.port;
    *guard = Some(child);
    drop(guard);
    // Chromium may need a moment to publish /json/list; bounded retries only on loopback.
    let mut last = None;
    for _ in 0..30 {
        match codex::fetch_targets(port) {
            Ok(targets) => return Ok(LaunchResult { port, targets }),
            Err(error) => { last = Some(error); std::thread::sleep(std::time::Duration::from_millis(100)); }
        }
    }
    if let Ok(mut guard) = managed(&state) { if let Some(mut process) = guard.take() { let _ = codex::stop_managed(&mut process); } }
    Err(last.unwrap_or_else(|| AppError::Cdp("Codex DevTools endpoint did not become ready".into())))
}

#[tauri::command]
fn list_codex_targets(state: State<'_, CodexState>) -> Result<Vec<DevtoolsTarget>, AppError> {
    let port = managed(&state)?.as_ref().ok_or_else(|| AppError::Validation("no managed Codex instance".into()))?.port;
    codex::fetch_targets(port)
}

fn trusted_target(state: &State<'_, CodexState>, target_id: &str) -> Result<(u16, DevtoolsTarget), AppError> {
    let port = managed(state)?.as_ref().ok_or_else(|| AppError::Validation("no managed Codex instance".into()))?.port;
    let target = codex::fetch_targets(port)?.into_iter().find(|t| t.id == target_id)
        .ok_or_else(|| AppError::Validation("target is not currently advertised by managed Codex".into()))?;
    codex::validate_target_ws_url(&target.web_socket_debugger_url, port, &target.id)?;
    Ok((port, target))
}

#[tauri::command]
fn apply_theme(state: State<'_, CodexState>, store: State<'_, StoreState>, target_id: String, theme: Theme) -> Result<(), AppError> {
    theme.validate()?;
    let (port, target) = trusted_target(&state, &target_id)?;
    cdp::apply(&target.web_socket_debugger_url, port, &target.id, &theme, store.0.root())
}

#[tauri::command]
fn restore_codex(state: State<'_, CodexState>, target_id: Option<String>) -> Result<(), AppError> {
    // Never let a renderer/validation failure skip process cleanup.
    let restore_result = match target_id {
        Some(id) => trusted_target(&state, &id).and_then(|(port, target)| {
            cdp::restore(&target.web_socket_debugger_url, port, &target.id)
        }),
        None => Ok(()),
    };
    let stop_result = managed(&state).and_then(|mut guard| {
        if let Some(mut process) = guard.take() { codex::stop_managed(&mut process) } else { Ok(()) }
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
        .invoke_handler(tauri::generate_handler![list_themes, save_theme, delete_theme, import_image, detect_codex, launch_codex, list_codex_targets, apply_theme, restore_codex])
        .run(tauri::generate_context!())
        .expect("error while running Codex Skin Workshop");
}
