use serde::{Deserialize, Serialize};
use rusqlite::{params, Connection};
use std::sync::Mutex;
use std::path::Path;

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
        let path = Path::new(db_path);
        let conn = Connection::open(path).map_err(|e| e.to_string())?;

        // 1. Get encryption key from DPAPI (on Windows) or Mock (on macOS/Linux)
        let encryption_key = get_or_create_db_key(path.parent())?;

        // 2. Set SQLCipher key
        let pragma_query = format!("PRAGMA key = '{}';", encryption_key.replace("'", "''"));
        conn.execute(&pragma_query, []).map_err(|e| format!("SQLCipher key pragma failed: {}", e))?;

        // 3. Initialize schema
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

// ── DPAPI Key Management Implementation ───────────────────────────────────────

#[cfg(target_os = "windows")]
fn get_or_create_db_key(parent_dir: Option<&Path>) -> Result<String, String> {
    use windows::Win32::Security::Cryptography::{CryptProtectData, CryptUnprotectData, CRYPTPROTECT_UI_FORBIDDEN};
    use windows::Win32::Security::Cryptography::CRYPTOAPI_BLOB;
    use std::fs::{self, File};
    use std::io::{Read, Write};

    let key_path = parent_dir.unwrap_or_else(|| Path::new(".")).join("scriberx.key");

    if key_path.exists() {
        // Read and decrypt the key
        let mut encrypted_bytes = Vec::new();
        File::open(&key_path)
            .and_then(|mut f| f.read_to_end(&mut encrypted_bytes))
            .map_err(|e| format!("Failed to read key file: {}", e))?;

        let mut input_blob = CRYPTOAPI_BLOB {
            cbData: encrypted_bytes.len() as u32,
            pbData: encrypted_bytes.as_mut_ptr(),
        };
        let mut output_blob = CRYPTOAPI_BLOB::default();

        unsafe {
            let success = CryptUnprotectData(
                &mut input_blob,
                None,
                None,
                None,
                None,
                CRYPTPROTECT_UI_FORBIDDEN.0,
                &mut output_blob,
            );
            if success.as_bool() {
                let decrypted_slice = std::slice::from_raw_parts(output_blob.pbData, output_blob.cbData as usize);
                let decrypted_key = String::from_utf8(decrypted_slice.to_vec())
                    .map_err(|e| format!("Key not valid UTF-8: {}", e))?;
                // Free memory allocated by CryptUnprotectData
                windows::Win32::System::Memory::LocalFree(windows::Win32::System::Memory::HLOCAL(output_blob.pbData as _));
                return Ok(decrypted_key);
            } else {
                return Err("Failed to decrypt database key via Windows DPAPI".to_string());
            }
        }
    } else {
        // Generate new random key
        let raw_key = uuid::Uuid::new_v4().to_string();
        let mut raw_bytes = raw_key.as_bytes().to_vec();

        let mut input_blob = CRYPTOAPI_BLOB {
            cbData: raw_bytes.len() as u32,
            pbData: raw_bytes.as_mut_ptr(),
        };
        let mut output_blob = CRYPTOAPI_BLOB::default();

        unsafe {
            let success = CryptProtectData(
                &mut input_blob,
                None,
                None,
                None,
                None,
                CRYPTPROTECT_UI_FORBIDDEN.0,
                &mut output_blob,
            );
            if success.as_bool() {
                let encrypted_slice = std::slice::from_raw_parts(output_blob.pbData, output_blob.cbData as usize);
                fs::write(&key_path, encrypted_slice)
                    .map_err(|e| format!("Failed to write encrypted key file: {}", e))?;
                windows::Win32::System::Memory::LocalFree(windows::Win32::System::Memory::HLOCAL(output_blob.pbData as _));
                return Ok(raw_key);
            } else {
                return Err("Failed to encrypt database key via Windows DPAPI".to_string());
            }
        }
    }
}

#[cfg(not(target_os = "windows"))]
fn get_or_create_db_key(parent_dir: Option<&Path>) -> Result<String, String> {
    use std::fs;
    let key_path = parent_dir.unwrap_or_else(|| Path::new(".")).join("scriberx.key");
    if key_path.exists() {
        fs::read_to_string(&key_path).map_err(|e| e.to_string())
    } else {
        let mock_key = "mac_mock_secure_sqlcipher_encryption_key_hash".to_string();
        let _ = fs::write(&key_path, &mock_key);
        Ok(mock_key)
    }
}
