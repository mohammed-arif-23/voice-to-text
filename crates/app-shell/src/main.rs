#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use core_audio::{AudioRecorder, CpalAudioRecorder};
use core_hotkey::{HotkeyListener, InjectionStrategy, TextInjector, WindowsTextInjector};
use drug_match::{AdvancedDrugMatcher, DrugMatcher};
use core_wakeword::{WakeWordDetector, PorcupineWakeWordDetector, WakeWordState};
use emr_adapter::{EmrAdapter, GenericEmrAdapter, EmrContext};
use clinical_safety::{SafetyEngine, FhirExporter, AmbientProcessor};
use medical_coding::{MedicalCoder, GenericMedicalCoder, CodingDecisionLog};
use std::sync::{Arc, Mutex};
use std::time::{Instant, SystemTime, UNIX_EPOCH};
use storage::{AuditLogEntry, SqliteStorageEngine, StorageEngine};
use stt_engine::{SttEngine, WindowsSapiEngine};
use tauri::Manager;
use tokio::io::{self, AsyncBufReadExt, BufReader};
use tokio::sync::mpsc;

struct AppState {
    recording: Mutex<bool>,
    audio: Mutex<CpalAudioRecorder>,
    stt: WindowsSapiEngine,
    matcher: Mutex<AdvancedDrugMatcher>,
    injector: WindowsTextInjector,
    storage: SqliteStorageEngine,
    active_child: Mutex<Option<tokio::task::JoinHandle<()>>>,
    target_window: Mutex<Option<windows::Win32::Foundation::HWND>>,
    
    // Core clinical modules
    emr_adapter: GenericEmrAdapter,
    safety_engine: SafetyEngine,
    coder: GenericMedicalCoder,
    wakeword_detector: Mutex<PorcupineWakeWordDetector>,
    cached_context: Mutex<Option<EmrContext>>,
}

#[tauri::command]
async fn confirm_and_inject(
    text: String,
    state: tauri::State<'_, Arc<AppState>>,
) -> Result<(), String> {
    println!("[Orchestrator] Confirming and injecting text: \"{}\"", text);

    // EMR Strategy Step 1: Capture and Revalidate context before write-back (T-01 guard)
    let cached = state.cached_context.lock().unwrap().clone();
    if let Some(ctx) = cached {
        if let Err(e) = state.emr_adapter.validate_context(&ctx) {
            return Err(format!("Focus Validation Failed: {}", e));
        }
    } else {
        return Err("Error: No active EMR context was cached before recording!".to_string());
    }

    // EMR Strategy Step 2: Inject text using EMR-compliant strategy
    state.injector.inject(&text, InjectionStrategy::ClipboardPaste)?;

    // EMR Strategy Step 3: Write FHIR compliant audit event and local logs
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let fhir_audit = FhirExporter::generate_audit_event("doc-anita", "PAT-CDSCO-1092", &text, timestamp);
    println!("[FHIR Event Logged]: {}", serde_json::to_string_pretty(&fhir_audit).unwrap());

    let log_entry = AuditLogEntry {
        timestamp,
        emr_app_name: "Active EMR Window".to_string(),
        injected_text: text,
        had_low_confidence: false,
    };
    state.storage.log_audit_entry(&log_entry)?;

    Ok(())
}

#[tauri::command]
async fn cancel_dictation(
    state: tauri::State<'_, Arc<AppState>>,
) -> Result<(), String> {
    println!("Dictation cancelled.");
    let mut recording = state.recording.lock().unwrap();
    *recording = false;
    let _ = state.audio.lock().unwrap().stop_recording();

    let mut active = state.active_child.lock().unwrap();
    if let Some(handle) = active.take() {
        handle.abort();
    }

    Ok(())
}

