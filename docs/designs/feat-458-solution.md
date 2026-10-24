---
issue_id: 458
title: "Solution Walkthrough: FEAT-42 Automated Source-Code AST Ingestion & 7-Section Safety Docstring Synthesizer"
version: "1.0"
date: "2026-10-24"
---

# Solution Walkthrough: FEAT-42 Automated Source-Code AST Ingestion & 7-Section Safety Docstring Synthesizer (Issue #458)

## 1. Overview & Architectural Context

Issue #458 ([FEAT-42](FEAT-42_source_code_ast_ingestion_and_docstring_synthesizer.md)) establishes the normative, deterministic compiler mechanisms for bidirectional parity between high-assurance source code (Rust and Dart) and canonical SysML v2 abstract syntax tree (AST) models under DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C standards:

1. **Automated Source-Code AST Ingestion (`ingest-sysml --source <DIR>`)**: A zero-regex, recursive-descent token-stream and character-scanner ingestion engine (`IngestSysml::SourceCodeAstIngestion`) that parses Dart and Rust source code trees directly into canonical SysML v2 `PackageDef` AST structures (`PartDef`, `AttributeDef`, `ActionDef`, `PortDef`, `StateDef`, `ConstraintDef`, and `RequirementDef`), enforces closed-world architectural scope and negative exclusion invariants, and writes the canonical model and FIPS 180-4 SHA-256 schema digest atomically with closed-loop read-back verification.
2. **7-Section Safety Docstring Synthesizer (`compile-sysml --inject-safety-docstrings <DIR>`)**: A deterministic forward synthesizer (`CompileSysml::SafetyDocstringSynthesizer`) that parses SysML v2 `requirement def`, `constraint def`, `part def`, and `action def` entities and injects standardized 7-section safety docstrings (`## 1. Safety Intent & Regulatory Scope` through `## 7. Realization & Traceability`) into target Rust and Dart source files with bit-for-bit idempotency and per-function normalized body uniqueness satisfying Check 34 and Check 36.

### Three-Layer Realization Architecture

- **Layer 1 (Domain State & AST Data Model)**: Implemented in [crates/deap-core/src/source_ast.rs](../../crates/deap-core/src/source_ast.rs) via strongly typed AST domain representations (`SourceLanguage`, `SourceItemKind`, `SourceParam`, `SourceField`, `SourceAstItem`) and stateful lexical scanner primitives (`ActiveStringState`, `InterpolationFrame`, `RustStringScannerState`).
- **Layer 2 (Logic, Lexing, Lowering & Docstring Synthesis)**: Implemented across [crates/deap-core/src/source_ast.rs](../../crates/deap-core/src/source_ast.rs), [crates/ingest-sysml/src/translators/code.rs](../../crates/ingest-sysml/src/translators/code.rs), and [crates/compile-sysml/src/docstring.rs](../../crates/compile-sysml/src/docstring.rs), providing zero-regex syntax validation, Rust/Dart declaration extraction, M2 metamodel lowering (`lower_source_ast_to_package`), recursive `PackageDef` part traversal, and 7-section docstring interpolation (`synthesize_7_section_docstring`).
- **Layer 3 (CLI & Atomic Persistence Interface Binding)**: Implemented across [crates/ingest-sysml/src/lib.rs](../../crates/ingest-sysml/src/lib.rs), [crates/ingest-sysml/src/main.rs](../../crates/ingest-sysml/src/main.rs), [crates/compile-sysml/src/docstring.rs](../../crates/compile-sysml/src/docstring.rs), and [crates/compile-sysml/src/main.rs](../../crates/compile-sysml/src/main.rs), enforcing deterministic exit code contracts (`0`, `1`, `2`), fail-closed `--schema` loading, unforced `Realises:` conflict detection, and atomic file replacement.

---

## 2. Detailed Summary of Changes Implemented

### 2.1 Layer 1 & Layer 2 Zero-Regex Source AST Lexer, Shared Rust Line Scanner & Lowering (`crates/deap-core/src/source_ast.rs`)

