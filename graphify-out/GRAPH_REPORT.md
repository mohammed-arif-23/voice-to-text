# Graph Report - /Users/mohammedarif/voice-to-text  (2026-06-30)

## Corpus Check
- Corpus is ~7,287 words - fits in a single context window. You may not need a graph.

## Summary
- 59 nodes · 83 edges · 16 communities (7 shown, 9 thin omitted)
- Extraction: 90% EXTRACTED · 10% INFERRED · 0% AMBIGUOUS · INFERRED: 8 edges (avg confidence: 0.9)
- Token cost: 0 input · 0 output

## Community Hubs (Navigation)
- [[_COMMUNITY_Core App Architecture|Core App Architecture]]
- [[_COMMUNITY_E2E Testing Track|E2E Testing Track]]
- [[_COMMUNITY_Build System and Interfaces|Build System and Interfaces]]
- [[_COMMUNITY_Targeting and Insertion|Targeting and Insertion]]
- [[_COMMUNITY_Agent Orchestration|Agent Orchestration]]
- [[_COMMUNITY_E2E Sub-Orchestrator|E2E Sub-Orchestrator]]
- [[_COMMUNITY_M1 Foundations Sub-Orch|M1 Foundations Sub-Orch]]
- [[_COMMUNITY_E2E Explorer Briefing|E2E Explorer Briefing]]
- [[_COMMUNITY_E2E Explorer Request|E2E Explorer Request]]
- [[_COMMUNITY_E2E Explorer Progress|E2E Explorer Progress]]
- [[_COMMUNITY_E2E Explorer Readme|E2E Explorer Readme]]
- [[_COMMUNITY_Orchestrator Request|Orchestrator Request]]
- [[_COMMUNITY_Orchestrator Readme|Orchestrator Readme]]
- [[_COMMUNITY_E2E Sub-Orch Readme|E2E Sub-Orch Readme]]
- [[_COMMUNITY_M1 Sub-Orch Request|M1 Sub-Orch Request]]
- [[_COMMUNITY_M1 Sub-Orch Readme|M1 Sub-Orch Readme]]

## God Nodes (most connected - your core abstractions)
1. `Universal Dictation Original User Request` - 17 edges
2. `Milestone 1: Solution Foundations & Core Interfaces` - 8 edges
3. `Universal Dictation Development Plan` - 7 edges
4. `Milestone 2: Core Logic & Logging Redaction` - 7 edges
5. `Milestone 3: Audio Capture, Hotkeys & Targeting` - 7 edges
6. `Milestone 4: Transcription Adapters & Reconciler` - 7 edges
7. `E2E Testing Sub-Orchestrator Agent` - 7 edges
8. `Milestone 1 Foundations Sub-Orchestrator Agent` - 7 edges
9. `WASAPI Audio Capture Service` - 5 edges
10. `Deepgram Nova-3 Streaming Adapter` - 5 edges

## Surprising Connections (you probably didn't know these)
- `E2E Testing Sub-Orchestrator Agent` --semantically_similar_to--> `Milestone 1 Foundations Sub-Orchestrator Agent`  [INFERRED] [semantically similar]
  .agents/sub_orch_e2e/BRIEFING.md → .agents/sub_orch_m1_foundations/BRIEFING.md
- `Azure Cognitive Services Speech Adapter` --references--> `ControlPlane.Api (ASP.NET Core Web API)`  [INFERRED]
  .agents/ORIGINAL_REQUEST.md → .agents/orchestrator/PROJECT.md
- `E2E Testing Sub-Orchestrator Agent` --references--> `Explorer Agent for E2E Codebase Investigation`  [EXTRACTED]
  .agents/sub_orch_e2e/BRIEFING.md → .agents/teamwork_preview_explorer_e2e_setup/BRIEFING.md
- `Sentinel Agent Briefing` --references--> `Universal Dictation Original User Request`  [EXTRACTED]
  .agents/BRIEFING.md → .agents/ORIGINAL_REQUEST.md
- `E2E Testing Sub-Orchestrator Agent` --references--> `Universal Dictation Original User Request`  [EXTRACTED]
  .agents/sub_orch_e2e/BRIEFING.md → .agents/ORIGINAL_REQUEST.md

## Import Cycles
- None detected.

## Hyperedges (group relationships)
- **Multi-level Agent Orchestration Hierarchy** — sentinel_agent, project_orchestrator_agent, e2e_sub_orchestrator_agent, m1_sub_orchestrator_agent [EXTRACTED 1.00]
- **Audio Capture to STT Provider Pipeline** — wasapi_audio_capture, deepgram_nova3_adapter, azure_speech_adapter, whisper_offline_adapter, transcript_reconciler [INFERRED 0.95]
- **Safe Text Insertion Pipeline (Target → Validate → Insert → Verify)** — target_context_service, insertion_adapter_chain, voice_command_parser, transcript_reconciler [INFERRED 0.85]