async fn toggle_dictation(
    app_handle: &tauri::AppHandle,
    state: &Arc<AppState>,
) -> Result<(), String> {
    let should_stop = {
        let mut recording_guard = state.recording.lock().unwrap();
        let main_window = app_handle.get_window("main").ok_or("Main window not found")?;

        if !*recording_guard {
            // EMR Context capture validation before recording (T-01 guard)
            let current_context = state.emr_adapter.capture_context()?;
            println!("[EMR Adapter] Captured patient context: ID: {}, Encounter: {}", current_context.patient_id, current_context.encounter_id);
            *state.cached_context.lock().unwrap() = Some(current_context.clone());

            // Start recording
            *recording_guard = true;
            println!("Starting dictation...");

            // Capture target window HWND
            #[cfg(target_os = "windows")]
            {
                let active_hwnd = unsafe { windows::Win32::UI::WindowsAndMessaging::GetForegroundWindow() };
                if active_hwnd.0 != 0 {
                    *state.target_window.lock().unwrap() = Some(active_hwnd);
                } else {
                    *state.target_window.lock().unwrap() = None;
                }
            }

            let mut audio = state.audio.lock().unwrap();
            audio.start_recording()?;

            // Position window around cursor/caret before showing it
            let (x, y) = state.injector.capture_caret_position().unwrap_or((100, 100));
            let x_pos = x - 190;
            let y_pos = y + 25;
            let _ = main_window.set_position(tauri::Position::Physical(tauri::PhysicalPosition {
                x: x_pos,
                y: y_pos,
            }));

            main_window.show().unwrap();
            main_window
                .emit("state-change", serde_json::json!({ "state": "listening" }))
                .unwrap();

            // Spawn a background thread to update mic levels and evaluate VAD silences
            let state_clone = Arc::clone(state);
            let app_handle_clone = app_handle.clone();

            let handle = tokio::spawn(async move {
                let start_time = Instant::now();
                let mut has_spoken = false;
                let mut silence_start: Option<Instant> = None;
                let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(50));

                loop {
                    interval.tick().await;

                    let rms = {
                        let audio = state_clone.audio.lock().unwrap();
                        audio.get_current_rms_level()
                    };

                    let _ = app_handle_clone.emit_all("mic-level", rms);

                    if rms > 0.003 {
                        if !has_spoken {
                            has_spoken = true;
                        }
                        silence_start = None;
                    } else if has_spoken {
                        if silence_start.is_none() {
                            silence_start = Some(Instant::now());
                        } else if silence_start.unwrap().elapsed() >= std::time::Duration::from_secs(2) {
                            println!("VAD: 2-second pause detected. Auto-stopping...");
                            let _ = stop_and_process(&app_handle_clone, &state_clone).await;
                            break;
                        }
                    } else {
                        if start_time.elapsed() >= std::time::Duration::from_secs(8) {
                            println!("VAD: 8-second initial silence. Aborting dictation...");
                            let mut rec = state_clone.recording.lock().unwrap();
                            *rec = false;
                            let _ = state_clone.audio.lock().unwrap().stop_recording();
                            let _ = main_window.hide();
                            let _ = main_window.emit("state-change", serde_json::json!({ "state": "idle" }));
                            break;
                        }
                    }
                }
            });

            *state.active_child.lock().unwrap() = Some(handle);
            false
        } else {
            true
        }
    };

    if should_stop {
        let _ = stop_and_process(app_handle, state).await;
    }

    Ok(())
}