- **Core Source AST Domain Types**:
  - `SourceLanguage`: Enumerates supported language modes (`Auto`, `Rust`, `Dart`) with deterministic flag parsing (`from_flag`) and extension detection (`detect_from_extension`).
  - `SourceItemKind`: Classifies extracted source constructs across its 10 variants (`Struct`, `Enum`, `Trait`, `Class`, `Mixin`, `Extension`, `Typedef`, `Function`, `Method`, `Constructor`), while recording enclosing `impl` or class owners in `SourceAstItem::owner_name` and widget/state supertypes (`StatefulWidget`, `StatelessWidget`, `State<...>`) in `SourceAstItem::parent_type`.
  - `SourceParam`, `SourceField`, and `SourceAstItem`: Capture typed parameters (`name`, `type_name`), struct/class fields (`name`, `type_name`, `is_callback_or_port`, `doc`), contiguous docstring lines (`doc_lines`), extracted `Realises:` requirement IDs (`satisfied_requirements`), runtime assertion expressions (`invariants`), and enum variants or state identifiers (`states`).
- **Stateful Lexical Scanners & String/Delimiter Hygiene**:
  - `ActiveStringState` and `InterpolationFrame`: Track standard, raw (`r'...'`, `r"..."`), and triple-quoted (`'''`, `"""`) Dart strings alongside arbitrarily nested `${...}` string interpolations (`has_matching_interpolation_brace_before_quote`) in `validate_source_syntax_for_language`.
  - `RustStringScannerState`, `single_char_literal_byte_len`, `rust_raw_string_hash_count`, and `scan_rust_line_preserving_lex_state`: Provide a unified, cross-line stateful Rust lexical scanner that distinguishes Rust character literals (`'{'`, `'\''`, `'\x7f'`, `'\u{1F600}'`) from lifetime annotations (`'a`, `'static`), tracks multi-line nested `/* ... */` block comments, tracks multi-line standard `"..."` strings, and tracks multi-line raw strings (`r#"..."#`, `br##"..."##`).
  - `strip_line_comment_preserving_strings` and `update_ast_delimiter_depths`: Operate strictly via iterator and byte/char slices (`get`, `split_at`) with zero direct indexing (`[i]`) or range slicing (`&s[a..b]`), satisfying Check 34.
- **Recursive-Descent AST Extraction & M2 Metamodel Lowering**:
  - `parse_rust_source_ast` and `parse_dart_source_ast`: Validate source syntax first via `validate_source_syntax_for_language` (reporting exact `line:col` coordinates on unclosed delimiters or strings) and extract structs, enums, traits, `impl` blocks, widgets, classes, fields, methods, callbacks, and `assert!` / `assert(...)` invariants.
  - `lower_source_ast_to_package`: Lowers extracted `SourceAstItem` records into a canonical SysML v2 `PackageDef` containing `PartDef`, `AttributeDef`, `PortDef`, `ActionDef`, `StateDef`, `ConstraintDef`, and `RequirementDef` nodes without hardcoded M1 domain vocabulary.

### 2.2 Source-Code AST Ingestion & Exit Code `0 / 1 / 2` Contract (`crates/ingest-sysml`)

- **Process Exit Status Classification ([crates/ingest-sysml/src/lib.rs](../../crates/ingest-sysml/src/lib.rs))**:
  - `IngestExitCode`: Defines `Clean = 0`, `ParseOrIoError = 1`, and `ScopeOrCryptoError = 2` with `const fn code(self) -> i32`.
  - `verify_scope_and_negative_invariants`: Verifies after `filter_ast_to_target_scope` that non-empty `--allowed-parts` filters retain at least one `PartDef` in the package hierarchy, and that no entity listed in `--negative-invariants` survives in the package. Violations return `Err(String)` mapped to `IngestExitCode::ScopeOrCryptoError` (`2`).
- **Source Code Translator ([crates/ingest-sysml/src/translators/code.rs](../../crates/ingest-sysml/src/translators/code.rs))**:
  - `CodeTranslator`: Wraps `SourceLanguage` and exposes `new`, `with_language`, `translate`, `translate_files`, and `translate_directory` to lower Rust and Dart source trees into `PackageDef` models.
- **Transactional CLI Orchestration ([crates/ingest-sysml/src/main.rs](../../crates/ingest-sysml/src/main.rs))**:
  - `run_ingestion_with_code` and `run_ingestion`: Support `--source <DIR>`, `--format code`, `--language <auto|dart|rust>`, `--allowed-parts`, and `--negative-invariants`.
  - Executes `verify_scope_and_negative_invariants` prior to writing output files so scope or negative-invariant failures never overwrite existing baseline artifacts.
  - Performs closed-loop post-write cryptographic verification by reading back `--out` and `--digest` from disk and verifying `compute_sha256(persisted_sysml.as_bytes()) == digest.sha256`.

