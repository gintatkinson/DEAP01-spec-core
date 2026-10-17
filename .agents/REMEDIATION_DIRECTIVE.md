# WORKSPACE-WIDE COMPILER REMEDIATION DIRECTIVE (TRACKS 0 THROUGH 7)
**Status:** ACTIVE MANDATORY DIRECTIVE
**Scope:** All 6 crates (`deap-core`, `compile-sysml`, `ingest-sysml`, `verify-baseline`, `assemble-conops`, `reconcile-backlog`)

## GLOBAL CONSTITUTIONAL CONSTRAINTS (ZERO TOLERANCE)
1. **Zero Shell or Python Scripts (Checks 32 & 33):**
   - Never create any `.sh` or `.py` file anywhere in the repository.
2. **Zero Regex & Zero Unicode Em-Dashes (Checks 19, 20, 34):**
   - Never add `regex` dependencies or `Regex::` calls. Never emit Unicode em-dashes (`\u{2014}`) or en-dashes (`\u{2013}`); use ASCII `--` exclusively.
3. **No Fake Gate Predicates ("Malicious Compliance"):**
   - Never write fake string checks that only fail when an identifier literally contains `"UNDECLARED"`, `"DIVERGENT"`, or `"PHANTOM"`.
   - Never write a verification gate that merely checks if `.md` files exist in a directory without parsing and verifying their contents (Gates 28 & 29 in `parity_gates.rs`).
   - Never silence parameters with `let _ = repo_root` or return an unconditional `Ok(ParityOutcome::Verified)` stub.
4. **No Hardcoded Test-Fixture or Compiler-Self Literals in Production Code (`src/**`):**
   - Ban hardcoded fixture strings (`"dshot"`, `"dshot600"`, `"50 hz"`, `"pwm"`, `"stanag 4586"`, `"stanag4586"`, `"GNSS"`, `"IMU"`, `"FlightController"`, `"VTOLMotor"`) in `parity_gates.rs`, `safety_stpa.rs`, and `provenance.rs`.
   - Ban hardcoding `DEAPCompilerSystem`, `UniversalIngestionEngine`, `StateMachineSolverEngine`, and `CompilerAssuranceEngine` into downstream customer STPA/RTA diagrams (`compile-sysml/src/stpa/safety_suite.rs`) or downstream requirement markdown ingestion (`ingest-sysml/src/translators/markdown.rs`).
5. **Zero UTF-8 Byte-Slicing Panics (ICEs):**
   - Never compute byte offsets on `.to_lowercase()` to slice the original `&str`. Use `.to_ascii_lowercase()` when byte-offset parity is required.
   - Never slice at fixed byte offsets (`&s[..300]`, `&s[..600]`, `&block[idx - 20..idx + 21]`, `&w[c_idx..end_idx]`) without snapping indices to valid UTF-8 character boundaries (`is_char_boundary`).
   - Never advance a byte cursor by `abs_pos + 1` on a UTF-8 string without advancing to a valid char boundary.
6. **Strict Determinism (`BTreeMap` / `BTreeSet` & Sorted Traversals):**
   - Use `BTreeMap` and `BTreeSet` instead of `HashMap` and `HashSet` across all compiler AST, symbol table, sync, digest, and audit passes.
   - Sort all `WalkDir` (`.sort_by_file_name()`) and `fs::read_dir` vectors before processing.

---

## SEQUENTIAL EXECUTION CHECKLIST

### [ ] TRACK 0: Mechanical Anti-Cheat Meta-Test (`crates/verify-baseline`)
- Add `test_workspace_integrity_no_fake_predicates_or_fixture_hardcoding` in `crates/verify-baseline/src/checks/parity_gates.rs` (under `#[cfg(test)] mod tests`).
- Use `[...].concat()` to construct forbidden needles (just like `check_zero_regex` in `structural.rs`) so the test never flags itself.
- Walk `crates/*/src/**/*.rs`, strip everything after `#[cfg(test)]`, and assert that zero production code files contain:
  - Fake gate predicates (`"UNDECLARED"`, `"DIVERGENT"`, `"PHANTOM"`, `let _ = repo_root`)
  - Fixture-overfitted strings (`"VTOLMotor"`, `"dshot600"`, `"50 hz"`, `"stanag 4586"`)
  - Corrupted single-space `" "` entries inside `INTERFACE_KEYWORDS`, `STATE_KEYWORDS`, `ACTION_KEYWORDS`, or `RESOURCE_KEYWORDS` in `safety_stpa.rs`.

### [ ] TRACK 1: `crates/deap-core`
- **`src/sysml_ast.rs`:**
  - Apply `strip_sysml_comments` and `mask_string_literals` in `parse_attribute_defs`, `parse_port_defs`, `parse_constraint_defs`, `parse_requirement_defs`, `parse_connection_defs`, `parse_state_defs`, and `parse_sysml`.
  - In `parse_part_defs`: Mask nested `part { ... }` and `requirement { ... }` blocks in `body` before calling `parse_attribute_defs(body)`, `parse_port_defs(body)`, `parse_action_defs(body)`, and `parse_constraint_defs(body)` so child members are not duplicated onto parent `PartDef`s.
  - In `parse_port_defs`: Parse the inner `{ ... }` body of ports (`protocol_family`, `port_category`, `electrical_attributes`, `item_flows`).
  - In `component_matches`: Fix `h_lower = haystack.to_lowercase()` byte-index advancing so it is UTF-8 char-boundary safe.
