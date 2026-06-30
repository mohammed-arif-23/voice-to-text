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
        conn.pragma_update(None, "key", &encryption_key).map_err(|e| format!("SQLCipher key pragma failed: {}", e))?;

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
    use windows::Win32::Security::Cryptography::{CryptProtectData, CryptUnprotectData, CRYPTPROTECT_UI_FORBIDDEN, CRYPT_INTEGER_BLOB};
    use windows::Win32::Foundation::{LocalFree, HLOCAL};
    use std::fs::{self, File};
    use std::io::Read;

    let key_path = parent_dir.unwrap_or_else(|| Path::new(".")).join("scriberx.key");

    if key_path.exists() {
        // Read and decrypt the key
        let mut encrypted_bytes = Vec::new();
        File::open(&key_path)
            .and_then(|mut f| f.read_to_end(&mut encrypted_bytes))
            .map_err(|e| format!("Failed to read key file: {}", e))?;

        let mut input_blob = CRYPT_INTEGER_BLOB {
            cbData: encrypted_bytes.len() as u32,
            pbData: encrypted_bytes.as_mut_ptr(),
        };
        let mut output_blob = CRYPT_INTEGER_BLOB::default();

        unsafe {
            let success = CryptUnprotectData(
                &mut input_blob,
                None,
                None,
                None,
                None,
                CRYPTPROTECT_UI_FORBIDDEN,
                &mut output_blob,
            );
            if success.is_ok() {
                let decrypted_slice = std::slice::from_raw_parts(output_blob.pbData, output_blob.cbData as usize);
                let decrypted_key = String::from_utf8(decrypted_slice.to_vec())
                    .map_err(|e| format!("Key not valid UTF-8: {}", e))?;
                // Free memory allocated by CryptUnprotectData
                let _ = LocalFree(HLOCAL(output_blob.pbData as _));
                return Ok(decrypted_key);
            } else {
                return Err("Failed to decrypt database key via Windows DPAPI".to_string());
            }
        }
    } else {
        // Generate new random key
        let raw_key = uuid::Uuid::new_v4().to_string();
        let mut raw_bytes = raw_key.as_bytes().to_vec();

        let mut input_blob = CRYPT_INTEGER_BLOB {
            cbData: raw_bytes.len() as u32,
            pbData: raw_bytes.as_mut_ptr(),
        };
        let mut output_blob = CRYPT_INTEGER_BLOB::default();

        unsafe {
            let success = CryptProtectData(
                &mut input_blob,
                None,
                None,
                None,
                None,
                CRYPTPROTECT_UI_FORBIDDEN,
                &mut output_blob,
            );
            if success.is_ok() {
                let encrypted_slice = std::slice::from_raw_parts(output_blob.pbData, output_blob.cbData as usize);
                fs::write(&key_path, encrypted_slice)
                    .map_err(|e| format!("Failed to write encrypted key file: {}", e))?;
                let _ = LocalFree(HLOCAL(output_blob.pbData as _));
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Read;

    #[test]
    fn test_key_creation_and_derivation() {
        let temp_dir = std::env::temp_dir();
        let unique_id = uuid::Uuid::new_v4().to_string();
        let test_dir = temp_dir.join(format!("scriberx_test_key_{}", unique_id));
        fs::create_dir_all(&test_dir).unwrap();

        // 1. Verify key file creation
        let key = get_or_create_db_key(Some(&test_dir)).unwrap();
        assert!(!key.is_empty(), "Key should not be empty");

        let key_file_path = test_dir.join("scriberx.key");
        assert!(key_file_path.exists(), "Key file scriberx.key should be created");

        // 2. Verify key derivation reads the same key on subsequent calls
        let read_key = get_or_create_db_key(Some(&test_dir)).unwrap();
        assert_eq!(key, read_key, "Subsequent calls should return the same key");

        // Clean up
        let _ = fs::remove_dir_all(&test_dir);
    }

    #[test]
    fn test_sqlite_sqlcipher_encryption() {
        let temp_dir = std::env::temp_dir();
        let unique_id = uuid::Uuid::new_v4().to_string();
        let test_dir = temp_dir.join(format!("scriberx_test_db_{}", unique_id));
        fs::create_dir_all(&test_dir).unwrap();

        let db_path = test_dir.join("test_secure.db");
        let db_path_str = db_path.to_str().unwrap();

        // 1. Initialize storage and write data
        {
            let storage = SqliteStorageEngine::new(db_path_str).unwrap();
            let entry = AuditLogEntry {
                timestamp: 1625097600,
                emr_app_name: "Test EMR".to_string(),
                injected_text: "Prescribed 50mg medicine".to_string(),
                had_low_confidence: false,
            };
            storage.log_audit_entry(&entry).unwrap();

            // Verify we can read it back
            let logs = storage.get_recent_audit_logs(10).unwrap();
            assert_eq!(logs.len(), 1);
            assert_eq!(logs[0].emr_app_name, "Test EMR");
            assert_eq!(logs[0].injected_text, "Prescribed 50mg medicine");
            assert_eq!(logs[0].had_low_confidence, false);
        } // Connection is closed here

        // 2. Verify that the SQLite file is indeed encrypted
        let mut file = fs::File::open(&db_path).unwrap();
        let mut header = [0u8; 16];
        file.read_exact(&mut header).unwrap();

        let sqlite_signature = b"SQLite format 3\0";
        assert_ne!(
            &header,
            sqlite_signature,
            "Database file must be encrypted and should not start with the SQLite signature"
        );

        // Clean up
        let _ = fs::remove_dir_all(&test_dir);
    }

    #[test]
    fn test_bad_key_failure() {
        let temp_dir = std::env::temp_dir();
        let unique_id = uuid::Uuid::new_v4().to_string();
        let test_dir = temp_dir.join(format!("scriberx_test_bad_key_{}", unique_id));
        fs::create_dir_all(&test_dir).unwrap();

        let db_path = test_dir.join("test_bad_key.db");
        let db_path_str = db_path.to_str().unwrap();

        // 1. Create database and write some data
        {
            let storage = SqliteStorageEngine::new(db_path_str).unwrap();
            let entry = AuditLogEntry {
                timestamp: 1625097600,
                emr_app_name: "Test EMR".to_string(),
                injected_text: "Secure data".to_string(),
                had_low_confidence: true,
            };
            storage.log_audit_entry(&entry).unwrap();
        } // Closed

        // 2. Modify the key file to contain a bad key
        let key_file_path = test_dir.join("scriberx.key");
        assert!(key_file_path.exists());

        // Overwrite it with a wrong key
        fs::write(&key_file_path, b"completely_different_and_wrong_key_value").unwrap();

        // 3. Attempt to open the database again. It should fail to initialize due to bad key.
        let result = SqliteStorageEngine::new(db_path_str);
        assert!(
            result.is_err(),
            "Opening the database with a modified/bad key must return an error"
        );

        // Clean up
        let _ = fs::remove_dir_all(&test_dir);
    }
}

