# Implementation Plan: Adversarial Code Auditor Execution & Upstream Defect Publication (`gintatkinson/DEAP01-spec-core`)

## 1. Objective & Scope Constraints
Execute `skills/adversarial-code-auditor/SKILL.md` (`PILLAR: Semantic Traceability`, `MODE: file-based`, `REPO: gintatkinson/DEAP01-spec-core`) across the 4 specified observable symptoms, write 4 compliant 7-section defect dossiers under `.pipeline/defects/`, and publish/verify each issue on `https://github.com/gintatkinson/DEAP01-spec-core/issues` via `./target/release/verify-baseline check-defect ... --publish`.

### Hard Constraints
- **Zero `crates/` Modifications**: Do not modify or revert any source files under `crates/`.
- **Single-Item Subagent Dispatches (`AGENTS.md:63-75`, `skills/adversarial-code-auditor/SKILL.md:17`)**: Each of the 4 symptoms is audited, written to `.pipeline/defects/<NAME>.md`, and published by its own dedicated, context-isolated `self` subagent. The coordinator collects the returned GitHub issue URLs and severities and never directly edits dossier body text. Every subagent is terminated/reclaimed immediately upon completion via `manage_subagents`.
- **12-Check Defect Dossier Compliance (`verify-baseline check-defect`)**:
  - Line 1 MUST start with `## 1. Context and References` (no `# ` H1 header).
  - Exact 7 section headers (`## 1. Context and References` through `## 7. Relationship to Existing Issues`) plus `## Audit Source` (`Adversarial Semantic Traceability Audit`, `SEVERITY: ...`, `FILE_LOCATION: <path>:<start>-<end>`).
  - Identical workspace-relative Rust test target path in both `<!-- test-target: crates/... -->` and `- **Test-Target**: \`crates/...\``.
  - Mandatory fenced ```` ```mermaid ```` diagram (`sequenceDiagram` or `classDiagram`) in `## 4. UML Diagrams` complying with Mermaid rules E1--E5 (no semicolons in `Note` or message text, no curly braces in `classDiagram` members, quoted angle brackets).
  - Workspace-relative paths only inside dossier text (no `/Users/...` absolute workstation paths) and zero Unicode em dashes (`--` ASCII only).

---

## 2. Work Packages

### Work Package 1: Symptom 1 -- Blueprint Markdown Translator Ingestion Gaps
- **Subagent Role**: `Adversarial Code Auditor -- ingest-sysml markdown.rs`
- **Target File (`FILE_PATH`)**: `crates/ingest-sysml/src/translators/markdown.rs`
- **Pillar / Mode / Repo**: `Semantic Traceability` | `file-based` | `gintatkinson/DEAP01-spec-core`
- **Target Dossier File**: `.pipeline/defects/AUDIT_INGEST_SYSML_MARKDOWN_TRANSLATOR.md`
- **Observable Symptoms to Audit & Trace**:
  1. Numbered functional section headings (e.g. `5.3 Acceptance Criteria`, `2.1 Operational Context`) are emitted as spurious `part def` blocks.
  2. Row-per-attribute component tables (`Component | Attribute | Type | Default / Bound | Description`) do not populate `PartDef.attributes` per row.
  3. Explicit constraint names in `Constraint Name` columns are ignored in favor of `assert_<param>_range`.
  4. Markdown backticks on interface `Port`, `Source`, and `Target` cells are not stripped, and `Latency Budget` columns are not parsed into `ConnectionDef.latency_ms`.
  5. Use-case tables using `Use Case ID` (`uc_id` / `use_case_id`), `Included Use Cases`, and `Verification Test Cases` columns, as well as multi-line numbered bullets (`1.`..`N.`) under `### UC-XX` prose subsections, are not extracted into `UseCaseDef`.
- **Verification & Publication Command**:
  ```bash
  ./target/release/verify-baseline check-defect .pipeline/defects/AUDIT_INGEST_SYSML_MARKDOWN_TRANSLATOR.md --repo "gintatkinson/DEAP01-spec-core" --title "[AUDIT] [markdown.rs]: Blueprint heading filter, row-per-attribute, constraint name, interface latency, and UC-XX extraction gaps" --publish
  ```

