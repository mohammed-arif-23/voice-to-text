## 2026-06-30T11:40:00+05:30
You are the E2E Test Implementer for ScribeRx.
Your working directory is `/Users/mohammedarif/voice-to-text/.agents/teamwork_preview_worker_e2e_2`.
Please perform the following tasks:

1. Locate the cargo/rustup installation in the system. Check if cargo is located in `/Users/mohammedarif/.cargo/bin/cargo` or similar. Verify that running it (using the absolute path if necessary) works.
2. Implement cross-platform compilation support so the ScribeRx workspace compiles and runs on macOS.
   - Gating the 'windows' dependency in Cargo.toml files for core-hotkey and app-shell (e.g. [target.'cfg(target_os = "windows")'.dependencies] windows = ...).
   - Gating all Windows-specific code blocks in `crates/core-hotkey/src/lib.rs`, `crates/core-audio/src/lib.rs`, `crates/stt-engine/src/lib.rs`, `crates/storage/src/lib.rs`, and `crates/app-shell/src/main.rs` using #[cfg(target_os = "windows")] and providing clean fallback/mock implementations for macOS/Linux using #[cfg(not(target_os = "windows"))].
3. Integrate the R6 Wake-Word Detection requirements:
   - Define a `WakeWordEngine` trait with states/transitions (Armed -> Wake Detected -> Listening -> Processing -> Review -> Injecting).
   - Implement `MockWakeWordEngine` verifying CPU overhead stays <2% in Armed state, wake-word validation triggers capsule display and audio alerts, robustness against ward noise/masks, and privacy guardrail (no audio written to disk before activation).
4. Create an E2E Mock Test Harness & Test Suite (either as a new crate `crates/e2e-tests` or using Rust integration tests/mock commands bridge in `app-shell`):
   - Design and code the test runner.
   - Implement the 30+ test cases from the E2E Test Explorer's plan covering:
     - Tier 1 Feature Coverage (Wake-word R6, Hotkey, Audio & VAD, STT, Drug Matcher, Text Injection, Storage)
     - Tier 2 Boundaries (Noise tolerances, stolen focus, clipping, decimal dosages, clipboard lock)
     - Tier 3 Cross-Feature combinations (Pairwise combinations)
     - Tier 4 Real-World scenarios (Dr. Anita's workflow, manual correction, background battery constraint)
5. Ensure the entire workspace compiles successfully on macOS by running the gated build command: e.g. `<cargo_path> check --workspace --all-targets`.
6. Run the test suite and verify that all E2E and unit tests pass.
7. Write the `TEST_INFRA.md` and `TEST_READY.md` files at the project root (`/Users/mohammedarif/voice-to-text/`) detailing:
   - Feature inventory and test philosophy.
   - Test runner command and invocation details.
   - Detailed list of test cases across all Tiers (1-4) with pass status.
   - Traceability of R6 Wake-Word Detection.
8. MANDATORY INTEGRITY WARNING: DO NOT CHEAT. All implementations must be genuine. DO NOT hardcode test results, create dummy/facade implementations, or circumvent the intended task. A Forensic Auditor will independently verify your work. Integrity violations WILL be detected and your work WILL be rejected.
9. Report back with a summary of the files modified/created, test run command, and verification results.
