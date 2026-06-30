# ScribeRx SQLCipher & DPAPI Storage Security Exploration Report

## Executive Summary
This report details the technical analysis and implementation design for migrating the ScribeRx audit log database (`crates/storage`) to an encrypted SQLite database using **SQLCipher** and securing its encryption key using **Windows Data Protection API (DPAPI)**. To support local development and continuous integration on macOS/Linux, a conditional compilation mocking strategy is designed. The proposed solution has zero external runtime dependencies on Windows and macOS, aligning with ScribeRx product guidelines.

---

## 1. SQLCipher Dependency & Configuration Analysis

To enable SQLCipher encryption in `crates/storage`, the SQLite driver must be compiled with SQLCipher support instead of the standard SQLite library. 

### 1.1 `rusqlite` Feature Selection
The `rusqlite` crate (currently at version `0.31` in `crates/storage/Cargo.toml`) provides features for SQLCipher:
- **`sqlcipher`**: Links against a system-installed SQLCipher library. This requires setup of `pkg-config` or `vcpkg` on the build machine and external DLLs on the target system.
- **`bundled-sqlcipher`**: Compiles SQLCipher directly from source and links it statically into the compiled Rust binary. 

For ScribeRx, **`bundled-sqlcipher`** is the required choice. It satisfies the Non-Functional Requirement (NFR) of a **single signed installer with no external runtime dependencies** (PRD Section 8) and ensures compilation works out-of-the-box on development machines.

### 1.2 Cryptographic Providers & Build Configuration
When building `bundled-sqlcipher`, `libsqlite3-sys` (the underlying FFI crate) automatically configures the platform's native cryptographic provider to avoid external OpenSSL dependencies:
- **Windows**: Statically links SQLCipher using the **Windows Cryptography Next Generation (CNG)** API and links against the native `bcrypt.lib`.
- **macOS**: Statically links SQLCipher using the **Apple CommonCrypto** framework and links against the native `Security` framework.
- **Linux**: Links against `libcrypto` (OpenSSL).

This native provider mapping means **no OpenSSL binaries or environment variables are required on Windows or macOS**, avoiding build-time and runtime headaches.

### 1.3 Keying the Database
SQLCipher requires the encryption key to be provided immediately after opening a connection, before any query is executed. In SQLite, this is done via the `PRAGMA key` statement.
For a 256-bit raw key (32 bytes), the key must be passed as a 64-character hex string prefixed with `x'`:
```sql
PRAGMA key = "x'4a6e3c...'";
```
By passing the raw bytes as a hex literal, SQLCipher bypasses its default PBKDF2 key derivation function (which would run 64,000 iterations to derive a key from a passcode), leading to instant database connections and faster app startup.

---

## 2. Windows DPAPI & macOS Mocking Analysis

### 2.1 Windows DPAPI Key Derivation
To avoid storing the database master key in plaintext or prompting the doctor for a password, we utilize the **Windows Data Protection API (DPAPI)**. DPAPI uses the logged-in Windows user's credentials to encrypt and decrypt data.
- **Mechanism**:
  1. If no key file exists, we generate a cryptographically secure 32-byte (256-bit) random key.
  2. We encrypt the 32-byte key using DPAPI's `CryptProtectData` function.
  3. We write the encrypted bytes to a secure key file (e.g., `scriberx_audit.key`).
  4. On subsequent application startups, we read the encrypted key file and decrypt it using DPAPI's `CryptUnprotectData` function to recover the 32-byte raw key.
- **Security Scope**: Since DPAPI is tied to the current OS user and machine, the key file cannot be decrypted by other users, nor can it be decrypted if copied to another machine.

### 2.2 Cross-Platform Mocking Strategy
Since DPAPI is a Windows-only subsystem, compiling it directly on macOS/Linux for development or unit testing will result in compilation failures. We use Rust's conditional compilation (`#[cfg]`) to separate platform-specific logic:
- **Windows (`#[cfg(target_os = "windows")`)**: Integrates DPAPI using the native `windows` crate.
- **macOS/Linux (`#[cfg(not(target_os = "windows"))`)**: Implements a plaintext pass-through mock. It reads and writes the 32-byte key in plaintext, allowing the entire file lifecycle and SQLCipher integration to be tested locally on macOS.

---

## 3. Key Derivation and File Storage Design

### 3.1 Key Lifecycle & Path Directory
In a production environment, writing database files to the current working directory of the application is unsafe, as the application directory (e.g., `C:\Program Files\ScribeRx`) is read-only.
- **Path Resolution**: The database and key files should be stored in the OS-designated local application data directory (`%LOCALAPPDATA%` on Windows, `~/Library/Application Support` on macOS).
- **File Matching**: The key file should be stored alongside the database file (e.g., `%LOCALAPPDATA%/ScribeRx/scriberx_audit.db` and `%LOCALAPPDATA%/ScribeRx/scriberx_audit.key`).

