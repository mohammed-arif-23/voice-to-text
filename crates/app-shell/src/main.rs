use core_audio::{AudioRecorder, CpalAudioRecorder};
use core_hotkey::{HotkeyListener, InjectionStrategy, TextInjector, WindowsTextInjector};
use drug_match::{AdvancedDrugMatcher, DrugMatcher};
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
    matcher: AdvancedDrugMatcher,
    injector: WindowsTextInjector,
    storage: SqliteStorageEngine,
    active_child: Mutex<Option<tokio::task::JoinHandle<()>>>,
    target_window: Mutex<Option<windows::Win32::Foundation::HWND>>,
}

#[tauri::command]
async fn confirm_and_inject(
    text: String,
    state: tauri::State<'_, Arc<AppState>>,
) -> Result<(), String> {
    println!("Confirming and injecting text: \"{}\"", text);

    // Inject text using ClipboardPaste
    state.injector.inject(&text, InjectionStrategy::ClipboardPaste)?;

    // Log to SQLite audit database
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let log_entry = AuditLogEntry {
        timestamp,
        emr_app_name: "Active Window".to_string(),
        injected_text: text,
        had_low_confidence: false,
    };
    let _ = state.storage.log_audit_entry(&log_entry);

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

    // Kill any active VAD tracking loop
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
            // Start recording
            *recording_guard = true;
            println!("Starting dictation...");

            // Capture currently focused active window HWND before showing the Tauri window
            let active_hwnd = unsafe { windows::Win32::UI::WindowsAndMessaging::GetForegroundWindow() };
            if active_hwnd.0 != 0 {
                *state.target_window.lock().unwrap() = Some(active_hwnd);
            } else {
                *state.target_window.lock().unwrap() = None;
            }

            let mut audio = state.audio.lock().unwrap();
            audio.start_recording()?;

            // Position window around cursor/caret before showing it
            let (x, y) = state.injector.capture_caret_position().unwrap_or((100, 100));
            let x_pos = x - 190; // Center horizontally (width 380)
            let y_pos = y + 25;  // Position slightly below cursor
            let _ = main_window.set_position(tauri::Position::Physical(tauri::PhysicalPosition {
                x: x_pos,
                y: y_pos,
            }));

            // Show window always on top
            main_window.show().unwrap();

            // Emit listening state to frontend
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

                    // Emit live mic level to animate Siri waveform
                    let _ = app_handle_clone.emit_all("mic-level", rms);

                    if rms > 0.003 {
                        if !has_spoken {
                            println!("Speech detected!");
                            has_spoken = true;
                        }
                        silence_start = None;
                    } else if has_spoken {
                        // Speech was active, now checking silence pause duration
                        if silence_start.is_none() {
                            silence_start = Some(Instant::now());
                        } else if silence_start.unwrap().elapsed() >= std::time::Duration::from_secs(2) {
                            println!("VAD: 2-second pause detected. Auto-stopping...");
                            let _ = stop_and_process(&app_handle_clone, &state_clone).await;
                            break;
                        }
                    } else {
                        // Initial silence timeout (if user doesn't start speaking)
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
    }; // recording_guard lock dropped here

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

        // Terminate VAD task
        let mut active = state.active_child.lock().unwrap();
        if let Some(handle) = active.take() {
            handle.abort();
        }

        state.audio.lock().unwrap().stop_recording()?
    };
    println!("Stopping dictation & transcribing...");
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

    // Call persistent Whisper daemon
    let res = state.stt.transcribe(&pcm, 16000).await?;
    if res.raw_text.is_empty() {
        main_window.hide().unwrap();
        main_window
            .emit("state-change", serde_json::json!({ "state": "idle" }))
            .unwrap();
        return Ok(());
    }

    println!("Recognized: \"{}\"", res.raw_text);

    // Apply clinical corrections
    let corrected = state.matcher.match_text(&res.raw_text);
    println!("Corrected : \"{}\"", corrected.formatted_text);

    // Hide window and reset UI state directly to idle first
    main_window.hide().unwrap();
    main_window
        .emit("state-change", serde_json::json!({ "state": "idle" }))
        .unwrap();

    // Explicitly restore foreground focus to the captured target window
    if let Some(target_hwnd) = *state.target_window.lock().unwrap() {
        unsafe {
            let _ = windows::Win32::UI::WindowsAndMessaging::SetForegroundWindow(target_hwnd);
        }
    }

    // Small delay to ensure focus restoration propagates in OS thread window manager
    tokio::time::sleep(std::time::Duration::from_millis(250)).await;

    // Inject immediately into the restored focus active caret field
    state.injector.inject(&corrected.formatted_text, InjectionStrategy::ClipboardPaste)?;

    // Log to SQLite audit database
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let log_entry = AuditLogEntry {
        timestamp,
        emr_app_name: "Active Window".to_string(),
        injected_text: corrected.formatted_text,
        had_low_confidence: false,
    };
    let _ = state.storage.log_audit_entry(&log_entry);

    Ok(())
}

fn main() -> Result<(), String> {
    println!("ScribeRx Desktop Shell Initializing...");

    // Initialize all components
    let audio = Mutex::new(CpalAudioRecorder::new());
    let mut stt = WindowsSapiEngine::new();
    
    // Spawn the persistent Whisper Python daemon
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .map_err(|e| e.to_string())?;

    rt.block_on(stt.load_model(""))?;

    let matcher = AdvancedDrugMatcher::new();
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
    });

    // Create channel for global hotkey thread
    let (tx, mut rx) = mpsc::channel::<()>(10);

    // Spawn Win32 hotkey listener thread
    std::thread::spawn(move || {
        let hotkey = HotkeyListener::new();
        if let Err(e) = hotkey.register() {
            eprintln!("Hotkey registration failed on thread: {}", e);
            return;
        }
        println!("Hotkey listener thread running successfully (Ctrl+Alt+F9).");
        while hotkey.wait_for_hotkey() {
            if let Err(e) = tx.blocking_send(()) {
                eprintln!("Failed to send hotkey event: {}", e);
                break;
            }
        }
    });

    // Start Tauri runner
    let state_clone = Arc::clone(&state);
    
    tauri::Builder::default()
        .manage(state)
        .invoke_handler(tauri::generate_handler![confirm_and_inject, cancel_dictation])
        .setup(move |app| {
            let app_handle = app.handle();
            let state_inner = Arc::clone(&state_clone);

            // Spawn task to listen to hotkeys and toggle dictation
            tauri::async_runtime::spawn(async move {
                while let Some(_) = rx.recv().await {
                    println!("\n[Hotkey Event Received]");
                    let _ = toggle_dictation(&app_handle, &state_inner).await;
                }
            });

            // Listen to terminal fallback Enter presses
            let app_handle_enter = app.handle();
            let state_enter = Arc::clone(&state_clone);
            tauri::async_runtime::spawn(async move {
                let mut stdin_lines = BufReader::new(io::stdin()).lines();
                while let Ok(Some(_)) = stdin_lines.next_line().await {
                    println!("\n[Terminal Enter Event Received]");
                    let _ = toggle_dictation(&app_handle_enter, &state_enter).await;
                }
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running ScribeRx tauri application");

    Ok(())
}
