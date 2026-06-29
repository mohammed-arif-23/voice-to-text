use serde::{Deserialize, Serialize};

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