### 3.2 Error Handling & Recovery Procedures
Since DPAPI relies on user credentials, certain events (e.g., domain migration, profile corruption, database copying to another computer) will cause `CryptUnprotectData` to fail.
- **Key Decryption Failure**: If the key file cannot be decrypted, or the SQLCipher key fails to open the database (returning `SQLITE_NOTADB` during schema creation), the database is inaccessible.
- **Recovery Policy**:
  - The storage engine must return an explicit, descriptive error rather than silently deleting the database (to protect audit trail integrity).
  - The host application (`app-shell`) should catch this initialization error and offer a "Safe Reset" option to the user, which deletes the corrupted database and key file and recreates them.

---

## 4. Proposed Implementation Plan & Code Structure

### 4.1 Dependency Modifications (`crates/storage/Cargo.toml`)
Update the storage crate dependencies to add `rand` (for key generation) and `windows` (conditionally for Windows DPAPI):

```toml
[package]
name = "storage"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = { workspace = true }
thiserror = { workspace = true }
# Enable bundled-sqlcipher to compile SQLCipher statically from source
rusqlite = { version = "0.31", features = ["bundled-sqlcipher"] }
# Cryptographically secure random number generation
rand = "0.8"

# Windows-specific dependencies for DPAPI APIs
[target.'cfg(target_os = "windows")'.dependencies]
windows = { version = "0.54.0", features = [
    "Win32_Security_Cryptography",
    "Win32_Foundation",
    "Win32_System_Memory",
] }
```

### 4.2 Code Structure Modifications (`crates/storage/src/lib.rs`)
The following structure replaces the existing SQLite engine in `lib.rs` with SQLCipher + DPAPI support while preserving the `StorageEngine` trait contract.

```rust
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
    /// Initializes a new SqliteStorageEngine using SQLCipher.
    ///
    /// # Arguments
    /// * `db_path` - Path to the SQLite database file.
    /// * `key_path` - Path to the DPAPI-encrypted key file.
    pub fn new(db_path: &str, key_path: &str) -> Result<Self, String> {
        // 1. Retrieve or generate the database encryption key (32 bytes)
        let key_bytes = key_store::get_or_create_key(Path::new(key_path))?;

        // 2. Open the connection
        let conn = Connection::open(db_path).map_err(|e| format!("Failed to open DB: {}", e))?;

        // 3. Set the SQLCipher encryption key
        // Format the 32-byte key as a hex literal PRAGMA key = "x'HEX_KEY'"
        // to bypass PBKDF2 key derivation and use the raw key directly.
        let hex_key: String = key_bytes.iter().map(|b| format!("{:02x}", b)).collect();
        let pragma_query = format!("PRAGMA key = \"x'{}'\"", hex_key);
        
        conn.execute(&pragma_query, [])
            .map_err(|e| format!("Failed to set SQLCipher key: {}", e))?;

        // 4. Initialize schema (serves as key verification)
        // SQLCipher decrypts page-by-page. A bad key will manifest here as an SQLITE_NOTADB (26) error.
        conn.execute(
            "CREATE TABLE IF NOT EXISTS audit_logs (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp INTEGER NOT NULL,
                emr_app_name TEXT NOT NULL,
                injected_text TEXT NOT NULL,
                had_low_confidence INTEGER NOT NULL
            )",
            [],
        ).map_err(|e| format!("Database decryption or initialization failed: {}", e))?;

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

/// Helper module for managing the DPAPI-protected encryption key.
mod key_store {
    use std::fs;
    use std::path::Path;
    use rand::Rng;

    pub fn get_or_create_key(key_path: &Path) -> Result<Vec<u8>, String> {
        if key_path.exists() {
            let encrypted_bytes = fs::read(key_path)
                .map_err(|e| format!("Failed to read key file: {}", e))?;
            decrypt_key(&encrypted_bytes)
        } else {
            // Generate a secure 32-byte key
            let mut plain_key = [0u8; 32];
            rand::thread_rng().fill(&mut plain_key);

            let encrypted_bytes = encrypt_key(&plain_key)?;

            if let Some(parent) = key_path.parent() {
                fs::create_dir_all(parent)
                    .map_err(|e| format!("Failed to create key directory: {}", e))?;
            }

            fs::write(key_path, &encrypted_bytes)
                .map_err(|e| format!("Failed to write key file: {}", e))?;

            Ok(plain_key.to_vec())
        }
    }

    #[cfg(target_os = "windows")]
    fn encrypt_key(plain_key: &[u8]) -> Result<Vec<u8>, String> {
        use std::ptr::null_mut;
        use windows::Win32::Security::Cryptography::{CryptProtectData, CRYPT_INTEGER_BLOB};
        use windows::Win32::Foundation::HLOCAL;
        use windows::Win32::System::Memory::LocalFree;
        use windows::core::PCWSTR;

        let mut input_blob = CRYPT_INTEGER_BLOB {
            cbData: plain_key.len() as u32,
            pbData: plain_key.as_ptr() as *mut u8,
        };

        let mut output_blob = CRYPT_INTEGER_BLOB {
            cbData: 0,
            pbData: null_mut(),
        };

        unsafe {
            let success = CryptProtectData(
                &mut input_blob,
                PCWSTR::null(),
                std::ptr::null(),
                std::ptr::null(),
                std::ptr::null(),
                0,
                &mut output_blob,
            );

            if success.as_bool() {
                let encrypted = std::slice::from_raw_parts(output_blob.pbData, output_blob.cbData as usize).to_vec();
                LocalFree(HLOCAL(output_blob.pbData as isize));
                Ok(encrypted)
            } else {
                Err(format!("DPAPI CryptProtectData failed: {:?}", windows::core::Error::from_win32()))
            }
        }
    }

    #[cfg(target_os = "windows")]
    fn decrypt_key(encrypted_key: &[u8]) -> Result<Vec<u8>, String> {
        use std::ptr::null_mut;
        use windows::Win32::Security::Cryptography::{CryptUnprotectData, CRYPT_INTEGER_BLOB};
        use windows::Win32::Foundation::HLOCAL;
        use windows::Win32::System::Memory::LocalFree;

        let mut input_blob = CRYPT_INTEGER_BLOB {
            cbData: encrypted_key.len() as u32,
            pbData: encrypted_key.as_ptr() as *mut u8,
        };

        let mut output_blob = CRYPT_INTEGER_BLOB {
            cbData: 0,
            pbData: null_mut(),
        };

        unsafe {
            let success = CryptUnprotectData(
                &mut input_blob,
                std::ptr::null_mut(),
                std::ptr::null(),
                std::ptr::null(),
                std::ptr::null(),
                0,
                &mut output_blob,
            );

            if success.as_bool() {
                let decrypted = std::slice::from_raw_parts(output_blob.pbData, output_blob.cbData as usize).to_vec();
                LocalFree(HLOCAL(output_blob.pbData as isize));
                Ok(decrypted)
            } else {
                Err(format!("DPAPI CryptUnprotectData failed: {:?}", windows::core::Error::from_win32()))
            }
        }
    }

    #[cfg(not(target_os = "windows"))]
    fn encrypt_key(plain_key: &[u8]) -> Result<Vec<u8>, String> {
        Ok(plain_key.to_vec()) // Passthrough mock for macOS/Linux testing
    }

    #[cfg(not(target_os = "windows"))]
    fn decrypt_key(encrypted_key: &[u8]) -> Result<Vec<u8>, String> {
        Ok(encrypted_key.to_vec()) // Passthrough mock for macOS/Linux testing
    }
}
```

