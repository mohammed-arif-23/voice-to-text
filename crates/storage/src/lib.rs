use serde::{Deserialize, Serialize};
use rusqlite::{params, Connection};
use std::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogEntry {
    pub timestamp: u64,
    pub emr_app_name: String,
    pub injected_text: String,
    pub had_low_confidence: bool,
}

pub trait StorageEngine: Send + Sync {
    fn log_audit_entry(&self, entry: &AuditLogEntry) -> Result<(), String>;
    fn get_recent_audit_logs(&self, limit: usize) -> Result<Vec<AuditLogEntry>, String>;
}

pub struct DummyStorageEngine;

impl StorageEngine for DummyStorageEngine {
    fn log_audit_entry(&self, _entry: &AuditLogEntry) -> Result<(), String> {
        Ok(())
    }

    fn get_recent_audit_logs(&self, _limit: usize) -> Result<Vec<AuditLogEntry>, String> {
        Ok(vec![])
    }
}

pub struct SqliteStorageEngine {
    conn: Mutex<Connection>,
}

impl SqliteStorageEngine {
    pub fn new(db_path: &str) -> Result<Self, String> {
        let conn = Connection::open(db_path).map_err(|e| e.to_string())?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS audit_logs (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp INTEGER NOT NULL,
                emr_app_name TEXT NOT NULL,
                injected_text TEXT NOT NULL,
                had_low_confidence INTEGER NOT NULL
            )",
            [],
        ).map_err(|e| e.to_string())?;

        Ok(Self {
            conn: Mutex::new(conn),
        })
    }
}

impl StorageEngine for SqliteStorageEngine {
    fn log_audit_entry(&self, entry: &AuditLogEntry) -> Result<(), String> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO audit_logs (timestamp, emr_app_name, injected_text, had_low_confidence)
             VALUES (?1, ?2, ?3, ?4)",
            params![
                entry.timestamp,
                entry.emr_app_name,
                entry.injected_text,
                if entry.had_low_confidence { 1 } else { 0 }
            ],
        ).map_err(|e| e.to_string())?;
        Ok(())
    }

    fn get_recent_audit_logs(&self, limit: usize) -> Result<Vec<AuditLogEntry>, String> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT timestamp, emr_app_name, injected_text, had_low_confidence
             FROM audit_logs
             ORDER BY id DESC
             LIMIT ?1"
        ).map_err(|e| e.to_string())?;

        let rows = stmt.query_map(params![limit], |row| {
            let had_low_int: i32 = row.get(3)?;
            Ok(AuditLogEntry {
                timestamp: row.get(0)?,
                emr_app_name: row.get(1)?,
                injected_text: row.get(2)?,
                had_low_confidence: had_low_int != 0,
            })
        }).map_err(|e| e.to_string())?;

        let mut logs = Vec::new();
        for row in rows {
            logs.push(row.map_err(|e| e.to_string())?);
        }
        Ok(logs)
    }
}

