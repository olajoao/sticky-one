use crate::config::{db_path, RETENTION_HOURS};
use crate::entry::{ContentType, Entry};
use crate::error::{Result, StickyError};
use rusqlite::{params, Connection};
use std::fs;

pub struct Storage {
    conn: Connection,
}

impl Storage {
    pub fn open() -> Result<Self> {
        let path = db_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let conn = Connection::open(&path)?;
        let storage = Self { conn };
        storage.init_schema()?;
        Ok(storage)
    }

    fn init_schema(&self) -> Result<()> {
        self.conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS entries (
                id INTEGER PRIMARY KEY,
                content_type TEXT NOT NULL,
                content TEXT,
                image_data BLOB,
                hash TEXT NOT NULL,
                created_at INTEGER NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_created_at ON entries(created_at);
            CREATE INDEX IF NOT EXISTS idx_hash ON entries(hash);",
        )?;
        Ok(())
    }

    pub fn insert(&self, entry: &Entry) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO entries (content_type, content, image_data, hash, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                entry.content_type.as_str(),
                entry.content,
                entry.image_data,
                entry.hash,
                entry.created_at,
            ],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn get_latest_hash(&self) -> Result<Option<String>> {
        let result = self.conn.query_row(
            "SELECT hash FROM entries ORDER BY created_at DESC LIMIT 1",
            [],
            |row| row.get(0),
        );

        match result {
            Ok(hash) => Ok(Some(hash)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    pub fn get_by_id(&self, id: i64) -> Result<Entry> {
        self.conn
            .query_row(
                "SELECT id, content_type, content, image_data, hash, created_at
                 FROM entries WHERE id = ?1",
                [id],
                |row| Ok(row_to_entry(row)),
            )
            .map_err(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => StickyError::NotFound(id),
                _ => e.into(),
            })
    }

    pub fn list(&self, limit: usize) -> Result<Vec<Entry>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, content_type, content, image_data, hash, created_at
             FROM entries ORDER BY created_at DESC LIMIT ?1",
        )?;

        let entries = stmt
            .query_map([limit], |row| Ok(row_to_entry(row)))?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(entries)
    }

    pub fn search(&self, query: &str, limit: usize) -> Result<Vec<Entry>> {
        let pattern = format!("%{}%", query);
        let mut stmt = self.conn.prepare(
            "SELECT id, content_type, content, image_data, hash, created_at
             FROM entries
             WHERE content LIKE ?1
             ORDER BY created_at DESC LIMIT ?2",
        )?;

        let entries = stmt
            .query_map(params![pattern, limit], |row| Ok(row_to_entry(row)))?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(entries)
    }

    pub fn cleanup_old(&self) -> Result<usize> {
        let cutoff = chrono::Utc::now().timestamp() - (RETENTION_HOURS * 3600);
        let deleted = self
            .conn
            .execute("DELETE FROM entries WHERE created_at < ?1", [cutoff])?;
        Ok(deleted)
    }

    pub fn clear(&self) -> Result<usize> {
        let deleted = self.conn.execute("DELETE FROM entries", [])?;
        Ok(deleted)
    }

    pub fn count(&self) -> Result<usize> {
        let count: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM entries", [], |row| row.get(0))?;
        Ok(count as usize)
    }
}

fn row_to_entry(row: &rusqlite::Row) -> Entry {
    Entry {
        id: row.get(0).unwrap_or(0),
        content_type: ContentType::from_str(row.get::<_, String>(1).unwrap_or_default().as_str())
            .unwrap_or(ContentType::Text),
        content: row.get(2).ok(),
        image_data: row.get(3).ok(),
        hash: row.get(4).unwrap_or_default(),
        created_at: row.get(5).unwrap_or(0),
    }
}
