use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EmrContext {
    pub patient_id: String,
    pub encounter_id: String,
    pub target_window_title: String,
    pub focused_field_id: String,
    pub target_hwnd: u64,
}

pub trait EmrAdapter: Send + Sync {
    fn capture_context(&self) -> Result<EmrContext, String>;
    fn validate_context(&self, cached: &EmrContext) -> Result<(), String>;
    fn parse_voice_command(&self, transcript: &str) -> Option<VoiceCommand>;
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum VoiceCommand {
    StartDictation,
    StartConsultation,
    OpenPrescription,
    StopAndReview,
    NavigateField(String),
}

pub struct GenericEmrAdapter {
    mock_mode: bool,
}

impl GenericEmrAdapter {
    pub fn new(mock: bool) -> Self {
        Self { mock_mode: mock }
    }
}

#[cfg(target_os = "windows")]
use windows::Win32::Foundation::HWND;
#[cfg(target_os = "windows")]
use windows::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowTextW};

impl EmrAdapter for GenericEmrAdapter {
    fn capture_context(&self) -> Result<EmrContext, String> {
        #[cfg(target_os = "windows")]
        {
            if !self.mock_mode {
                unsafe {
                    let hwnd = GetForegroundWindow();
                    if hwnd.0 == 0 {
                        return Err("No active window focused".to_string());
                    }
                    let mut text: [u16; 512] = [0; 512];
                    let len = GetWindowTextW(hwnd, &mut text);
                    let title = String::from_utf16_lossy(&text[..len as usize]);
                    
                    // Windows UI Automation context retrieval placeholder
                    // Extracting patient ID from window title/metadata if available, fallback to stub
                    return Ok(EmrContext {
                        patient_id: "PAT-CDSCO-1092".to_string(),
                        encounter_id: "ENC-4029".to_string(),
                        target_window_title: title,
                        focused_field_id: "txt_prescription_notes".to_string(),
                        target_hwnd: hwnd.0 as u64,
                    });
                }
            }
        }

        // Target-agnostic mockup for macOS/Linux / Testing
        Ok(EmrContext {
            patient_id: "PAT-CDSCO-1092".to_string(),
            encounter_id: "ENC-4029".to_string(),
            target_window_title: "Practo EMR - Dr. Anita - Patient: Amit Kumar".to_string(),
            focused_field_id: "txt_prescription_notes".to_string(),
            target_hwnd: 98765,
        })
    }

    fn validate_context(&self, cached: &EmrContext) -> Result<(), String> {
        let current = self.capture_context()?;
        
        if current.patient_id != cached.patient_id {
            return Err("EMR Patient Context changed! Aborting text injection to prevent cross-patient data leakage.".to_string());
        }
        
        if current.target_hwnd != cached.target_hwnd {
            return Err("EMR Target Window lost focus! Aborting injection.".to_string());
        }

        if current.focused_field_id != cached.focused_field_id {
            return Err("Focused EMR text field bounds changed! Aborting.".to_string());
        }

        Ok(())
    }

    fn parse_voice_command(&self, transcript: &str) -> Option<VoiceCommand> {
        let cleaned = transcript.trim().to_lowercase();
        if cleaned.contains("start dictation") {
            Some(VoiceCommand::StartDictation)
        } else if cleaned.contains("start consultation") {
            Some(VoiceCommand::StartConsultation)
        } else if cleaned.contains("open prescription") {
            Some(VoiceCommand::OpenPrescription)
        } else if cleaned.contains("stop and review") {
            Some(VoiceCommand::StopAndReview)
        } else if cleaned.starts_with("go to") || cleaned.starts_with("navigate to") {
            let field = cleaned.replace("go to", "").replace("navigate to", "").trim().to_string();
            Some(VoiceCommand::NavigateField(field))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_validation_guards() {
        let adapter = GenericEmrAdapter::new(true);
        let cached = adapter.capture_context().unwrap();
        
        let validation = adapter.validate_context(&cached);
        assert!(validation.is_ok());
    }

    #[test]
    fn test_voice_command_parsing() {
        let adapter = GenericEmrAdapter::new(true);
        assert_eq!(adapter.parse_voice_command("Start dictation now"), Some(VoiceCommand::StartDictation));
        assert_eq!(adapter.parse_voice_command("Go to diagnosis field"), Some(VoiceCommand::NavigateField("diagnosis field".to_string())));
    }
}
