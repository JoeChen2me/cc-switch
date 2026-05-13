//! Plugin 管理 Tauri 命令

use crate::app_config::{AppType, InstalledPlugin};
use crate::services::PluginService;
use std::sync::Arc;

fn service(db: Arc<crate::database::Database>) -> PluginService {
    PluginService::new(db)
}

fn parse_app_type(s: &str) -> Result<AppType, String> {
    s.parse::<AppType>().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn scan_plugins(
    app_type: String,
    db: tauri::State<'_, Arc<crate::database::Database>>,
) -> Result<Vec<InstalledPlugin>, String> {
    let app = parse_app_type(&app_type)?;
    service(db.inner().clone())
        .scan_plugins(&app)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn enable_plugin(
    id: String,
    app_type: String,
    db: tauri::State<'_, Arc<crate::database::Database>>,
) -> Result<InstalledPlugin, String> {
    let app = parse_app_type(&app_type)?;
    service(db.inner().clone())
        .enable_plugin(&id, &app)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn disable_plugin(
    id: String,
    app_type: String,
    db: tauri::State<'_, Arc<crate::database::Database>>,
) -> Result<InstalledPlugin, String> {
    let app = parse_app_type(&app_type)?;
    service(db.inner().clone())
        .disable_plugin(&id, &app)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn uninstall_plugin(
    id: String,
    app_type: String,
    db: tauri::State<'_, Arc<crate::database::Database>>,
) -> Result<bool, String> {
    let app = parse_app_type(&app_type)?;
    service(db.inner().clone())
        .uninstall_plugin(&id, &app)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn install_plugin_from_zip(
    file_path: String,
    app_type: String,
    db: tauri::State<'_, Arc<crate::database::Database>>,
) -> Result<Vec<InstalledPlugin>, String> {
    let app = parse_app_type(&app_type)?;
    service(db.inner().clone())
        .install_from_zip(&file_path, &app)
        .map_err(|e| e.to_string())
}