### 2.3 7-Section Safety Docstring Synthesizer & Conflict/Schema Exit Codes (`crates/compile-sysml`)

- **Synthesizer Exit Code & Configuration ([crates/compile-sysml/src/docstring.rs](../../crates/compile-sysml/src/docstring.rs))**:
  - `DocstringExitCode`: Defines `Clean` (`0`), `Error` (`1`), and `IdempotencyOrConflict` (`2`).
  - `DocstringInjectionOptions` and `DocstringInjectionReport`: Configure `--inject-safety-docstrings <DIR>`, `--dry-run`, `--force`, and optional `--schema <FILE>`, reporting `files_scanned`, `files_modified`, `items_enriched`, and `modified_files`.
  - `run_extract_source`: Implements `--extract-source <DIR>` reverse synchronization probing into canonical SysML v2 and SHA-256 digest files.
- **Schema Context Resolution & Unforced Conflict Detection ([crates/compile-sysml/src/docstring.rs](../../crates/compile-sysml/src/docstring.rs))**:
  - `collect_all_parts` and `resolve_schema_context`: Recursively traverse nested `pkg.packages` and `part.parts` to bind matching `PartDef`, `ActionDef`, `RequirementDef`, and `ConstraintDef` nodes strictly without leaking unrelated package-first requirements onto unmatched items.
  - `extract_rust_impl_header_prefix` and `extract_rust_impl_target_name`: Accumulate multi-line generic and `where`-clause Rust `impl` headers until `{` and extract the target type identifier across generic brackets and trait `for` clauses.
  - `inject_rust_file_docstrings`, `inject_dart_file_docstrings`, and `synthesize_7_section_docstring`: Synthesize per-item unique 7-section docstrings (`# Summary`, `# Preconditions`, `# Postconditions`, `# Invariants`, `# Failure Modes`, `# Side Effects`, `# Determinism`, and `## 1.` through `## 7.`), detect conflicting existing `Realises:` tags when `!opts.force` (returning `DocstringExitCode::IdempotencyOrConflict` / exit code `2`), and verify second-pass idempotency in memory prior to atomic disk write.
- **Fail-Closed `--schema` CLI Handling ([crates/compile-sysml/src/main.rs](../../crates/compile-sysml/src/main.rs))**:
  - When `--schema` (or positional `FILE`) is explicitly supplied with `--inject-safety-docstrings`, any missing path, unreadable file, or SysML parse failure logs a structured diagnostic to `stderr` and exits immediately with `DocstringExitCode::Error` (`1`).

---

## 3. Code Realization Table

