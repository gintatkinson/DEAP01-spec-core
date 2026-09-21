# Implementation Plan: Level 0 OEM Prose/Markdown Ingestion Support & Pipeline 0 Sequence Remediation

## Goal Description
Implement Level 0 OEM prose/markdown ingestion support in `sysmlv2_ingest.py` and update the Operator Prompt Catalog / Pipeline 0 sequence in `DEAP01-spec-core` so that downstream customer projects starting with unstructured/prose manuals (markdown, PDF, BOM tables) have a sanctioned, deterministic path to generate `schema/model.sysml` and satisfy the compilation gate without deadlocking.

## User Review Required
> **Strict Planning Gate Mandate:** In accordance with `.agents/AGENTS.md` § *Strict Planning Gate*, `rules/user-authorization-lock.md`, and `.pipeline/constitution.md` § *Strict Planning Mode Gate*, execution and subagent dispatches are locked until this implementation plan is explicitly reviewed and approved by the user/parent sentinel. Please review the proposed work packages, subagent dispatches, and verification steps below.

---

## Proposed Changes

### Work Package 1: Level 0 Markdown / BOM Schema Ingestion in sysmlv2_ingest.py (R1)
- **Target Files:**
  - Create `skills/spec-orchestrator/scripts/translators/markdown_translator.py`
  - Modify `skills/spec-orchestrator/scripts/sysmlv2_ingest.py`
- **Action Details:**
  1. **Implement `MarkdownTranslator`:**
     - Develop `skills/spec-orchestrator/scripts/translators/markdown_translator.py` inheriting/following canonical translator patterns in `translators/`.
     - Implement parsing for structured Markdown tables (e.g., BOM tables, component lists, port/signal interfaces, and parametric range specifications).
     - Extract system components into canonical `PartDef` nodes with typed `AttributeDef` items (e.g. mass, power, dimensions, part numbers).
     - Extract interface definitions into `PortDef` nodes with directional flow typing (`in`, `out`, `inout`) and item types.
     - Extract parametric ranges and operational limits into `SysMLConstraintDef` assertion blocks (e.g., range inequalities, voltage/current limits, operational bounds).
     - Generate a valid `SysMLPackage` containing all parsed elements with full `to_sysml()` serialization compatibility.
  2. **Integrate into `sysmlv2_ingest.py`:**
     - Import `MarkdownTranslator`.
     - Update `detect_format()`:
       - When a file has `.md` extension or contains structured Markdown table headers (`| Component |`, `| Part |`, `| BOM |`, `| Port |`, `| Signal |`, `| Parameter |`, `| Interface |`), detect format as `"markdown"`.
       - Add `"markdown"` to supported CLI format choices in `--format`.
     - Update `ingest_schema()`:
       - Add handler for `fmt == "markdown"` dispatching to `MarkdownTranslator().translate(content_text, default_name=file_basename)`.
       - Support auto-discovery of markdown tables / BOM specifications in `schema/` and `schema/extracted/`.
       - Produce output canonical SysML v2 textual model (e.g. `schema/model.sysml` / `.pipeline/schema.sysml`) and generate `.pipeline/schema-digest.json`.

### Work Package 2: Operator Prompt Catalog & Pipeline 0 Sequence Remediation (R2)
- **Target Files:**
  - Modify `docs/OPERATOR_PROMPT_CATALOG.md`
  - Modify `README.md`
  - Modify `scripts/install_pipeline.sh`
  - Modify `skills/spec-orchestrator/SKILL.md`
- **Action Details:**
  1. **Formalize Step 0.0 in `docs/OPERATOR_PROMPT_CATALOG.md`:**
     - Add `### Worker 00: OEM Prose / BOM Ingestion & Model Synthesizer (Step 0.0)` under `## Section for "Pipeline 0"`.
     - Document the entrypoint for customer projects starting with prose/PDF documentation.
     - Clarify that extracting BOM and physical parameters into `schema/extracted/` and synthesizing `schema/model.sysml` is authorized under Check 23 and is the required precursor to running `compile_sysml.py --compile`.
     - Include full, unabridged context-isolated subagent prompt for Worker 00.
  2. **Update `README.md`:**
     - Update Section 9.1 (Pipeline 0 Prompts) to insert `Worker 00: OEM Prose / BOM Ingestion & Model Synthesis Prompt (Step 0.0)` prior to Worker 0A.
  3. **Update `scripts/install_pipeline.sh`:**
     - In the scaffolded downstream README template (around line 687+), update the Mermaid topology diagram to show Step 0.0 preceding Step 0:
       `Step00["Step 0.0: Level 0 OEM Ground Truth Ingestion (sysmlv2_ingest.py)"] --> Step0["Step 0: SysML Model Compilation Gate (compile_sysml.py --compile)"]`
     - Add Worker 00 to the downstream README prompt catalog.
  4. **Update `skills/spec-orchestrator/SKILL.md`:**
     - Update the sequence diagram in `## Multi-Agent Orchestration Lifecycle` to show Step 0.0 / Worker 00 for Level 0 OEM Ground Truth Ingestion.
     - Update `## Phase 0: Pre-Flight / Pre-computation` to formalize Step 0.0 as the entrypoint for customer projects starting with unstructured/prose manuals, citing Check 23 authorization.