## Communities (16 total, 9 thin omitted)

### Community 0 - "Core App Architecture"
Cohesion: 0.29
Nodes (13): Sentinel Agent Briefing, Universal Dictation Original User Request, Azure Cognitive Services Speech Adapter, ControlPlane.Api (ASP.NET Core Web API), Deepgram Nova-3 Streaming Adapter, Dictation Session State Machine, Serilog Logging Redaction Pipeline, Milestone 2: Core Logic & Logging Redaction (+5 more)

### Community 1 - "E2E Testing Track"
Cohesion: 0.22
Nodes (9): 4-Tier E2E Testing Methodology, E2E Testing Sub-Orchestrator Agent, E2E Testing Track, Milestone 7: Integration & Hardening, Project Orchestrator Briefing, Orchestrator Progress Tracker, Universal Dictation Project Plan (PROJECT.md), TEST_INFRA.md (Test Strategy Document) (+1 more)

### Community 2 - "Build System and Interfaces"
Cohesion: 0.31
Nodes (9): Central Package Management Pattern, Core Port Interfaces (Desktop.Core), Directory.Build.props (Shared Build Settings), Directory.Packages.props (Central Package Management), .NET Solution Structure (UniversalDictation.sln), Explorer Agent for E2E Codebase Investigation, Explorer E2E Setup Handoff Report, Milestone 1 Foundations Sub-Orchestrator Agent (+1 more)

### Community 3 - "Targeting and Insertion"
Cohesion: 0.32
Nodes (8): Global Hotkey Service, Text Insertion Adapter Chain, Milestone 3: Audio Capture, Hotkeys & Targeting, Milestone 5: Safe Text Insertion Adapter Chain, Milestone 6: WPF Desktop UI Overlay & Composition, No-Activate WPF Overlay Window, Universal Dictation Development Plan, Target Context Service

### Community 4 - "Agent Orchestration"
Cohesion: 0.40
Nodes (5): Sentinel Liveness Check Handoff Report, Project Orchestrator Agent (teamwork_preview_orchestrator), Rust/Tauri/ScribeRx Prior Prototype (Removed), Sentinel Agent, Universal Voice-to-Text Desktop Application

### Community 5 - "E2E Sub-Orchestrator"
Cohesion: 0.67
Nodes (3): E2E Testing Sub-Orchestrator Briefing, E2E Sub-Orchestrator Original Request, E2E Sub-Orchestrator Progress

### Community 6 - "M1 Foundations Sub-Orch"
Cohesion: 0.67
Nodes (3): Milestone 1 Sub-Orchestrator Briefing, Milestone 1 Sub-Orchestrator Progress, Milestone 1 Scope Document

## Knowledge Gaps
- **18 isolated node(s):** `Sentinel Agent Briefing`, `Sentinel Liveness Check Handoff Report`, `Project Orchestrator Original Request`, `Universal Dictation Project Plan (PROJECT.md)`, `Orchestrator Directory README` (+13 more)
  These have ≤1 connection - possible missing edges or undocumented components.
- **9 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `Universal Dictation Original User Request` connect `Core App Architecture` to `E2E Testing Track`, `Build System and Interfaces`, `Targeting and Insertion`, `Agent Orchestration`?**
  _High betweenness centrality (0.279) - this node is a cross-community bridge._
- **Why does `E2E Testing Sub-Orchestrator Agent` connect `E2E Testing Track` to `Core App Architecture`, `Build System and Interfaces`, `Agent Orchestration`?**
  _High betweenness centrality (0.132) - this node is a cross-community bridge._
- **Why does `Milestone 1 Foundations Sub-Orchestrator Agent` connect `Build System and Interfaces` to `E2E Testing Track`, `Agent Orchestration`?**
  _High betweenness centrality (0.096) - this node is a cross-community bridge._
- **Are the 2 inferred relationships involving `Milestone 1: Solution Foundations & Core Interfaces` (e.g. with `Core Port Interfaces (Desktop.Core)` and `.NET Solution Structure (UniversalDictation.sln)`) actually correct?**
  _`Milestone 1: Solution Foundations & Core Interfaces` has 2 INFERRED edges - model-reasoned connections that need verification._
- **What connects `Sentinel Agent Briefing`, `Sentinel Liveness Check Handoff Report`, `Project Orchestrator Original Request` to the rest of the system?**
  _19 weakly-connected nodes found - possible documentation gaps or missing edges._