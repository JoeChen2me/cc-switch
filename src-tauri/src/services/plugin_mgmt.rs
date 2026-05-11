//! Claude Code Plugin 管理服务
//!
//! 直接操作 ~/.claude/plugins/ 目录，数据库仅缓存元数据。
//! 禁用时将 plugin 移到 ~/.cc-switch/plugins-disabled/。

use crate::app_config::InstalledPlugin;
use crate::config::get_app_config_dir;
use crate::database::Database;
use crate::error::AppError;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::path::PathBuf;
use std::sync::Arc;

/// plugin.json 中的 author 字段可以是字符串或对象
#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum AuthorValue {
    String(String),
    Object { name: String },
}

#[derive(Debug, Deserialize)]
struct PluginManifest {
    name: String,
    #[serde(default)]
    version: Option<String>,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    author: Option<AuthorValue>,
}

/// Plugin 管理服务
pub struct PluginService {
    db: Arc<Database>,
}

/// 验证 id 不含路径穿越字符
fn validate_id(id: &str) -> Result<(), AppError> {
    if id.is_empty()
        || id.contains('/')
        || id.contains('\\')
        || id.contains("..")
        || id == "."
    {
        return Err(AppError::Message(format!("非法插件标识: {id}")));
    }
    Ok(())
}

/// 移动目录，跨文件系统时回退到 copy + remove
fn move_dir(src: &std::path::Path, dst: &std::path::Path) -> Result<(), AppError> {
    if std::fs::rename(src, dst).is_ok() {
        return Ok(());
    }
    // rename 失败（可能跨文件系统），回退到 copy + remove
    copy_dir_recursive(src, dst)?;
    std::fs::remove_dir_all(src)
        .map_err(|e| AppError::Message(format!("删除源目录失败: {e}")))?;
    Ok(())
}

