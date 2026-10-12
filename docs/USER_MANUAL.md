| Attribute | Specification Detail |
| :--- | :--- |
| **Document** | Operations and User Manual |
| **Status** | Approved |
| **Version** | 1.0.0 |
| **Role** | Downstream Systems Engineering & Compiler Operations |
| **Framework** | Digital Engineering Agent Platform (DEAP) |
| **Applicability** | Cross-Platform Engineering Environments (macOS, Linux, Windows) |

# Operations and User Manual

This manual provides authoritative, plain-language guidance for developers, systems engineers, and autonomous agents operating within the Digital Engineering Agent Platform (DEAP) downstream workspace.

---

## 1. Quick Start Guide

This section outlines the basic setup and verification workflow required to initialize and validate the workspace.

### 1.1 Prerequisites

Before executing operations within this repository, ensure the following core tools are installed:

- **Rust Toolchain**: Rust 1.75+ (edition 2021) with `cargo` and `rustc`.
- **Git**: Git 2.30+ installed and available in the system PATH.
- **Platform Development Runtimes**: Depending on your project platform profile (e.g. Flutter, React/TypeScript, MATLAB/Simulink), ensure target development SDKs are configured.

### 1.2 Toolchain Workspace Compilation

The repository provides high-performance native Rust binaries across five specialized crates (`crates/verify-baseline`, `crates/reconcile-backlog`, `crates/compile-sysml`, `crates/ingest-sysml`, and `crates/assemble-conops`). Compile all workspace binaries with release optimizations:

```bash
# Compile optimized release binaries across the entire workspace
cargo build --release --workspace

# Alternatively, verify type correctness without full link
cargo check --workspace
```

Upon successful compilation, native binaries reside in `./target/release/` (`verify-baseline`, `reconcile-backlog`, `compile-sysml`, `ingest-sysml`, `assemble-conops`).

### 1.3 Native Git Hook Initialization

Git hooks enforce commit message invariants and automated baseline checks before code reaches version control. Run the native Rust hook configuration command:

```bash
# Install cross-platform Git hooks natively
./target/release/verify-baseline setup-hooks
```

What `setup-hooks` executes:
- Configures `.git/hooks/pre-commit` to run baseline conformance checks (`./target/release/verify-baseline . --no-domain`) and markdown output auditing (`check-output`).
- Configures `.git/hooks/commit-msg` to enforce the commit message non-closure invariant (`check-commit-msg`).
- Strips OS-generated metadata artifacts (`.DS_Store`) across the entire repository tree.
- Automatically updates `.gitignore` to whitelist essential pipeline directories (`.pipeline/`, `skills/`, `rules/`, `.agents/`).
- Sets executable permissions on Unix and macOS platforms while remaining 100% compatible with Windows.

### 1.4 Downstream Baseline Verification

Run baseline conformance verification to ensure all repository contracts, governance rules, and schema fixtures pass:

```bash
# Run baseline conformance verification (checks 10-32)
./target/release/verify-baseline . --no-domain
```

A successful execution evaluates 32 mechanical quality gates and concludes with:
`Success: Check 32 verified (Zero Shell Scripts Invariant passed -- zero .sh files exist on disk).`
`Success: Specification Quality & Coverage Audit passed`.

---

## 2. The "Zero-Shell" & Cross-Platform Policy

DEAP strictly enforces a zero-shell-script invariant across all workspaces. This section details the operational rationale, risk mitigations, and architectural implementation of this policy.

### 2.1 Plain English Translation of Check 32

Internal **Check 32** is an automated quality gate executed by `verify-baseline`. During verification, the tool scans the entire repository tree (excluding build artifacts in `target/` and metadata in `.git/`).

If **any file with a `.sh` extension** is discovered anywhere in the repository, the baseline check immediately fails closed with an error code and generates a defect dossier. Shell scripts are completely prohibited from being checked into the repository or used in operational workflows.

### 2.2 Why Shell Scripts Are Prohibited

The prohibition of `.sh` shell scripts is a deliberate engineering decision based on cross-platform reliability, safety-critical governance, and environmental determinism:

1. **Windows Portability Failures**: Shell scripts depend on POSIX shell interpreters (`/bin/sh`, `/bin/bash`, `zsh`) that are absent from native Windows environments. Developers and CI runners on Windows using PowerShell or `cmd.exe` cannot execute `.sh` files without external compatibility layers (WSL, MSYS2, Git Bash, Cygwin). These external layers introduce path translation errors, drive-letter discrepancies, and environment contamination.
2. **Line Ending (CRLF vs LF) Corruption**: On Windows workstations, Git configurations often convert newline delimiters to Carriage Return Line Feed (`CRLF` / `\r\n`). When a shell script containing CRLF line endings is executed on Unix or Linux CI runners, the interpreter fails immediately with cryptic errors such as:
   ```text
   /bin/sh^M: bad interpreter: No such file or directory
   ```
   This failure occurs before a single line of script code executes, creating spurious build breakages across distributed engineering teams.
3. **Executable Bit Tracking Fragility**: POSIX file permissions (`chmod +x` / file mode `100755`) are not supported natively by Windows file systems (FAT32, NTFS). Files created or modified on Windows frequently lose their executable bit when committed to Git. When checked out on Linux or macOS, the scripts fail with `Permission denied` errors.
4. **Untyped, Fragile Execution**: Shell scripts lack static type checking, structured error handling, memory safety, and standard library consistency. Differences between shell versions (GNU Bash 3.2 on macOS vs. Bash 5.x on Linux) cause silent syntax discrepancies, unhandled pipeline errors, and argument injection vulnerabilities.

### 2.3 The Compiled Rust Architecture

To guarantee deterministic cross-platform execution, all operational tooling, verification gates, and automation tasks are implemented exclusively in native compiled Rust:

- **Single Executable Artifact**: Rust compiles to self-contained native binaries (`.exe` on Windows, Mach-O on macOS, ELF on Linux) with zero runtime dependencies.
- **Strict Type Safety**: All CLI arguments, flags, and configuration inputs are parsed and validated through strongly typed interfaces using `clap`.
- **Sub-Second Performance**: Critical path verification runs in milliseconds, making automated pre-commit and CI verification imperceptible to developer velocity.
- **Identical Behavior Across Operating Systems**: File system access, line ending handling, path normalization, and process spawning behave identically across Windows, macOS, and Linux.

---

## 3. Complete CLI Operations & Tooling Reference

The repository provides five primary CLI tools compiled under `./target/release/`.

### 3.1 `verify-baseline` -- Baseline Conformance and Quality Gate Verifier

The `verify-baseline` utility is the central quality assurance engine, enforcing repository invariants, schema consistency, mathematical syntax, and security boundaries.

#### Syntax & Primary Options

```bash
./target/release/verify-baseline [OPTIONS] [DESTINATION] [COMMAND]
```

- `[DESTINATION]`: Repository root or directory to evaluate [default: `.`].
- `--no-domain`: Skips domain-specific compiler builds and tests; executes baseline repository checks (Checks 10-32).
- `--strict`: Fails closed if any specification documents are missing or pending.
- `--allow-missing-specs`: Tolerates missing specifications when operating in initial downstream onboarding phases.
- `--spec-only`: Executes only the specification quality and coverage audit suite, bypassing structural checks.
- `--gate <GATE>`: Filters execution to a single specific quality gate (e.g. `--gate 13`).
- `--only <PATH>`: Audits only the specified Markdown document.
- `--output <PATH>`: Writes detailed audit logs or defect dossiers to an output file.

#### Subcommands

##### 1. `setup-hooks`
Installs native Git pre-commit and commit-msg hooks, cleans `.DS_Store` files, and updates `.gitignore`:
```bash
./target/release/verify-baseline setup-hooks .
```

##### 2. `check-secrets`
Scans compiled build assets and deployment directories for accidental credential leaks:
```bash
# Scan default build directories (dist, build/web, build/macos)
./target/release/verify-baseline check-secrets .

# Scan custom build asset directories
./target/release/verify-baseline check-secrets --dirs dist build/target .
```
- **Lexical Pattern Detection**: Detects PEM private keys (`-----BEGIN [TYPE ]PRIVATE KEY-----`) and AWS Access Key IDs (`AKIA...`).
- **Zero Regex Invariant**: Employs deterministic pure-Rust token scanners without regex dependencies.

##### 3. `check-commit-msg`
Validates that commit messages satisfy the non-closure invariant and do not contain premature auto-closing keywords:
```bash
# Validate message inside a file (used by Git hook)
./target/release/verify-baseline check-commit-msg --msg-file .git/COMMIT_EDITMSG

# Check a raw string directly
./target/release/verify-baseline check-commit-msg --check-text "feat(engine): add telemetry parser (refs #42)"

# Check the latest commit on HEAD
./target/release/verify-baseline check-commit-msg --head

# Check a revision range
./target/release/verify-baseline check-commit-msg --range origin/main..HEAD
```