async fn stop_and_process(
    app_handle: &tauri::AppHandle,
    state: &Arc<AppState>,
) -> Result<(), String> {
    let pcm = {
        let mut recording_guard = state.recording.lock().unwrap();
        if !*recording_guard {
            return Ok(());
        }
        *recording_guard = false;

        let mut active = state.active_child.lock().unwrap();
        if let Some(handle) = active.take() {
            handle.abort();
        }

        state.audio.lock().unwrap().stop_recording()?
    };

    let main_window = app_handle.get_window("main").ok_or("Main window not found")?;
    main_window
        .emit("state-change", serde_json::json!({ "state": "processing" }))
        .unwrap();

    if pcm.is_empty() {
        main_window.hide().unwrap();
        main_window
            .emit("state-change", serde_json::json!({ "state": "idle" }))
            .unwrap();
        return Ok(());
    }

    let res = state.stt.transcribe(&pcm, 16000).await?;
    if res.raw_text.is_empty() {
        main_window.hide().unwrap();
        main_window
            .emit("state-change", serde_json::json!({ "state": "idle" }))
            .unwrap();
        return Ok(());
    }

    // 1. Terminology correction & Layered vocab lookup
    let corrected = state.matcher.lock().unwrap().match_text(&res.raw_text);

    // 2. Medication safety checks (duplicate therapy, allergies, over-dosage bounds)
    for term in &corrected.terms {
        if let Some(ref matched_name) = term.matched_name {
            let alerts = state.safety_engine.check_medication(matched_name, term.dosage.as_deref().unwrap_or(""));
            for alert in alerts {
                println!("[Medication Safety Warning]: Source: {} - Description: {}", alert.source, alert.description);
            }
        }
    }

    // 3. Ambient consultation processing (SOAP Note traces Generation)
    let ambient_dialogue = format!("Doctor: Let's prescribe Dolo 650 mg thrice daily.\nPatient: Thank you Doctor.");
    let soap_note = AmbientProcessor::process_dialogue(&ambient_dialogue);
    println!("[Ambient Consultation Note Generated]:\nPlan: {}\nTraces: {:?}", soap_note.plan, soap_note.traces);

    // 4. Medical Coding suggestions
    let coding_suggestions = state.coder.suggest_codes(&corrected.formatted_text);
    for suggestion in coding_suggestions {
        println!("[Coding Suggestion]: Code: {} - {} (System: {})", suggestion.code, suggestion.description, suggestion.system);
        
        let decision_log = CodingDecisionLog {
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            clinician_id: "doc-anita".to_string(),
            code: suggestion.code.clone(),
            accepted: true,
            terminology_version: "ICD-10-AM 2026".to_string(),
        };
        let _ = state.coder.log_decision(&decision_log);
    }

    // 5. Context Verification before write-back (T-01)
    let cached = state.cached_context.lock().unwrap().clone();
    if let Some(ref ctx) = cached {
        if let Err(e) = state.emr_adapter.validate_context(ctx) {
            println!("[Context Validation Alert]: Context changed post-processing: {}", e);
            main_window.hide().unwrap();
            main_window.emit("state-change", serde_json::json!({ "state": "idle" })).unwrap();
            return Err(e);
        }
    }

    main_window.hide().unwrap();
    main_window
        .emit("state-change", serde_json::json!({ "state": "idle" }))
        .unwrap();

    // 6. Text injection using caret strategy
    state.injector.inject(&corrected.formatted_text, InjectionStrategy::ClipboardPaste)?;

    // Log transaction to storage
    let log_entry = AuditLogEntry {
        timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
        emr_app_name: "Active EMR Field".to_string(),
        injected_text: corrected.formatted_text,
        had_low_confidence: false,
    };
    let _ = state.storage.log_audit_entry(&log_entry);

    Ok(())
}

fn main() -> Result<(), String> {
    println!("ScribeRx Desktop Shell Initializing...");

    let audio = Mutex::new(CpalAudioRecorder::new());
    let mut stt = WindowsSapiEngine::new();
    
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .map_err(|e| e.to_string())?;

    rt.block_on(stt.load_model(""))?;

    let matcher = Mutex::new(AdvancedDrugMatcher::new());
    let injector = WindowsTextInjector::new();
    let storage = SqliteStorageEngine::new("scriberx_audit.db")?;

    let state = Arc::new(AppState {
        recording: Mutex::new(false),
        audio,
        stt,
        matcher,
        injector,
        storage,
        active_child: Mutex::new(None),
        target_window: Mutex::new(None),
        emr_adapter: GenericEmrAdapter::new(true),
        safety_engine: SafetyEngine::new(&["Penicillin".to_string()], &["Dolo 650".to_string()]),
        coder: GenericMedicalCoder::new(),
        wakeword_detector: Mutex::new(PorcupineWakeWordDetector::new()),
        cached_context: Mutex::new(None),
    });

    let (tx, mut rx) = mpsc::channel::<()>(10);

    // Spawn hotkey listener thread
    std::thread::spawn(move || {
        let hotkey = HotkeyListener::new();
        if let Err(e) = hotkey.register() {
            eprintln!("Hotkey registration failed: {}", e);
            return;
        }
        while hotkey.wait_for_hotkey() {
            if let Err(_) = tx.blocking_send(()) {
                break;
            }
        }
    });

    // Start Tauri App runner
    let state_clone = Arc::clone(&state);
    
    tauri::Builder::default()
        .manage(state)
        .invoke_handler(tauri::generate_handler![confirm_and_inject, cancel_dictation])
        .setup(move |app| {
            let app_handle = app.handle();
            let state_inner = Arc::clone(&state_clone);

            tauri::async_runtime::spawn(async move {
                while let Some(_) = rx.recv().await {
                    let _ = toggle_dictation(&app_handle, &state_inner).await;
                }
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running ScribeRx tauri application");

    Ok(())
}
