# ScribeRx — Testing & Validation Guide
**Instructions for testing ScribeRx after transferring repository to Windows**

This document provides step-by-step procedures to build, test, and validate ScribeRx across all crates and target EMR applications on Windows 10/11.

---

## 1. Environment Prerequisites (Windows 10/11 x64)

Ensure the target Windows development machine has the following tools installed:

1. **Rust toolchain** (MSVC target):
   ```cmd
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   rustup default stable-x86_64-pc-windows-msvc
   ```
2. **C++ Build Tools** (Visual Studio 2022 Community with "Desktop development with C++" workload for compiling `whisper.cpp` C bindings).
3. **Node.js 18+** & `npm` (for Tauri frontend bundling).
4. **Tauri CLI**:
   ```cmd
   cargo install tauri-cli
   ```

---

## 2. Workspace Automated Test Suite

Run unit and E2E integration tests across all workspace crates:

```cmd
cargo test --workspace
```

### Expected Output
- `drug-match`: Verifies Levenshtein matching, scoped medical vocabularies (specialty, doctor, hospital), corrections learning logic, and dosage immutability checks.
- `core-audio`: Verifies RMS calculation, channel downmix, and sample rate conversion.
- `core-wakeword`: Verifies state machine transitions (Armed → Wake Detected → Listening → Processing → Review → Injecting) and microphone software mute control.
- `emr-adapter`: Verifies EMR window, active patient context validation, and voice command parser.
- `clinical-safety`: Verifies drug allergy warnings, formulary check alerts, and SOAP note trace mappings.
- `medical-coding`: Verifies SNOMED-CT / ICD-10 suggestions and audit logs.
- `app-shell`: Verifies E2E clinical workflow simulation.

---

## 3. Validating Text Injection & EMR Compatibility

To verify text injection against host applications without crashing or clobbering native undo buffers:

### Test Case A: Win32 Application (Notepad)
1. Launch `app-shell`:
   ```cmd
   cargo run --package app-shell
   ```
2. Focus Notepad cursor inside an empty text area.
3. Trigger dictation hotkey (`Ctrl+Alt+Space`) or speak "Hey ScribeRx" (if wake-word listening is enabled).
4. **Verification**: Text appears inside Notepad. Press `Ctrl+Z` to verify Notepad's native undo stack removes the injected text cleanly.

### Test Case B: Chromium Browser EMR
1. Open Chrome/Edge and navigate to any web-based EHR input form.
2. Click into a `<textarea>` or contenteditable field.
3. Trigger dictation hotkey.
4. **Verification**: Text injects accurately without lost characters or skipped dosage numbers.

---

## 4. Performance & Resource Benchmarking

Target metrics defined in PRD Section 5:

| Metric | Target | Verification Method |
|---|---|---|
| Idle RAM | < 30 MB | Open Windows Task Manager / Process Hacker, monitor `app-shell.exe` memory working set while idle. |
| Latency | < 3 sec on 5-sec audio | Measure timestamp from audio stop to text insertion in console logs. |
| Wake-Word Idle CPU | < 2% | Monitor CPU utilization during "Armed" wake-word listening state. |
| General WER | < 10% | Run `scripts/benchmark` against test audio fixtures. |
