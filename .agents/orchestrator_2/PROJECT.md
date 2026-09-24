# Project: Level 0 OEM Prose/Markdown Ingestion Support & Pipeline 0 Sequence Remediation

## Architecture
Deterministic Level 0 OEM Ground Truth Ingestion architecture providing downstream customer projects starting with unstructured/prose manuals (markdown, PDF, BOM tables) a sanctioned, deterministic path to generate `schema/model.sysml` and satisfy the compilation gate:
- Level 0 Markdown/BOM Ingestion: `skills/spec-orchestrator/scripts/translators/markdown_translator.py` and `skills/spec-orchestrator/scripts/sysmlv2_ingest.py`
- Step 0.0 Governance & Operator Guidance: `docs/OPERATOR_PROMPT_CATALOG.md`, `README.md`, `scripts/install_pipeline.sh`, and `skills/spec-orchestrator/SKILL.md`
- Gate Remediation Guidance: `scripts/compile_sysml.py`
- Verification Suite: `tests/test_sysmlv2_markdown_ingest.py` and `scripts/verify_downstream_baseline.py`

## Feature Inventory
| # | Feature | Description | Milestone | Source |
|---|---------|-------------|-----------|--------|
| 1 | R1: Markdown / BOM Translator | Parse structured Markdown tables into canonical SysML v2 AST (packages, part defs, port defs, constraints) | M1 | ORIGINAL_REQUEST § R1 |
| 2 | R1: CLI Ingestion Integration | Extend sysmlv2_ingest.py with --format markdown and auto-detection for markdown/BOMs in schema/ and schema/extracted/ | M1 | ORIGINAL_REQUEST § R1 |
| 3 | R2: Operator Prompt Catalog & Pipeline 0 Sequence | Update prompt catalog across docs/OPERATOR_PROMPT_CATALOG.md, README.md, scripts/install_pipeline.sh, and SKILL.md for Step 0.0 | M2 | ORIGINAL_REQUEST § R2 |
| 4 | R3: Helpful Compilation Gate Error Messages | Update compile_sysml.py to emit actionable remediation instructions pointing to Step 0.0 / sysmlv2_ingest.py | M3 | ORIGINAL_REQUEST § R3 |
| 5 | Verification & Conformance | Add unit tests in tests/, verify sysmlv2_ingest.py CLI output parseable by compile_sysml.py, and verify downstream baseline | M4 | ORIGINAL_REQUEST § Acceptance Criteria |

## Milestones
| # | Name | Scope | Dependencies | Status |
|---|------|-------|-------------|--------|
| 1 | M1: R1 Markdown/BOM Ingestion | skills/spec-orchestrator/scripts/translators/markdown_translator.py, skills/spec-orchestrator/scripts/sysmlv2_ingest.py | none | DONE |
| 2 | M2: R2 Prompt Catalog & Pipeline 0 | docs/OPERATOR_PROMPT_CATALOG.md, README.md, scripts/install_pipeline.sh, skills/spec-orchestrator/SKILL.md | none | DONE |
| 3 | M3: R3 Gate Fallback & Error Messages | scripts/compile_sysml.py | none | DONE |
| 4 | M4: Verification Suite & Baseline Conformance | tests/test_sysmlv2_markdown_ingest.py, scripts/verify_downstream_baseline.py | M1, M2, M3 | DONE |

## Interface Contracts
### markdown_translator.py ↔ sysmlv2_ingest.py
- `MarkdownTranslator.translate(content: str, default_name: str = "Markdown_Package") -> SysMLPackage`
- `detect_format` identifies markdown files via extension (`.md`) or content analysis (presence of markdown table headers like `| Component |`, `| BOM |`, `| Part |`, `| Port |`, `| Signal |`, `| Parameter |`, `| Interface |`).
- Ingested package serializes via `pkg.to_sysml()` into valid textual SysML v2 parseable by `SysMLParser.parse_text()` and `compile_sysml.py`.

### compile_sysml.py ↔ sysmlv2_ingest.py / Step 0.0
- When no `.sysml` file is found in `schema/`, `compile_sysml.py` outputs actionable error pointing to:
  `python3 skills/spec-orchestrator/scripts/sysmlv2_ingest.py --schema <path_to_markdown_or_bom> --format markdown --out schema/model.sysml`
