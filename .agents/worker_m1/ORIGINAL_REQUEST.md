## 2026-06-30T04:56:46Z
You are a worker for ScribeRx Milestone 1 (Security & Storage).
Your working directory is `/Users/mohammedarif/voice-to-text/.agents/worker_m1`.
Your task is to implement SQLCipher encryption and Windows DPAPI key derivation in `crates/storage`.
You MUST follow the trait contract `StorageEngine` in `crates/storage/src/lib.rs`.

Here is the aggregated plan based on Explorers 2 & 3:
1. Update `crates/storage/Cargo.toml`:
   - Change `rusqlite` dependency feature to `bundled-sqlcipher` instead of `bundled`.
   - Add `rand = "0.8"` for key generation.
   - Add target-conditional dependency for `windows` crate on Windows:
     ```toml
     [target.'cfg(target_os = "windows")'.dependencies]
     windows = { version = "0.54.0", features = [
         "Win32_Security_Cryptography",
         "Win32_Foundation",
         "Win32_System_Memory"
     ] }
     ```
2. Update `crates/storage/src/lib.rs`:
   - Keep the existing `StorageEngine` and `AuditLogEntry` definitions.
   - Implement `new(db_path: &str) -> Result<Self, String>`: Resolves/creates DPAPI-wrapped key, opens connection, sets SQLCipher key via `pragma_update(None, "key", format!("x'{}'", hex_key))`, runs schema verification by executing `SELECT count(*) FROM sqlite_master`.
   - Implement `new_with_key(db_path: &str, key: &[u8]) -> Result<Self, String>`: Accepts raw 32-byte key, sets SQLCipher key, runs verification.
   - Implement key management helper:
     - On Windows (`#[cfg(target_os = "windows")]`): Check if `<db_path>.key` exists. If yes, decrypt it using DPAPI `CryptUnprotectData` (with `CRYPTPROTECT_UI_FORBIDDEN` flag). If not, generate 32 random bytes, encrypt via DPAPI `CryptProtectData`, write to `<db_path>.key`.
     - On other platforms (`#[cfg(not(target_os = "windows"))]`): Implement a passthrough mock that reads/writes the raw 32-byte key in plaintext to `<db_path>.key`.
   - Add unit tests verifying:
     - Decryption fails when opening the database with an incorrect key.
     - Automatic key generation, persistence, and subsequent successful decrypt works as expected.
3. Verify your implementation by running `cargo test --package storage` and `cargo test --workspace` on the current machine (macOS). Ensure everything builds and tests pass cleanly.

MANDATORY INTEGRITY WARNING:
> DO NOT CHEAT. All implementations must be genuine. DO NOT
> hardcode test results, create dummy/facade implementations, or
> circumvent the intended task. A Forensic Auditor will independently
> verify your work. Integrity violations WILL be detected and your
> work WILL be rejected.
