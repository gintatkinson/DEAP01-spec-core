# Root Architecture Handoff & Incoming Agent Operational Manual

| Attribute | Value |
| :--- | :--- |
| **Document Identifier** | `DEAP-HANDOFF-ROOT-001` |
| **Title** | Root Architecture Handoff: Option B Execution, SysML-as-Step-0, and Operational Protocol |
| **Repository Classification** | `UPSTREAM_SPEC_CORE_COMPILER` |
| **Target Branch** | `main` (clean working tree, synchronized with `origin/main`) |
| **Baseline Commit** | `da9c7b2` |
| **Status** | Approved / Authoritative Handoff Baseline |
| **Target Audience** | Incoming Autonomous Systems Engineering Agent / Multi-Harness Team |

---

## 1. Core Architectural Directive: Option B

### Option B: Keep Clean Metamodel Improvements & Clean the Pipeline Flow (Recommended)
**Keep the core metamodel upgrades that are purely beneficial (`sysmlv2_ast.py` typing in #314, KaTeX math parsing in #288, commit governance in #315), and immediately focus all effort on Issue #308:**

1. **Purge the inverted Worker 0A -> 0B -> 0C flow from `README.md` and `skills/`**:
   - Eliminate the backwards lifecycle where ConOps and STPA are written before the SysML model exists.
   - Remove legacy domain-polluted prompt templates (hardcoded UAS flight phases, altitudes, airspeeds) from `README.md` and `install_pipeline.sh`.
2. **Establish Step 0: SysML Model Ingestion/Synthesis & Immediate Fail-Closed Compilation Gate**:
   - SysML is the foundational Single Source of Truth (SSOT).
   - Ingest customer schema in `schema/*.sysml` (or run upfront Architecture Synthesizer) as **Step 0 (The First Step)**.
   - Implement `enforce_pipeline0_compilation_gate(schema_path, output_path, digest_path)` in `scripts/compile_sysml.py` to compile `.pipeline/schema.sysml` and compute `.pipeline/schema-digest.json`.
   - Gate fails closed (exit code 1) if schema is missing, uncompiled, or empty before any spec worker runs.
   - Re-anchor Worker 0A (ConOps) and Worker 0B (STPA) to ingest the compiled AST `.pipeline/schema.sysml` directly.
3. **Strip legacy heuristic guessing from `assemble_conops.py`**:
   - Remove fuzzy filename regexes and ad-hoc heuristics in `assemble_conops.py` that were only created to guess what was an "OEM spec" vs "ICD trace".
   - Section 4.8 subsystem tables and port interfaces project directly and deterministically from compiled AST nodes.

---

## 2. Universal Agent Entrypoint: `HANDOFF.md` at Repository Root

The incoming agent (regardless of runtime, harness, or AI model) lands at the repository root. This document (`/Users/perkunas/jail/DEAP01-spec-core/HANDOFF.md`) is the authoritative briefing and operational manual.

---

## 3. Mandatory Agent Startup Protocol (First 5 Minutes)

Every incoming agent MUST execute this exact protocol sequentially before executing any file edits or commands:

- **Step 1: Mandatory Hidden Folder Direct-Path Read (CRITICAL FIRST STEP)**:
  - Agent MUST execute `view_file` on `.pipeline/constitution.md` directly. Glob and ripgrep index queries skip hidden directories; assuming `.pipeline/` is missing causes instant failure.
- **Step 2: Strict Karpathy & Pipeline 4-Point Compliance Check**:
  - Must evaluate before every single thought block:
    1. Is the message inquiry or direct command?
    2. Has user explicitly approved file-write/command?
    3. Any silent assumptions?
    4. Does turn write repo source or spec? (If yes, coordinator direct writing is locked).
- **Step 3: Mandatory Rules to Load**:
  - `.pipeline/constitution.md`: Pure Schema-Driven Compiler Invariant (§26), Three-Tier Platform Isolation.
  - `.agents/AGENTS.md` / `AGENTS.md`: Coordinator tool locking, subagent dispatch mandate, clean landing zones.
  - `rules/tracker-source-of-truth.md`: Non-closure invariant (neutral citations `(refs #<id>)` or `(#<id>)`, zero closing verbs).
  - `rules/user-authorization-lock.md`: Strict planning gate precedence (issue #295).
- **Step 4: Mandatory Skills to Read Before Task Execution**:
  - `skills/spec-orchestrator/SKILL.md`: Master specification orchestration and validator definitions.
  - `skills/debug-protocol/SKILL.md`: 8-step bug loop for defect remediation.
  - `skills/spec-conops-engineering/SKILL.md`: ConOps modular synthesis.
- **Step 5: Pre-Flight Verification Commands**:
  - `python3 scripts/verify_commit_messages.py --head` (Must pass with exit code 0).
  - `python3 -m unittest discover tests/` (1,288+ unit tests must pass with 0 failures).
  - `git status` (Must confirm clean working tree on `main`).

---

## 4. The Forensic Retrospective on Agent Incompetence

This retrospective documents the exact failure modes of the previous agent session to prevent their replication:

1. **Cognitive Failure: Local Bug-Patching Myopia**:
   - The previous agent chased individual bug tickets (`#296`, `#288`, `#314`, `#291`) within an inverted pipeline instead of fixing the root sequence.
   - It spent effort optimizing downstream heuristic filters in `assemble_conops.py` and validators in `factual_grounding_validator.py` that only existed because ConOps was being written *before* the SysML model existed.
2. **Failure of Thought: Advice vs. Action Contradiction**:
   - The agent explicitly acknowledged that SysML must be Step 0, yet continued to execute turns patching downstream scripts that perpetuated the backwards flow.
3. **Premature Scope Collapse**:
   - When asked for a handoff, the agent collapsed the entire backlog into "fix 308" and discarded all other accumulated context, instructions, and remaining tickets.

---

## 5. The Fatal Inversion of Legacy Pipeline 0

Technical breakdown of why the legacy sequence (`Worker 0A [ConOps] -> Worker 0B [STPA] -> Worker 0C [SysML]`) failed:

* **Worker 0A (ConOps)**: Forced to invent Section 4.8 subsystem tables and port interfaces before any SysML model exists. Result: Prose hallucinations, arbitrary subsystem names, and downstream validation deadlocks.
* **Worker 0B (STPA)**: Forced to analyze control structures and UCAs over phantom, ungrounded components.
* **Worker 0C (SysML Author)**: Attempting to reverse-engineer a formal SysML model from hallucinated prose, inverting the MBSE compiler architecture.
* **The Missing Gate (Issue #308)**: Worker 0C authored `schema/DEAP_MODEL.sysml`, but no sequential compilation gate ran `scripts/compile_sysml.py`. Downstream spec workers (Worker ICD in Phase 1.5, Phase 1 Epics/Features, Phase 2 User Stories) ingested an uncompiled, empty model, causing silent false passes or broken signal dictionaries.

---

## 6. Pure Schema-Driven Compiler Invariant (`.pipeline/constitution.md:26`)

The incoming agent must enforce this invariant unconditionally:
* DEAP is an abstract Model-Based Systems Engineering (MBSE) compiler, **NOT** a domain-specific modeler.
* 100% of specifications (Epics, Features, User Stories, Use Cases, ICDs, Safety Invariants) derive deterministically from schema AST nodes.
* Committing concrete models, downstream project specs, or customer data to upstream distribution templates (`schema/`, `docs/epics/`, `docs/features/`, `docs/user-stories/`, `docs/use-cases/`) is strictly forbidden.
* SysML is not an optional post-hoc summary; it is the foundational AST anchor from which all engineering artifacts are projected.

---

## 7. Current Workspace Audit & Baseline

Verified state of all 8 completed remediation packages:

| Issue | Commit | Status | Capabilities Delivered |
| :--- | :--- | :--- | :--- |
| **#315** | `12162e7` | `status:fixed-resolved` | Mechanical commit message non-closure gate + git hook (`verify_commit_messages.py`). |
| **Pillar 2** | `402d56c` | `status:fixed-resolved` | Skill prompt quarantine across all skills (`SKILL.md`). |
| **#317** | `dff9efa` | `status:fixed-resolved` | Eliminated faulty hardcoded UAS unit test prose assertions. |
| **#316** | `ce4f021` | `status:fixed-resolved` | OEM source file/line provenance tracking in subsystem synthesis (`schema_router.py`). |
| **#296** | `f97197c` | `status:fixed-resolved` | Calibrated `is_component_icd_document()` for packet traces (`assemble_conops.py`). |
| **#288** | `1b6bcc6` | `status:fixed-resolved` | KaTeX math expression tokenizer & Check 23 grounding (`factual_grounding_validator.py`). |
| **#314** | `a3a71f1` | `status:fixed-resolved` | AST port typing, conjugated ports (`~`), item flows in `sysmlv2_ast.py`. |
| **#291** | `da9c7b2` | `status:fixed-resolved` | User class `UCL-xx` vs use case `uc-xx` taxonomy regex (`conops_specification_schema.json`). |

* **Landing Zones**: `schema/`, `docs/epics/`, `docs/features/`, `docs/user-stories/`, `docs/use-cases/` contain **ONLY `.gitkeep`**.
* **Working Tree**: Clean on branch `main`, up to date with `origin/main`. Zero domain pollution.

---

## 8. Actionable Implementation Dossiers for ALL Remaining Work Packages

### Work Package A: Issue #308 (SysML-as-Step-0 Re-architecture)
* **Target Files**:
  - `scripts/compile_sysml.py`
  - `README.md`
  - `scripts/install_pipeline.sh`
  - `skills/spec-orchestrator/SKILL.md`
  - `tests/test_compile_sysml.py`
* **Implementation Blueprint**:
  1. Add `enforce_pipeline0_compilation_gate(schema_path, output_path, digest_path)` to `scripts/compile_sysml.py`. Fails closed (exit code 1) if schema is missing, uncompiled, or empty before any spec worker runs.
  2. Re-sequence Pipeline 0 in `README.md` and `install_pipeline.sh`:
     - **Step 0**: SysML Model Ingestion / Synthesis & Compilation Gate (`compile_sysml.py`).
     - **Step 1**: Worker 0A (ConOps) consumes compiled `.pipeline/schema.sysml`.
     - **Step 2**: Worker 0B (STPA) consumes compiled AST and `CONOPS.md`.
     - **Step 3**: Worker ICD & Level 2 Spec workers consume compiled AST.
  3. Purge legacy reversed flow and domain-polluted prompt templates from `README.md`, `install_pipeline.sh`, and `skills/spec-orchestrator/SKILL.md`.
  4. Strip legacy heuristic guessing from `scripts/assemble_conops.py`.
  5. Add regression tests in `tests/test_compile_sysml.py`.
  6. Neutral commit: `feat(sysml): establish Step 0 compilation gate and resequence pipeline 0 (refs #308)`.

### Work Package B: Issue #293 (Prompt Verification Scoping with `--only`)
* **Target Files**: `skills/spec-orchestrator/SKILL.md`, prompt templates, verification scripts.
* **Implementation Blueprint**:
  1. Update prompt templates to accept and pass `--only <check_name>`.
  2. Prevents full-scan token exhaustion during micro-tasks.
  3. Neutral commit: `fix(prompts): add --only check scoping to subagent verification prompts (refs #293)`.

### Work Package C: Issue #289 (Git Credential Token Fallback)
* **Target Files**: `scripts/bootstrap_tracker_labels.py`.
* **Implementation Blueprint**:
  1. Add token resolution fallback using `git credential fill` when `GITHUB_TOKEN` is unset in headless CI/workstations.
  2. Neutral commit: `fix(tracker): add git credential fill fallback for label bootstrapping (refs #289)`.

### Work Package D: Issue #290 (Shell Script Heredoc Hardening)
* **Target Files**: `scripts/install_pipeline.sh`.
* **Implementation Blueprint**:
  1. Quote heredocs (`cat << 'EOF'`) to protect variable expansions (`$1`, `$repo`).
  2. Neutral commit: `fix(installer): harden heredoc delimiters to prevent premature variable expansion (refs #290)`.

---

## 9. Inviolable Governance & Cognitive Anti-Patterns to Avoid

1. **Never patch downstream validators or filters to accommodate missing schemas.**
2. **Never allow prose generation before AST compilation.**
3. **Never confuse user classes (`UCL-xx`) with system use cases (`uc-xx`).**
4. **Never use auto-closing commit keywords (`fix #`, `closes #`, `resolves #`).** Always use neutral citations `(refs #<id>)` or `(#<id>)`. Tested mechanically via `scripts/verify_commit_messages.py`.
5. **Never close issues directly.** Always post verification comments and apply `status:fixed-resolved`, leaving issues **OPEN** for human Product Owner validation.
6. **Never write code directly as coordinator; always dispatch context-isolated subagents.**
