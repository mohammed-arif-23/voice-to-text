# ScribeRx Storage Encrypted SQLCipher and DPAPI Analysis Report

## Summary
This report analyzes the migration of `crates/storage` from plain SQLite to SQLCipher encrypted SQLite, utilizing Windows DPAPI for user-tied key derivation, and a macOS mock fallback for local testing and cross-platform compatibility.

---

## 1. SQLCipher Integration & Dependencies

### 1.1 Cargo.toml Requirements
To support SQLCipher, we must update the `Cargo.toml` of `crates/storage`.
* **SQLCipher compilation**: Change the feature on `rusqlite` from `bundled` to `bundled-sqlcipher`. This instructs `libsqlite3-sys` to compile SQLCipher statically instead of standard SQLite.
* **OpenSSL Portability**: SQLCipher requires a cryptographic backend for its core algorithms (like AES-256). By default, it builds against OpenSSL. To prevent compilation errors on developer machines (especially Windows) due to missing OpenSSL libraries, we must add the `openssl` dependency with the `vendored` feature enabled. This downloads and builds OpenSSL statically during cargo compilation.
* **Windows API bindings**: We target DPAPI functions only on Windows using target-specific dependencies.
* **Entropy Source**: The `rand` crate is used for cryptographically secure database key generation.
* **Testing Harness**: The `tempfile` crate (already in cargo cache) is added as a dev-dependency to test key validation on temp databases.

### 1.2 SQLCipher Initialization Flow
After opening a connection via `Connection::open(db_path)`, we must immediately apply the encryption key before executing any queries:
1. **Passphrase vs. Raw Key**: By default, SQLCipher derives a key from a string passphrase using PBKDF2. To skip the overhead of PBKDF2 and use our high-entropy 256-bit DPAPI-derived key directly, we format the key as a hex-encoded string prefixed with `x'`, yielding: `PRAGMA key = "x'a1b2c3d4...'"`.
2. **First Query Verification**: SQLCipher does not verify key correctness during the `PRAGMA key` execution. The key is only validated when database page reads/writes are first performed. Therefore, immediately after setting the key, we must query `sqlite_master` (e.g. `SELECT count(*) FROM sqlite_master`). If this fails, the key is incorrect or the database file is corrupt.

---

## 2. Windows DPAPI Key Derivation

### 2.1 API Selection
We leverage the native Windows Data Protection API (DPAPI) via the `windows` crate (version `0.54.0`).
* **Functions**:
  * `windows::Win32::Security::Cryptography::CryptProtectData` (encrypts bytes using current OS user credentials).
  * `windows::Win32::Security::Cryptography::CryptUnprotectData` (decrypts bytes).
  * `windows::Win32::System::Memory::LocalFree` (releases memory allocated by the DPAPI calls).
* **Flags**:
  * We use `CRYPTPROTECT_UI_FORBIDDEN` (value `0x1`) to ensure no visual prompts interrupt the application execution.

### 2.2 Key Wrapping Flow
Rather than deriving a key from static strings, we generate a random 256-bit (32-byte) key on the first run, encrypt it via DPAPI, and save it as `<db_path>.key`. On subsequent runs, we read `<db_path>.key` and decrypt it:
1. Check if `<db_path>.key` exists.
2. **If it exists**: Read the file content, run it through `CryptUnprotectData` to decrypt it, and return the 32-byte raw key.
3. **If it does not exist**: Generate 32 cryptographically secure random bytes using `rand::thread_rng().fill_bytes()`. Encrypt the bytes using `CryptProtectData`. Write the ciphertext to `<db_path>.key` and return the 32-byte raw key.

This ensures the database key is tied to the current OS user and cannot be decrypted by other users or on different machines.

---

## 3. macOS Mocking Design

For development and automated testing on macOS:
* We define a conditional compilation module `crypto` using `#[cfg(target_os = "windows")]` and `#[cfg(not(target_os = "windows"))]`.
* On macOS/Linux, the mock implementation does not perform actual DPAPI encryption. Instead, it reads/writes the raw 32-byte key directly in plain-text to the `<db_path>.key` file.
* This maintains the stateful behavior of key persistence (checking file existence, reading, and writing) without depending on Windows-specific APIs.

---

## 4. Implementation Plan & Code Structure

