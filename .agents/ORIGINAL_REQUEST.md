# Original User Request

## Request — 2026-06-30T04:58:59Z

Convert ScribeRx into a production-grade clinical automation platform suitable for hospital deployment in India.

Working directory: /Users/mohammedarif/voice-to-text
Integrity mode: development

## Requirements

### R1. Deep EMR/HMS Integration & Validation
- Implement a pluggable EMR adapter architecture supporting: browser-based EMRs (Practo, Web EMRs) via Extension/JS injection, and native Windows Win32/WPF EMRs via UI Automation.
- Capture, parse, and validate patient context, encounter ID, target application window, and focus field bounds before recording starts, and revalidate immediately before write-back to prevent cross-patient data insertion.
- Parse allowlisted voice commands for navigation, clinical templates, investigations, and prescriptions.

### R2. Scoped Medical Terminology & Safety Engine
- Support layered vocabularies (global medical terminology, hospital-specific formulary, department, specialty, and individual doctor dictionary).
- Support approved learning from clinician corrections, version rollback, import/export, and explainable confidence scores.
- Immutability constraint: Do not silently alter numbers, dosages, units, routes, frequencies, durations, or drug strengths.

### R3. Ambient Consultation Mode & FHIR Compliance
- Implement a visible consent, speaker separation (clinician vs. patient), and session state machine (cancellation, timeout, recovery).
- Generate clinical drafts (History, Examination, Assessment, Plan) where every statement traceably maps back to the corresponding segment of the transcript.
- Produce compliant FHIR R4 resources: Patient, Encounter, Observation, MedicationRequest, ServiceRequest, DocumentReference, Task, AuditEvent, and Provenance.

### R4. Medical Coding & Formulary Safety Integrations
- Implement a replaceable terminology-service interface for ICD-10 and SNOMED editions.
- Implement a drug safety checker verifying allergies, duplicate therapies, interactions, and contraindications against the target patient context, returning clear clinical warning levels without causing alert fatigue.

### R5. Authentication & Database Key Management
- Authenticate user credentials via Windows Hello / DPAPI integration.
- Encrypt SQLite storage at rest via SQLCipher using a key derived from Windows DPAPI (tied to the current OS user).

### R6. On-Device Wake-Word Detection
- Implement a lightweight, on-device wake-word detection crate (`core-wakeword`) integrating with the existing CPAL audio stream.
- Support "Hey ScribeRx" wake-word with minimal CPU overhead, activating the popup window accompanied by an audible tone.
- Implement allowlisted commands: "Start dictation", "Start consultation", "Open prescription", "Stop and review".
- Maintain explicit state machine transition logic: Armed → Wake Detected → Listening → Processing → Review → Injecting.
- Implement strict privacy guardrails: maintain only a short sliding in-memory window of audio before activation, never persist it, provide software mute control, and retain the keyboard shortcut fallback.

## Acceptance Criteria

### Security & Privacy Guardrails
- [ ] PHI/PII is scrubbed and never present in logs, crash reports, or telemetry.
- [ ] Database is fully encrypted via SQLCipher using DPAPI-derived key.
- [ ] Transient audio files are processed entirely in memory, removing any raw on-disk WAV files.
- [ ] Clipboard injection preserves all prior non-text clipboard formats on restoration.
- [ ] No pre-activation audio is written to disk or sent off-device.

### EMR Injection Verification
- [ ] Compatibility harness verifies injection reliability across browser textareas, contenteditable elements, Win32, and WPF mock applications.
- [ ] Injection halts immediately and alerts the clinician if the active patient or field focus changes during processing.

### FHIR & Compliance Logs
- [ ] Audit logs produce compliant, tamper-evident FHIR AuditEvent and Provenance records for every text injection event.

### Wake-Word & State Machine Verification
- [ ] State machine correctly transitions along Armed → Wake Detected → Listening → Processing → Review → Injecting.
- [ ] Wake-word integration triggers window display and audio alert upon validation, keeping CPU overhead under 2% during the idle "Armed" state.
- [ ] Tests verify wake-word performance against ward noise, masks, and Bluetooth microphone profiles.
