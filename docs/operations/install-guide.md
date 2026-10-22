# Installation & Setup Guide

| Attribute | Specification Detail |
| :--- | :--- |
| **Title** | Installation & Setup Guide |
| **Version** | 1.0.0 |
| **Date** | 2026-09-16 |
| **Lifecycle Standard** | ISO/IEC/IEEE 15288:2023 §6.4.13 |
| **Classification** | UPSTREAM_SPEC_CORE_COMPILER |

> Comprehensive operational installation and environment provisioning guide for the Digital Engineering Agent Platform (DEAP) Specification Core Compiler, conforming to ISO/IEC/IEEE 15288:2023 §6.4.13 (Maintenance and Operations Documentation).

---

## Table of Contents

1. [Overview & Architecture](#1-overview--architecture)
   - [1.1 Pure Schema-Driven Compiler Invariant](#11-pure-schema-driven-compiler-invariant)
   - [1.2 Three-Tier Platform Isolation](#12-three-tier-platform-isolation)
   - [1.3 Primary Commercial Toolchain Integration Context](#13-primary-commercial-toolchain-integration-context)
2. [Prerequisites & System Requirements](#2-prerequisites--system-requirements)
   - [2.1 Supported Operating Systems](#21-supported-operating-systems)
   - [2.2 Software Dependencies](#22-software-dependencies)
3. [Upstream Repository Setup](#3-upstream-repository-setup)
   - [Step 3.1: Clone the Upstream Compiler Repository](#step-31-clone-the-upstream-compiler-repository)
   - [Step 3.2: Rust 1.80+ Toolchain Verification](#step-32-rust-180-toolchain-verification)
   - [Step 3.3: Build Pure-Rust Workspace Crates](#step-33-build-pure-rust-workspace-crates)
   - [Step 3.4: Configure Git Pre-Commit Hooks via verify-baseline](#step-34-configure-git-pre-commit-hooks-via-verify-baseline)
4. [Downstream Workspace Installation & Platform Bootstrapping](#4-downstream-workspace-installation--platform-bootstrapping)
   - [4.1 Initializing a Downstream Workspace](#41-initializing-a-downstream-workspace)
   - [4.2 Downstream Workspace Structure](#42-downstream-workspace-structure)
   - [4.3 Clean Landing Zone Invariant Distinction](#43-clean-landing-zone-invariant-distinction)
   - [4.4 Platform Profile Bootstrapping (.pipeline/profiles/)](#44-platform-profile-bootstrapping-pipelineprofiles)
   - [4.5 Environment Variables Reference](#45-environment-variables-reference)
   - [4.6 Downstream Governance & Toolchain Propagation Protocol](#46-downstream-governance--toolchain-propagation-protocol)
5. [Verification & Quality Gates](#5-verification--quality-gates)
   - [5.1 Running the Native Baseline Verifier](#51-running-the-native-baseline-verifier)
   - [5.2 Running the Specification Verification Gate](#52-running-the-specification-verification-gate)
   - [5.3 Summary of Automated Quality Gates](#53-summary-of-automated-quality-gates)
6. [Git Pre-Commit Hook Configuration](#6-git-pre-commit-hook-configuration)
   - [6.1 Hook Installation](#61-hook-installation)
   - [6.2 Pre-Commit Validations](#62-pre-commit-validations)
7. [Troubleshooting & FAQ](#7-troubleshooting--faq)
   - [7.1 Tracker CLI Authentication Issues](#71-tracker-cli-authentication-issues)
   - [7.2 Binary Execution Permissions](#72-binary-execution-permissions)
   - [7.3 KaTeX / LaTeX Font Rendering Failures](#73-katex--latex-font-rendering-failures)
   - [7.4 Check 16 Clean Landing Zone Violation](#74-check-16-clean-landing-zone-violation)
   - [7.5 Rust Test Execution & Filtering](#75-rust-test-execution--filtering)
   - [7.6 Cargo Build and Toolchain Resolution](#76-cargo-build-and-toolchain-resolution)

---

## 1. Overview & Architecture

The Digital Engineering Agent Platform (DEAP) Specification Core Compiler (`gintatkinson/DEAP01-spec-core`) is an abstract Model-Based Systems Engineering (MBSE) compiler, validation framework, and governance automation engine. It transforms SysML v2 / KerML Abstract Syntax Trees (ASTs), abstract interface schemas, and systems engineering constraints into verified specification suites (Epics, Features, User Stories, Use Cases, ICDs, and Safety Invariants).

### 1.1 Pure Schema-Driven Compiler Invariant

The compiler adheres strictly to the **Pure Schema-Driven Compiler Invariant**:
- The platform contains **zero hardcoded domain concepts**.
- Domain-specific concepts (such as unmanned aerial systems, automotive chassis, or medical devices) belong exclusively in downstream customer workspaces and domain templates.
- The upstream repository functions purely as a schema compiler, AST validator, and quality gate governance harness.

### 1.2 Three-Tier Platform Isolation

The DEAP architecture enforces strict three-tier platform isolation:

```
+------------------------------------------------------------------------+
|               Tier 1: Core Specification Metamodel                     |
|  - SysML v2 / KerML AST Parser & Metamodel Ingest                     |
|  - Abstract Specification Schemas (Epics, Features, US, UC, ICDs)      |
|  - ISO/IEC/IEEE 15288:2023 Systems Engineering Lifecycle Contracts    |
+-----------------------------------┬------------------------------------+
                                    |
                                    v
+------------------------------------------------------------------------+
|               Tier 2: AST Transformation & Quality Auditor             |
|  - Parity Auditor Engine (Profile Scoping, Schema Mapping)             |
|  - Coverage Digest Generator & Witness Registry Validator             |
|  - Cross-Document Diagram Parity & Metrology Verification Gates        |
+-----------------------------------┬------------------------------------+
                                    |
                                    v
+------------------------------------------------------------------------+
|               Tier 3: Target Platform Projections                      |
|  - Flutter Cross-Platform Client (app_flutter)                         |
|  - React High-Density Web Console (web_react)                          |
|  - Headless CI Digital Twin Engine & Verification Harness              |
+------------------------------------------------------------------------+
```

### 1.3 Primary Commercial Toolchain Integration Context

All engineering workflows, control law synthesis models, safety statecharts, and software generation across the DEAP framework declare:

> **Primary Commercial Toolchain Integration Context:**
> **MATLAB / Simulink / Stateflow / Embedded Coder** (Model-Based Design, Control Law Synthesis, DO-178C C/SPARK Ada code generation).

In downstream workspaces, physical and control features maintain dual-track Model-Based Design (MBD):
- **Track A (Native MATLAB/Simulink Synthesis)**: Programmatic model scripts (`.m`), parameter dictionaries (`.m`), and Simulink Data Dictionaries (`.sldd`) configured for deterministic discrete solvers (`FixedStepDiscrete`).
- **Track B (Headless CI Digital Twin Engine)**: Standalone, license-free discrete-time Pure-Rust execution engines validating identical loop rates, polynomial curves, and transition guards offline.

---

## 2. Prerequisites & System Requirements

Before configuring the compiler, ensure your workstation or CI environment meets the following minimum requirements.

### 2.1 Supported Operating Systems

- **macOS**: 13.0+ (Ventura, Sonoma, Sequoia) on Apple Silicon (`arm64`) or Intel (`x86_64`).
- **Linux**: Ubuntu 22.04 LTS / 24.04 LTS, Debian 12, or RHEL 9 (`x86_64` or `aarch64`).
- **Windows**: Windows 11 with Windows Subsystem for Linux 2 (WSL2 Ubuntu 22.04+). Direct Windows native execution is supported with PowerShell 7+.

### 2.2 Software Dependencies

| Dependency | Minimum Version | Required For | Verification Command |
| :--- | :--- | :--- | :--- |
| **Rust & Cargo** | 1.80+ | Workspace compiler crates, baseline verification, AST compilation | `cargo --version` |
| **Git** | 2.38+ | Repository governance, restoration points, hook enforcement | `git --version` |
| **Flutter SDK** | 3.22.0+ (Dart 3.4+) | Downstream Flutter cross-platform desktop/mobile client | `flutter --version` |
| **Node.js** | 18.0.0+ LTS (npm 9+) | Web React console, Firebase emulator CLI, markdown linters | `node --version` |
| **Java OpenJDK** | 17 LTS | SysML v2 CLI parsers, PlantUML rendering, Firestore emulator | `java -version` |
| **GitHub CLI** | 2.40+ | GitHub issue tracking, milestone syncing, backlog reconciliation | `gh --version` |
| **GitLab CLI** | 1.35+ (optional) | GitLab issue tracking and CI/CD provider syncing | `glab --version` |
| **Graphviz** | 2.40+ (optional) | Architecture diagram layout and PlantUML rendering | `dot -V` |

---

## 3. Upstream Repository Setup

Follow these steps to establish a clean development environment for the upstream specification compiler.

### Step 3.1: Clone the Upstream Compiler Repository

```bash
git clone https://github.com/gintatkinson/DEAP01-spec-core.git
cd DEAP01-spec-core
```

Verify that the upstream sentinel exists:
```bash
test -d .pipeline/upstream && echo "Upstream Sentinel Confirmed"
```

### Step 3.2: Rust 1.80+ Toolchain Verification

Verify that a modern Rust toolchain is installed:

```bash
rustc --version
cargo --version
```

If Rust is not yet installed on your system, install it via `rustup`:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Step 3.3: Build Pure-Rust Workspace Crates

Build the complete suite of workspace compiler and verification binaries in release mode:

```bash
cargo build --release --workspace
```

This compiles the native binaries under `./target/release/`:
- `verify-baseline`: Mechanical baseline validator and gate verification engine
- `compile-sysml`: SysML v2 compiler, AST extractor, and STPA engine
- `ingest-sysml`: Level 0 AST ingestion engine (supporting SysML, KerML, and YANG)
- `assemble-conops`: Deterministic ConOps and Mission Intent document assembler
- `reconcile-backlog`: Backlog reconciler and tracker synchronizer

### Step 3.4: Configure Git Pre-Commit Hooks via verify-baseline

Install the OS-level git hooks directly using the native verifier:

```bash
./target/release/verify-baseline setup-hooks .
```

Verify that the CLI entrypoints operate correctly:

```bash
./target/release/verify-baseline --help
./target/release/reconcile-backlog --help
```

---

## 4. Downstream Workspace Installation & Platform Bootstrapping

The upstream specification core compiler (`UPSTREAM_SPEC_CORE_COMPILER`) serves as the central abstract MBSE engine. Downstream domain projects instantiate from clean distribution templates (`DEAP-*`).

### 4.1 Initializing a Downstream Workspace

To initialize and bootstrap a downstream repository:

1. Clone or instantiate a downstream workspace from the repository template.
2. Verify baseline conformance and initialize OS Git hooks using the native Rust binary:
   ```bash
   ./target/release/verify-baseline setup-hooks .
   ./target/release/verify-baseline . --no-domain
   ```
3. Populate `schema/` with your domain SysML v2 / KerML models.

### 4.2 Downstream Workspace Structure

The downstream repository contains:
- `.pipeline/`: Governance constitution, platform profiles, and validation configuration.
- `rules/`: Hardcoded systems engineering discipline and verification rules.
- `skills/`: Agent skills (`feature-driven-implementation`, `spec-orchestrator`, etc.).
- `schema/`: Clean landing zone ready to receive domain SysML v2 / KerML models.
- `docs/`: Clean landing zones for derived specifications (`docs/conops/`, `docs/safety/`, `docs/epics/`, `docs/features/`, `docs/user-stories/`, `docs/use-cases/`).

### 4.3 Clean Landing Zone Invariant Distinction

| Repository Role | Sentinel File | Landing Zone Rule |
| :--- | :--- | :--- |
| **Upstream Compiler** (`DEAP01-spec-core`) | `.pipeline/upstream/` present | **STRICT CLEAN**: `schema/` and `docs/{epics,features,...}` must contain ONLY `.gitkeep` and `README.md`. Zero domain models allowed. |
| **Downstream Workspace** (`my-project`) | `.pipeline/upstream/` absent | **CONCRETE SPECS**: User populates `schema/` with domain models; derived specifications populate `docs/`. |

### 4.4 Platform Profile Bootstrapping (.pipeline/profiles/)

Downstream codebases select a target runtime profile located in `.pipeline/profiles/`:
- **`flutter.md`**: Cross-platform desktop and mobile Logical UI with 60 FPS viewport guarantees and decoupled state management.
- **`react.md`**: High-density operator web console architecture with CSS Container Queries and REST/WebSocket interfaces.
- **`ros2_cpp.md`**: Embedded real-time robotics lifecycle nodes with deterministic memory allocation.
- **`px4_module.md`**: Flight control autopilot modules with uORB messaging and ASTM F3269-17 run-time assurance monitors.
- **`misra_c_spark.md`**: Safety-critical firmware synthesized via MATLAB / Simulink Embedded Coder.

To bootstrap and lock a platform profile in a downstream workspace:
1. Configure `.pipeline/codebase_rules.json` with the chosen platform profile identifier.
2. Ensure the downstream agent adopts the platform profile rules during Step 1 of the feature implementation workflow.
3. Validate profile conformance using `./target/release/verify-baseline . --spec-only`.

### 4.5 Environment Variables Reference

The DEAP toolchain and downstream runtimes support the following environment variables:

| Variable Name | Default Value | Description |
| :--- | :--- | :--- |
| `GITHUB_TOKEN` | *None* | Personal access token for GitHub API issue tracking and milestone synchronization. |
| `GITLAB_TOKEN` | *None* | Personal access token for GitLab REST API queries. |
| `JIRA_API_TOKEN` | *None* | Basic Auth API token for Atlassian Jira Cloud instances. |
| `JIRA_EMAIL` | *None* | User email associated with `JIRA_API_TOKEN`. |
| `DATA_SOURCE` | `sqlite` | Runtime backend selector (`sqlite`, `firebase`, `grpc`, `fallback`). |
| `FLUTTER_TEST` | *Unset* | When set to `true`, limits database seed rows and accelerates test execution. |
| `FIRESTORE_EMULATOR_HOST` | *Unset* | Host and port for connecting to the local Firebase Firestore emulator (e.g. `localhost:8080`). |
| `DEAP_WORKSPACE` | `.` | Absolute or workspace-relative path to the target repository root. |

### 4.6 Downstream Governance & Toolchain Propagation Protocol

#### Scope & Objective
The Downstream Governance & Toolchain Propagation Protocol establishes the mechanical synchronization process across repository tiers:
- **Tier 0 (Upstream Specification Core Compiler)**: `DEAP01-spec-core` (authoritative single source of truth for systems engineering metamodels, compiler crates, and verification gates).
- **Tier 1 (Platform Template Repositories)**: Distribution templates such as `DEAP-profile-flutter-app`, maintaining clean landing zones and verified platform projections.
- **Tier 2 (Customer Application Workspaces)**: Active engineering workspaces such as `DEAP01-cuas-coa-engine`, consuming concrete SysML schemas and implementing platform projections.

This protocol propagates the unified Turn-0 session initialization receipt, the 5-point Karpathy and Pipeline compliance check, and compiled native Rust binaries from Tier 0 to Tier 1 platform templates and Tier 2 customer workspaces.

#### Synchronized Governance Artifacts
To maintain uniform agent governance and fail-closed quality gates across all downstream repositories, the following core governance assets must be synchronized from Tier 0:
- **`AGENTS.md` and symlink `.agents/AGENTS.md`**: Master agent instruction entrypoint containing coordinator tool locking, intent classification, and compliance check protocols.
- **`.pipeline/constitution.md`**: Platform-independent functional governance constitution and zero-mocking persistence mandates.
- **`.pipeline/ACTIVE_RULES_BUNDLE.md`**: Consolidated active governance rules bundle ingested in a single read during Turn-0 initialization.
- **`rules/*.md`**: Granular governance rules, especially:
  - `rules/constitution-first.md`: Platform profile scoping and loading constraints.
  - `rules/user-authorization-lock.md`: Plan approval locks and continuous execution gates.
  - `rules/role-boundary-lock.md`: Coordinator delegation duty and subagent execution boundaries.

#### Precompiled Release Binaries
Downstream repositories do not rebuild the compiler crates from source; instead, they receive precompiled release binaries built in Tier 0:
1. Build release binaries in Tier 0 (`DEAP01-spec-core`):
   ```bash
   cargo build --release --workspace
   ```
2. Copy the resulting binaries into the downstream repository's `target/release/` directory:
   - `target/release/verify-baseline`
   - `target/release/compile-sysml`
   - `target/release/ingest-sysml`
   - `target/release/assemble-conops`
   - `target/release/reconcile-backlog`
3. Ensure binary execution permissions in downstream:
   ```bash
   chmod +x target/release/*
   ```

#### Downstream Verification Command Sequence
Once governance files and release binaries are staged in the downstream repository, execute the following bootstrap verification sequence in the downstream root:
```bash
# In downstream repository root:
./target/release/verify-baseline setup-hooks .
./target/release/verify-baseline . --no-domain
```
This installs the pre-commit git hook and validates all baseline gates (hygiene, entrypoints, binary executability, syntax, and governance consistency) against the downstream repository.

#### Turn-0 Initialization Expectation
Every downstream coordinator agent on Turn 0 MUST execute Phase A of the Mandatory Session Initialization Gate:
1. Detect repository scope by inspecting the presence of `.pipeline/upstream/`. In downstream repositories, `.pipeline/upstream/` is absent, evaluating deterministically to `DOWNSTREAM_CUSTOMER_PROJECT`.
2. Ingest `.pipeline/constitution.md` and `.pipeline/ACTIVE_RULES_BUNDLE.md`.
3. Check working tree state via read-only `git status --short --branch`.
4. Output the verbatim Canonical Session Initialization Receipt Table before planning or coding.

---

## 5. Verification & Quality Gates

The DEAP platform executes a comprehensive, 35-checkpoint native Rust verification suite to guarantee fail-closed governance and architectural compliance.

### 5.1 Running the Downstream Baseline Verification

Run the primary verification harness against the active workspace:

```bash
./target/release/verify-baseline . --no-domain
```

Expected passing output:
```text
Success: Check 10 verified (.gitignore exists in repository root).
Success: Check 11 verified (zero .DS_Store files found).
Success: Check 12 verified (Master core / upstream repository detected).
Success: Check 13 verified (KaTeX / LaTeX mathematical syntax valid across all markdown files).
Success: Mermaid syntax verified across all markdown files.
Success: Check 14 verified (README.md, agent instruction entrypoints exist).
Success: Check 15 verified (Native reconcile-backlog binary exists and is executable).
Success: Check 16 verified (Upstream distribution template landing zones are clean).
...
Success: Checks 30A-35 verified (Architecture, binary, pure-Rust, and integrity invariants).
Success: Build and test suite execution passed. Conformance gate verified.
Tagging restoration point...
```

### 5.2 Running the Specification Verification Gate

Execute the baseline verifier in specification-only mode:

```bash
./target/release/verify-baseline . --spec-only
```

### 5.3 Summary of Automated Quality Gates

| Check ID | Gate Name | ISO 15288 Clause | Invariant Description |
| :--- | :--- | :--- | :--- |
| **Check 10** | `.gitignore` Hygiene | §6.4.2 | Prevents OS artifacts (`.DS_Store`) and temporary build outputs from being tracked. |
| **Check 11** | Artifact Cleanliness | §6.4.2 | Zero `.DS_Store` or stray binary files across all folders. |
| **Check 12** | Master Blueprint Isolation | §6.4.3 | Architecture blueprints remain synchronized without duplication. |
| **Check 13** | Mathematical Syntax Integrity | §6.4.3 | All LaTeX / KaTeX expressions are valid; literal \$ properly escaped. |
| **Check 14** | Entrypoint & Rules Check | §6.4.1 | Verifies presence of `README.md`, `AGENTS.md`, and core governance rules. |
| **Check 15** | Backlog Tooling Executability | §6.4.13 | Ensures `target/release/reconcile-backlog` is executable. |
| **Check 16** | Clean Landing Zone Gate | §6.4.3 | Upstream template directories must contain zero concrete specifications. |
| **Check 17** | Safety Invariant Landing Zone | §6.4.14 | Upstream `docs/safety/` contains zero concrete domain hazards. |
| **Check 18** | Abstract Blueprint Cleanliness | §6.4.3 | No domain concept papers hardcoded in upstream architecture. |
| **Check 19** | Closed-Grammar Metamodel Gate | §6.4.3 | Verifies pure AST-derived schema generation with zero domain strings. |
| **Checks 20-23** | WBS & SysML AST Gates | §6.4.5 | Validates Work Breakdown Structure and SysML parser AST node coverage. |
| **Check 24** | Operational Allocation Gate | §6.4.5 | Validates that all operational activities map to architectural resources. |
| **Check 25** | Parameter Metrology & Diagram Parity | §6.4.5 | Validates SI 7D units and diagram node/port parity across documents. |
| **Check 26** | ConOps & Mission Intent Completeness | §6.4.1 | Verifies all mandatory ConOps sections and METL rosters. |
| **Check 27** | Cited Research & Traceability Gate | §6.4.2 | Enforces 100% citation grounding against declared research inventories. |
| **Check 28** | Coverage-Digest Population Gate | §6.4.2 | Enforces bidirectional traceability from schema nodes to specs. |
| **Check 29** | Obligation-Witness Registry Gate | §6.4.2 | Validates that all contractual obligations have verified witnesses. |
| **Check 30** | Architecture Viewpoint Completeness | §6.4.5 | Enforces 11 canonical architecture diagrams across 5 viewpoints. |
| **Check 31** | Baseline Release Binary Invariant | §6.4.2 | Ensures `./target/release/verify-baseline` exists and executes. |
| **Check 32** | Zero Shell Scripts Invariant | §6.4.2 | Zero `*.sh` scripts permitted in repository. |
| **Check 33** | Zero Python Scripts Invariant | §6.4.2 | Zero `*.py` scripts permitted in repository. |
| **Check 34** | Zero Dynamic Regex Across Crates | §6.4.2 | Zero regex dependency or invocation across Rust crates. |
| **Check 35** | Handoff & Backlog Integrity Gate | §6.4.13 | Zero broken links or retired script references in active docs. |

---

## 6. Git Pre-Commit Hook Configuration

To prevent committing malformed markdown, unescaped KaTeX delimiters, broken Mermaid diagrams, or premature issue-closing keywords, configure the DEAP git hooks.

### 6.1 Hook Installation

Run the hook setup via the native baseline binary:

```bash
./target/release/verify-baseline setup-hooks .
```

Expected output:
```text
Configuring DEAP Git pre-commit hook...
Installed .git/hooks/pre-commit successfully.
Permissions set to 0755 (executable).
```

### 6.2 Pre-Commit Validations

The installed hook automatically enforces:
1. **Commit Message Non-Closure Invariant**: Commit messages must use neutral issue references `(#<id>)` or `(refs #<id>)`. Issue auto-closing keywords (`fixes #123`, `closes #456`) are blocked to prevent premature server-side closure before Product Owner review.
2. **Landing Zone Hygiene**: Prevents committing domain-specific files into upstream landing zones.
3. **KaTeX & LaTeX Syntax**: Validates all modified `.md` files for unescaped `\$` symbols.
4. **Mermaid Block Syntax**: Ensures all Mermaid diagrams have valid opening headers and explicit closing fences.

---

## 7. Troubleshooting & FAQ

### 7.1 Tracker CLI Authentication Issues

**Symptom**: `gh issue view` or `glab issue list` fails with `authentication required` or `401 Unauthorized`.

**Resolution**:
- For GitHub:
  ```bash
  gh auth login
  # Or set an environment variable:
  export GITHUB_TOKEN="ghp_yourPersonalAccessTokenHere"
  ```
- For GitLab:
  ```bash
  glab auth login --hostname gitlab.com
  # Or set an environment variable:
  export GITLAB_TOKEN="glpat-yourPersonalAccessTokenHere"
  ```
- For Jira:
  ```bash
  export JIRA_API_TOKEN="yourJiraApiToken"
  export JIRA_EMAIL="your-email@domain.com"
  ```

### 7.2 Binary Execution Permissions

**Symptom**: Executing `./target/release/verify-baseline` returns `permission denied`.

**Resolution**:
Restore execution permissions on release binaries:
```bash
chmod +x target/release/*
```

### 7.3 KaTeX / LaTeX Font Rendering Failures

**Symptom**: Check 13 fails with: `LaTeX Syntax Error: Unescaped dollar sign found in markdown file`.

**Resolution**:
In markdown prose and documentation tables, standalone currency symbols or shell variables must be escaped with a backslash (`\$`) or enclosed in inline code backticks (`` `$PATH` ``). Two unescaped `$` characters in a paragraph are interpreted by KaTeX as math delimiters and break document compilation.

### 7.4 Check 16 Clean Landing Zone Violation

**Symptom**: `ERROR: Check 16 failed: Upstream distribution template landing zones contain concrete specification files: docs/features/FEAT-001.md`.

**Resolution**:
You are working in the upstream specification compiler repository (`DEAP01-spec-core`). Upstream landing zones (`schema/`, `docs/epics/`, `docs/features/`, `docs/user-stories/`, `docs/use-cases/`, `docs/conops/`, `docs/safety/`) must remain clean landing zones with only `.gitkeep` and `README.md` files. Move concrete project specifications to your downstream workspace instantiated from distribution templates.

### 7.5 Rust Test Execution & Filtering

**Symptom**: Long test runs when running the full workspace test suite.

**Resolution**:
Target specific packages or test names when debugging:
```bash
cargo test -p verify-baseline -- test_issue_441
cargo test -p compile-sysml
```
Or execute the entire workspace test suite:
```bash
cargo test --workspace
```

### 7.6 Cargo Build and Toolchain Resolution

**Symptom**: Compilation fails with feature or edition errors on older Rust compilers.

**Resolution**:
Ensure your Rust toolchain is updated to the latest stable version:
```bash
rustup update stable
cargo check --workspace
```
