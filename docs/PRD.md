# ScribeRx — Product Requirements Document
**Voice-to-EMR Dictation Assistant for Windows**

| | |
|---|---|
| Version | 1.0 |
| Status | Draft for build kickoff |
| Platform | Windows 10/11 (x64) |
| Owner | Dev |

---

## 1. Problem Statement

Doctors lose consultation time and attention typing notes and medicine names directly into HMS/EMR systems. Existing dictation tools either require switching applications, don't understand Indian drug nomenclature, or are cloud-only (privacy risk). ScribeRx sits invisibly behind any EMR, is summoned by a hotkey while the doctor's cursor is already in the target field, transcribes spoken clinical notes/prescriptions with high drug-name accuracy, and injects clean text directly into that field — without the doctor ever leaving their EMR.

## 2. Goals

- G1: Reduce per-consultation documentation time by ≥40%.
- G2: ≥95% accuracy on structured drug name + dosage dictation (the safety-critical metric).
- G3: Works against any EMR (native Win32, WPF, Electron, or browser-based) without per-EMR custom integration.
- G4: Runs smoothly on a 5-year-old dual-core/4GB RAM clinic PC.
- G5: Patient audio/text never leaves the device by default (privacy-first, DPDP Act 2023 aligned).

## 3. Non-Goals (v1)

- Not a full EMR replacement or note-structuring AI assistant (no auto-diagnosis, no clinical decision support).
- No mobile app in v1 (Windows desktop only).
- No multi-speaker diarization (single-speaker, doctor-only dictation assumed).
- No live streaming captions — push-to-talk / pause-and-transcribe model only in v1.

## 4. Primary Persona

**Dr. Anita, General Physician, Tier-2 city clinic.** Sees 40–60 patients/day, uses a browser-based EMR, types prescriptions with two fingers, frequently prescribes from a personal list of ~80 drugs, uses a wired earphone mic, runs a 4-year-old desktop PC also running the EMR + WhatsApp Web + billing software simultaneously.

## 5. Success Metrics (post-launch tracking)

| Metric | Target |
|---|---|
| Drug-name word error rate (WER) | < 5% |
| General dictation WER | < 10% |
| Hotkey → popup appear latency | < 100 ms |
| 5-sec utterance → injected text latency | < 3 sec on min-spec device |
| App idle RAM | < 30 MB |
| Crash-free sessions | > 99.5% |
| Doctor-reported time saved/consult | ≥ 1.5 min |

---

## 6. End-to-End User Workflow

1. Doctor opens EMR (any app/browser), clicks into the notes or Rx input field.
2. Doctor presses global hotkey (default `Ctrl+Alt+Space`, remappable).
3. A small floating popup appears near the cursor with a listening indicator (waveform).
4. Doctor speaks naturally: *"Tab Amlokind 5 mg once daily, Tab Dolo 650 if fever."*
5. Doctor presses hotkey again (or pauses ≥1.2s) to stop capture.
6. Popup shows the draft transcript for ≤1 second with confidence highlighting:
 - Plain text = high confidence, auto-accepted.
 - Underlined = medium confidence, accepted but flagged.
 - Yellow chip with 2–3 alternatives = low confidence, requires one click/tap to resolve.
7. On confirmation (auto after 1s if nothing flagged, or manual click), text is injected verbatim into the originally focused field, formatted (line breaks, dosage structure).
8. Popup auto-dismisses. Focus returns to the EMR field exactly where it was.
9. One-keystroke undo (`Ctrl+Z` inside the EMR field works normally since text was inserted as real keystrokes/paste).
10. Entry is logged locally (encrypted) for audit/undo history — no raw audio retained.

---

## 7. Functional Requirements

### 7.1 Global Hotkey Module
- Register configurable global hotkey, conflict-detection against common EMR shortcuts.
- Toggle mode (press to start/stop) and hold mode (push-to-talk) — user-selectable.
- Must work regardless of which window/app has focus.