### 4.3 Integration in `crates/app-shell/src/main.rs`
The main application shell initialization needs to be updated to pass both paths:
```rust
// In main.rs (before Tauri starts, or inside Tauri setup):
let local_data = std::env::var("LOCALAPPDATA")
    .map(std::path::PathBuf::from)
    .unwrap_or_else(|_| std::env::current_dir().unwrap_or_default());

let db_path = local_data.join("ScribeRx").join("scriberx_audit.db");
let key_path = local_data.join("ScribeRx").join("scriberx_audit.key");

let storage = SqliteStorageEngine::new(
    db_path.to_str().unwrap_or("scriberx_audit.db"),
    key_path.to_str().unwrap_or("scriberx_audit.key")
)?;
```

---

## 5. Migration Strategy for Plaintext Databases

If an unencrypted database file exists from a previous installation of ScribeRx, opening it with SQLCipher and a key will trigger an `SQLITE_NOTADB` error. To avoid data loss:

1. **Detection**:
   If `SqliteStorageEngine::new` fails with `Database decryption or initialization failed`, check if the file can be successfully opened as a standard (unencrypted) SQLite connection.
2. **Export Migration**:
   If it is a plaintext database, we perform an in-place migration to SQLCipher:
   ```rust
   // Open plaintext connection
   let conn = Connection::open(db_path)?;
   // Attach new encrypted database
   let attach_query = format!("ATTACH DATABASE '{}' AS encrypted KEY \"x'{}'\"", temp_encrypted_db_path, hex_key);
   conn.execute(&attach_query, [])?;
   // Export data schema and tables
   conn.execute("SELECT sqlcipher_export('encrypted')", [])?;
   // Detach
   conn.execute("DETACH DATABASE encrypted", [])?;
   // Replace the plaintext database file with the newly encrypted one
   std::fs::rename(temp_encrypted_db_path, db_path)?;
   ```
This migration flow guarantees smooth transitions for existing users during software updates.