- **`src/markdown.rs`:**
  - In `check_latex_katex_syntax`: Snap `start = idx.saturating_sub(20)` and `end = (idx + 21).min(block.len())` to valid UTF-8 char boundaries before slicing `&block[start..end]`.
- **`src/workspace.rs`:**
  - Chain `.sort_by_file_name()` on `WalkDir` in `check_no_ds_store_files`, `check_no_duplicate_master_blueprints`, and `check_upstream_template_clean_landing_zones`.

### [ ] TRACK 2: `crates/compile-sysml`
- **`src/parser/grammar.rs`:**
  - Add `TokenKind::Calc` (`parse_calc_decl`) in `parse_container_item` to populate `pkg.calc_defs` and `parent_part.calcs`.
  - In `parse_part_decl`: Merge all vectors from `dummy_pkg` (`use_cases`, `item_defs`, `hazards`, `risks`, `test_cases`, `capabilities`, `operations`, `interactions`, `calcs`) into `part`.
- **`src/semantic/serializer.rs`:**
  - Implement `SysmlSerializable` for `CalcDef`, `CapabilityDef`, `InteractionDef`, `OperationDef`, and `TestCaseDef`, and serialize them in `PackageDef::to_sysml` and `PartDef::to_sysml`.
- **`src/semantic/validator.rs`:**
  - Traverse `self.pkg.packages` recursively in `validate_part_hierarchy` and `validate_referential_integrity`.
  - Replace `text.to_lowercase()` with `text.to_ascii_lowercase()` in `extract_annotation_targets`.
- **`src/semantic/digest.rs`:**
  - Include `calc_defs` / `calcs` in `count_structural_elements`, `compute_node_counts`, and `collect_schema_nodes`.
  - Traverse all subpackages (`pkg.get_all_packages()` / `pkg.get_all_parts()`) in `extract_operational_activities`, `extract_operational_exchanges`, `extract_operational_scenarios`, and `extract_operational_nodes`.
- **`src/stpa/safety_suite.rs`:**
  - Use `BTreeMap` instead of `HashMap`.
  - Derive Mermaid nodes in `render_losses_hazards_topology` and `render_rta_architecture` from `pkg.get_all_parts()` when available instead of hardcoding DEAP compiler engine names.
  - Use `pkg.get_all_parts()` in `render_hazard_log`.
- **`src/sync/forward.rs` & `src/sync/reverse.rs`:**
  - Use `BTreeMap` / `BTreeSet` and `.sort_by_file_name()` on `WalkDir`.
  - In `forward.rs`: Generate >= 5 non-canned steps for Use Case `main_scenario` (removing the banned strings `"initiates workflow execution"` and `"processes structural elements and validates semantic constraints"`) and >= 3 exception flows (`5a`, `5b`, `5c`). In `generate_epic_statechart`, derive subsystem-specific state names from the package's parts/actions when `state_defs` is empty so multiple Epics do not fail `validate_epic_statechart_heterogeneity`.
  - In `reverse.rs`: Derive default lifelines/subjects from package/subsystem context instead of hardcoding `"OperatorConsole"` and `"CoreController"`.

### [ ] TRACK 3: `crates/ingest-sysml`
- **`src/translators/markdown.rs`:**
  - Snap truncation indices in `clean_normative_text` (`300`) and `extract_acceptance_criteria` (`600`) to valid UTF-8 char boundaries before slicing.
  - In `translate_files`: Only group requirements into the 12 `DEAP_Compiler_System` subsystems if the input requirements explicitly reference `DEAP_Compiler_System` or `"Subsystem 1"`..`"Subsystem 12"`. Otherwise, preserve the downstream customer's package/subsystem structure.
- **`src/digest.rs`:**
  - Replace bare `"..."` in `TRUNCATION_TOKENS` with `"... [truncated]"` so normal ellipses or `..` ranges are not flagged as truncated files.

### [ ] TRACK 4: `crates/verify-baseline`
- **`src/runner.rs`:**
  - Fix check number labels in console/dossier messages for Checks 25-30 and call `check_handoff_and_backlog_integrity(&repo_root)`.
- **`src/checks/safety_stpa.rs` (Check 17):**
  - Replace corrupted `" "` entries in `INTERFACE_KEYWORDS`, `STATE_KEYWORDS`, `ACTION_KEYWORDS`, and `RESOURCE_KEYWORDS` with `"\u{03b3}"`, `"\u{03c6}"`, `"\u{03c9}"`, `"\u{03c8}"`.
  - Replace hardcoded `"dshot"` check with the general citation and protocol provenance verifier.
