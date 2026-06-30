# Scope: Milestone 1: Foundations

## Architecture
- Root Directory containing C# Solution and solution-level files: `UniversalDictation.sln`, `global.json`, `Directory.Build.props`, `Directory.Packages.props`, `.editorconfig`, `.gitignore`.
- Subdirectories:
  - `src/`: Production C# projects (e.g. Desktop.Core and others)
  - `tests/`: Test C# projects

## Milestones
| # | Name | Scope | Dependencies | Status |
|---|------|-------|-------------|--------|
| 1 | Infrastructure | Create Solution, Projects, .editorconfig, and .gitignore. Keep global.json, Directory.Build.props, Directory.Packages.props. | None | FAILED (Integrity Violation) |
| 2 | Core Ports & Domain | Implement interfaces, domain types, and error types in Desktop.Core. | Infrastructure | FAILED (Integrity Violation) |
| 3 | Verification | Compile in Release and verify zero warnings/analyzer errors. Run challenger & auditor checks. | Core Ports & Domain | FAILED (Integrity Violation) |

## Interface Contracts
- All interfaces and types in `Desktop.Core` must use namespace `Desktop.Core`.
- All exception classes must extend `DictationException` and include a `DiagnosticCode`.