### Work Package 3: Pipeline 0 Compilation Gate Fallback & Helpful Error Messages (R3)
- **Target Files:**
  - Modify `scripts/compile_sysml.py`
- **Action Details:**
  - In `enforce_pipeline0_compilation_gate()` and schema discovery routines:
  - When no `.sysml` file is found in `schema/`, replace generic or unhelpful errors with clear, actionable remediation guidance:
    ```
    Error: No .sysml schema file found in schema/.
    If starting from unstructured OEM prose manuals, PDF documentation, or BOM markdown tables:
      1. Place your OEM documentation or extract tables into schema/ or schema/extracted/.
      2. Execute Step 0.0 Level 0 OEM Ground Truth Ingestion:
         python3 skills/spec-orchestrator/scripts/sysmlv2_ingest.py --schema <path_to_markdown> --format markdown --out schema/model.sysml
      3. Re-run compile_sysml.py --compile to satisfy the compilation gate.
    ```
  - Ensure return code remains 1 (fail-closed) while providing unambiguous developer instructions.

### Work Package 4: Verification Suite & Baseline Conformance
- **Target Files:**
  - Create `tests/test_sysmlv2_markdown_ingest.py`
- **Action Details:**
  - Add comprehensive unit tests in `tests/test_sysmlv2_markdown_ingest.py` covering:
    - Ingestion of markdown tables representing components/BOMs, port interfaces, and parametric constraints.
    - Translation into canonical `SysMLPackage`, `PartDef`, `PortDef`, `AttributeDef`, and `SysMLConstraintDef`.
    - Verification that generated `.sysml` content parses successfully with `SysMLParser.parse_text()` and `compile_sysml.py`.
    - Auto-detection of `--format markdown` for markdown files.
    - CLI test: `sysmlv2_ingest.py --schema <fixture> --format markdown --out <temp_out>`.
    - Verify that `compile_sysml.py` outputs the actionable remediation message when no `.sysml` file exists.

---

## Subagent Delegation & Execution Strategy
Following `.agents/AGENTS.md` § *Mandatory Subagent Dispatch for Research, Specification & Implementation Loops* and orchestrator dispatch-only constraints:
1. **Explorer Subagents (`teamwork_preview_explorer`)**:
   - Inspect existing translators, CLI argparsing, and prompt catalogs to establish precise line ranges and integration hooks.
2. **Worker Subagents (`teamwork_preview_worker`)**:
   - Worker 1: Implement `skills/spec-orchestrator/scripts/translators/markdown_translator.py` and integrate into `sysmlv2_ingest.py` (WP1).
   - Worker 2: Update `docs/OPERATOR_PROMPT_CATALOG.md`, `README.md`, `scripts/install_pipeline.sh`, and `skills/spec-orchestrator/SKILL.md` (WP2).
   - Worker 3: Update `scripts/compile_sysml.py` error handling and remediation guidance (WP3).
   - Worker 4: Create `tests/test_sysmlv2_markdown_ingest.py` and run test suite (WP4).
3. **Reviewer Subagents (`teamwork_preview_reviewer`)**:
   - Reviewer 1 & 2: Review all code changes for SysML v2 AST fidelity, schema-driven compliance (zero hardcoded domain concepts), shell script quotation, and clean Markdown code blocks.
4. **Challenger Subagents (`teamwork_preview_challenger`)**:
   - Challenger 1 & 2: Execute empirical stress tests with complex nested markdown tables, edge-case table formatting, missing columns, and empty schema directories.
5. **Forensic Auditor Subagent (`teamwork_preview_auditor`)**:
   - Run full integrity audit verifying zero mocks, genuine AST generation, valid error handling, and clean baseline conformance.

---

## Verification Plan

### Automated Tests
- Run new unit tests:
  `python3 -m unittest tests/test_sysmlv2_markdown_ingest.py`
- Test sample markdown ingestion into SysML v2 AST and compile verification:
  `python3 skills/spec-orchestrator/scripts/sysmlv2_ingest.py --schema <sample_markdown> --format markdown --out .pipeline/schema.sysml`
  `python3 scripts/compile_sysml.py --compile`
- Run baseline verification:
  `python3 scripts/verify_downstream_baseline.py --no-domain`
- Verify markdown code blocks:
  - Assert zero unescaped parentheses in shell comments.
  - Assert zero unquoted angle-bracket placeholders in shell code fences.

### Manual / Integration Verification
- Inspect `git diff` against `origin/main` to confirm clean, precise diffs conforming to all repository invariants.
