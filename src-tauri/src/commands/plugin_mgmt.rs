//! Plugin 管理 Tauri 命令

use crate::app_config::InstalledPlugin;
use crate::services::PluginService;
use std::sync::Arc;

fn service(db: Arc<crate::database::Database>) -> PluginService {
    PluginService::new(db)
}

#[tauri::command]
pub async fn scan_plugins(db: tauri::State<'_, Arc<crate::database::Database>>) -> Result<Vec<InstalledPlugin>, String> {
    service(db.inner().clone()).scan_plugins().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn enable_plugin(id: String, db: tauri::State<'_, Arc<crate::database::Database>>) -> Result<InstalledPlugin, String> {
    service(db.inner().clone()).enable_plugin(&id).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn disable_plugin(id: String, db: tauri::State<'_, Arc<crate::database::Database>>) -> Result<InstalledPlugin, String> {
    service(db.inner().clone()).disable_plugin(&id).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn uninstall_plugin(id: String, db: tauri::State<'_, Arc<crate::database::Database>>) -> Result<bool, String> {
    service(db.inner().clone()).uninstall_plugin(&id).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn install_plugin_from_zip(
    file_path: String,
    db: tauri::State<'_, Arc<crate::database::Database>>,
) -> Result<Vec<InstalledPlugin>, String> {
    service(db.inner().clone())
        .install_from_zip(&file_path)
        .map_err(|e| e.to_string())
}