##### 4. `check-output`
Verifies that generated markdown documents satisfy file integrity rules:
```bash
# Audit all markdown files in docs/
./target/release/verify-baseline check-output --dir docs

# Audit specific files
./target/release/verify-baseline check-output --files docs/USER_MANUAL.md README.md
```
- Validates non-zero file length.
- Verifies code fence balance (even number of ```` ``` ```` occurrences).
- Ensures zero unresolved template escape tokens (such as `{{` + `REQUIRED_*}}` placeholders).

##### 5. `check-defect`
Validates defect dossiers against the 7-section Adversarial Audit format before submission:
```bash
# Validate a defect file locally without submitting
./target/release/verify-baseline check-defect --body-file docs/reports/DEFECT_DOSSIER.md --dry-run
```

---

### 3.2 `reconcile-backlog` -- Backlog & Issue Tracker Synchronization

The `reconcile-backlog` engine maintains bidirectional synchronization between local Markdown specifications and remote issue tracking platforms (GitHub, GitLab).

#### Syntax & Primary Options

```bash
./target/release/reconcile-backlog [OPTIONS] [COMMAND]
```

- `--workspace <PATH>`: Workspace root path [default: `.`].
- `--offline`: Synchronizes local markdown checklists without network calls to external APIs.
- `--provider <PROVIDER>`: Issue tracker platform (`github` or `gitlab`) [default: `github`].
- `--dry-run`: Simulates synchronization and reports diffs without modifying files or remote issues.
- `--sync-bodies`: Pushes updated local markdown specification content to remote tracker issue descriptions.

#### Subcommands

##### `create-issue`
Creates or reopens backlog issues idempotently, verifying preconditions before submission:
```bash
./target/release/reconcile-backlog create-issue \
  --body-file docs/features/feat-01.md \
  --label feature \
  --title "Feature 01: Telemetry Streaming" \
  --repo gintatkinson/dumpiler-01
```

---

### 3.3 `compile-sysml` -- SysML v2 / KerML Compiler & Transpiler

The `compile-sysml` binary compiles SysML v2 textual models, verifies semantic integrity, synchronizes AST elements with markdown specifications, and transpile safety models into STPA matrices.

#### Syntax & Primary Options

```bash
./target/release/compile-sysml [OPTIONS] [FILE]
```

- `--compile`: Executes the Pipeline 0 compilation gate. Parses SysML syntax, performs semantic validation, and serializes the AST digest into `.pipeline/schema-digest.json`.
- `--forward-sync`: Synchronizes structural AST definitions from SysML textual models into markdown specifications under `docs/`.
- `--reverse-sync`: Extracts acceptance criteria and model definitions from markdown specifications and synchronizes them into the SysML Single Source of Truth (SSOT).
- `--stpa-transpile`: Transpiles SysML AST model elements into the 10-pillar STPA safety specification suite.
- `--stpa <PATH>`: Compiles an STPA markdown matrix into formal SysML constraint blocks.
- `--schema <PATH>`: Specifies input schema file or directory [default searches `schema/*.sysml`].
- `--out <PATH>`: Sets the target schema output path [default: `.pipeline/schema.sysml`].
- `--digest <PATH>`: Sets the output schema digest path [default: `.pipeline/schema-digest.json`].
- `--dry-run`: Simulates synchronization without writing modifications to disk.

---

### 3.4 `ingest-sysml` -- Ground 0.0 Universal Ingestion Engine

The `ingest-sysml` binary ingests heterogeneous, unstructured OEM specifications, flight manuals, interface control tables, OpenAPI schemas, and BOM tables into standardized SysML v2 textual definitions.

#### Syntax & Primary Options

```bash
./target/release/ingest-sysml [OPTIONS]
```

- `-s, --schema <PATH>`: Path to input documentation or schemas [default: `schema/`].
- `-f, --format <FORMAT>`: Ingestion parser format (`auto`, `sysml`, `idl`, `openapi`, `markdown`, `autosar`, `protobuf`) [default: `auto`].
- `-o, --out <PATH>`: Output SysML v2 file path [default: `.pipeline/schema.sysml`].
- `-d, --digest <PATH>`: Output AST digest path [default: `.pipeline/schema-digest.json`].
- `--allowed-parts <NAMES>`: Comma-separated list of allowed part definition names to retain in a closed-world scope.
- `--negative-invariants <NAMES>`: Comma-separated list of entities to assert negative exclusion for (`!exists(...)`).

---

## 4. Developer Workflows & Git Conventions

All engineering contributions within DEAP workspaces must follow strict Git conventions to maintain traceability and prevent accidental lifecycle disruptions.

### 4.1 Git Branching Conventions

- **Feature & Implementation Branches**: Name branches using the pattern `feat/<issue-id>-<slug>` (for example: `feat/42-telemetry-pipeline` or `feat/105-zero-shell-audit`).
- **Specification Work**: Authored on `main` or targeted specification branches (`spec/<module>`).
- **Workspace-Relative Paths**: All scripts, configurations, and commands must use repository-relative paths (`.`, `./...`). Never hardcode workstation-specific absolute paths.

### 4.2 Commit Message Formatting & Conventional Commits

Commits must follow the Conventional Commits specification:
- Implementation commits: `feat:`, `fix:`, `test:`, `refactor:`, `chore:`.
- Specification and documentation commits: `docs: [action] [artifact type] -- [brief description] (refs #<id>)`.

Examples:
```text
feat(compiler): add AST node validation for payload boundaries (refs #102)
docs(manual): author operations and user manual (refs #108)
chore(toolchain): update baseline quality gate verifier (refs #112)
```

### 4.3 Neutral Issue Citations vs. Forbidden Auto-Closing Keywords

Git commit messages and pull request descriptions must use **neutral issue citations**:
- **Permitted Syntax**: `(#<id>)` or `(refs #<id>)` (e.g. `(refs #42)`).

The following auto-closing keywords are **strictly prohibited** in commit messages:
- `fix #`, `fixes #`, `fixed #`
- `close #`, `closes #`, `closed #`
- `resolve #`, `resolves #`, `resolved #`

#### Why Auto-Closing Keywords Are Banned:
In GitHub and GitLab, committing text containing `fixes #42` or `closes #42` triggers server-side automation that automatically closes the issue upon push or pull request merge.

Under **CMMI Level 3** and safety-critical engineering processes (such as RTCA DO-178C), there is a strict separation between **Verification** and **Validation**:
1. **Verification (Development Gate)**: Verifies that code compiles cleanly, passes linters, satisfies test suites, and integrates into the target branch. Completing verification moves an issue to `status:fixed-resolved`.
2. **Validation (Customer / Product Owner Gate)**: Requires formal customer or Product Owner inspection and acceptance before the issue can transition to `Closed`.

Auto-closing keywords bypass the validation gate, prematurely closing issues before acceptance criteria have been validated.

### 4.4 The Zero Em Dash Invariant

Unicode em dashes (`\u2014`) are strictly forbidden across all files, commit messages, and documentation. Em dashes cause rendering artifacts, font discrepancies, and encoding mismatches across terminals and operating systems.
- Use standard ASCII `--` or `-` exclusively.

### 4.5 Remote Synchronization Mandate

Every development and documentation task must be verified against the remote tracking branch before declaring completion. Always verify:
```bash
git diff origin/<branch>
```
The diff must be completely empty before completing the task.

---

## 5. Practical Troubleshooting Guide

This section provides diagnostic symptoms, root causes, and explicit remediation steps for common pipeline verification issues.

### 5.1 Symptom: Shell Script Detected (Check 32 Failure)

**Diagnostic Output:**
```text
ERROR: Check 32 failed (Zero Shell Scripts Invariant violated -- .sh files found on disk):
  scripts/deploy.sh
```

- **Root Cause**: A shell script (`*.sh`) was committed or generated in the workspace.
- **Remediation**:
  1. Remove the shell script from the repository:
     ```bash
     git rm -f scripts/deploy.sh
     ```
  2. If the script performed operational automation, port its functionality to a subcommand in `crates/verify-baseline` or a standalone Rust binary in `crates/`.
  3. Re-run verification:
     ```bash
     ./target/release/verify-baseline . --no-domain
     ```

---

### 5.2 Symptom: Commit Message Rejected (Non-Closure Violation)

**Diagnostic Output:**
```text
ERROR: Commit message non-closure invariant violated:
  offending text: 'fixes #104'
  forbidden verb: 'fixes' preceding issue '#104'
```

- **Root Cause**: The commit message used an auto-closing keyword (`fix`, `fixes`, `close`, `resolve`) immediately preceding an issue number.
- **Remediation**:
  1. Rewrite the commit message using neutral citations:
     ```bash
     # Amend the latest commit message
     git commit --amend -m "feat(sensor): calibrate IMU rate clamps (refs #104)"
     ```
  2. Test the message using the verification binary:
     ```bash
     ./target/release/verify-baseline check-commit-msg --check-text "feat(sensor): calibrate IMU rate clamps (refs #104)"
     ```

---

### 5.3 Symptom: Secret Leak Scanner Finding (`check-secrets` Failure)

**Diagnostic Output:**
```text
ERROR: Leak detected in dist/bundle.js: AWS Access Key ID 'AKIAIOSFODNN7EXAMPLE'
```

- **Root Cause**: A private cryptographic key (PEM format) or cloud access key (AWS Access Key ID) was discovered in post-build assets.
- **Remediation**:
  1. Immediately revoke and rotate the exposed credential in your cloud management console.
  2. Remove the hardcoded secret from configuration files or environment definitions.
  3. Clean the build output directory and verify `.gitignore` contains build output paths:
     ```bash
     cargo clean
     ./target/release/verify-baseline check-secrets .
     ```

---

### 5.4 Symptom: KaTeX / LaTeX Formula Syntax Error (Check 13 Failure)

**Diagnostic Output:**
```text
ERROR: KaTeX mathematical syntax error in docs/features/feat-02.md:
  top-level \begin{align} is forbidden inside $$ display blocks
```

- **Root Cause**: A display math block used `\begin{align}` instead of `\begin{aligned}`, had non-isolated `$$` delimiters, or used LaTeX math delimiters around alphanumeric requirement keys (`$SC-01$`).
- **Remediation**:
  1. Change `\begin{align}` to `\begin{aligned}`:
     ```markdown
     $$
     \begin{aligned}
     v_{nominal} &= \frac{d_{travel}}{t_{elapsed}}
     \end{aligned}
     $$
     ```
  2. Ensure display math delimiters `$$` reside on their own dedicated lines.
  3. Remove math delimiters from requirement tags. Use bold formatting (`**SC-01**`) instead of math formatting (`$SC-01$`).

---

### 5.5 Symptom: Mermaid Diagram Syntax Error (Check 13B Failure)

**Diagnostic Output:**
```text
ERROR: Mermaid syntax violation in docs/architecture/overview.md:
  unquoted special characters '<' or '>' in node label
```

- **Root Cause**: A Mermaid diagram contains unquoted comparison operators (`<`, `>`), special punctuation (`/`, `:`, `()`), unescaped curly braces in class members, or an unbalanced closing fence.
- **Remediation**:
  1. Enclose node labels containing special characters in double quotes:
     ```text
     # Correct:
     node1["Threshold (< 50 ms)"]
     
     # Incorrect:
     node1[Threshold (< 50 ms)]
     ```
  2. Ensure the first line of the code block declares a valid diagram header (`flowchart TD`, `sequenceDiagram`, `stateDiagram-v2`, `classDiagram`, `erDiagram`).
  3. Verify code fence closing delimiters match opening delimiters.

---

### 5.6 Symptom: Subagent Output Verification Failure (`check-output`)

**Diagnostic Output:**
```text
ERROR: Subagent output verification failed:
  Unbalanced code fences in: docs/reports/summary.md (found 3 fences)
```

- **Root Cause**: A markdown document contains an odd number of ```` ``` ```` code fences or contains unreplaced required placeholder tokens (e.g. `{{` + `REQUIRED_*}}`).
- **Remediation**:
  1. Open the identified document and count code fence delimiters to ensure every opening fence has a corresponding closing fence.
  2. Search for any residual template tokens (e.g. `grep -n "REQUIRED_" docs/reports/summary.md`) and substitute concrete values.
  3. Re-verify using `./target/release/verify-baseline check-output --files docs/reports/summary.md`.

---

## 6. Source References

- Repository Governance Constitution: [.pipeline/constitution.md](../.pipeline/constitution.md)
- Consolidated Governance Manifest: [.pipeline/ACTIVE_RULES_BUNDLE.md](../.pipeline/ACTIVE_RULES_BUNDLE.md)
- Codebase Compliance Rules: [rules/codebase-compliance.md](../rules/codebase-compliance.md)
- Platform Independence Rules: [rules/platform-independence.md](../rules/platform-independence.md)
- LaTeX / KaTeX Integrity Invariants: [rules/latex-katex-integrity.md](../rules/latex-katex-integrity.md)
- Tracker Source of Truth & Non-Closure Rules: [rules/tracker-source-of-truth.md](../rules/tracker-source-of-truth.md)
- Check 32 Zero Shell Scripts Invariant Source: [crates/verify-baseline/src/checks/structural.rs](../crates/verify-baseline/src/checks/structural.rs)
