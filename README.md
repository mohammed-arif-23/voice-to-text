# ScribeRx — Voice-to-EMR Dictation Assistant

**ScribeRx** is an open-source, privacy-first, on-device voice dictation assistant designed specifically for physicians and healthcare professionals typing into electronic medical record (EMR/HMS) systems on Windows.

---

## Key Features

- **Invisible & EMR-Agnostic**: Works with any Windows EMR (Win32, WPF, Electron, browser-based) using global hotkey summoning (`Ctrl+Alt+Space`) and direct caret text injection.
- **Indian Drug Name Accuracy**: Built-in fuzzy and phonetic matching against CDSCO / NLEM drug databases (e.g., *Amlokind*, *Dolo*, *Glycomet*, *Pantocid*).
- **Safety-Critical Dosage Protection**: Numeric dosage values are strictly protected and never silently auto-corrected.
- **Privacy-First (DPDP Act 2023 Aligned)**: Audio is processed on-device; no raw audio or patient identifiers are ever sent to external clouds or stored beyond the active buffer.
- **Calm Precision Design**: Frameless, minimal floating acrylic popup interface optimized for keyboard-only speed.

---

## Repository Structure

```text
/voice-to-text
 ├── Cargo.toml               # Root Rust workspace configuration
 ├── crates/
 │    ├── core-hotkey/        # Global hotkeys, caret positioning & text injection
 │    ├── core-audio/         # CPAL audio capture stream & RMS VAD
 │    ├── stt-engine/         # whisper.cpp bindings & transcription engine
 │    ├── drug-match/         # Levenshtein/phonetic matcher & dosage verification
 │    ├── drug-db-builder/    # CDSCO/NLEM ingestion utility tool
 │    ├── storage/            # SQLCipher encrypted audit logs & settings
 │    └── app-shell/          # Tauri desktop application shell
 ├── ui/                      # Floating popup frontend (HTML/CSS/TS)
 ├── docs/                    # PRD, Architecture, Design Tokens, Testing Guide
 └── graphify-out/            # Generated codebase knowledge graph
```

---

## Quick Start & Testing

For full build instructions, native Windows dependency installation (WASAPI, `windows-rs`, `whisper.cpp`), and EMR field injection testing, please see the **[Testing Instructions](docs/TESTING_INSTRUCTIONS.md)**.

### Running Workspace Tests
```bash
cargo test --workspace
```
