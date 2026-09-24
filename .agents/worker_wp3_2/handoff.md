# Handoff Report — Work Package 3 (Requirement R3)

## 1. Observation
- `scripts/compile_sysml.py:4094-4097` previously handled missing schemas in `enforce_pipeline0_compilation_gate()` by searching `glob.glob("schema/*.sysml")` and printing:
  ```
  Error: No schema file provided and none found in schema/
  ```
- Running `python3 scripts/compile_sysml.py --compile` in the upstream clean repository (where `schema/` only contains `.gitkeep`) produced that generic error with exit code 1, offering no guidance on how downstream projects or agents starting with unstructured prose/BOM manuals should proceed.
- Line 4101 in `scripts/compile_sysml.py` printed `Error: Schema file does not exist: {schema_path}` when a specific schema path was passed but missing.
- Other entrypoints (`target_file` at line 4244, `reverse_sync` at line 4213, `forward_sync` at line 4223, and `transpile_stpa` at line 5038) printed generic file-not-found errors or raised `FileNotFoundError` without remediation instructions when `schema/*.sysml` was absent.

## 2. Logic Chain
1. Requirement R3 specifies that when no `.sysml` schema file is found in `schema/` (or default path), generic errors or unhandled `FileNotFoundError` must be replaced with clear, helpful remediation instructions directing the user or agent to Step 0.0 Level 0 OEM Ground Truth Ingestion (`skills/spec-orchestrator/scripts/sysmlv2_ingest.py`).
2. Defined `SCHEMA_REMEDIATION_MESSAGE` in `scripts/compile_sysml.py:4080` verbatim matching the specified prompt:
   ```
   Error: No .sysml schema file found in schema/.
   If starting from unstructured OEM prose manuals, PDF documentation, or BOM markdown tables:
     1. Place your OEM documentation or extract tables into schema/ or schema/extracted/.
     2. Execute Step 0.0 Level 0 OEM Ground Truth Ingestion:
        python3 skills/spec-orchestrator/scripts/sysmlv2_ingest.py --schema <path_to_markdown> --format markdown --out schema/model.sysml
     3. Re-run compile_sysml.py --compile to satisfy the compilation gate.
   ```
3. Updated `enforce_pipeline0_compilation_gate()` in `scripts/compile_sysml.py:4086-4120` so that:
   - If `schema_path is None`, it checks `glob.glob("schema/*.sysml")` (and relative to `PROJECT_ROOT`). If empty, it prints `SCHEMA_REMEDIATION_MESSAGE` to `sys.stderr` and returns `1`.
   - If `schema_path` is explicitly passed but does not exist, and no `.sysml` exists in `schema/`, it prints `SCHEMA_REMEDIATION_MESSAGE` to `sys.stderr` and returns `1`. If other `.sysml` files do exist in `schema/`, it prints `Error: Schema file does not exist: {schema_path}` and returns `1`.
4. Updated CLI main routines in `scripts/compile_sysml.py`:
   - Positional missing schema file handler checks if schema files are absent in `schema/` and outputs `SCHEMA_REMEDIATION_MESSAGE` to `sys.stderr` with exit code 1.
   - `--reverse-sync` and `--forward-sync` exception handling catches missing schema `FileNotFoundError` and outputs `SCHEMA_REMEDIATION_MESSAGE` to `sys.stderr` with exit code 1 if `schema/` has no `.sysml` files.
   - `transpile_stpa()` outputs `SCHEMA_REMEDIATION_MESSAGE` to `sys.stderr` with return code 1 if schema file does not exist and `schema/` has no `.sysml` files.
5. In all failure cases, the gate remains strictly fail-closed (`return 1` / `sys.exit(1)`).

## 3. Caveats
- No caveats. Upstream template clean landing zones (`schema/.gitkeep`) are strictly preserved.
- No changes made outside exclusively owned file `scripts/compile_sysml.py`.

## 4. Conclusion
Work Package 3 (Requirement R3) is complete. The compilation gate and schema resolution paths in `scripts/compile_sysml.py` now provide clear, actionable remediation instructions pointing developers and subagents to Step 0.0 (`sysmlv2_ingest.py`) whenever `schema/` contains no `.sysml` models, while strictly preserving the fail-closed exit code 1 contract.

## 5. Verification Method
1. Run compilation gate in absence of schema files:
   ```bash
   python3 scripts/compile_sysml.py --compile
   ```
   Confirm exit code is 1 and stderr contains the exact 6-line remediation message:
   ```
   Error: No .sysml schema file found in schema/.
   If starting from unstructured OEM prose manuals, PDF documentation, or BOM markdown tables:
     1. Place your OEM documentation or extract tables into schema/ or schema/extracted/.
     2. Execute Step 0.0 Level 0 OEM Ground Truth Ingestion:
        python3 skills/spec-orchestrator/scripts/sysmlv2_ingest.py --schema <path_to_markdown> --format markdown --out schema/model.sysml
     3. Re-run compile_sysml.py --compile to satisfy the compilation gate.
   ```
2. Verify explicit non-existent schema invocation:
   ```bash
   python3 scripts/compile_sysml.py --compile --schema schema/model.sysml
   ```
   Confirm exit code is 1 and stderr contains the remediation message.
3. Verify compilation syntax and baseline verification suite:
   ```bash
   python3 -m py_compile scripts/compile_sysml.py
   python3 scripts/verify_downstream_baseline.py --no-domain
   ```
   Confirm all checks pass with exit code 0.
