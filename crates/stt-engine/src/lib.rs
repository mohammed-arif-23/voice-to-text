use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptionResult {
    pub raw_text: String,
    pub tokens: Vec<TokenConfidence>,
    pub processing_time_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenConfidence {
    pub token: String,
    pub confidence: f32,
    pub start_time_ms: u64,
    pub end_time_ms: u64,
}

#[async_trait]
pub trait SttEngine: Send + Sync {
    async fn load_model(&mut self, model_path: &str) -> Result<(), String>;
    async fn transcribe(&self, audio_pcm: &[f32], sample_rate: u32) -> Result<TranscriptionResult, String>;
}

pub struct DummySttEngine;

#[async_trait]
impl SttEngine for DummySttEngine {
    async fn load_model(&mut self, _model_path: &str) -> Result<(), String> {
        Ok(())
    }

    async fn transcribe(&self, _audio_pcm: &[f32], _sample_rate: u32) -> Result<TranscriptionResult, String> {
        Ok(TranscriptionResult {
            raw_text: "Tab Amlokind 5 mg once daily, Tab Dolo 650 if fever.".to_string(),
            tokens: vec![],
            processing_time_ms: 150,
        })
    }
}
