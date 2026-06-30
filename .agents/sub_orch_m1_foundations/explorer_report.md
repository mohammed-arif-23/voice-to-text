# Infrastructure Explorer Report: Milestone 1 Foundations

## 1. Executive Summary
This report documents the architectural findings, build requirements, and project configurations for the **Universal Dictation** solution. Based on an analysis of the root directory files (`global.json`, `Directory.Build.props`, `Directory.Packages.props`), repository git history, and design specifications, we define the directory layout, dependency graph, and tooling settings required to establish a production-grade C#/.NET 10 build foundation.

Key accomplishments of this investigation:
- Analyzed and verified central build settings, confirming .NET 10 compilation requirements and central package configurations.
- Uncovered the legacy Rust/Tauri prototype ("ScribeRx") in git history, identifying core architectural lessons and explicit prohibitions.
- Resolved the ambiguity regarding the project count (12 vs. 16 projects), recommending the creation of all 16 projects to satisfy the full requirements of R1.
- Detailed the precise folder structure, SDK types, framework targets, project references, and package dependencies for all 16 projects.
- Designed production-ready configurations for `.editorconfig` and `.gitignore` tailored for .NET 10 and WPF development.

---

## 2. Build System and Central Package Analysis

### 2.1 global.json
The `global.json` file pins the SDK version as follows:
```json
{
  "sdk": {
    "version": "8.0.0",
    "rollForward": "latestFeature"
  }
}
```
*Analysis:* While the pinned SDK is `8.0.0`, the `rollForward` policy is set to `latestFeature`. Since the target framework for the codebase is `net10.0-windows`, a .NET 10 SDK must be installed on the compilation system. The `rollForward` setting ensures that the build system will roll forward to the installed .NET 10 SDK band during compilation.