| FEAT-42 Construct / Capability | Source File | Concrete Symbols (Structs / Enums / Functions) | Verification Gate |
| :--- | :--- | :--- | :--- |
| Layer 1 & 2 Source AST Domain & Zero-Regex Lexer | [crates/deap-core/src/source_ast.rs](../../crates/deap-core/src/source_ast.rs) | `SourceLanguage`, `SourceItemKind`, `SourceParam`, `SourceField`, `SourceAstItem`, `ActiveStringState`, `InterpolationFrame`, `RustStringScannerState`, `single_char_literal_byte_len`, `scan_rust_line_preserving_lex_state`, `validate_source_syntax_for_language`, `parse_rust_source_ast`, `parse_dart_source_ast`, `lower_source_ast_to_package`, `strip_line_comment_preserving_strings`, `update_ast_delimiter_depths`, `rust_raw_string_hash_count`, `has_matching_interpolation_brace_before_quote` | `cargo test -p deap-core`, Check 34, Check 36 |
| Ingestion Exit Code & Scope / Negative-Invariant Gate | [crates/ingest-sysml/src/lib.rs](../../crates/ingest-sysml/src/lib.rs) | `IngestExitCode` (`Clean`, `ParseOrIoError`, `ScopeOrCryptoError`), `IngestExitCode::code`, `verify_scope_and_negative_invariants`, `filter_ast_to_target_scope`, `filter_ast_to_realized_or_allowed` | `cargo test -p ingest-sysml`, Check 34, Check 36 |
| Ingestion CLI & Closed-Loop SHA-256 Persistence | [crates/ingest-sysml/src/main.rs](../../crates/ingest-sysml/src/main.rs) | `CliArgs`, `resolve_ingested_package_name`, `run_ingestion`, `run_ingestion_with_code`, `main` | `cargo test -p ingest-sysml`, Check 34, Check 36 |
| Rust & Dart Source Code AST Translator | [crates/ingest-sysml/src/translators/code.rs](../../crates/ingest-sysml/src/translators/code.rs) | `CodeTranslator`, `CodeTranslator::new`, `CodeTranslator::with_language`, `CodeTranslator::translate`, `CodeTranslator::translate_files`, `CodeTranslator::translate_directory` | `cargo test -p ingest-sysml`, Check 34, Check 36 |
| Source Ingestion BDD & Exit Code Integration Suite | [crates/ingest-sysml/tests/code_ingestion_tests.rs](../../crates/ingest-sysml/tests/code_ingestion_tests.rs) | `test_ac01_dart_stateful_widget_extraction_into_canonical_packagedef_ast`, `test_ac02_rust_struct_and_impl_block_ast_ingestion`, `test_ac05_zero_regex_lexer_error_recovery_halts_closed_leaving_outputs_unmodified`, `test_ingest_sysml_cli_exit_codes_0_1_and_2` | 4/4 integration tests green |
| 7-Section Safety Docstring Synthesizer & Injector | [crates/compile-sysml/src/docstring.rs](../../crates/compile-sysml/src/docstring.rs) | `DocstringExitCode` (`Clean`, `Error`, `IdempotencyOrConflict`), `DocstringInjectionOptions`, `DocstringInjectionReport`, `run_docstring_injection`, `run_extract_source`, `inject_rust_file_docstrings`, `inject_dart_file_docstrings`, `synthesize_7_section_docstring`, `resolve_schema_context`, `collect_all_parts`, `extract_rust_impl_header_prefix`, `extract_rust_impl_target_name` | `cargo test -p compile-sysml`, Check 34, Check 36 |
| Compiler CLI Fail-Closed `--schema` & Docstring Dispatch | [crates/compile-sysml/src/main.rs](../../crates/compile-sysml/src/main.rs) | `Cli`, `main` (`inject_safety_docstrings` and `extract_source` branches) | `cargo test -p compile-sysml`, Check 34, Check 36 |
| Docstring Synthesizer BDD & Conflict Integration Suite | [crates/compile-sysml/tests/docstring_injector_tests.rs](../../crates/compile-sysml/tests/docstring_injector_tests.rs) | `test_ac03_and_ac04_rust_and_dart_docstring_synthesis_and_idempotency`, `test_duplicate_function_docstrings_are_disambiguated`, `test_extract_source_and_cli_flags`, `test_injector_handles_multiline_dart_classes_string_slashes_interpolated_utf8_quotes_and_dart3_items`, `test_cli_exit_code_1_on_broken_schema_and_exit_code_2_on_conflicting_unforced_docstring` | 5/5 integration tests green |

---

## 4. Raw Verification & Testing Evidence

### 4.1 `cargo test -p deap-core`

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.03s

running 78 tests
test source_ast::tests::test_scan_rust_line_preserving_lex_state_char_literals_and_lifetimes ... ok
test source_ast::tests::test_syntax_error_unclosed_brace_fails_closed_with_line_and_col ... ok
test source_ast::tests::test_parse_rust_struct_and_impl_and_lower_to_sysml ... ok
test source_ast::tests::test_validate_dart_raw_strings_and_interpolated_nested_quotes ... ok
test source_ast::tests::test_parse_dart_stateful_widget_and_lower_to_sysml ... ok
test source_ast::tests::test_strip_comment_and_delimiter_depths_zero_range_slice_indexing ... ok
...
test result: ok. 78 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
```

### 4.2 `cargo test -p ingest-sysml`

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.04s

running 40 tests
test result: ok. 40 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

running 1 test
test tests::test_cli_cache_ast_flag ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/code_ingestion_tests.rs (target/debug/deps/code_ingestion_tests-8ab427988105cc81)

running 4 tests
test test_ac05_zero_regex_lexer_error_recovery_halts_closed_leaving_outputs_unmodified ... ok
test test_ac01_dart_stateful_widget_extraction_into_canonical_packagedef_ast ... ok
test test_ac02_rust_struct_and_impl_block_ast_ingestion ... ok
test test_ingest_sysml_cli_exit_codes_0_1_and_2 ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.45s

     Running tests/code_realization_scoping_tests.rs (target/debug/deps/code_realization_scoping_tests-867476da1ae6ef78)

running 3 tests
test test_issue_472_check17_fmeca_closure_ignores_realization_only_code_parts ... ok
test test_issue_472_source_with_reference_schema_scopes_to_architectural_and_realized_parts ... ok
test test_issue_472_parent_qualified_realization_package_naming_and_override ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
```