impl PluginService {
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }

    /// ~/.claude/plugins/ 路径
    fn plugins_dir() -> Result<PathBuf, AppError> {
        let home = dirs::home_dir()
            .ok_or_else(|| AppError::Message("无法获取 HOME 目录".into()))?;
        Ok(home.join(".claude").join("plugins"))
    }

    /// ~/.cc-switch/plugins-disabled/ 路径
    fn disabled_dir() -> Result<PathBuf, AppError> {
        Ok(get_app_config_dir().join("plugins-disabled"))
    }

    /// 扫描所有已安装的 plugin，同步数据库并返回列表
    pub fn scan_plugins(&self) -> Result<Vec<InstalledPlugin>, AppError> {
        let plugins_dir = Self::plugins_dir()?;
        let disabled_dir = Self::disabled_dir()?;

        std::fs::create_dir_all(&disabled_dir)
            .map_err(|e| AppError::Message(format!("创建 plugins-disabled 目录失败: {e}")))?;

        let mut result = Vec::new();

        // 获取数据库已有记录，用于保留 installed_at
        let existing = self.db.get_all_plugins()?;
        let existing_map: std::collections::HashMap<String, i64> = existing
            .iter()
            .map(|p| (p.id.clone(), p.installed_at))
            .collect();

        if plugins_dir.exists() {
            Self::scan_directory(&plugins_dir, true, &existing_map, &mut result)?;
        }
        if disabled_dir.exists() {
            Self::scan_directory(&disabled_dir, false, &existing_map, &mut result)?;
        }

        // 事务性同步数据库
        self.db.sync_plugins_batch(&result)?;

        Ok(result)
    }

    fn scan_directory(
        dir: &std::path::Path,
        enabled: bool,
        existing: &std::collections::HashMap<String, i64>,
        result: &mut Vec<InstalledPlugin>,
    ) -> Result<(), AppError> {
        let entries = std::fs::read_dir(dir)
            .map_err(|e| AppError::Message(format!("读取插件目录失败: {e}")))?;

        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }

            let manifest_path = path.join(".claude-plugin").join("plugin.json");
            let manifest_path = if manifest_path.exists() {
                manifest_path
            } else {
                let fallback = path.join("plugin.json");
                if fallback.exists() {
                    fallback
                } else {
                    continue;
                }
            };

            let directory_name = path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();

            match Self::parse_plugin(&directory_name, &manifest_path, enabled, existing) {
                Ok(plugin) => result.push(plugin),
                Err(e) => {
                    log::warn!("解析插件 {} 的 plugin.json 失败: {e}", directory_name);
                }
            }
        }
        Ok(())
    }

    fn parse_plugin(
        directory_name: &str,
        manifest_path: &std::path::Path,
        enabled: bool,
        existing: &std::collections::HashMap<String, i64>,
    ) -> Result<InstalledPlugin, AppError> {
        let raw = std::fs::read_to_string(manifest_path)
            .map_err(|e| AppError::Message(format!("读取 plugin.json 失败: {e}")))?;

        let manifest: PluginManifest = serde_json::from_str(&raw)
            .map_err(|e| AppError::Message(format!("解析 plugin.json 失败: {e}")))?;

        let author = manifest.author.map(|a| match a {
            AuthorValue::String(s) => s,
            AuthorValue::Object { name } => name,
        });

        let content_hash = {
            let mut hasher = Sha256::new();
            hasher.update(raw.as_bytes());
            format!("{:x}", hasher.finalize())
        };

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;

        // 保留数据库中已有的 installed_at，避免每次扫描覆盖首次安装时间
        let installed_at = existing.get(directory_name).copied().unwrap_or(now);

        Ok(InstalledPlugin {
            id: directory_name.to_string(),
            name: manifest.name,
            version: manifest.version,
            description: manifest.description,
            author,
            directory_name: directory_name.to_string(),
            enabled,
            installed_at,
            content_hash: Some(content_hash),
            plugin_json_raw: Some(raw),
        })
    }

    pub fn disable_plugin(&self, id: &str) -> Result<InstalledPlugin, AppError> {
        validate_id(id)?;
        let src = Self::plugins_dir()?.join(id);
        let dst = Self::disabled_dir()?.join(id);

        if !src.exists() {
            return Err(AppError::Message(format!("插件 {id} 不存在于启用目录")));
        }

        std::fs::create_dir_all(Self::disabled_dir()?)
            .map_err(|e| AppError::Message(format!("创建禁用目录失败: {e}")))?;

        if dst.exists() {
            std::fs::remove_dir_all(&dst)
                .map_err(|e| AppError::Message(format!("删除已有禁用目录失败: {e}")))?;
        }

        move_dir(&src, &dst)
            .map_err(|e| AppError::Message(format!("移动插件到禁用目录失败: {e}")))?;

        self.db.update_plugin_enabled(id, false)?;

        self.scan_plugins()?
            .into_iter()
            .find(|p| p.id == id)
            .ok_or_else(|| AppError::Message(format!("禁用后未找到插件 {id}")))
    }

    pub fn enable_plugin(&self, id: &str) -> Result<InstalledPlugin, AppError> {
        validate_id(id)?;
        let src = Self::disabled_dir()?.join(id);
        let dst = Self::plugins_dir()?.join(id);

        if !src.exists() {
            return Err(AppError::Message(format!("插件 {id} 不存在于禁用目录")));
        }

        std::fs::create_dir_all(Self::plugins_dir()?)
            .map_err(|e| AppError::Message(format!("创建插件目录失败: {e}")))?;

        if dst.exists() {
            std::fs::remove_dir_all(&dst)
                .map_err(|e| AppError::Message(format!("删除已有插件目录失败: {e}")))?;
        }

        move_dir(&src, &dst)
            .map_err(|e| AppError::Message(format!("移动插件到启用目录失败: {e}")))?;

        self.db.update_plugin_enabled(id, true)?;

        self.scan_plugins()?
            .into_iter()
            .find(|p| p.id == id)
            .ok_or_else(|| AppError::Message(format!("启用后未找到插件 {id}")))
    }

    pub fn uninstall_plugin(&self, id: &str) -> Result<bool, AppError> {
        validate_id(id)?;
        let enabled_path = Self::plugins_dir()?.join(id);
        let disabled_path = Self::disabled_dir()?.join(id);

        let mut deleted = false;
        if enabled_path.exists() {
            std::fs::remove_dir_all(&enabled_path)
                .map_err(|e| AppError::Message(format!("删除插件目录失败: {e}")))?;
            deleted = true;
        }
        if disabled_path.exists() {
            std::fs::remove_dir_all(&disabled_path)
                .map_err(|e| AppError::Message(format!("删除禁用插件目录失败: {e}")))?;
            deleted = true;
        }

        if !deleted {
            return Err(AppError::Message(format!("插件 {id} 不存在")));
        }

        self.db.delete_plugin(id)?;
        Ok(true)
    }

    pub fn install_from_zip(&self, zip_path: &str) -> Result<Vec<InstalledPlugin>, AppError> {
        let plugins_dir = Self::plugins_dir()?;
        std::fs::create_dir_all(&plugins_dir)
            .map_err(|e| AppError::Message(format!("创建插件目录失败: {e}")))?;

        let existing = self.db.get_all_plugins()?;
        let existing_map: std::collections::HashMap<String, i64> = existing
            .iter()
            .map(|p| (p.id.clone(), p.installed_at))
            .collect();

        let file = std::fs::File::open(zip_path)
            .map_err(|e| AppError::Message(format!("打开 ZIP 文件失败: {e}")))?;

        let mut archive = zip::ZipArchive::new(file)
            .map_err(|e| AppError::Message(format!("解析 ZIP 文件失败: {e}")))?;

        let temp_dir = tempfile::tempdir()
            .map_err(|e| AppError::Message(format!("创建临时目录失败: {e}")))?;

        archive
            .extract(temp_dir.path())
            .map_err(|e| AppError::Message(format!("解压 ZIP 失败: {e}")))?;

        let mut installed = Vec::new();
        Self::find_plugin_dirs(temp_dir.path(), &mut installed)?;

        if installed.is_empty() {
            return Err(AppError::Message(
                "ZIP 文件中未找到包含 plugin.json 的插件".into(),
            ));
        }

        let mut results = Vec::new();
        for plugin_src in installed {
            let dir_name = plugin_src
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();

            // 防止 ZIP 路径穿越
            if dir_name.contains('/') || dir_name.contains('\\') || dir_name.contains("..") {
                log::warn!("跳过非法目录名: {dir_name}");
                continue;
            }

            let dest = plugins_dir.join(&dir_name);

            if dest.exists() {
                std::fs::remove_dir_all(&dest)
                    .map_err(|e| AppError::Message(format!("删除已有插件失败: {e}")))?;
            }

            copy_dir_recursive(&plugin_src, &dest)?;

            let manifest_path = dest
                .join(".claude-plugin")
                .join("plugin.json")
                .exists()
                .then(|| dest.join(".claude-plugin").join("plugin.json"))
                .or_else(|| dest.join("plugin.json").exists().then(|| dest.join("plugin.json")));

            if let Some(mp) = manifest_path {
                match Self::parse_plugin(&dir_name, &mp, true, &existing_map) {
                    Ok(plugin) => {
                        self.db.save_plugin(&plugin)?;
                        results.push(plugin);
                    }
                    Err(e) => {
                        log::warn!("解析安装后的 plugin.json 失败: {e}");
                    }
                }
            }
        }

        Ok(results)
    }

    fn find_plugin_dirs(
        dir: &std::path::Path,
        results: &mut Vec<PathBuf>,
    ) -> Result<(), AppError> {
        let has_manifest = dir.join(".claude-plugin").join("plugin.json").exists()
            || dir.join("plugin.json").exists();

        if has_manifest {
            results.push(dir.to_path_buf());
            return Ok(());
        }

        let entries = std::fs::read_dir(dir)
            .map_err(|e| AppError::Message(format!("读取目录失败: {e}")))?;

        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                Self::find_plugin_dirs(&path, results)?;
            }
        }
        Ok(())
    }
}

fn copy_dir_recursive(src: &std::path::Path, dst: &std::path::Path) -> Result<(), AppError> {
    std::fs::create_dir_all(dst)
        .map_err(|e| AppError::Message(format!("创建目录失败: {e}")))?;

    let entries = std::fs::read_dir(src)
        .map_err(|e| AppError::Message(format!("读取源目录失败: {e}")))?;

    for entry in entries.flatten() {
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        // 跳过符号链接，防止跟随到插件目录外的路径
        if src_path.symlink_metadata().map_or(false, |m| m.file_type().is_symlink()) {
            log::warn!("跳过符号链接: {}", src_path.display());
            continue;
        }

        if src_path.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            std::fs::copy(&src_path, &dst_path)
                .map_err(|e| AppError::Message(format!("复制文件失败: {e}")))?;
        }
    }
    Ok(())
}
