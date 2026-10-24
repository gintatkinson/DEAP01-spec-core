# Implementation Plan: FEAT-42 Source-Code AST Ingestion & 7-Section Safety Docstring Synthesizer (Issue #458)

## 1. Upstream Design Specification & Gap Analysis

### 1.1 Established Design Baseline (`docs/designs/FEAT-42_source_code_ast_ingestion_and_docstring_synthesizer.md`)
The upstream design specification [FEAT-42_source_code_ast_ingestion_and_docstring_synthesizer.md](file:///Users/perkunas/jail/DEAP01-spec-core/docs/designs/FEAT-42_source_code_ast_ingestion_and_docstring_synthesizer.md) is already authored in the repository and defines the normative contracts for `ingest-sysml --source <DIR>`, `compile-sysml --inject-safety-docstrings <DIR>`, and `deap-core::source_ast`.

An empirical audit of the codebase against the design specification and GitHub Issue `#458` confirms that the core architecture is present, while **four normative code/test gaps** remain to achieve 100% compliance:

| Area | Normative Requirement in FEAT-42 (`#458`) | Current Workspace State | Gap to Remediate |
| :--- | :--- | :--- | :--- |
| **1. Zero-Regex Source AST Lexer & Check 34 Purity (`deap-core::source_ast`)** | AC-01, AC-02, AC-05 & Check 34: Zero-regex recursive-descent lexer/parser for Rust and Dart with zero panicking slices and zero lint suppressions. | `crates/deap-core/src/source_ast.rs` has `#[allow(clippy::type_complexity)]` at line 545, misses `r#"..."#` hashed raw strings in `strip_line_comment_preserving_strings` / `update_ast_delimiter_depths`, corrupts `paren_depth` on unclosed `"${"` literals in Rust, and checks only 3 hardcoded strings in its Check 34 unit test (`#492` / `DEFECT-AUDIT-492`). | Replace `#[allow(clippy::type_complexity)]` with `ActiveStringState` / `InterpolationFrame` type aliases; expose shared `RustStringScannerState`, `scan_rust_line_preserving_lex_state`, and `single_char_literal_byte_len`; handle `r#"..."#` and guard `${` interpolation with `has_matching_interpolation_brace_before_quote`; scan all production lines in `source_ast.rs` with `contains_direct_range_slice`. |
| **2. `ingest-sysml` Exit Code Contract (Section 2.1)** | Exit codes: `0` (Clean), `1` (Parsing or I/O failure), `2` (Architectural scope or cryptographic verification failure). | `crates/ingest-sysml/src/main.rs:220-226` only exits with `0` or `1`. | Introduce `IngestExitCode` (`Clean = 0`, `ParseOrIoError = 1`, `ScopeOrCryptoError = 2`) in `ingest-sysml`; return exit code `2` when a `--negative-invariants` forbidden entity exists in the output AST scope, when a non-empty `--allowed-parts` filter matches `0` parts, or when post-write SHA-256 digest verification fails. |
| **3. `compile-sysml` Schema Error & Conflict Exit Codes (Section 2.2)** | Exit codes: `0` (Clean), `1` (Compilation or syntax error in source or SysML schema), `2` (Idempotency invariant violation or conflicting unforced docstring detected). | `crates/compile-sysml/src/main.rs:214-220` silently swallows `--schema` read/parse errors via `.ok()`, and `docstring.rs` ignores conflicting `Realises:` bindings when `!force`. | Fail closed with exit code `1` (`DocstringExitCode::Error`) when `--schema` (or discovered `schema/*.sysml`) fails to read or parse; return exit code `2` (`DocstringExitCode::IdempotencyOrConflict`) when `!force` and an existing 7-section docstring's explicit `Realises:` requirement ID conflicts with the authoritative requirement bound to that entity in `schema_model` (and overwrite when `--force` is supplied). |
| **4. 7-Section Docstring Synthesizer Traceability & Scanner (`compile-sysml::docstring`)** | Section 3.2 & AC-03, AC-04: Deterministic schema-to-docstring synthesis and bit-for-bit idempotency without false-positive requirement attribution. | `crates/compile-sysml/src/docstring.rs` retains `.first()` package fallback on `matched_entity_pkg` (`lines 1700-1736`), matches method `item_name` against unrelated `PartDef` nodes when `owner_name.is_some()`, skips items after inline-closed `*/` comments (`line 621`), misparses `->` in `impl<F: Fn() -> R>` headers (`line 2083`), duplicates `scan_rust_line_preserving_lex_state` with magic offset `close_pos <= 10` (`line 2012`), and uses `std::mem::size_of` / `#![allow(clippy::...)]` in tests (`#498` / `DEFECT-AUDIT-498`). | Remove `matched_entity_pkg` `.first()` fallbacks; scope `PartDef` lookup to `owner_name` when `owner_name.is_some()` and traverse nested `PartDef::parts`, inline `PartDef::requirements`, `RequirementDef::satisfied_by`, and `owner_part.actions`; delegate Rust line scanning to `deap_core::source_ast`; fix inline `*/` closure and multiline/generic `impl` target parsing; remove `std::mem::size_of` and `#![allow(clippy::...)]` from test files. |

---

## 2. Architectural Mapping (3-Layer Semantic Realization)

1. **Layer 1 — Domain State & AST Data Model (`crates/deap-core/src/source_ast.rs`)**:
   - `SourceLanguage`, `SourceItemKind`, `SourceParam`, `SourceField`, `SourceAstItem`, `ActiveStringState`, `InterpolationFrame`, and `RustStringScannerState`.
   - Deterministic lowering to `deap_core::sysml_ast::{PackageDef, PartDef, AttributeDef, ActionDef, PortDef, StateDef, ConstraintDef, RequirementDef}`.
2. **Layer 2 — Logic, Lexing, Lowering & Docstring Synthesis (`crates/deap-core/src/source_ast.rs`, `crates/ingest-sysml/src/lib.rs`, `crates/ingest-sysml/src/translators/code.rs`, `crates/compile-sysml/src/docstring.rs`)**:
   - Zero-regex token-stream validation (`validate_source_syntax_for_language`), comment/literal scanning (`strip_line_comment_preserving_strings`, `scan_rust_line_preserving_lex_state`, `update_ast_delimiter_depths`), and recursive-descent extraction (`parse_rust_source_ast`, `parse_dart_source_ast`).
   - 7-section safety docstring synthesis (`synthesize_7_section_docstring`, `resolve_schema_context`, `inject_rust_file_docstrings`, `inject_dart_file_docstrings`) with Check 34/36 `seen_docblocks` uniqueness and conflict detection.
3. **Layer 3 — CLI & Atomic Persistence Interface Binding (`crates/ingest-sysml/src/main.rs`, `crates/compile-sysml/src/main.rs`)**:
   - `ingest-sysml --source <DIR>` with `IngestExitCode` (`0`, `1`, `2`), atomic model/digest persistence (`write_atomic`, `write_digest_atomic`), and scope/negative-invariant verification.
   - `compile-sysml --inject-safety-docstrings <DIR>` and `--extract-source <DIR>` with `DocstringExitCode` (`0`, `1`, `2`), strict `--schema` validation, `--dry-run`, and `--force` conflict resolution.

---

## 3. Subagent Dispatch Plan & Governance Protocol

In strict compliance with [AGENTS.md](file:///Users/perkunas/jail/DEAP01-spec-core/AGENTS.md#L126-L153) (*Coordinator Delegation & Shell Mutation Lock* and *Mandatory Subagent Dispatch*) and [SKILL.md](file:///Users/perkunas/jail/DEAP01-spec-core/.agents/skills/feature-driven-implementation/SKILL.md#L107-L233) (§ 3.2 *Per-Task Dispatch Loop*, § 3.3 *Two-Stage Review Gate*, § 4.6 *Independent Subagent Validation Check*, and § 5.2 *Solution Walkthrough*), the coordinator will execute **zero direct repository file writes** and instead orchestrate the following **serial, context-isolated subagent dispatch sequence** via `invoke_subagent`:

### 3.1 Serial Subagent Dispatch Schedule

| Dispatch # | Phase / Gate | Subagent `Role` | `TypeName` | `Model` / `Workspace` | Single-Item Scope & Target Files | Verification / Exit Criteria |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| **Dispatch 1** | **Step 3 — Micro-Task 1 Implementer** | `Source AST Lexer Implementer` | `self` | `inherit` / `inherit` | **Single crate (`deap-core`)**: [crates/deap-core/src/source_ast.rs](file:///Users/perkunas/jail/DEAP01-spec-core/crates/deap-core/src/source_ast.rs) (`RED -> GREEN -> REFACTOR`). | Failing test demonstrated RED, then GREEN; `cargo test -p deap-core` and `cargo clippy -p deap-core --all-targets -- -D warnings` pass (`0` warnings). |
| **Dispatch 2** | **Step 3.3 — Micro-Task 1 Two-Stage Reviewer** | `Source AST Two-Stage Reviewer` | `research` | `inherit` / `inherit` | Read-only **Stage 1 (Spec Compliance)** + **Stage 2 (Code Quality)** audit of `crates/deap-core/src/source_ast.rs` diff against FEAT-42 AC-01, AC-02, AC-05, Check 34, and Check 36. | Explicit PASS on both Stage 1 and Stage 2 before Dispatch 3 begins. |
| **Dispatch 3** | **Step 3 — Micro-Task 2 Implementer** | `Ingest-SysML Exit Contract Implementer` | `self` | `inherit` / `inherit` | **Single crate (`ingest-sysml`)**: [crates/ingest-sysml/src/lib.rs](file:///Users/perkunas/jail/DEAP01-spec-core/crates/ingest-sysml/src/lib.rs), [crates/ingest-sysml/src/main.rs](file:///Users/perkunas/jail/DEAP01-spec-core/crates/ingest-sysml/src/main.rs), [crates/ingest-sysml/tests/code_ingestion_tests.rs](file:///Users/perkunas/jail/DEAP01-spec-core/crates/ingest-sysml/tests/code_ingestion_tests.rs) (`RED -> GREEN -> REFACTOR`). | Failing test demonstrated RED, then GREEN; `cargo test -p ingest-sysml` and `cargo clippy -p ingest-sysml --all-targets -- -D warnings` pass (`0` warnings). |
| **Dispatch 4** | **Step 3.3 — Micro-Task 2 Two-Stage Reviewer** | `Ingest-SysML Two-Stage Reviewer` | `research` | `inherit` / `inherit` | Read-only **Stage 1 (Spec Compliance)** + **Stage 2 (Code Quality)** audit of `crates/ingest-sysml` diff against FEAT-42 Section 2.1 exit codes (`0`, `1`, `2`), Check 34, and Check 36. | Explicit PASS on both Stage 1 and Stage 2 before Dispatch 5 begins. |
| **Dispatch 5** | **Step 3 — Micro-Task 3 Implementer** | `Compile-SysML Docstring Implementer` | `self` | `inherit` / `inherit` | **Single crate (`compile-sysml`)**: [crates/compile-sysml/src/docstring.rs](file:///Users/perkunas/jail/DEAP01-spec-core/crates/compile-sysml/src/docstring.rs), [crates/compile-sysml/src/main.rs](file:///Users/perkunas/jail/DEAP01-spec-core/crates/compile-sysml/src/main.rs), [crates/compile-sysml/tests/docstring_injector_tests.rs](file:///Users/perkunas/jail/DEAP01-spec-core/crates/compile-sysml/tests/docstring_injector_tests.rs) (`RED -> GREEN -> REFACTOR`). | Failing test demonstrated RED, then GREEN; `cargo test -p compile-sysml` and `cargo clippy -p compile-sysml --all-targets -- -D warnings` pass (`0` warnings). |
| **Dispatch 6** | **Step 3.3 — Micro-Task 3 Two-Stage Reviewer** | `Compile-SysML Two-Stage Reviewer` | `research` | `inherit` / `inherit` | Read-only **Stage 1 (Spec Compliance)** + **Stage 2 (Code Quality)** audit of `crates/compile-sysml` diff against FEAT-42 Section 2.2 exit codes (`0`, `1`, `2`), Section 3.2 traceability, Check 34, and Check 36. | Explicit PASS on both Stage 1 and Stage 2 before Step 4 verification. |
| **Dispatch 7** | **Step 5.2 — Solution Walkthrough Author** | `FEAT-42 Solution Walkthrough Author` | `self` | `inherit` / `inherit` | Author [docs/designs/feat-458-solution.md](file:///Users/perkunas/jail/DEAP01-spec-core/docs/designs/feat-458-solution.md) and update `Status` to `Implemented` in [docs/designs/FEAT-42_source_code_ast_ingestion_and_docstring_synthesizer.md](file:///Users/perkunas/jail/DEAP01-spec-core/docs/designs/FEAT-42_source_code_ast_ingestion_and_docstring_synthesizer.md). | `./target/release/verify-baseline . --no-domain` passes with `0` errors across all checks (Checks 12, 13, 13B, 18, 28, 30, 34, 36). |
| **Dispatch 8** | **Step 4.6 — Independent Walkthrough Validator** | `Solution Walkthrough Validator` | `research` | `inherit` / `inherit` | Read-only audit of `docs/designs/feat-458-solution.md` cross-referencing every structural identifier, Code Realization Table entry, and link target against the live codebase (`SKILL.md` § 4.6). | 100% verbatim structural identifier and link target verification before git push and issue closure. |

### 3.2 Mandatory Subagent Prompt Governance Preamble Template
Every implementer subagent prompt (`Dispatches 1, 3, 5, 7`) will begin with the verbatim, un-degraded governance preamble mandated by [SKILL.md](file:///Users/perkunas/jail/DEAP01-spec-core/.agents/skills/feature-driven-implementation/SKILL.md#L132-L141):

```text
Adopt the feature-driven-implementation skill (referencing .pipeline/constitution.md).
1. Execute read_file (view_file) on /Users/perkunas/jail/DEAP01-spec-core/.agents/skills/feature-driven-implementation/SKILL.md as step 1 (note: this repository is UPSTREAM_SPEC_CORE_COMPILER as .pipeline/upstream/ exists; platform profiles under .pipeline/profiles/ are bypassed per AGENTS.md).
2. Enforce Section 1.9 Zero-Mocking Live Persistence Mandate (.pipeline/constitution.md).
3. Enforce the 3-Layer Definition of Done (DoD) (Layer 1: Domain State & AST Data Model; Layer 2: Logic, Lexing, Lowering & Docstring Synthesis; Layer 3: CLI & Atomic Persistence Interface Binding) and continuous SysML part def architectural grounding (IngestSysml::SourceCodeAstIngestion and CompileSysml::SafetyDocstringSynthesizer).
4. Execute strict TDD RED-GREEN-REFACTOR cycle: write failing unit/integration tests first, run cargo test to confirm RED, implement minimal production code to reach GREEN, and refactor while keeping tests GREEN.
5. Build/Test Verification Commands: cargo clippy --workspace --all-targets -- -D warnings (0 issues), cargo test --workspace (all pass), ./target/release/verify-baseline . --no-domain (0 issues).
6. Every generated/modified struct, enum, trait, function, method, and type alias MUST carry a unique, non-boilerplate 7-section safety docstring (/// Realises:, /// # Summary, /// # Preconditions, /// # Postconditions, /// # Invariants, /// # Failure Modes, /// # Side Effects, /// # Determinism) and strictly obey Check 34 (zero regex, zero unwrap/expect/panic, zero direct indexing/range slicing &s[a..b], and zero #[allow(...)] suppressions).
---GOVERNANCE-END---
```

Each subagent prompt will also require the **Mandatory Governance Acknowledgment** (`(a)` through `(d)`) prior to any file edit and terminate with `PROCEED`.

### 3.3 Output Integrity, Retry & Immediate Subagent Termination Protocol
1. **Output Integrity Gate (`SKILL.md` § 3.2.A.2)**: After each implementer returns, the coordinator verifies that the response contains:
   - Complete 4-point Governance Acknowledgment (`(a)`–`(d)`).
   - Explicit RED phase test failure output and GREEN phase passing output.
   - Clean `git diff` limited strictly to the micro-task's target files.
2. **Bounded Retry Guard**: Track `subagent_dispatch_retry` per micro-task (maximum 2 retries on empty/truncated output before halting and escalating via "The Grill").
3. **Immediate Termination & Cleanup (`AGENTS.md` § *Mandatory Subagent Termination & Cleanup*)**: Immediately after each subagent dispatch finishes, the coordinator invokes `manage_subagents(Action: "list")` (and `Action: "kill"` if any subagent remains active) so zero subagents are left idle or dormant between dispatches.

---

## 4. Step 3: Sequential TDD Implementation Micro-Tasks

### Micro-Task 1 (Dispatch 1 -> Dispatch 2): Remediate `deap-core::source_ast` Lexer, Shared Rust Line Scanner & Check 34 Self-Verification
- **3-Layer DoD Coverage**:
  - **Layer 1 (Domain State & Data Model)**: Define `ActiveStringState`, `InterpolationFrame`, and `pub enum RustStringScannerState` in `crates/deap-core/src/source_ast.rs`.
  - **Layer 2 (Logic & Lexing)**: Expose `pub fn single_char_literal_byte_len` and `pub fn scan_rust_line_preserving_lex_state`, update `parse_rust_source_ast`, and add `rust_raw_string_hash_count` + `has_matching_interpolation_brace_before_quote` in `strip_line_comment_preserving_strings` and `update_ast_delimiter_depths`.
  - **Layer 3 (CLI & Interface Binding)**: Export `RustStringScannerState`, `scan_rust_line_preserving_lex_state`, and `single_char_literal_byte_len` across the `deap_core::source_ast` public crate interface for `ingest-sysml` and `compile-sysml` (completed in Micro-Tasks 2 and 3).
- **Target File**:
  - [crates/deap-core/src/source_ast.rs](file:///Users/perkunas/jail/DEAP01-spec-core/crates/deap-core/src/source_ast.rs) (lines `534-595`, `852-915`, `3490-3579`, `3723-3842`, `4010-4058`)
- **Mandatory Subagent Task Checkboxes**:
  - [ ] Replace `#[allow(clippy::type_complexity)]` at line `545` with `type ActiveStringState = (u8, bool, bool, usize, usize);` and `type InterpolationFrame = (ActiveStringState, usize);` (with unique 7-section docstrings).
  - [ ] Make `single_char_literal_byte_len` `pub fn` and expose `pub enum RustStringScannerState` and `pub fn scan_rust_line_preserving_lex_state(line: &str, state: &mut RustStringScannerState) -> String` using `single_char_literal_byte_len` with zero magic byte offsets (`close_pos <= 10` eliminated).
  - [ ] Update `parse_rust_source_ast` (`lines 857-905`) to use `RustStringScannerState` and `scan_rust_line_preserving_lex_state`.
  - [ ] Add `rust_raw_string_hash_count` and `has_matching_interpolation_brace_before_quote` so `strip_line_comment_preserving_strings` and `update_ast_delimiter_depths` handle `r#"..."#` hashed raw strings and unclosed `"${"` literals in Rust without corrupting `paren_depth` or `brace_depth`.
  - [ ] Upgrade `test_strip_comment_and_delimiter_depths_zero_range_slice_indexing` to scan all non-test production lines of `source_ast.rs` with `contains_direct_range_slice`, assert `!prod_source.contains("#[allow(clippy::type_complexity)]")`, and add `test_scan_rust_line_preserving_lex_state_char_literals_and_lifetimes`.
- **Driving Tests (RED -> GREEN)**:
  - `cargo test -p deap-core --lib -- test_strip_comment_and_delimiter_depths_zero_range_slice_indexing`
  - `cargo test -p deap-core --lib -- test_scan_rust_line_preserving_lex_state_char_literals_and_lifetimes`
- **Verification**:
  - `cargo test -p deap-core` and `cargo clippy -p deap-core --all-targets -- -D warnings`.

### Micro-Task 2 (Dispatch 3 -> Dispatch 4): Enforce `ingest-sysml` Exit Code `0 / 1 / 2` Contract & Clean Test Lint Hygiene
- **3-Layer DoD Coverage**:
  - **Layer 1 (Domain State & Data Model)**: Define `pub enum IngestExitCode { Clean = 0, ParseOrIoError = 1, ScopeOrCryptoError = 2 }` with `pub const fn code(self) -> i32` in `crates/ingest-sysml/src/lib.rs`.
  - **Layer 2 (Logic & Verification)**: Implement `pub fn verify_scope_and_negative_invariants(pkg: &PackageDef, allowed_parts: Option<&HashSet<String>>, negative_invariants: Option<&[String]>) -> Result<(), String>` in `crates/ingest-sysml/src/lib.rs`.
  - **Layer 3 (CLI & Persistence Interface Binding)**: Wire `run_ingestion_with_code(args: CliArgs) -> Result<(), (IngestExitCode, String)>` and `main()` in `crates/ingest-sysml/src/main.rs` to exit with `0`, `1`, or `2` per FEAT-42 Section 2.1.
- **Target Files**:
  - [crates/ingest-sysml/src/lib.rs](file:///Users/perkunas/jail/DEAP01-spec-core/crates/ingest-sysml/src/lib.rs) (lines `218-331`, `502-616`)
  - [crates/ingest-sysml/src/main.rs](file:///Users/perkunas/jail/DEAP01-spec-core/crates/ingest-sysml/src/main.rs) (lines `220-498`)
  - [crates/ingest-sysml/tests/code_ingestion_tests.rs](file:///Users/perkunas/jail/DEAP01-spec-core/crates/ingest-sysml/tests/code_ingestion_tests.rs) (lines `1-269`)
- **Mandatory Subagent Task Checkboxes**:
  - [ ] Define `pub enum IngestExitCode` (`Clean = 0`, `ParseOrIoError = 1`, `ScopeOrCryptoError = 2`) with `pub const fn code(self) -> i32` and full 7-section docstrings.
  - [ ] Implement `pub fn verify_scope_and_negative_invariants` in `crates/ingest-sysml/src/lib.rs` to reject empty `--allowed-parts` matches and forbidden `--negative-invariants` entities remaining in `pkg`.
  - [ ] Update `run_ingestion_with_code` and `main()` in `crates/ingest-sysml/src/main.rs` so parsing/I/O failures return exit code `1`, scope/negative-invariant or post-write SHA-256 digest failures return exit code `2`, and clean runs return `0`.
  - [ ] Remove `#![allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing, clippy::panic)]` from `crates/ingest-sysml/tests/code_ingestion_tests.rs`, convert tests to return `Result<(), String>`, and add `test_ingest_sysml_cli_exit_codes_0_1_and_2`.
- **Driving Tests (RED -> GREEN)**:
  - `cargo test -p ingest-sysml --test code_ingestion_tests -- test_ingest_sysml_cli_exit_codes_0_1_and_2`
- **Verification**:
  - `cargo test -p ingest-sysml` and `cargo clippy -p ingest-sysml --all-targets -- -D warnings`.

### Micro-Task 3 (Dispatch 5 -> Dispatch 6): Remediate `compile-sysml::docstring` Traceability, Conflict Detection (Exit Code `2`), Schema Validation (Exit Code `1`) & Test Hygiene
- **3-Layer DoD Coverage**:
  - **Layer 1 (Domain State & Data Model)**: Re-export and bind `deap_core::source_ast::{RustStringScannerState, scan_rust_line_preserving_lex_state}` and `DocstringExitCode` (`Clean = 0`, `Error = 1`, `IdempotencyOrConflict = 2`) in `crates/compile-sysml/src/docstring.rs`.
  - **Layer 2 (Logic & Docstring Synthesis)**: Remediate `resolve_schema_context` (eliminating `matched_entity_pkg` `.first()` fallbacks and method-name `PartDef` collisions; traversing nested `PartDef::parts`, inline `PartDef::requirements`, `PartDef::safety_realises`, `RequirementDef::satisfied_by`, and `owner_part.actions`), add `!force` conflicting `Realises:` detection returning `DocstringExitCode::IdempotencyOrConflict` (`2`), fix inline `*/` closure in `inject_rust_file_docstrings`, and fix `->` / multiline `impl` target extraction in `extract_rust_impl_target_name`.
  - **Layer 3 (CLI & Persistence Interface Binding)**: Update `crates/compile-sysml/src/main.rs:212-227` so `--inject-safety-docstrings` fails closed with `DocstringExitCode::Error` (`1`) when `--schema` (or discovered `schema/*.sysml`) fails to read or parse.
- **Target Files**:
  - [crates/compile-sysml/src/docstring.rs](file:///Users/perkunas/jail/DEAP01-spec-core/crates/compile-sysml/src/docstring.rs) (lines `348-807`, `1083-1148`, `1587-2111`, `2366-2561`)
  - [crates/compile-sysml/src/main.rs](file:///Users/perkunas/jail/DEAP01-spec-core/crates/compile-sysml/src/main.rs) (lines `212-244`)
  - [crates/compile-sysml/tests/docstring_injector_tests.rs](file:///Users/perkunas/jail/DEAP01-spec-core/crates/compile-sysml/tests/docstring_injector_tests.rs) (lines `1-405`)
- **Mandatory Subagent Task Checkboxes**:
  - [ ] Fail closed with `DocstringExitCode::Error` (`1`) in `crates/compile-sysml/src/main.rs` when `--schema` (or discovered `schema/*.sysml`) cannot be read or fails `parse_sysml` / `parse_sysml_tree`.
  - [ ] Delegate `RustStringScannerState` and `scan_rust_line_preserving_lex_state` in `crates/compile-sysml/src/docstring.rs` to `deap_core::source_ast` (removing duplicate scanner and `close_pos <= 10`).
  - [ ] Remove `matched_entity_pkg` `.first()` fallbacks from `resolve_schema_context`, scope `PartDef` matching to `owner_name` when `owner_name.is_some()`, and traverse nested `PartDef::parts`, inline `PartDef::requirements`, `PartDef::safety_realises`, `RequirementDef::satisfied_by`, and `owner_part.actions`.
  - [ ] Detect conflicting unforced docstrings when `!force` (returning `DocstringExitCode::IdempotencyOrConflict` = `2` when an existing 7-section docstring's explicit `Realises:` list conflicts with the entity's schema requirement, and overwriting when `--force` is passed).
  - [ ] Fix `if !started_in_normal && lex_state != RustStringScannerState::Normal` in `inject_rust_file_docstrings` and update `extract_rust_impl_target_name` for `Fn(...) -> Ret` bounds and multiline `impl ... where ... {` headers.
  - [ ] Replace the `std::mem::size_of::<PartDef>()` test in `docstring.rs` with an explicit multi-requirement `PackageDef` test, remove `#![allow(clippy::...)]` from `crates/compile-sysml/tests/docstring_injector_tests.rs`, and add CLI exit code `1` and `2` integration tests.
- **Driving Tests (RED -> GREEN)**:
  - `cargo test -p compile-sysml --lib`
  - `cargo test -p compile-sysml --test docstring_injector_tests`
- **Verification**:
  - `cargo test -p compile-sysml` and `cargo clippy -p compile-sysml --all-targets -- -D warnings`.

---

## 5. Step 4 & Step 5: Post-Implementation Verification, Solution Walkthrough & Issue Closure

1. **Step 4 — Full Compilation Build & Baseline Gate (Coordinator Read-Only Verification)**:
   - Build release binaries (`cargo build --release --workspace`) and execute `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, and `./target/release/verify-baseline . --no-domain`.
2. **Step 5.2 — Solution Walkthrough Authoring (Dispatch 7) & Independent Validation (Dispatch 8)**:
   - **Dispatch 7 (`FEAT-42 Solution Walkthrough Author`)**: Author `docs/designs/feat-458-solution.md` (with YAML frontmatter, Code Realization Table, raw test/baseline outputs, step-by-step manual testing instructions, and `## Source References`) and update `Status` to `Implemented` in [FEAT-42_source_code_ast_ingestion_and_docstring_synthesizer.md](file:///Users/perkunas/jail/DEAP01-spec-core/docs/designs/FEAT-42_source_code_ast_ingestion_and_docstring_synthesizer.md).
   - **Dispatch 8 (`Solution Walkthrough Validator`)**: Independently cross-reference every structural identifier and link target in `docs/designs/feat-458-solution.md` against the codebase per `SKILL.md` § 4.6.
3. **Step 5.3–5.5 — Backlog Reconciliation, Remote Push & Tracker Transition**:
   - Run `./target/release/verify-baseline . --no-domain` and `./target/release/reconcile-backlog`.
   - Commit and push all verified changes to `origin/main` using neutral citations (`(refs #458, refs #492, refs #498)`), verify `git diff origin/main` is empty, apply `status:fixed-resolved` on GitHub Issue `#458`, post the resolution comment, and verify the live payload via `gh issue view 458 --repo gintatkinson/DEAP01-spec-core`.