### 4.3 `cargo test -p compile-sysml`

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.03s

running 128 tests
test docstring::tests::test_defect_09_unmatched_items_do_not_inherit_first_package_req_or_constraint ... ok
test docstring::tests::test_defect_09_multihash_raw_and_multiline_standard_strings_do_not_confuse_scanner ... ok
...
test result: ok. 128 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

running 3 tests
test tests::test_compile_sysml_main_7_section_rustdoc ... ok
test tests::test_cli_emit_flutter_contracts_flag ... ok
test tests::test_cli_export_mbd_and_slice_package_flags_and_flatten ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/docstring_injector_tests.rs (target/debug/deps/docstring_injector_tests-9284f4ef5d215e7f)

running 5 tests
test test_duplicate_function_docstrings_are_disambiguated ... ok
test test_injector_handles_multiline_dart_classes_string_slashes_interpolated_utf8_quotes_and_dart3_items ... ok
test test_ac03_and_ac04_rust_and_dart_docstring_synthesis_and_idempotency ... ok
test test_extract_source_and_cli_flags ... ok
test test_cli_exit_code_1_on_broken_schema_and_exit_code_2_on_conflicting_unforced_docstring ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.56s

     Running tests/multi_file_compilation_tests.rs (target/debug/deps/multi_file_compilation_tests-fd63a67b28fb1150)

running 3 tests
test test_multi_file_top_level_imports_and_unsorted_slices_unwrap_sysml_model ... ok
test test_multi_file_with_top_level_doc_and_imports_preserves_package_scopes_and_dual_schema_parity ... ok
test test_multi_file_single_and_multi_package_no_extraneous_sysml_model_wrapper ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

### 4.4 `cargo clippy --workspace --all-targets -- -D warnings` & `./target/release/verify-baseline . --no-domain`