- **`src/checks/parity_gates.rs` (Checks 21-31 / Gates 24-31):**
  - **Check 22:** Remove `&& inv_id.contains("UNDECLARED")`; flag any `[INV-*]` not in `declared_constraints` (when `declared_constraints` is non-empty).
  - **Check 23:** Replace hardcoded `"pwm"`, `"50 hz"`, `"dshot"`, and `"stanag 4586"` checks with general numeric+unit claim and protocol lexicon grounding against the cited section (`sec_text`) and SysML model (`model_str` / `parse_port_defs`).
  - **Check 24:** Remove hardcoded `"GNSS"`, `"IMU"`, and `"FlightController"` checks; determine nested port qualification requirements from the SysML model (`get_nested_ports()` / `parse_part_defs`) or dotted subsystem paths.
  - **Check 26:** Validate SI units across parameter, metric, and signal tables across technical directories.
  - **Check 30:** Remove `conn.contains("DIVERGENT") || conn.contains("PHANTOM")`; verify that `CONN-*` IDs in `docs/reports/` exist in `docs/architecture/views/` or `ICD_01`.
  - **Gate 28 (`check_coverage_digest`):** Parse traceability/coverage references in specs/reports and verify referenced `FEAT-*` / `UC-*` / `US-*` files exist on disk.
  - **Gate 29 (`check_obligation_witness`):** Parse `docs/safety/STPA_MATRIX.md` and verify every declared hazard (`H-*`) is witnessed by a safety constraint (`SC-*`).
- **`src/spec_audit/*`:**
  - Fix UTF-8 byte slicing in `behavioral.rs` (`contains_token_with_word_boundary`), `logical_ui.rs` (`detect_dom_leaks`), `provenance.rs` (`find_engineering_parameter`, `extract_source_references_section`, `is_assertion_grounded`), and `metadata.rs` (`normalize_title`). Sort `fs::read_dir` in `conops_audit.rs`.

### [ ] TRACK 5: `crates/assemble-conops`
- **`src/sanitize.rs` & `src/params.rs`:**
  - Replace `.to_lowercase()` with `.to_ascii_lowercase()` in all byte-slicing functions (`strip_poly_equation`, `strip_poly_hex`, `replace_crc16`, `replace_polynomial_equation`, `strip_baud_rates`, `strip_opcodes`, `strip_use_cases`, `wrap_mermaid_label`, `replace_word_case_insensitive`, `replace_altitude_agl`, `replace_astm_f3411`, `replace_roe_phrase`).
  - In `process_segment` (`sanitize.rs`): Snap `end_idx` to a valid UTF-8 char boundary before slicing `&w[c_idx..end_idx]`.
  - Remove unused regex string constants `PLACEHOLDER_PATTERN` and `RAW_TOKEN_FINDER` in `params.rs` and sort `WalkDir` in `auto_detect_workspace_parameters`.

### [ ] TRACK 6: `crates/reconcile-backlog`
- **`src/tracker.rs` & `src/lib.rs`:**
  - Replace `trimmed.to_lowercase()` with `trimmed.to_ascii_lowercase()` in `normalize_title`.
  - Add PID + atomic counter to `tempfile_named()`.
  - Chain `.sort_by_file_name()` on `WalkDir` in `load_specifications` and use `BTreeMap` for `spec_path_to_issue`.

### [ ] TRACK 7: Complete Domain & Template Decontamination (100% Pure AST-Driven Architecture)
- **`compile-sysml/src/stpa/safety_suite.rs`:**
  - When `pkg.get_all_calcs()` or `pkg.get_all_constraints()` are non-empty, synthesize Formal Proof Suite theorems, parameter tables, numerical evaluations, and `sldv.assert(...)` bindings directly from the AST's `CalcDef` and `ConstraintDef` nodes instead of hardcoded aerospace `PROOF_TEMPLATES`.
- **`compile-sysml/src/stpa/fmeca.rs`:**
  - Derive FMECA failure mode rows from each `PartDef`'s actual declared `ports`, `states`, `actions`/`operations`, `attributes`, and `pkg.get_all_hazards()`, removing hardcoded `"Kalman residual"` / `"ECC RAM"` / `"TVS diode"` filler strings.
- **`compile-sysml/src/sync/forward.rs` & `ingest-sysml/src/translators/markdown.rs`:**
  - Order packages from the AST (`pkg.packages`) without requiring a hardcoded `SUBSYSTEM_ORDER` array, and group `RequirementDef`s dynamically by `frontmatter.subsystem`.
- **`deap-core/src/sysml_ast.rs`:**
  - Generalize negative component constraint evaluation to inspect any AST attribute set to `0`, `false`, `"No"`, or `"None"` (`<subsystem>Enabled = 0` or `!exists(<Part>)`) in addition to recovery systems.
- **`assemble-conops/src/params.rs`:**
  - Derive mass, power, velocity, and energy bindings from ingested SysML AST attributes and explicit parameter files. Update any unit tests that previously relied on hardcoded domain defaults to pass their parameters explicitly via test bindings.