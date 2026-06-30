# Original User Request

## Initial Request — 2026-06-30T10:18:54+05:30

You are the E2E Testing Track Orchestrator for ScribeRx.
Your working directory is `/Users/mohammedarif/voice-to-text/.agents/e2e_orch`.
Read PROJECT.md, plan.md, and ORIGINAL_REQUEST.md at `/Users/mohammedarif/voice-to-text`.
Your task is to design and implement a comprehensive, opaque-box, requirement-driven E2E test suite for ScribeRx.
Design test cases across all Tiers (Tier 1 Feature Coverage, Tier 2 Boundaries, Tier 3 Cross-feature, Tier 4 Real-world workloads) based on the criteria in PROJECT.md.
Create a test runner / mock harness that enables testing the application shell and injection strategies.
When complete, write TEST_READY.md and handoff your results.
Your parent is `c86685ba-0e81-4f1c-8b80-38b87e48c6f9`. Report progress to your parent.
Keep your BRIEFING.md updated under your working directory.

## Follow-up — 2026-06-30T04:55:39Z

**Context**: Requirement Update: R6 On-Device Wake-Word Detection
**Content**: ScribeRx has been updated with a new requirement R6. You must incorporate R6 into your E2E test plan:
- Verify state transitions: Armed -> Wake Detected -> Listening -> Processing -> Review -> Injecting.
- Verify CPU overhead is kept under 2% during the idle "Armed" state.
- Verify wake-word triggers window display and audio alert upon validation.
- Verify wake-word performance against ward noise, masks, and Bluetooth microphone profiles.
- Verify privacy guardrails: no pre-activation audio is written to disk or sent off-device.
Please update your test infrastructure and test cases to include R6.
**Action**: Update test infrastructure and write test cases for R6.