```text
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.09s
Success: Check 10 verified (.gitignore exists in repository root).
Success: Check 11 verified (Zero .DS_Store files in working tree).
Success: Check 12 verified (Zero duplicate master core blueprints in downstream repository).
Success: Check 13 verified (KaTeX / LaTeX mathematical syntax conforms to markdown rendering rules across all markdown files).
Success: Check 13B verified (Mermaid syntax and diagram ergonomics verified across all markdown files).
Success: Check 14 verified (Downstream repository contains valid instructions: README.md, AGENTS.md, and rules/sysml-ssot-completeness.md).
Success: Check 15 verified (Reconcile backlog tooling exists: target/release/reconcile-backlog or crates/reconcile-backlog).
Success: Check 16 verified (Upstream template clean landing zone gate passed).
Success: Check 17 verified (Upstream distribution template safety landing zone is clean).
Success: Check 18 verified (Upstream architecture blueprints are clean with zero domain concept papers or sysml models).
Success: Check 19 verified (Domain-Agnostic AST Cleanliness & Closed-Grammar Metamodel Gate passed -- pure dynamic schema AST architecture verified).
Success: Check 20 verified (WBS & Enterprise Deliverables Suite pending or not present).
Success: Check 21 verified (SysML model pending or landing zone clean).
Success: Check 22 verified (SysML model pending or landing zone clean).
Success: Check 23 verified (SysML model pending or landing zone clean).
Success: Level 1C ICD Completeness verified (zero dangling ports, 100% port contract parity).
Success: Check 25 verified (Operational-to-Resource Allocation passed -- zero orphan activities or phantom allocation tags).
Success: Check 26 verified (Standards & SI 7D Parameter Metrology passed -- all parameter dimensions, units, and SDO baselines valid).
Success: Check 27 verified (Cross-Document Diagram Parity Gate passed -- zero disparity in subgraphs, nodes, ports, or connections).
Success: Check 28 verified (ConOps & Mission Intent Completeness passed -- all mandatory sections, tables, and METL rosters valid).
Success: Check 29 verified (Cited Research Inventory & Declared-Total Population Register passed).
Success: Check 30 verified (Executive Deliverable Traceability Gate passed -- all tables and diagrams anchored to SSOT).
Success: Check 30A verified (Downstream repository detected -- coverage digest pending or landing zone clean).
Success: Check 30B verified (Downstream repository detected -- obligation witness pending or landing zone clean).
Success: Check 30C verified (Architecture Viewpoint & Diagram Completeness Gate passed -- all 11 canonical diagrams verified).
Success: Check 31 verified (Dual-schema SSOT parity gate passed -- single schema or landing zone clean).
Success: Check 32 verified (Zero Shell Scripts Invariant passed -- zero .sh files exist on disk).
Success: Check 33 verified (Zero Python Scripts Invariant passed -- zero .py files exist on disk).
Success: Check 34 verified (Zero Regex Invariant passed -- zero regex occurrences exist across crates).
Success: Check 35 verified (Handoff & Backlog Integrity passed).
Success: Check 36 verified (Per-Item 7-Section Safety Docstring Completeness Gate passed -- 100% per-item 7-section docstrings verified across Rust and Dart sources).
Success: Check 37 verified (Architecture Blueprint Grounding Gate passed -- all 13 blueprints grounded in workspace crates and paths).
Success: Check 38 verified (Presentation Anti-Hardcoding AST Gate passed).
Success: Check 39 verified (Zero-Mocking Test AST & SQLite Disposal Gate passed).
Success: Check 40 verified (Interactive Gesture, 1080p-6K Viewport & Pairwise 2D HUD Non-Overlap Gate passed).
Success: Check 41 verified (Track B C-ABI Solver Bridge SSOT & 5-Table SQLite Seed Parity Gate passed).
Downstream repository root verified (non-framework platform or root orchestration workspace).
```

---

## 5. Step-by-Step Human Manual Testing Instructions

### 5.1 Manual Verification of `ingest-sysml --source <DIR>` (Exit Codes `0`, `1`, and `2`)

1. **Create a temporary scratch workspace outside the repository landing zones**:
   ```bash
   TMP_DIR="$(mktemp -d)"
   mkdir -p "$TMP_DIR/src"
   cat << 'EOF' > "$TMP_DIR/src/sample.rs"
   /// Realises: [REQ-SAMPLE-01]
   pub struct SampleController {
       pub cycle_count: u32,
   }

   impl SampleController {
       pub fn step(&mut self, delta: u32) -> u32 {
           assert!(delta > 0);
           self.cycle_count = self.cycle_count.saturating_add(delta);
           self.cycle_count
       }
   }
   EOF
   ```

2. **Verify Clean Source AST Ingestion (Expected Exit Code `0`)**:
   ```bash
   ./target/release/ingest-sysml --source "$TMP_DIR/src" --format code --language rust --out "$TMP_DIR/schema.sysml" --digest "$TMP_DIR/schema-digest.json"
   echo "Exit code: $?"
   ```
   - **Expected Output**: Prints `[SysML v2 Ingestion] Successfully ingested source AST...` and exits with status `0`. Inspecting `"$TMP_DIR/schema.sysml"` confirms `part def SampleController`, `attribute cycle_count : u32;`, `action def step`, and `satisfy requirement REQ-SAMPLE-01;`.

3. **Verify Scope or Negative-Invariant Violation Detection (Expected Exit Code `2`)**:
   ```bash
   ./target/release/ingest-sysml --source "$TMP_DIR/src" --format code --language rust --negative-invariants "SampleController" --out "$TMP_DIR/schema.sysml" --digest "$TMP_DIR/schema-digest.json"
   echo "Exit code: $?"
   ```
   - **Expected Output**: Prints `Architectural negative-invariant violation: forbidden entity 'SampleController' exists...` to `stderr` and exits with status `2`.

