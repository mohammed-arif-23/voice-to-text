use core_audio::{AudioRecorder, DummyAudioRecorder};
use core_hotkey::{DummyInjector, TextInjector};
use drug_match::{AdvancedDrugMatcher, DrugMatcher};
use core_wakeword::{WakeWordDetector, PorcupineWakeWordDetector, WakeWordState};
use emr_adapter::{EmrAdapter, GenericEmrAdapter};
use clinical_safety::{SafetyEngine, WarningLevel, AmbientProcessor};
use medical_coding::{MedicalCoder, GenericMedicalCoder};
use storage::{AuditLogEntry, DummyStorageEngine};

#[test]
fn test_clinical_integration_flow() {
    // 1. Setup mock components
    let mut audio = DummyAudioRecorder::new();
    let matcher = AdvancedDrugMatcher::new();
    let injector = DummyInjector;
    let emr_adapter = GenericEmrAdapter::new(true);
    let safety = SafetyEngine::new(&["Penicillin".to_string()], &["Dolo 650".to_string()]);
    let coder = GenericMedicalCoder::new();
    let mut wakeword = PorcupineWakeWordDetector::new();

    // 2. State Machine Validation (R6)
    assert_eq!(wakeword.get_state(), WakeWordState::Armed);
    wakeword.set_state(WakeWordState::WakeDetected);
    assert_eq!(wakeword.get_state(), WakeWordState::WakeDetected);

    // 3. EMR Validation Validation (R1)
    let cached_context = emr_adapter.capture_context().unwrap();
    assert_eq!(cached_context.patient_id, "PAT-CDSCO-1092");
    assert!(emr_adapter.validate_context(&cached_context).is_ok());

    // 4. Audio Capture Validation
    assert!(audio.start_recording().is_ok());
    let samples = audio.stop_recording().unwrap();
    assert!(!samples.is_empty());

    // 5. Speech Correction & Safety Validation (R2, R5)
    let res = matcher.match_text("Tab Penicillin 250 mg once daily");
    assert_eq!(res.terms[0].matched_name.as_deref(), Some("Penicillin"));
    
    let alerts = safety.check_medication("Penicillin V", "250 mg");
    assert_eq!(alerts[0].warning_level, WarningLevel::Blocking); // Patient is allergic

    // 6. Ambient Consultation mapping (R3)
    let dialogue = "Doctor: We will prescribe Penicillin for the infection.\nPatient: I am fine with it.";
    let soap = AmbientProcessor::process_dialogue(dialogue);
    assert!(soap.traces.get("history").is_none()); // No cough symptoms
    assert!(soap.traces.get("plan").is_some()); // Plan details match

    // 7. Medical Coding recommendations (R4)
    let suggestions = coder.suggest_codes("Patient exhibits severe cough and body pain.");
    assert_eq!(suggestions[0].code, "J06.9");
}
