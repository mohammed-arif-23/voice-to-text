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

Run unit tests across all workspace crates (`drug-match`, `core-audio`, `stt-engine`, `core-hotkey`, `storage`):

```cmd
cargo test --workspace
```

### Expected Output
- `drug-match`: Verifies Levenshtein distance calculation, dosage extraction isolation (e.g. "500 mg"), and confidence classification.
- `core-audio`: Verifies RMS audio level feedback calculation.
- `stt-engine`: Verifies mock transcription payload structure.

---

## 3. Validating Text Injection & EMR Compatibility

To verify text injection against host applications without crashing or clobbering native undo buffers:

### Test Case A: Win32 Application (Notepad)
1. Launch `app-shell`:
   ```cmd
   cargo run --package app-shell
   ```
2. Focus Notepad cursor inside an empty text area.
3. Trigger dictation hotkey (`Ctrl+Alt+Space`).
4. **Verification**: Text `"Tab Amlokind 5 mg once daily, Tab Dolo 650 if fever."` appears inside Notepad. Press `Ctrl+Z` to verify Notepad's native undo stack removes the injected text cleanly.

### Test Case B: Chromium Browser EMR
1. Open Chrome/Edge and navigate to any web-based EHR input form.
2. Click into a `<textarea>` or contenteditable field.
3. Trigger dictation hotkey (`Ctrl+Alt+Space`).
4. **Verification**: Text injects accurately without lost characters or skipped dosage numbers.

---

## 4. Performance & Resource Benchmarking

Target metrics defined in PRD Section 5:

| Metric | Target | Verification Method |
|---|---|---|
| Idle RAM | < 30 MB | Open Windows Task Manager / Process Hacker, monitor `app-shell.exe` memory working set while idle. |
| Latency | < 3 sec on 5-sec audio | Measure timestamp from audio stop to text insertion in console logs. |
| General WER | < 10% | Run `scripts/benchmark` against test audio fixtures. |