### 7.2 Floating Popup UI
- Appears anchored near text cursor (caret position via UI Automation where available, else near mouse/active window).
- States: idle → listening → processing → review → injected/dismissed.
- Always-on-top, frameless, click-through outside its bounds.
- Keyboard-only operable (doctors shouldn't need a mouse).

### 7.3 Audio Capture
- Supports default mic, USB headset mic, and Bluetooth headset mic (handle BT codec/profile switching, e.g., A2DP→HFP switch on mic activation).
- Voice activity detection to auto-trim silence at start/end.
- Visual feedback for input level (so doctor knows mic is live).

### 7.4 Speech-to-Text Engine
- On-device by default; cloud STT as an explicit opt-in fallback only.
- Quantized medical-tuned model (see Tech Stack).
- Custom vocabulary biasing using the doctor's personal drug list + master drug DB.

### 7.5 Medical Term Correction Layer
- NER-style flagging of drug-shaped tokens in raw transcript.
- Fuzzy + phonetic match against local CDSCO/NLEM drug database.
- Confidence scoring per detected drug + dosage; numeric dosages get extra scrutiny (never silently "corrected").
- Per-doctor adaptive vocabulary (boost frequently prescribed drugs).

### 7.6 Text Injection Engine
- Primary: `SendInput` synthetic keystrokes.
- Fallback: clipboard set + simulated `Ctrl+V` (restores prior clipboard contents after, to avoid clobbering doctor's clipboard).
- UI Automation `TextPattern.SetValue` as a third path for apps that support it cleanly (most reliable, fewest side effects).
- Must preserve the field's native undo stack (so EMR's own `Ctrl+Z` works).

### 7.7 Settings
- Hotkey remap, mic device selection, language (English / Tamil-English code-switch), personal drug list import/edit, confidence-threshold tuning, light/dark mode, data retention toggle (audit log on/off), cloud-fallback opt-in toggle.

### 7.8 Local History & Audit
- Encrypted local log: timestamp, EMR app name (not content of other fields), final injected text, confidence flags, whether doctor edited post-injection.
- No raw audio persisted beyond the transcription step (deleted from memory buffer immediately after processing).
- Exportable audit log for medico-legal defensibility, doctor-initiated only.

---

## 8. Non-Functional Requirements

| Category | Requirement |
|---|---|
| Performance | Idle RAM < 30MB; CPU < 1% idle; transcription ≤ real-time×3 on dual-core/4GB target |
| Compatibility | Windows 10 64-bit (1909+) and Windows 11; works against native Win32, WPF, Electron, Chromium-based EMRs |
| Privacy | No audio leaves device by default; DPDP Act 2023-aligned consent + data minimization; encrypted at rest (SQLCipher) |
| Reliability | Graceful degradation if mic/permissions unavailable; never crash the host EMR application |
| Installability | Single signed installer, no external runtime dependency (no .NET/Python required on target machine) |
| Accessibility | Fully keyboard-operable popup; high-contrast mode support |

---

## 9. System Architecture

```
┌─────────────────────────────────────────────────────────┐
│                      ScribeRx.exe                        │
│  ┌───────────────┐   ┌────────────────┐                  │
│  │ Hotkey Listener│──▶│ Popup UI (Tauri)│                 │
│  └───────────────┘   └────────┬───────┘                  │
│                                ▼                          │
│                       ┌────────────────┐                  │
│                       │ Audio Capture  │ (cpal/WASAPI)    │
│                       └────────┬───────┘                  │
│                                ▼                          │
│                  ┌─────────────────────────┐              │
│                  │ STT Engine (whisper.cpp) │             │
│                  └────────────┬────────────┘              │
│                                ▼                          │
│              ┌────────────────────────────────┐           │
│              │ Drug Match + Confidence Scorer │            │
│              │  (Rust, local SQLite drug DB)  │            │
│              └────────────────┬───────────────┘           │
│                                ▼                          │
│                    ┌────────────────────┐                 │
│                    │ Text Injection Core │                │
│                    └─────────┬──────────┘                 │
│                                ▼                          │
│                    [ Target EMR input field ]             │
│                                                            │
│   ┌─────────────────────────────────────────────────┐     │
│   │ Local Encrypted Store (SQLite + SQLCipher)       │     │
│   │  — audit log, per-doctor vocab, settings         │     │
│   └─────────────────────────────────────────────────┘     │
└─────────────────────────────────────────────────────────┘
```

**Rust workspace layout**:

```
/scriberx
 ├── crates/
 │    ├── core-hotkey/        # global hotkey + injection (windows-rs)
 │    ├── core-audio/         # cpal/WASAPI capture, VAD
 │    ├── stt-engine/         # whisper.cpp bindings, model loading
 │    ├── drug-match/         # fuzzy/phonetic matcher, confidence scoring
 │    ├── drug-db-builder/    # offline tool: CDSCO/NLEM ingestion → SQLite
 │    ├── storage/            # SQLite + SQLCipher wrapper, audit log
 │    └── app-shell/          # Tauri app, ties everything together
 ├── ui/                      # frontend (HTML/CSS/TS) for Tauri webview
 ├── docs/
 │    ├── PRD.md              # this document
 │    ├── ARCHITECTURE.md     # module contracts (source of truth for agents)
 │    ├── STATUS.md           # live task board
 │    └── DESIGN_SYSTEM.md    # UI tokens (Section 11)
 └── scripts/                 # fine-tuning, benchmarking, packaging
```
