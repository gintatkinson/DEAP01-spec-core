# Goal Description

Implement Work Package 4.3 to resolve Issue #300 (Scaffolding and Prompt Catalog) and Issue #295 (Raw document schema ingestion). 

## User Review Required

Please review this implementation plan. As required by the Strict Planning Gate, I must verify the implementation plan with you first before making code modifications, even if PROCEED was provided.

## Open Questions

None. The requirements are clear.

## Proposed Changes

### Issue #300 - Scaffolding and Prompt Catalog

#### [MODIFY] scripts/install_pipeline.sh
- **Path:** `/Users/perkunas/jail/DEAP01-spec-core/scripts/install_pipeline.sh`
- **Changes:** Update line 194's `mkdir -p` command to include `"$TARGET_DIR/docs/conops/units/conops"`, `"$TARGET_DIR/docs/conops/units/mission_intent"`, and `"$TARGET_DIR/docs/interfaces"`.

#### [NEW] docs/OPERATOR_PROMPT_CATALOG.md
- **Path:** `/Users/perkunas/jail/DEAP01-spec-core/docs/OPERATOR_PROMPT_CATALOG.md`
- **Changes:** Create a new markdown file containing all standardized operator usage prompts extracted and aligned with `README.md` Section 9 and `scripts/install_pipeline.sh` Section 4.2.
  - Section for Pipeline 0 (Workers 0A, 0B, 0C, 0D).
  - Section for Pipeline 1 (Workers 1A, 1B, 1C, 1D).
  - Section for Pipeline 2 and Synthesis Driver (Workers 2A, 2B).
  - These prompts will satisfy DEAP prompt integrity checks (`tests/test_prompt_catalog_integrity.py`).

### Issue #295 - Raw document schema ingestion

#### [MODIFY] skills/spec-orchestrator/scripts/sysmlv2_ingest.py
- **Path:** `/Users/perkunas/jail/DEAP01-spec-core/skills/spec-orchestrator/scripts/sysmlv2_ingest.py`
- **Changes:**
  - Define `RAW_EXTENSIONS = {".md", ".pdf", ".txt", ".doc", ".docx"}`.
  - In `detect_format`, check if the extension is in `RAW_EXTENSIONS` and return `"raw"`.
  - In `ingest_schema`, when `fmt == "raw"`, raise a structured, actionable SSOT reporting exception that names the raw document and indicates AST translation is required instead of raising the generic `ValueError: Unsupported schema format`.

#### [MODIFY] tests/test_sysmlv2_ingest.py
- **Path:** `/Users/perkunas/jail/DEAP01-spec-core/tests/test_sysmlv2_ingest.py`
- **Changes:** Add unit tests to verify that passing raw `.md` or `.txt` files to the ingestor yields the structured actionable raw document handling/reporting exception, ensuring the generic `ValueError` is not thrown.

## Verification Plan

### Automated Tests
- Run `python3 -m unittest tests/test_prompt_catalog_integrity.py`
- Run `python3 -m unittest discover -s tests -p "test_sysml*.py"`
- Ensure all tests pass.

### Commit and Tracker Updates
- When verified, commit using neutral reference `fix(scaffolding): add modular units, prompt catalog, and raw doc ingestion (refs #300, refs #295)`
- Push to `origin/main`
- Apply `status:fixed-resolved` label and post verification evidence comments to #300 and #295 on GitHub.
