use anyhow::{Context, Result};
use rusqlite::{params, Connection, OptionalExtension};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use crate::models::{PhotoMetadataRecord, WorkspaceRecord};

/// 遵循“源卡纯只读（Read-Only）”原则的本地嵌入式 SQLite 数据库引擎
#[derive(Clone)]
pub struct Database {
    conn: Arc<Mutex<Connection>>,
    pub db_path: PathBuf,
}

impl Database {
    /// 初始化本地 SQLite 数据库，自动检测操作系统标准数据存储路径
    pub fn init() -> Result<Self> {
        let app_data_dir = Self::resolve_app_data_dir()?;
        std::fs::create_dir_all(&app_data_dir)
            .with_context(|| format!("Failed to create app data dir: {:?}", app_data_dir))?;

        let db_path = app_data_dir.join("photoselect.db");
        let conn = Connection::open(&db_path)
            .with_context(|| format!("Failed to open SQLite database: {:?}", db_path))?;

        // 启用 WAL 模式提高并发性能与抗灾能力
        let _ = conn.execute_batch("PRAGMA journal_mode = WAL; PRAGMA synchronous = NORMAL;");

        // 初始化数据表架构
        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS workspaces (
                id TEXT PRIMARY KEY,
                folder_path TEXT NOT NULL UNIQUE,
                volume_uuid TEXT,
                last_opened_at DATETIME DEFAULT CURRENT_TIMESTAMP
            );

            CREATE TABLE IF NOT EXISTS photo_metadata (
                file_signature TEXT PRIMARY KEY,
                base_name TEXT NOT NULL,
                workspace_id TEXT,
                folder_path TEXT NOT NULL,
                rating INTEGER DEFAULT 0,
                flag INTEGER DEFAULT 0,
                color_label TEXT DEFAULT 'none',
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
            );

            CREATE TABLE IF NOT EXISTS app_settings (
                setting_key TEXT PRIMARY KEY,
                setting_value TEXT NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_workspace_filter 
                ON photo_metadata(workspace_id, flag, rating, color_label);
            CREATE INDEX IF NOT EXISTS idx_folder_path 
                ON photo_metadata(folder_path);
            CREATE INDEX IF NOT EXISTS idx_base_name 
                ON photo_metadata(base_name);
            "#,
        )?;

        // 动态增加 tags 字段支持生态与通用关键字 (若已存在则忽略)
        let _ = conn.execute("ALTER TABLE photo_metadata ADD COLUMN tags TEXT DEFAULT ''", []);

        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
            db_path,
        })
    }

    /// 解析系统标准数据存储目录
    pub fn resolve_app_data_dir() -> Result<PathBuf> {
        #[cfg(target_os = "windows")]
        {
            if let Ok(local_app_data) = std::env::var("LOCALAPPDATA") {
                return Ok(PathBuf::from(local_app_data).join("PhotoSelect"));
            }
            if let Ok(user_profile) = std::env::var("USERPROFILE") {
                return Ok(PathBuf::from(user_profile)
                    .join("AppData")
                    .join("Local")
                    .join("PhotoSelect"));
            }
        }

        #[cfg(target_os = "linux")]
        {
            if let Ok(xdg_data) = std::env::var("XDG_DATA_HOME") {
                return Ok(PathBuf::from(xdg_data).join("photoselect"));
            }
            if let Ok(home) = std::env::var("HOME") {
                return Ok(PathBuf::from(home)
                    .join(".local")
                    .join("share")
                    .join("photoselect"));
            }
        }

        Ok(PathBuf::from(".photoselect_data"))
    }

    /// 记录或刷新工作区/目录访问时间
    pub fn record_workspace(&self, folder_path: &str, volume_uuid: Option<&str>) -> Result<String> {
        let norm_path = folder_path.replace('\\', "/");
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT id FROM workspaces WHERE folder_path = ?1")?;
        let existing_id: Option<String> = stmt.query_row(params![norm_path], |row| row.get(0)).ok();

        if let Some(id) = existing_id {
            conn.execute(
                "UPDATE workspaces SET last_opened_at = CURRENT_TIMESTAMP, volume_uuid = ?2 WHERE id = ?1",
                params![id, volume_uuid],
            )?;
            Ok(id)
        } else {
            let new_id = uuid::Uuid::new_v4().to_string();
            conn.execute(
                "INSERT INTO workspaces (id, folder_path, volume_uuid, last_opened_at) VALUES (?1, ?2, ?3, CURRENT_TIMESTAMP)",
                params![new_id, norm_path, volume_uuid],
            )?;
            Ok(new_id)
        }
    }

    /// 获取最近打开的工作区列表
    pub fn get_recent_workspaces(&self) -> Result<Vec<WorkspaceRecord>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, folder_path, volume_uuid, last_opened_at FROM workspaces ORDER BY last_opened_at DESC LIMIT 20",
        )?;

        let rows = stmt.query_map([], |row| {
            Ok(WorkspaceRecord {
                id: row.get(0)?,
                folder_path: row.get(1)?,
                volume_uuid: row.get(2)?,
                last_opened_at: row.get(3)?,
            })
        })?;

        let mut list = Vec::new();
        for r in rows {
            list.push(r?);
        }
        Ok(list)
    }

    pub fn get_app_setting(&self, key: &str) -> Result<Option<String>> {
        let conn = self.conn.lock().unwrap();
        let value = conn
            .query_row(
                "SELECT setting_value FROM app_settings WHERE setting_key = ?1",
                params![key],
                |row| row.get(0),
            )
            .optional()?;
        Ok(value)
    }

    pub fn set_app_settings(&self, settings: &[(&str, &str)]) -> Result<()> {
        let mut conn = self.conn.lock().unwrap();
        let tx = conn.transaction()?;
        {
            let mut stmt = tx.prepare(
                "INSERT INTO app_settings (setting_key, setting_value) VALUES (?1, ?2) ON CONFLICT(setting_key) DO UPDATE SET setting_value = excluded.setting_value",
            )?;
            for (key, value) in settings {
                stmt.execute(params![key, value])?;
            }
        }
        tx.commit()?;
        Ok(())
    }

    /// 原子单独更新星级评分 (绝不破坏已打的旗标与色标)
    pub fn update_rating(
        &self,
        file_signature: &str,
        base_name: &str,
        folder_path: &str,
        rating: u8,
    ) -> Result<()> {
        let norm_folder = folder_path.replace('\\', "/");
        let conn = self.conn.lock().unwrap();
        conn.execute(
            r#"
            INSERT INTO photo_metadata (file_signature, base_name, folder_path, rating, flag, color_label, updated_at)
            VALUES (?1, ?2, ?3, ?4, 0, 'none', CURRENT_TIMESTAMP)
            ON CONFLICT(file_signature) DO UPDATE SET
                rating = excluded.rating,
                updated_at = CURRENT_TIMESTAMP
            "#,
            params![file_signature, base_name, norm_folder, rating],
        )?;
        Ok(())
    }

    /// 原子单独更新旗标状态 (绝不破坏已打的星级与色标)
    pub fn update_flag(
        &self,
        file_signature: &str,
        base_name: &str,
        folder_path: &str,
        flag: i8,
    ) -> Result<()> {
        let norm_folder = folder_path.replace('\\', "/");
        let conn = self.conn.lock().unwrap();
        conn.execute(
            r#"
            INSERT INTO photo_metadata (file_signature, base_name, folder_path, rating, flag, color_label, updated_at)
            VALUES (?1, ?2, ?3, 0, ?4, 'none', CURRENT_TIMESTAMP)
            ON CONFLICT(file_signature) DO UPDATE SET
                flag = excluded.flag,
                updated_at = CURRENT_TIMESTAMP
            "#,
            params![file_signature, base_name, norm_folder, flag],
        )?;
        Ok(())
    }

    /// 原子单独更新色彩标签 (绝不破坏已打的星级与旗标)
    pub fn update_color_label(
        &self,
        file_signature: &str,
        base_name: &str,
        folder_path: &str,
        color_label: &str,
    ) -> Result<()> {
        let norm_folder = folder_path.replace('\\', "/");
        let conn = self.conn.lock().unwrap();
        conn.execute(
            r#"
            INSERT INTO photo_metadata (file_signature, base_name, folder_path, rating, flag, color_label, updated_at)
            VALUES (?1, ?2, ?3, 0, 0, ?4, CURRENT_TIMESTAMP)
            ON CONFLICT(file_signature) DO UPDATE SET
                color_label = excluded.color_label,
                updated_at = CURRENT_TIMESTAMP
            "#,
            params![file_signature, base_name, norm_folder, color_label],
        )?;
        Ok(())
    }

    /// 全量更新或插入照片的星级、旗标与色标
    pub fn upsert_full_metadata(
        &self,
        file_signature: &str,
        base_name: &str,
        folder_path: &str,
        rating: u8,
        flag: i8,
        color_label: &str,
    ) -> Result<()> {
        let norm_folder = folder_path.replace('\\', "/");
        let conn = self.conn.lock().unwrap();
        conn.execute(
            r#"
            INSERT INTO photo_metadata (file_signature, base_name, folder_path, rating, flag, color_label, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, CURRENT_TIMESTAMP)
            ON CONFLICT(file_signature) DO UPDATE SET
                rating = excluded.rating,
                flag = excluded.flag,
                color_label = excluded.color_label,
                updated_at = CURRENT_TIMESTAMP
            "#,
            params![file_signature, base_name, norm_folder, rating, flag, color_label],
        )?;
        Ok(())
    }

    /// 批量事务写入照片标记数据
    pub fn batch_upsert_metadata(
        &self,
        records: &[(String, String, String, u8, i8, String)],
    ) -> Result<()> {
        let mut conn = self.conn.lock().unwrap();
        let tx = conn.transaction()?;

        {
            let mut stmt = tx.prepare(
                r#"
                INSERT INTO photo_metadata (file_signature, base_name, folder_path, rating, flag, color_label, updated_at)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, CURRENT_TIMESTAMP)
                ON CONFLICT(file_signature) DO UPDATE SET
                    rating = excluded.rating,
                    flag = excluded.flag,
                    color_label = excluded.color_label,
                    updated_at = CURRENT_TIMESTAMP
                "#,
            )?;

            for (sig, base_name, folder, rating, flag, color) in records {
                let norm = folder.replace('\\', "/");
                stmt.execute(params![sig, base_name, norm, *rating, *flag, color])?;
            }
        }

        tx.commit()?;
        Ok(())
    }

    /// 更新照片的关键字与生态标签 (保留既有星级、旗标与色标)
    pub fn update_tags(
        &self,
        file_signature: &str,
        base_name: &str,
        folder_path: &str,
        tags: &[String],
    ) -> Result<()> {
        let norm_folder = folder_path.replace('\\', "/");
        let tags_json = serde_json::to_string(tags).unwrap_or_else(|_| "[]".to_string());
        let conn = self.conn.lock().unwrap();
        conn.execute(
            r#"
            INSERT INTO photo_metadata (file_signature, base_name, folder_path, rating, flag, color_label, tags, updated_at)
            VALUES (?1, ?2, ?3, 0, 0, 'none', ?4, CURRENT_TIMESTAMP)
            ON CONFLICT(file_signature) DO UPDATE SET
                tags = excluded.tags,
                updated_at = CURRENT_TIMESTAMP
            "#,
            params![file_signature, base_name, norm_folder, tags_json],
        )?;
        Ok(())
    }

    /// 查询指定物理目录下所有已保存的照片状态
    /// 自动容错匹配正斜杠与反斜杠路径
    pub fn get_metadata_for_folder(
        &self,
        folder_path: &str,
    ) -> Result<HashMap<String, PhotoMetadataRecord>> {
        let norm_folder = folder_path.replace('\\', "/");
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            r#"
            SELECT file_signature, base_name, folder_path, rating, flag, color_label, updated_at, tags
            FROM photo_metadata
            WHERE folder_path = ?1 OR folder_path = ?2
            "#,
        )?;

        let alt_folder = folder_path.replace('/', "\\");
        let rows = stmt.query_map(params![norm_folder, alt_folder], |row| {
            let tags_str: Option<String> = row.get(7).ok();
            let tags: Vec<String> = tags_str
                .filter(|s| !s.trim().is_empty())
                .and_then(|s| serde_json::from_str(&s).ok())
                .unwrap_or_default();

            Ok(PhotoMetadataRecord {
                file_signature: row.get(0)?,
                base_name: row.get(1)?,
                folder_path: row.get(2)?,
                rating: row.get(3)?,
                flag: row.get(4)?,
                color_label: row.get(5)?,
                tags,
                updated_at: row.get(6)?,
            })
        })?;

        let mut map = HashMap::new();
        for r in rows {
            let record = r?;
            map.insert(record.file_signature.clone(), record);
        }
        Ok(map)
    }
}