4. **Verify Fail-Closed Syntax Error Recovery (Expected Exit Code `1`)**:
   ```bash
   printf 'pub struct Broken {\n' > "$TMP_DIR/src/broken.rs"
   ./target/release/ingest-sysml --source "$TMP_DIR/src" --format code --language rust --out "$TMP_DIR/schema.sysml" --digest "$TMP_DIR/schema-digest.json"
   echo "Exit code: $?"
   rm -f "$TMP_DIR/src/broken.rs"
   ```
   - **Expected Output**: Reports the unclosed delimiter with `line:col` coordinates to `stderr`, leaves `"$TMP_DIR/schema.sysml"` untouched, and exits with status `1`.

### 5.2 Manual Verification of `compile-sysml --inject-safety-docstrings <DIR>` (Exit Codes `0`, `1`, and `2`)

1. **Synthesize 7-Section Safety Docstrings and Verify Idempotency (Expected Exit Code `0`)**:
   ```bash
   ./target/release/compile-sysml --inject-safety-docstrings "$TMP_DIR/src" --schema "$TMP_DIR/schema.sysml"
   echo "First run exit code: $?"
   ./target/release/compile-sysml --inject-safety-docstrings "$TMP_DIR/src" --schema "$TMP_DIR/schema.sysml"
   echo "Second (idempotent) run exit code: $?"
   ```
   - **Expected Output**: First run enriches `SampleController` and `step` and exits with `0`. Second run reports `modified 0 file(s), enriched 0 item(s)` and exits with `0`.

2. **Verify Unforced Conflicting `Realises:` Detection (Expected Exit Code `2`)**:
   ```bash
   cat << 'EOF' > "$TMP_DIR/conflict.sysml"
   package ConflictPkg {
       requirement def <'REQ-AUTHORITATIVE-99'> ReqAuth {
           doc /* Authoritative requirement */
       }
       part def SampleController {
           satisfy requirement 'REQ-AUTHORITATIVE-99';
       }
   }
   EOF
   ./target/release/compile-sysml --inject-safety-docstrings "$TMP_DIR/src" --schema "$TMP_DIR/conflict.sysml"
   echo "Conflict exit code: $?"
   ```
   - **Expected Output**: Detects that `SampleController` carries `REQ-SAMPLE-01` while the schema requires `REQ-AUTHORITATIVE-99` without `--force`, prints a conflict diagnostic to `stderr`, and exits with status `2`.

3. **Verify Fail-Closed Missing/Broken `--schema` Handling (Expected Exit Code `1`)**:
   ```bash
   ./target/release/compile-sysml --inject-safety-docstrings "$TMP_DIR/src" --schema "$TMP_DIR/nonexistent.sysml"
   echo "Missing schema exit code: $?"
   rm -rf "$TMP_DIR"
   ```
   - **Expected Output**: Prints an error diagnostic to `stderr` and exits with status `1`.

---

## 6. Source References

- **Governance Constitution**: [.pipeline/constitution.md](../../.pipeline/constitution.md)
- **SysML SSOT Completeness**: [rules/sysml-ssot-completeness.md](../../rules/sysml-ssot-completeness.md)
- **Domain Engineering Standards**: [rules/domain-engineering-standards.md](../../rules/domain-engineering-standards.md)
- **Feature Design Specification**: [FEAT-42_source_code_ast_ingestion_and_docstring_synthesizer.md](FEAT-42_source_code_ast_ingestion_and_docstring_synthesizer.md)
- **Core Source AST Lexer & Lowering Engine**: [crates/deap-core/src/source_ast.rs](../../crates/deap-core/src/source_ast.rs)
- **Ingestion Library & Scope/Exit-Code Gates**: [crates/ingest-sysml/src/lib.rs](../../crates/ingest-sysml/src/lib.rs)
- **Ingestion CLI Entrypoint**: [crates/ingest-sysml/src/main.rs](../../crates/ingest-sysml/src/main.rs)
- **Source Code AST Translator**: [crates/ingest-sysml/src/translators/code.rs](../../crates/ingest-sysml/src/translators/code.rs)
- **7-Section Safety Docstring Synthesizer**: [crates/compile-sysml/src/docstring.rs](../../crates/compile-sysml/src/docstring.rs)
- **Compiler CLI Entrypoint**: [crates/compile-sysml/src/main.rs](../../crates/compile-sysml/src/main.rs)
