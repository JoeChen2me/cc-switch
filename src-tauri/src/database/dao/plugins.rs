//! Plugins 数据访问对象

use crate::app_config::InstalledPlugin;
use crate::database::{lock_conn, Database};
use crate::error::AppError;
use rusqlite::params;

impl Database {
    /// 获取指定应用的所有已记录 Plugins
    pub fn get_plugins_by_app(&self, app_type: &str) -> Result<Vec<InstalledPlugin>, AppError> {
        let conn = lock_conn!(self.conn);
        let mut stmt = conn
            .prepare(
                "SELECT id, app_type, name, version, description, author, directory_name,
                        enabled, installed_at, content_hash, plugin_json_raw
                 FROM plugins WHERE app_type = ?1 ORDER BY name ASC",
            )
            .map_err(|e| AppError::Database(e.to_string()))?;

        let iter = stmt
            .query_map(params![app_type], |row| {
                Ok(InstalledPlugin {
                    id: row.get(0)?,
                    app_type: {
                        let s: String = row.get(1)?;
                        s.parse().unwrap_or(crate::app_config::AppType::Claude)
                    },
                    name: row.get(2)?,
                    version: row.get(3)?,
                    description: row.get(4)?,
                    author: row.get(5)?,
                    directory_name: row.get(6)?,
                    enabled: row.get(7)?,
                    installed_at: row.get(8)?,
                    content_hash: row.get(9)?,
                    plugin_json_raw: row.get(10)?,
                })
            })
            .map_err(|e| AppError::Database(e.to_string()))?;

        let mut plugins = Vec::new();
        for p in iter {
            plugins.push(p.map_err(|e| AppError::Database(e.to_string()))?);
        }
        Ok(plugins)
    }

    /// 保存 Plugin（INSERT OR REPLACE）
    pub fn save_plugin(&self, plugin: &InstalledPlugin) -> Result<(), AppError> {
        let conn = lock_conn!(self.conn);
        conn.execute(
            "INSERT OR REPLACE INTO plugins
             (id, app_type, name, version, description, author, directory_name,
              enabled, installed_at, content_hash, plugin_json_raw)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![
                plugin.id,
                plugin.app_type.as_str(),
                plugin.name,
                plugin.version,
                plugin.description,
                plugin.author,
                plugin.directory_name,
                plugin.enabled,
                plugin.installed_at,
                plugin.content_hash,
                plugin.plugin_json_raw,
            ],
        )
        .map_err(|e| AppError::Database(e.to_string()))?;
        Ok(())
    }

    /// 删除 Plugin 记录
    pub fn delete_plugin(&self, id: &str, app_type: &str) -> Result<bool, AppError> {
        let conn = lock_conn!(self.conn);
        let affected = conn
            .execute(
                "DELETE FROM plugins WHERE id = ?1 AND app_type = ?2",
                params![id, app_type],
            )
            .map_err(|e| AppError::Database(e.to_string()))?;
        Ok(affected > 0)
    }

    /// 更新 Plugin 的启用状态
    pub fn update_plugin_enabled(
        &self,
        id: &str,
        app_type: &str,
        enabled: bool,
    ) -> Result<bool, AppError> {
        let conn = lock_conn!(self.conn);
        let affected = conn
            .execute(
                "UPDATE plugins SET enabled = ?1 WHERE id = ?2 AND app_type = ?3",
                params![enabled, id, app_type],
            )
            .map_err(|e| AppError::Database(e.to_string()))?;
        Ok(affected > 0)
    }

    /// 事务性同步：删除不在列表中的旧记录，upsert 所有新记录
    pub fn sync_plugins_batch(
        &self,
        app_type: &str,
        plugins: &[InstalledPlugin],
    ) -> Result<(), AppError> {
        let conn = lock_conn!(self.conn);
        conn.execute_batch("BEGIN")
            .map_err(|e| AppError::Database(e.to_string()))?;

        let result = (|| -> Result<(), rusqlite::Error> {
            let mut stmt = conn.prepare("SELECT id FROM plugins WHERE app_type = ?1")?;
            let existing_ids: Vec<String> = stmt
                .query_map(params![app_type], |r| r.get(0))?
                .filter_map(|r| r.ok())
                .collect();
            drop(stmt);

            let new_ids: std::collections::HashSet<&str> =
                plugins.iter().map(|p| p.id.as_str()).collect();

            for id in &existing_ids {
                if !new_ids.contains(id.as_str()) {
                    conn.execute(
                        "DELETE FROM plugins WHERE id = ?1 AND app_type = ?2",
                        params![id, app_type],
                    )?;
                }
            }

            for plugin in plugins {
                conn.execute(
                    "INSERT OR REPLACE INTO plugins
                     (id, app_type, name, version, description, author, directory_name,
                      enabled, installed_at, content_hash, plugin_json_raw)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
                    params![
                        plugin.id,
                        plugin.app_type.as_str(),
                        plugin.name,
                        plugin.version,
                        plugin.description,
                        plugin.author,
                        plugin.directory_name,
                        plugin.enabled,
                        plugin.installed_at,
                        plugin.content_hash,
                        plugin.plugin_json_raw,
                    ],
                )?;
            }
            Ok(())
        })();

        match result {
            Ok(()) => conn
                .execute_batch("COMMIT")
                .map_err(|e| AppError::Database(e.to_string())),
            Err(e) => {
                let _ = conn.execute_batch("ROLLBACK");
                Err(AppError::Database(e.to_string()))
            }
        }
    }
}