### Work Package 2: Symptom 2 -- SysML Serializer & Parser Grammar Gaps
- **Subagent Role**: `Adversarial Code Auditor -- compile-sysml serializer.rs & grammar.rs`
- **Target Files (`FILE_PATH`)**: `crates/compile-sysml/src/semantic/serializer.rs` and `crates/compile-sysml/src/parser/grammar.rs`
- **Pillar / Mode / Repo**: `Semantic Traceability` | `file-based` | `gintatkinson/DEAP01-spec-core`
- **Target Dossier File**: `.pipeline/defects/AUDIT_COMPILE_SYSML_SERIALIZER_GRAMMAR.md`
- **Observable Symptoms to Audit & Trace**:
  1. In `crates/compile-sysml/src/semantic/serializer.rs`, when `./target/release/compile-sysml --compile` serializes a source SysML schema into `.pipeline/schema.sysml`, `include use case <target>;` statements (`UseCaseDef.includes`) inside `use case def` blocks are dropped from the serialized `.pipeline/schema.sysml` output.
  2. In `crates/compile-sysml/src/parser/grammar.rs`, `port def` parsing rejects `flow in <name>: <Type>;` and `flow out <name>: <Type>;` syntax unless written as `in flow` / `out flow` (or requires symmetric handling of both token orderings).
- **Verification & Publication Command**:
  ```bash
  ./target/release/verify-baseline check-defect .pipeline/defects/AUDIT_COMPILE_SYSML_SERIALIZER_GRAMMAR.md --repo "gintatkinson/DEAP01-spec-core" --title "[AUDIT] [serializer.rs]: Dropped include use case serialization and port def flow in/out grammar asymmetry" --publish
  ```

### Work Package 3: Symptom 3 -- Forward Sync Omits Use Cases & User Stories and Overwrites Authored Epic/Feature Sections
- **Subagent Role**: `Adversarial Code Auditor -- compile-sysml forward.rs`
- **Target File (`FILE_PATH`)**: `crates/compile-sysml/src/sync/forward.rs`
- **Pillar / Mode / Repo**: `Semantic Traceability` | `file-based` | `gintatkinson/DEAP01-spec-core`
- **Target Dossier File**: `.pipeline/defects/AUDIT_COMPILE_SYSML_FORWARD_SYNC.md`
- **Observable Symptoms to Audit & Trace**:
  1. `crates/compile-sysml/src/sync/forward.rs` only projects `docs/epics/EPIC-XX.md` and `docs/features/FEAT-XX.md` from `Package` and `RequirementDef` AST nodes and never projects or updates `docs/use-cases/UC-XX.md` or `docs/user-stories/US-XX.md` from `UseCaseDef` (`model.use_cases`) and behavioral AST nodes, leaving downstream `docs/use-cases/` and GitHub tracker issues out of sync with `.pipeline/schema.sysml`.
  2. Re-running forward sync (`fs::write(&epic_path, content)` / `fs::write(&feat_path, content)`) unconditionally overwrites existing authored sections and checklists inside `docs/epics/EPIC-XX.md` and `docs/features/FEAT-XX.md`.
- **Verification & Publication Command**:
  ```bash
  ./target/release/verify-baseline check-defect .pipeline/defects/AUDIT_COMPILE_SYSML_FORWARD_SYNC.md --repo "gintatkinson/DEAP01-spec-core" --title "[AUDIT] [forward.rs]: Forward sync omits Use Case and User Story projection and overwrites authored Epic/Feature sections" --publish
  ```

### Work Package 4: Symptom 4 -- Baseline Verifier Passes (`exit code == 0`) When Schema Use Cases Lack Specification Files or Issues
- **Subagent Role**: `Adversarial Code Auditor -- verify-baseline parity_gates.rs & spec_audit/mod.rs`
- **Target Files (`FILE_PATH`)**: `crates/verify-baseline/src/checks/parity_gates.rs` and `crates/verify-baseline/src/spec_audit/mod.rs`
- **Pillar / Mode / Repo**: `Semantic Traceability` | `file-based` | `gintatkinson/DEAP01-spec-core`
- **Target Dossier File**: `.pipeline/defects/AUDIT_VERIFY_BASELINE_USE_CASE_PARITY.md`
- **Observable Symptoms to Audit & Trace**:
  1. When `.pipeline/schema.sysml` and `schema/*.sysml` declare `use case def` nodes (e.g., 9 Use Cases) while `docs/use-cases/` only contains a subset (e.g., 5 `UC-01`..`UC-05` files), `./target/release/verify-baseline .` exits `0` (`ALL CHECKS PASSED`) without enforcing 100% 1:1 parity between AST `UseCaseDef` declarations and `docs/use-cases/*.md` specification files.
- **Verification & Publication Command**:
  ```bash
  ./target/release/verify-baseline check-defect .pipeline/defects/AUDIT_VERIFY_BASELINE_USE_CASE_PARITY.md --repo "gintatkinson/DEAP01-spec-core" --title "[AUDIT] [parity_gates.rs]: Baseline verifier exits 0 when schema UseCaseDef nodes lack docs/use-cases specification files" --publish
  ```

---

## 3. Final Output Contract
Return the 4 published `https://github.com/gintatkinson/DEAP01-spec-core/issues/<N>` URLs with their corresponding severities and closed-loop verification status.
