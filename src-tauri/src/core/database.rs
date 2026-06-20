use rusqlite::{params, Connection, Result};
use std::path::PathBuf;
use uuid::Uuid;
use crate::core::config::get_config_dir;

pub struct Database {
    db_path: PathBuf,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct HistoryEntry {
    pub id: String,
    pub source_lang: String,
    pub target_lang: String,
    pub original_text: String,
    pub translated_text: String,
    pub is_favorite: bool,
    pub created_at: String,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct LanguagePackInfo {
    pub lang_code: String,
    pub lang_name: String,
    pub version: String,
    pub status: String,
    pub local_path: Option<String>,
    pub model_size_bytes: i64,
}

impl Database {
    pub fn new() -> Self {
        let mut db_path = get_config_dir();
        db_path.push("langflow.db");
        
        let db = Database { db_path };
        db.init().expect("Failed to initialize database");
        db
    }

    fn connect(&self) -> Result<Connection> {
        let conn = Connection::open(&self.db_path)?;
        // Enable WAL mode for high performance and concurrency
        conn.execute_batch(
            "PRAGMA journal_mode=WAL;
             PRAGMA synchronous=NORMAL;"
        )?;
        Ok(conn)
    }

    fn init(&self) -> Result<()> {
        let conn = self.connect()?;
        
        // 1. Create history table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS translation_history (
                id TEXT PRIMARY KEY,
                source_lang TEXT NOT NULL,
                target_lang TEXT NOT NULL,
                original_text TEXT NOT NULL,
                translated_text TEXT NOT NULL,
                is_favorite INTEGER NOT NULL DEFAULT 0,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            );",
            [],
        )?;
        
        // Create indexes for history
        conn.execute("CREATE INDEX IF NOT EXISTS idx_history_text ON translation_history(original_text);", [])?;
        conn.execute("CREATE INDEX IF NOT EXISTS idx_history_created ON translation_history(created_at DESC);", [])?;

        // 2. Create cache table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS translation_cache (
                key_hash TEXT PRIMARY KEY,
                source_lang TEXT NOT NULL,
                target_lang TEXT NOT NULL,
                original_text TEXT NOT NULL,
                translated_text TEXT NOT NULL,
                hit_count INTEGER NOT NULL DEFAULT 1,
                last_accessed TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            );",
            [],
        )?;
        
        conn.execute("CREATE INDEX IF NOT EXISTS idx_cache_accessed ON translation_cache(last_accessed DESC);", [])?;

        // 3. Create language packs table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS language_packs (
                lang_code TEXT PRIMARY KEY,
                lang_name TEXT NOT NULL,
                version TEXT NOT NULL,
                status TEXT NOT NULL,
                local_path TEXT,
                model_size_bytes INTEGER NOT NULL DEFAULT 0,
                installed_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            );",
            [],
        )?;

        Ok(())
    }

    // --- Cache Operations ---
    fn calculate_hash(source: &str, target: &str, text: &str) -> String {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(format!("{}:{}:{}", source, target, text).as_bytes());
        format!("{:x}", hasher.finalize())
    }

    pub fn get_cache(&self, source: &str, target: &str, text: &str) -> Option<String> {
        let conn = match self.connect() {
            Ok(c) => c,
            Err(_) => return None,
        };
        let key_hash = Self::calculate_hash(source, target, text);
        
        let mut stmt = conn.prepare(
            "SELECT translated_text FROM translation_cache WHERE key_hash = ?1"
        ).ok()?;
        
        let result: Result<String> = stmt.query_row(params![key_hash], |row| row.get(0));
        
        if let Ok(translated) = result {
            // Update last accessed time and increment hit count
            let _ = conn.execute(
                "UPDATE translation_cache SET hit_count = hit_count + 1, last_accessed = CURRENT_TIMESTAMP WHERE key_hash = ?1",
                params![key_hash]
            );
            Some(translated)
        } else {
            None
        }
    }

    pub fn set_cache(&self, source: &str, target: &str, text: &str, translated: &str) {
        if let Ok(conn) = self.connect() {
            let key_hash = Self::calculate_hash(source, target, text);
            let _ = conn.execute(
                "INSERT OR REPLACE INTO translation_cache (key_hash, source_lang, target_lang, original_text, translated_text, last_accessed)
                 VALUES (?1, ?2, ?3, ?4, ?5, CURRENT_TIMESTAMP)",
                params![key_hash, source, target, text, translated]
            );
        }
    }

    // --- History Operations ---
    pub fn add_history(&self, source: &str, target: &str, original: &str, translated: &str) -> Result<String> {
        let conn = self.connect()?;
        let id = Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO translation_history (id, source_lang, target_lang, original_text, translated_text, is_favorite)
             VALUES (?1, ?2, ?3, ?4, ?5, 0)",
            params![id, source, target, original, translated],
        )?;
        Ok(id)
    }

    pub fn get_history(&self, search_query: Option<String>) -> Result<Vec<HistoryEntry>> {
        let conn = self.connect()?;
        let mut stmt = match search_query {
            Some(ref q) if !q.is_empty() => {
                conn.prepare(
                    "SELECT id, source_lang, target_lang, original_text, translated_text, is_favorite, datetime(created_at, 'localtime')
                     FROM translation_history
                     WHERE original_text LIKE ?1 OR translated_text LIKE ?1
                     ORDER BY created_at DESC"
                )?
            }
            _ => {
                conn.prepare(
                    "SELECT id, source_lang, target_lang, original_text, translated_text, is_favorite, datetime(created_at, 'localtime')
                     FROM translation_history
                     ORDER BY created_at DESC"
                )?
            }
        };

        let rows = if let Some(ref q) = search_query {
            if !q.is_empty() {
                stmt.query(params![format!("%{}%", q)])?
            } else {
                stmt.query([])?
            }
        } else {
            stmt.query([])?
        };

        let mut history = Vec::new();
        let mut iter = rows;
        while let Some(row) = iter.next()? {
            history.push(HistoryEntry {
                id: row.get(0)?,
                source_lang: row.get(1)?,
                target_lang: row.get(2)?,
                original_text: row.get(3)?,
                translated_text: row.get(4)?,
                is_favorite: row.get::<_, i32>(5)? != 0,
                created_at: row.get(6)?,
            });
        }
        Ok(history)
    }

    pub fn toggle_favorite(&self, id: &str) -> Result<bool> {
        let conn = self.connect()?;
        let current_state: i32 = conn.query_row(
            "SELECT is_favorite FROM translation_history WHERE id = ?1",
            params![id],
            |row| row.get(0)
        )?;
        
        let new_state = if current_state == 0 { 1 } else { 0 };
        conn.execute(
            "UPDATE translation_history SET is_favorite = ?1 WHERE id = ?2",
            params![new_state, id]
        )?;
        Ok(new_state != 0)
    }

    pub fn delete_history(&self, id: &str) -> Result<()> {
        let conn = self.connect()?;
        conn.execute("DELETE FROM translation_history WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn clear_history(&self) -> Result<()> {
        let conn = self.connect()?;
        conn.execute("DELETE FROM translation_history", [])?;
        Ok(())
    }

    // --- Language Pack Operations ---
    pub fn get_language_packs(&self) -> Result<Vec<LanguagePackInfo>> {
        let conn = self.connect()?;
        let mut stmt = conn.prepare(
            "SELECT lang_code, lang_name, version, status, local_path, model_size_bytes FROM language_packs"
        )?;
        let rows = stmt.query([])?;
        let mut packs = Vec::new();
        let mut iter = rows;
        while let Some(row) = iter.next()? {
            packs.push(LanguagePackInfo {
                lang_code: row.get(0)?,
                lang_name: row.get(1)?,
                version: row.get(2)?,
                status: row.get(3)?,
                local_path: row.get(4)?,
                model_size_bytes: row.get(5)?,
            });
        }
        Ok(packs)
    }

    pub fn register_language_pack(&self, info: &LanguagePackInfo) -> Result<()> {
        let conn = self.connect()?;
        conn.execute(
            "INSERT OR REPLACE INTO language_packs (lang_code, lang_name, version, status, local_path, model_size_bytes)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                info.lang_code,
                info.lang_name,
                info.version,
                info.status,
                info.local_path,
                info.model_size_bytes
            ],
        )?;
        Ok(())
    }

    pub fn delete_language_pack(&self, lang_code: &str) -> Result<()> {
        let conn = self.connect()?;
        conn.execute("DELETE FROM language_packs WHERE lang_code = ?1", params![lang_code])?;
        Ok(())
    }
}
