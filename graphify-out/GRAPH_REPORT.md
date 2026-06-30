# Graph Report - .  (2026-06-30)

## Corpus Check
- Corpus is ~8,770 words - fits in a single context window. You may not need a graph.

## Summary
- 173 nodes · 281 edges · 13 communities (11 shown, 2 thin omitted)
- Extraction: 99% EXTRACTED · 1% INFERRED · 0% AMBIGUOUS · INFERRED: 2 edges (avg confidence: 0.8)
- Token cost: 0 input · 0 output

## Community Hubs (Navigation)
- [[_COMMUNITY_Audio Capture & Recording|Audio Capture & Recording]]
- [[_COMMUNITY_Windows Hotkey & Text Injection|Windows Hotkey & Text Injection]]
- [[_COMMUNITY_STT subprocess & Whisper integration|STT subprocess & Whisper integration]]
- [[_COMMUNITY_Tauri App Shell & Command Orchestrator|Tauri App Shell & Command Orchestrator]]
- [[_COMMUNITY_Drug Matcher & Dosage Safety Logic|Drug Matcher & Dosage Safety Logic]]
- [[_COMMUNITY_SQLite Encrypted Audit Database|SQLite Encrypted Audit Database]]
- [[_COMMUNITY_System Architecture & Documentation|System Architecture & Documentation]]
- [[_COMMUNITY_Package Management|Package Management]]
- [[_COMMUNITY_Floating Popup UI State Machine|Floating Popup UI State Machine]]
- [[_COMMUNITY_Workspace Agent Configurations|Workspace Agent Configurations]]

## God Nodes (most connected - your core abstractions)
1. `CpalAudioRecorder` - 15 edges
2. `AppState` - 14 edges
3. `WindowsSapiEngine` - 10 edges
4. `ScribeRxUI` - 9 edges
5. `toggle_dictation()` - 8 edges
6. `AdvancedDrugMatcher` - 8 edges
7. `stop_and_process()` - 7 edges
8. `SqliteStorageEngine` - 7 edges
9. `ScribeRx PRD` - 7 edges
10. `confirm_and_inject()` - 6 edges

## Surprising Connections (you probably didn't know these)
- `Floating Popup HTML UI` --implements--> `ScribeRx Design System`  [EXTRACTED]
  ui/index.html → docs/DESIGN_SYSTEM.md
- `get_clipboard_text()` --calls--> `HWND`  [INFERRED]
  crates/core-hotkey/src/lib.rs → crates/app-shell/src/main.rs
- `set_clipboard_text()` --calls--> `HWND`  [INFERRED]
  crates/core-hotkey/src/lib.rs → crates/app-shell/src/main.rs
- `ScribeRx Architecture` --references--> `ScribeRx PRD`  [EXTRACTED]
  docs/ARCHITECTURE.md → docs/PRD.md
- `ScribeRx Status Board` --references--> `ScribeRx PRD`  [EXTRACTED]
  docs/STATUS.md → docs/PRD.md

## Import Cycles
- 1-file cycle: `crates/core-audio/src/lib.rs -> crates/core-audio/src/lib.rs`
- 1-file cycle: `crates/storage/src/lib.rs -> crates/storage/src/lib.rs`
- 1-file cycle: `crates/stt-engine/src/lib.rs -> crates/stt-engine/src/lib.rs`

## Communities (13 total, 2 thin omitted)

### Community 0 - "Audio Capture & Recording"
Cohesion: 0.14
Nodes (15): AtomicBool, AtomicU32, Arc, Mutex, Option, Result, Self, Send (+7 more)

### Community 1 - "Windows Hotkey & Text Injection"
Cohesion: 0.16
Nodes (17): Option, Result, Self, Send, String, Sync, HWND, DummyInjector (+9 more)

### Community 2 - "STT subprocess & Whisper integration"
Cohesion: 0.14
Nodes (19): BufReader, ChildStdin, ChildStdout, Mutex, Option, Result, Self, Send (+11 more)

### Community 3 - "Tauri App Shell & Command Orchestrator"
Cohesion: 0.20
Nodes (19): AdvancedDrugMatcher, AppHandle, CpalAudioRecorder, Arc, Mutex, Option, Result, main() (+11 more)

### Community 4 - "Drug Matcher & Dosage Safety Logic"
Cohesion: 0.18
Nodes (13): Option, Self, Send, String, Sync, Vec, AdvancedDrugMatcher, ConfidenceLevel (+5 more)

### Community 5 - "SQLite Encrypted Audit Database"
Cohesion: 0.23
Nodes (12): Connection, Mutex, Result, Self, Send, String, Sync, Vec (+4 more)

### Community 6 - "System Architecture & Documentation"
Cohesion: 0.18
Nodes (11): Calm Precision Design Language, Dosage Safety & Immutability, Fuzzy & Phonetic Drug Matching, On-Device Privacy & DPDP Compliance, EMR Caret Text Injection, ScribeRx Architecture, ScribeRx Design System, ScribeRx PRD (+3 more)

### Community 7 - "Package Management"
Cohesion: 0.20
Nodes (9): dependencies, @tauri-apps/api, description, devDependencies, @tauri-apps/cli, name, scripts, tauri (+1 more)

## Knowledge Gaps
- **48 isolated node(s):** `Mutex`, `CpalAudioRecorder`, `WindowsSapiEngine`, `AdvancedDrugMatcher`, `WindowsTextInjector` (+43 more)
  These have ≤1 connection - possible missing edges or undocumented components.
- **2 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `AppState` connect `Tauri App Shell & Command Orchestrator` to `Windows Hotkey & Text Injection`?**
  _High betweenness centrality (0.042) - this node is a cross-community bridge._
- **Why does `HWND` connect `Windows Hotkey & Text Injection` to `Tauri App Shell & Command Orchestrator`?**
  _High betweenness centrality (0.034) - this node is a cross-community bridge._
- **What connects `Mutex`, `CpalAudioRecorder`, `WindowsSapiEngine` to the rest of the system?**
  _48 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `Audio Capture & Recording` be split into smaller, more focused modules?**
  _Cohesion score 0.13846153846153847 - nodes in this community are weakly interconnected._
- **Should `STT subprocess & Whisper integration` be split into smaller, more focused modules?**
  _Cohesion score 0.13666666666666666 - nodes in this community are weakly interconnected._