### 4.1 Cargo.toml Update Proposal
```toml
[package]
name = "storage"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = { workspace = true }
thiserror = { workspace = true }
# Use bundled-sqlcipher to build with SQLCipher support
rusqlite = { version = "0.31", features = ["bundled-sqlcipher"] }
# Static vendored OpenSSL to make compilation cross-platform and portable
openssl = { version = "0.10", features = ["vendored"] }
rand = "0.8"

[target.'cfg(target_os = "windows")'.dependencies]
windows = { version = "0.54.0", features = [
    "Win32_Security_Cryptography",
    "Win32_Foundation",
    "Win32_System_Memory"
] }

[dev-dependencies]
tempfile = "3.27.0"
```

### 4.2 lib.rs Design Proposal

```rust
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
    /// Opens the SQLCipher database, deriving the key automatically via DPAPI (or macOS mock)
    pub fn new(db_path: &str) -> Result<Self, String> {
        let key = crypto::get_or_create_key(db_path)?;
        Self::new_with_key(db_path, &key)
    }

    /// Opens the SQLCipher database using an explicit raw key (useful for test verification)
    pub fn new_with_key(db_path: &str, key: &[u8]) -> Result<Self, String> {
        let conn = Connection::open(db_path).map_err(|e| e.to_string())?;

        // 1. Convert key to hex string format for SQLCipher PRAGMA
        let key_hex: String = key.iter().map(|b| format!("{:02x}", b)).collect();
        let pragma_val = format!("x'{key_hex}'");

        // 2. Set SQLCipher key
        conn.pragma_update(None, "key", pragma_val).map_err(|e| e.to_string())?;

        // 3. Verify key validity by querying sqlite_master
        conn.execute("SELECT count(*) FROM sqlite_master", []).map_err(|e| {
            format!("Invalid decryption key or database corrupted: {}", e)
        })?;

        // 4. Initialize schema
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

#[cfg(target_os = "windows")]
mod crypto {
    use std::ptr;
    use std::fs;
    use std::path::Path;
    use windows::Win32::Foundation::{LocalFree, HLOCAL};
    use windows::Win32::Security::Cryptography::{
        CryptProtectData, CryptUnprotectData, CRYPTOAPI_BLOB, CRYPTPROTECT_UI_FORBIDDEN
    };
    use rand::RngCore;

    fn encrypt(data: &[u8]) -> Result<Vec<u8>, String> {
        let mut input_blob = CRYPTOAPI_BLOB {
            cbData: data.len() as u32,
            pbData: data.as_ptr() as *mut u8,
        };
        let mut output_blob = CRYPTOAPI_BLOB {
            cbData: 0,
            pbData: ptr::null_mut(),
        };

        let success = unsafe {
            CryptProtectData(
                &input_blob,
                None,
                ptr::null(),
                ptr::null_mut(),
                ptr::null(),
                CRYPTPROTECT_UI_FORBIDDEN,
                &mut output_blob,
            )
        };

        if success.as_bool() {
            let encrypted = unsafe {
                std::slice::from_raw_parts(output_blob.pbData, output_blob.cbData as usize).to_vec()
            };
            unsafe {
                let _ = LocalFree(HLOCAL(output_blob.pbData as *mut _));
            }
            Ok(encrypted)
        } else {
            Err(format!("DPAPI encrypt failed: {:?}", std::io::Error::last_os_error()))
        }
    }

    fn decrypt(data: &[u8]) -> Result<Vec<u8>, String> {
        let mut input_blob = CRYPTOAPI_BLOB {
            cbData: data.len() as u32,
            pbData: data.as_ptr() as *mut u8,
        };
        let mut output_blob = CRYPTOAPI_BLOB {
            cbData: 0,
            pbData: ptr::null_mut(),
        };

        let success = unsafe {
            CryptUnprotectData(
                &input_blob,
                ptr::null_mut(),
                ptr::null(),
                ptr::null_mut(),
                ptr::null(),
                CRYPTPROTECT_UI_FORBIDDEN,
                &mut output_blob,
            )
        };

        if success.as_bool() {
            let decrypted = unsafe {
                std::slice::from_raw_parts(output_blob.pbData, output_blob.cbData as usize).to_vec()
            };
            unsafe {
                let _ = LocalFree(HLOCAL(output_blob.pbData as *mut _));
            }
            Ok(decrypted)
        } else {
            Err(format!("DPAPI decrypt failed: {:?}", std::io::Error::last_os_error()))
        }
    }

    pub fn get_or_create_key(db_path: &str) -> Result<Vec<u8>, String> {
        let key_path = format!("{}.key", db_path);
        let key_path = Path::new(&key_path);

        if key_path.exists() {
            let encrypted = fs::read(key_path)
                .map_err(|e| format!("Failed to read key file: {}", e))?;
            let decrypted = decrypt(&encrypted)?;
            if decrypted.len() != 32 {
                return Err("Decrypted key must be exactly 32 bytes".to_string());
            }
            Ok(decrypted)
        } else {
            let mut key = vec![0u8; 32];
            rand::thread_rng().fill_bytes(&mut key);
            let encrypted = encrypt(&key)?;
            fs::write(key_path, encrypted)
                .map_err(|e| format!("Failed to write key file: {}", e))?;
            Ok(key)
        }
    }
}

#[cfg(not(target_os = "windows"))]
mod crypto {
    use std::fs;
    use std::path::Path;
    use rand::RngCore;

    pub fn get_or_create_key(db_path: &str) -> Result<Vec<u8>, String> {
        let key_path = format!("{}.key", db_path);
        let key_path = Path::new(&key_path);

        if key_path.exists() {
            let key = fs::read(key_path)
                .map_err(|e| format!("Failed to read mock key file: {}", e))?;
            if key.len() != 32 {
                return Err("Mock key must be exactly 32 bytes".to_string());
            }
            Ok(key)
        } else {
            let mut key = vec![0u8; 32];
            rand::thread_rng().fill_bytes(&mut key);
            fs::write(key_path, &key)
                .map_err(|e| format!("Failed to write mock key file: {}", e))?;
            Ok(key)
        }
    }
}
```

