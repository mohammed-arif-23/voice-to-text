# plan.md — Project Orchestration Plan for ScribeRx

This document defines the execution strategy and milestone dispatches for ScribeRx.

## Parallel Track Strategy

We run two parallel tracks:
1. **E2E Testing Track**: Designs a comprehensive requirement-driven test suite. Output: `TEST_READY.md` and complete E2E tests in a separate module/harness.
2. **Implementation Track**: Iteratively builds the core modules and integrates them, concluding with passing 100% of the E2E tests and undergoing adversarial hardening.

---

## E2E Testing Track Milestones
- **E2E-M1: Test Infrastructure & Mock EMR Setup**
  - Create a command-line test runner or web mock interface representing the EMR client.
  - Setup validation harness supporting textareas, contenteditable elements, and Win32/WPF inputs.
- **E2E-M2: Tier 1 & Tier 2 Test Cases**
  - Implement Tier 1 (Feature Coverage): at least 5 test cases per feature (R1-R6, including wake-word detection).
  - Implement Tier 2 (Boundary & Corner Cases): at least 5 test cases per feature (including invalid state transitions, ward noise, and mask interference for wake-word).
- **E2E-M3: Tier 3 & Tier 4 Test Cases**
  - Implement Tier 3 (Cross-Feature Combinations): pairwise coverage (e.g., wake-word triggered during target window context validation).
  - Implement Tier 4 (Real-World Application Scenarios): complete clinical consult workflows using verbal triggers.
  - Publish `TEST_READY.md`.

---

## Implementation Track Milestones

### Milestone 1 (M1): Security & Storage Foundation
- **Goal**: Implement Windows Hello/DPAPI credential auth and SQLCipher SQLite encryption.
- **Worker Task**:
  - Update `crates/storage` to use SQLCipher instead of plain SQLite.
  - Derive database encryption key using Windows DPAPI (tied to current OS user).
  - Mock DPAPI / Windows Hello key derivation logic if compiled/tested on macOS.
- **Verification**: Run unit tests in `storage` to verify database creates, write/reads encrypted records, and fails when using an incorrect key.

### Milestone 2 (M2): EMR Adapter & Validation
- **Goal**: Implement pluggable browser & Win32/WPF EMR adapters with active window focus re-validation.
- **Worker Task**:
  - Implement `EmrAdapter` trait.
  - Implement browser adapter (simulate via CLI focus/browser focus detection) and Win32/WPF adapter via `windows-rs` / UI Automation.
  - Implement pre-flight context checks (patient context, encounter ID, target window, focus fields) and post-flight validation before text write-back.
  - Integrate allowlisted voice commands (navigation, clinical templates, investigations).
- **Verification**: Verify that injection halts and warns user if the target window changes between transcription starting and ending.

### Milestone 3 (M3): Medical Terminology & Safety Engine
- **Goal**: Implement layered vocabularies, CDSCO/NLEM drug safety checking, and strict dosing immutability.
- **Worker Task**:
  - Update `crates/drug-match` to load a SQLite database of NLEM/CDSCO drugs.
  - Add layered dictionary support: doctor, department, specialty, global.
  - Implement drug safety checker interface (verifying duplicate therapies, interactions, allergies).
  - Enforce absolute immutability on numeric dosages (throw warnings on fuzzy alterations).
- **Verification**: Check confidence levels in `drug-match` unit tests; verify no autocorrection is done on numeric dosages.

### Milestone 4 (M4): Ambient Consultation & FHIR Compliance
- **Goal**: Implement speaker separation, SOAP clinical drafts, transcript tracing, and FHIR resource generation.
- **Worker Task**:
  - Integrate speaker separation markers (clinician vs. patient).
  - Generate structured SOAP notes (History, Examination, Assessment, Plan) where text traces back to raw transcript segments.
  - Generate compliant FHIR R4 JSON payloads: Patient, Encounter, Observation, MedicationRequest, ServiceRequest, DocumentReference, Task, AuditEvent, and Provenance.
  - Log events as FHIR AuditEvent and Provenance records in the encrypted storage.
- **Verification**: Validate output JSONs against FHIR schemas.

### Milestone 5 (M5): Floating Popup UI & Dual-Track Integration
- **Goal**: Connect Tauri commands, render Siri-like UI, verify 100% of E2E tests, and execute Phase 2 Adversarial Hardening.
- **Worker Task**:
  - Hook UI consent checkbox, speaker separation toggle, and confidence corrections panel.
  - Integrate all backend crates with `crates/app-shell`.
  - Pass all Tier 1-4 E2E tests.
  - Execute Phase 2 (Adversarial Coverage Hardening) with Challengers finding white-box coverage gaps.
- **Verification**: Full manual and automated E2E run. Forensic Auditor integrity verification.

### Milestone 6 (M6): Wake-Word Detection
- **Goal**: Implement a lightweight, on-device wake-word detection crate (`core-wakeword`) integrating with the existing CPAL audio stream.
- **Worker Task**:
  - Implement `core-wakeword` crate containing the `WakeWordListener` trait.
  - Support "Hey ScribeRx" wake-word with CPU overhead under 2% during idle "Armed" state.
  - Activate the popup window accompanied by an audible alert tone upon validation.
  - Implement allowlisted verbal commands: "Start dictation", "Start consultation", "Open prescription", "Stop and review".
  - Maintain the explicit state transitions: Armed -> Wake Detected -> Listening -> Processing -> Review -> Injecting.
  - Implement strict privacy guardrails: sliding in-memory window of audio before activation, software mute control, keyboard shortcut fallback.
- **Verification**: Verify state transitions and low CPU usage (<2%) under simulated ward noise and Bluetooth microphone profiles.