### 2.2 Directory.Build.props
This file applies global compilation policies to all projects in the solution:
- **Target Framework:** `net10.0-windows` (C# 14 / .NET 10 with Windows Desktop extensions by default).
- **Code Quality Policies:**
  - Nullable reference types are enabled (`<Nullable>enable</Nullable>`).
  - Implicit Usings are enabled (`<ImplicitUsings>enable</ImplicitUsings>`).
  - Warnings are treated as errors (`<TreatWarningsAsErrors>true</TreatWarningsAsErrors>`).
  - Code analysis mode is set to all (`<AnalysisMode>All</AnalysisMode>`), and code styles are enforced during the build (`<EnforceCodeStyleInBuild>true</EnforceCodeStyleInBuild>`).
- **Reproducible Builds:** Enabled via `<Deterministic>true</Deterministic>`.
- **Debugging & Source Link:** Configured with GitHub source link mappings and untracked source embedding.
- **Central NuGet Security Auditing:** Enabled (`<NuGetAudit>true</NuGetAudit>`) with moderate severity triggers and auditing across all transitive dependencies.

### 2.3 Directory.Packages.props
Central Package Management (CPM) is active (`ManagePackageVersionsCentrally: true`). The versions of all NuGet packages are centrally controlled. Individual project files (`.csproj`) must reference packages using `<PackageReference Include="..." />` and **never** include a `Version` attribute.

Key dependencies available for reference:
- **Hosting & DI:** `Microsoft.Extensions.*` packages at version `10.0.0` (aligned with .NET 10).
- **Logging:** `Serilog` (4.1.0) with console and file sinks, and thread enrichers.
- **Audio Capture:** `NAudio` & `NAudio.Wasapi` at version `3.0.0` (supporting WASAPI capture and the new WasapiRecorder API).
- **Transcription Cloud Adapters:** `Deepgram` (4.4.0) and `Microsoft.CognitiveServices.Speech` (1.50.0 - upgraded past the deprecated 1.48.2 threshold to ensure stability before July 1, 2026).
- **Offline Transcription:** `Whisper.net` and `Whisper.net.Runtime` at version `1.9.0` (CPU-only GGUF model execution).
- **Database & Persistence:** `Npgsql.EntityFrameworkCore.PostgreSQL` (10.0.0) and EF Core packages (10.0.0) for the Control Plane.
- **Testing:** `xunit` (2.9.3), `Moq` (4.20.72), `FluentAssertions` (7.0.0), and `Bogus` (35.6.1).

---

## 3. Legacy Analysis (ScribeRx / Rust Prototype)

A forensic examination of the git history (specifically commit `abea0626cf04825ba82363832f5d30659e22391f`) shows that the repository previously housed **ScribeRx**, a multi-crate Rust workspace with a Tauri frontend:
- `crates/app-shell` (Tauri host orchestrating IPC and session states)
- `crates/core-audio` (CPAL-based audio capture)
- `crates/stt-engine` (whisper.cpp engine bindings)
- `crates/drug-match` (NLEM/CDSCO drug dictionary phonetic matcher)
- `crates/core-hotkey` (Win32 global hotkeys and UI Automation caret tracking)
- `crates/storage` (SQLCipher log auditing with DPAPI-derived key)
- `crates/core-wakeword` ("Hey ScribeRx" wake-word engine)

### 3.1 Reusable Concepts
- **Windows API Interop:** Using Win32 APIs for global hotkey hooks (`RegisterHotKey`) and UI Automation (`System.Windows.Automation`) for caret context tracking and text injection.
- **Layered Validation:** Capturing the target application's metadata (ProcessId, HWND, AutomationId, class names) and performing pre-flight check validation prior to text insertion.
- **Local Auditing:** Storing audit trails locally with encryption.

### 3.2 Rejected Patterns & Rationale (Prohibitions)
1. **Blind Clipboard-Only Insertion:** Overwriting the system clipboard without validation destroys user data and risks race conditions. *Resolution:* Implement a prioritized insertion chain: Browser bridge, UI Automation (`ValuePattern`), raw simulated keyboard input (`SendInput`), with clipboard copy-paste only as a final, validated fallback.
2. **Mock EMR Context:** Relying on mock contexts instead of live window analysis causes text to be written into the wrong window if focus shifts. *Resolution:* Read active system window properties and process integrity levels dynamically.
3. **Broad Fuzzy Word Replacement:** Autocorrecting numbers and dosage units can result in dangerous medical dosage changes (e.g., changing "5.0 mg" to "50 mg"). *Resolution:* Enforce absolute immutability on numeric values and dosages.
4. **Hardcoded Paths:** Referencing absolute local user paths prevents multi-user operation and deployment. *Resolution:* Dynamically resolve paths using `Environment.SpecialFolder.LocalApplicationData` or `ApplicationData`.
5. **Raw Transcript Logging:** Logging verbatim spoken audio transcripts, window titles, or clipboard text violates patient privacy (HIPAA/DPDP Act). *Resolution:* Implement a Serilog redaction pipeline using destructuring policies to strip out sensitive data.
6. **Predictable Temporary Audio Files:** Writing raw audio buffer files to disk risks unauthorized access to patient speech. *Resolution:* Hold audio frames strictly in-memory using a bounded ring buffer.
7. **Direct Insertion Without Validation:** Injecting text into fields that have changed focus since dictation began. *Resolution:* Throw `TargetValidationException` if the process ID, window handle, or integrity level has changed between hotkey capture and final insertion.

---

## 4. Resolution of Project Count Ambiguity

### 4.1 The Ambiguity
- **R1 Requirement:** Specifies a list of 13 projects under `src/` and 3 projects under `tests/`, totaling **16 projects**.
- **Milestone 1 Objectives:** Instructs to "Create the Visual Studio solution `UniversalDictation.sln` containing the 12 projects under `src/` and `tests/` as specified in R1".

### 4.2 Reconciling Options
1. **Option A: 12 Projects Total (Client Only + Api + Tests)**
   - Includes: 8 client-side projects (`DesktopApp`, `Desktop.Core`, `Desktop.Audio`, `Desktop.Transcription`, `Desktop.Targeting`, `Desktop.Insertion`, `Desktop.NativeInterop`, `NativeMessagingHost`), `ControlPlane.Api`, and 3 test projects (`Unit`, `Integration`, `Contract`).
   - Total: 12 projects.
   - *Limitation:* Omit 4 control-plane helper libraries (`ControlPlane.Application`, `ControlPlane.Domain`, `ControlPlane.Infrastructure`, `AdminPortal`), violating the core requirement of R1.
2. **Option B: 12 Projects under `src/` + 3 under `tests/` (15 Projects Total)**
   - Excludes only `AdminPortal` under `src/`.
   - Total: 15 projects.
   - *Limitation:* Does not fully satisfy R1's table, which explicitly demands `AdminPortal`.
3. **Option C: All 16 Projects Specified in R1 (Recommended)**
   - Creates all 13 source projects and 3 test projects.
   - *Rationale:* R1 represents the authoritative architectural specification. The mention of "12 projects" in the milestone text is likely a typo or referred to an earlier iteration that excluded the control plane libraries. Creating all 16 projects guarantees that:
     1. Database versions (PostgreSQL, EF Core 10) declared in `Directory.Packages.props` are properly consumed by their target libraries (`ControlPlane.Infrastructure`).
     2. All dependencies are configured with correct central settings at the outset.
     3. Subsequent development milestones can progress without requiring a solution-level structural rewrite.

We recommend implementing **Option C (16 projects)**.

---

## 5. Target Project Layout and Configuration

All 16 projects are organized inside the workspace under the `src/` and `tests/` folders.

| Folder / Project Directory | Project Name | SDK Type | Target Framework | Project References | Primary Packages Used |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **`src/` (Production Projects)** | | | | | |
| `src/DesktopApp` | `DesktopApp` | `Microsoft.NET.Sdk` | `net10.0-windows` | `Desktop.Core`, `Desktop.Audio`, `Desktop.Transcription`, `Desktop.Targeting`, `Desktop.Insertion`, `Desktop.NativeInterop` | WPF App enabled (`<UseWPF>true</UseWPF>`), `Microsoft.Extensions.Hosting`, `Serilog.Sinks.File`, `OpenTelemetry` |
| `src/Desktop.Core` | `Desktop.Core` | `Microsoft.NET.Sdk` | `net10.0-windows` | *None* | `Serilog`, `Microsoft.Extensions.Options`, `OpenTelemetry.Api` |
| `src/Desktop.Audio` | `Desktop.Audio` | `Microsoft.NET.Sdk` | `net10.0-windows` | `Desktop.Core` | `NAudio`, `NAudio.Wasapi` |
| `src/Desktop.Transcription` | `Desktop.Transcription` | `Microsoft.NET.Sdk` | `net10.0-windows` | `Desktop.Core` | `Deepgram`, `Microsoft.CognitiveServices.Speech`, `Whisper.net`, `Whisper.net.Runtime` |
| `src/Desktop.Targeting` | `Desktop.Targeting` | `Microsoft.NET.Sdk` | `net10.0-windows` | `Desktop.Core`, `Desktop.NativeInterop` | `Microsoft.Extensions.Logging.Abstractions` |
| `src/Desktop.Insertion` | `Desktop.Insertion` | `Microsoft.NET.Sdk` | `net10.0-windows` | `Desktop.Core`, `Desktop.NativeInterop` | `Microsoft.Extensions.Logging.Abstractions` |
| `src/Desktop.NativeInterop` | `Desktop.NativeInterop` | `Microsoft.NET.Sdk` | `net10.0-windows` | `Desktop.Core` | `System.Security.Cryptography.Pkcs` |
| `src/NativeMessagingHost` | `NativeMessagingHost` | `Microsoft.NET.Sdk` | `net10.0-windows` | `Desktop.Core` | Console App (`<OutputType>Exe</OutputType>`), `System.Text.Json` |
| `src/ControlPlane.Domain` | `ControlPlane.Domain` | `Microsoft.NET.Sdk` | `net10.0` | *None* | Core entities, domain models |
| `src/ControlPlane.Application`| `ControlPlane.Application` | `Microsoft.NET.Sdk` | `net10.0` | `ControlPlane.Domain` | `Microsoft.Extensions.Logging.Abstractions` |
| `src/ControlPlane.Infrastructure`| `ControlPlane.Infrastructure`| `Microsoft.NET.Sdk` | `net10.0` | `ControlPlane.Application`, `ControlPlane.Domain` | `Microsoft.EntityFrameworkCore.Relational`, `Npgsql.EntityFrameworkCore.PostgreSQL` |
| `src/ControlPlane.Api` | `ControlPlane.Api` | `Microsoft.NET.Sdk.Web` | `net10.0` | `ControlPlane.Infrastructure`, `ControlPlane.Application`, `ControlPlane.Domain` | `Microsoft.AspNetCore.Authentication.JwtBearer`, `Microsoft.AspNetCore.OpenApi`, `Microsoft.EntityFrameworkCore.Design` |
| `src/AdminPortal` | `AdminPortal` | `Microsoft.NET.Sdk.Web` | `net10.0` | `ControlPlane.Infrastructure` (or API client) | Razor Pages App, `Microsoft.Extensions.Hosting` |
| **`tests/` (Test Projects)** | | | | | |
| `tests/Unit` | `Unit` | `Microsoft.NET.Sdk` | `net10.0-windows` | `Desktop.Core`, `Desktop.Audio`, `Desktop.Transcription`, `Desktop.Targeting`, `Desktop.Insertion`, `Desktop.NativeInterop` | `xunit`, `Moq`, `FluentAssertions`, `Bogus`, `Microsoft.NET.Test.Sdk` |
| `tests/Integration` | `Integration` | `Microsoft.NET.Sdk` | `net10.0-windows` | `Desktop.Core`, `Desktop.Audio`, `Desktop.Transcription`, `Desktop.Targeting`, `Desktop.Insertion`, `Desktop.NativeInterop` | `xunit`, `Moq`, `FluentAssertions`, `Microsoft.NET.Test.Sdk` |
| `tests/Contract` | `Contract` | `Microsoft.NET.Sdk` | `net10.0-windows` | `Desktop.Core`, `Desktop.Transcription` | `xunit`, `Moq`, `FluentAssertions`, `Microsoft.NET.Test.Sdk` |

---

## 6. Project Reference Dependency Graph

### 6.1 Desktop Client Stack
```text
                  +-------------------+
                  |    DesktopApp     | (WPF App)
                  +--+---+---+---+--+
                     |   |   |   |
     +---------------+   |   |   +---------------+
     |                   |   |                   |
+----+----+  +-----------+   +-----------+  +----+----+
|Targeting|  |Transcription| |   Audio   |  |Insertion|
+----+----+  +-----+-------+ +-----+-----+  +----+----+
     |             |               |             |
     |             |  +------------+             |
     |             |  |                          |
     |        +----+--+---+                      |
     |        |   Core    | (Contracts/Domain)   |
     |        +----+------+                      |
     |             ^                             |
     +-----+       |       +---------------------+
           |       |       |
      +----+-------+-------+----+
      |      NativeInterop      | (Win32 P/Invoke, DPAPI)
      +-------------------------+
```

### 6.2 Control Plane Stack
```text
           +--------------------+
           |  ControlPlane.Api  | (ASP.NET Web API)
           +---------+----------+
                     |
           +---------v------------------+
           | ControlPlane.Infrastructure|
           +---------+----------+-------+
                     |          |
           +---------v----+     |
           |  Application |     |
           +---------+----+     |
                     |          |
           +---------v----------v+
           |  ControlPlane.Domain|
           +---------------------+
```

---

## 7. Draft Specifications for Codebase Hygiene

### 7.1 .editorconfig
Create `.editorconfig` in the root directory to enforce the 4-space indent for C#, system import sorting, and specific C# var guidelines.

```ini
# Rules for the entire solution repository
root = true

# All files - general defaults
[*]
indent_style = space
charset = utf-8
trim_trailing_whitespace = true
insert_final_newline = true

# C# source code files (.cs)
[*.cs]
indent_size = 4
# Disable var for built-in types (e.g. use 'int' instead of 'var')
csharp_style_var_for_built_in_types = false:warning
# Sort System directives alphabetically first in using blocks
dotnet_sort_system_directives_first = true:warning

# C# project and MSBuild configuration files (.csproj, .props, .targets)
[*.{csproj,props,targets,xml}]
indent_size = 2

# Configuration files (.json, .yml, .yaml)
[*.{json,yml,yaml}]
indent_size = 2
```

### 7.2 .gitignore
Create `.gitignore` in the root directory to prevent local compilation artifacts, caches, and logs from entering source control.

```text
# =======================================================================
# .NET Core / Visual Studio Build Artifacts
# =======================================================================
[Dd]ebug/
[Rr]elease/
x64/
x86/
bld/
[Bb]in/
[Oo]bj/
Generated Files/
TestResults/
*.suo
*.user
*.userosscache
*.sln.docstates

# =======================================================================
# Visual Studio Cache & Metadata Directories
# =======================================================================
.vs/
.vs-*

# =======================================================================
# NuGet Packages Cache
# =======================================================================
project.lock.json
project.assets.json
.nuget/
packages/

# =======================================================================
# Rider / ReSharper / VS Code Configurations
# =======================================================================
.idea/
*.sln.iml
_ReSharper*/
.vscode/

# =======================================================================
# Operating System files
# =======================================================================
.DS_Store
Thumbs.db
ehthumbs.db
desktop.ini

# =======================================================================
# Build / Publish Output Directories
# =======================================================================
artifacts/
publish/

# =======================================================================
# Application Logs (Serilog Redaction/Logging Target)
# =======================================================================
logs/
*.log

# =======================================================================
# Native Debug / Intermediate Files
# =======================================================================
*.pdb
*.iobj
*.ipdb

# =======================================================================
# Configuration Secrets / Environment Files
# =======================================================================
appsettings.Development.json
appsettings.local.json
*.user.config
secrets.json

# =======================================================================
# Node.js (Web Frontend, AdminPortal build tools)
# =======================================================================
node_modules/
npm-debug.log*
yarn-debug.log*
yarn-error.log*
.pnpm-debug.log*

# =======================================================================
# Whisper Models (Offline GGUF Model Files)
# =======================================================================
*.gguf
*.bin
models/
```

---

## 8. Recommendations for Implementation (Milestone 1 Next Steps)

1. **Solution and Project Creation:**
   - Execute `dotnet new sln -n UniversalDictation` at the root.
   - Create directories: `src/` and `tests/`.
   - Create projects matching Section 5 using `dotnet new classlib`, `dotnet new wpf`, `dotnet new webapi`, `dotnet new web`, `dotnet new console`, and `dotnet new xunit`.
   - Link all projects to the solution using `dotnet sln add`.
2. **Setup References:**
   - Run `dotnet add reference` commands as mapped in Section 5.
   - Wire up package references in each project. Verify that **no** version attributes exist inside `<PackageReference>` nodes, as Central Package Management will fail build validation otherwise.
3. **Commit Configuration Files:**
   - Create `.editorconfig` and `.gitignore` at the root using the exact templates provided in Section 7.
4. **Core Interface Implementation:**
   - Move to implementing the interfaces (`IStreamingTranscriptionProvider`, etc.), domain types (`TargetContext`, etc.), and exceptions in `Desktop.Core` project, adhering to strict namespace guidelines.