### 4.3 Unit Testing Verification Strategy
To verify the implementation compiles and functions correctly, we add standard integration tests. By invoking `new_with_key`, we can verify the SQLCipher encryption behaves as expected (refusing incorrect keys):

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_encryption_and_decryption_failures() {
        let dir = tempdir().expect("Failed to create temp dir");
        let db_path = dir.path().join("test_audit.db");
        let db_path_str = db_path.to_str().unwrap();

        let key_a = [1u8; 32];
        let key_b = [2u8; 32];

        // 1. Create database and write entry using Key A
        {
            let engine = SqliteStorageEngine::new_with_key(db_path_str, &key_a)
                .expect("Failed to initialize database with Key A");
            let entry = AuditLogEntry {
                timestamp: 123456,
                emr_app_name: "Test EMR".to_string(),
                injected_text: "Amoxicillin 500mg".to_string(),
                had_low_confidence: false,
            };
            engine.log_audit_entry(&entry).expect("Failed to write audit entry");
        }

        // 2. Try to open the database using Key B (should fail)
        {
            let open_result = SqliteStorageEngine::new_with_key(db_path_str, &key_b);
            assert!(
                open_result.is_err(),
                "Opening the database with an incorrect key should fail"
            );
        }

        // 3. Open the database using Key A (should succeed and retrieve the entry)
        {
            let engine = SqliteStorageEngine::new_with_key(db_path_str, &key_a)
                .expect("Failed to open database with correct Key A");
            let logs = engine.get_recent_audit_logs(10).expect("Failed to query audit logs");
            assert_eq!(logs.len(), 1);
            assert_eq!(logs[0].timestamp, 123456);
            assert_eq!(logs[0].emr_app_name, "Test EMR");
            assert_eq!(logs[0].injected_text, "Amoxicillin 500mg");
            assert_eq!(logs[0].had_low_confidence, false);
        }
    }

    #[test]
    fn test_automatic_key_generation_and_persistence() {
        let dir = tempdir().expect("Failed to create temp dir");
        let db_path = dir.path().join("test_auto.db");
        let db_path_str = db_path.to_str().unwrap();

        // 1. Open database (first run, should generate key file automatically)
        {
            let engine = SqliteStorageEngine::new(db_path_str).expect("Failed to open db first time");
            let entry = AuditLogEntry {
                timestamp: 987654,
                emr_app_name: "Auto App".to_string(),
                injected_text: "Paracetamol 650mg".to_string(),
                had_low_confidence: true,
            };
            engine.log_audit_entry(&entry).expect("Failed to write audit entry");
        }

        // Verify key file was created
        let key_file_path = format!("{}.key", db_path_str);
        assert!(std::path::Path::new(&key_file_path).exists(), "Key file should be generated");

        // 2. Open database again (second run, should read existing key and decrypt successfully)
        {
            let engine = SqliteStorageEngine::new(db_path_str).expect("Failed to open db second time");
            let logs = engine.get_recent_audit_logs(10).expect("Failed to query audit logs");
            assert_eq!(logs.len(), 1);
            assert_eq!(logs[0].timestamp, 987654);
            assert_eq!(logs[0].injected_text, "Paracetamol 650mg");
            assert_eq!(logs[0].had_low_confidence, true);
        }
    }
}
```
