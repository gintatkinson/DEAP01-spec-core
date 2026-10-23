# Implementation Plan: Full Remediation of Unfixed Issues `#483`–`#489`, `#465`–`#472`, and `#481`

## 0. Forensic Audit Summary (Why These 16 Issues Were Never Fixed)

An empirical line-by-line audit of `gintatkinson/DEAP01-spec-core` (`HEAD ab70f44`), `gintatkinson/DEAP01-cuas-coa-engine` (`HEAD ecd9b473`), and the live GitHub issue tracker confirmed two systemic failures:

1. **Tracker Closure & Premature Label Fraud Across All 16 Issues (`#465`–`#472`, `#481`, `#483`–`#489`)**:
   - Every single one of `#465`, `#466`, `#467`, `#468`, `#469`, `#470`, `#471`, `#472`, `#481`, `#483`, `#484`, `#485`, `#486`, `#487`, `#488`, and `#489` is still **`state: OPEN`** on GitHub while carrying the `status:fixed-resolved` label.
2. **Hollow Stubs, Check 19 Domain Violations, Silent Early-Return Bypass Bugs, and Missing Code Across `#483`–`#489` and `#468`**:
   - **`#483` & `#484` (`crates/compile-sysml/src/flutter_codegen.rs` & `crates/verify-baseline/src/checks/codebase.rs`)**:
     - `generate_solver_bridge_dart` ([flutter_codegen.rs](file:///Users/perkunas/jail/DEAP01-spec-core/crates/compile-sysml/src/flutter_codegen.rs#L1060-L1152)) is a **90-line hollow stub**:
       - It does **not** generate the Rust C-ABI shim (`crates/<domain-engine>/src/ffi.rs` and `include/deap_solver_abi.h`) with `#[repr(C)] pub struct DeapByteBuffer`, `DeapFfiStatus`, `std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| ...))` returning `DeapFfiStatus::InternalPanic = 5`, `deap_get_abi_version`, `deap_get_schema_hash`, `deap_evaluate_port`, and `deap_free_buffer`.
       - Its generated `solver_bridge.g.dart` has **zero** `DynamicLibrary.open` / `lookupFunction` FFI bindings, **zero** IEEE 802.3 CRC32 verification (`computeCrc32`), **zero** `try / finally { deap_free_buffer(buf); }`, **zero** `NativeFinalizer`, and **zero** background worker isolate (`Isolate.run` / `TransferableTypedData`) execution.
       - Its generated port DTOs ignore SysML `item def` attributes and `calc def` parameters, emitting only an untyped `final Map<String, dynamic> payload;`.
     - `check_solver_bridge_ssot_delegation` ([codebase.rs](file:///Users/perkunas/jail/DEAP01-spec-core/crates/verify-baseline/src/checks/codebase.rs#L873-L955)) **violates Check 19 (`Pure Schema-Driven Compiler Invariant`)** by hardcoding C-UAS identifiers (`_buildCandidates`, `alphaCovOffset`, `reserveBoostPct`, `extraDensityPenalty`, `paretoScore`, `violationCount: 0`) at lines `914-922`, and fails to require `app_flutter/lib/data/generated/solver_bridge.g.dart` when a Rust solver crate exists.
   - **`#485` & `#489` (`crates/compile-sysml/src/flutter_codegen.rs` & `crates/verify-baseline/src/checks/codebase.rs`)**:
     - `check_bijective_sqlite_seed_parity` ([codebase.rs](file:///Users/perkunas/jail/DEAP01-spec-core/crates/verify-baseline/src/checks/codebase.rs#L981-L1052)) contains a **silent early-return bypass** at lines `1003-1006`: `let seeds_dir = repo_root.join("app_flutter/lib/data/seeds"); if !seeds_dir.is_dir() { return; }`. When run against `DEAP01-cuas-coa-engine` (which has `app_flutter/lib/data/domain_seed_strategy.dart` and no `seeds/` directory), **Check 41 silently passes (`0 errors`)** without inspecting `domain_seed_strategy.dart` and without requiring `generated_domain_seed.g.dart`!
     - `emit_flutter_contracts` ([flutter_codegen.rs](file:///Users/perkunas/jail/DEAP01-spec-core/crates/compile-sysml/src/flutter_codegen.rs#L1393-L1395)) hashes an empty string `""` when `topology_json` is `None` (loaded from disk) instead of hashing the full `.pipeline/schema.sysml` + `topology_data.json` contents, and `generate_domain_seed_dart` fails to emit `type_definitions` rows for `topology_data.json` node `type_name` values not present in `model.parts` (which breaks SQLite `FOREIGN KEY (type_name) REFERENCES type_definitions(type_name)`).
   - **`#486` (`crates/compile-sysml/src/flutter_codegen.rs` & `crates/verify-baseline/src/checks/codebase.rs`)**:
     - `generate_ui_contract_test_dart` ([flutter_codegen.rs](file:///Users/perkunas/jail/DEAP01-spec-core/crates/compile-sysml/src/flutter_codegen.rs#L1183-L1321)) emits **fake hardcoded rectangles** (`const Rect.fromLTWH(16, 16, 320, 64)`, `const Rect.fromLTWH(100, 100, 48, 48)`), **commented-out `tester.tap` calls** (`// await tester.tap(find.byKey(...));`), and an unformatted literal `'{}'` bug (`expect(SchemaWidgetKeys.entityRow('{}'), isNotNull);`) instead of pumping the widget tree across `1920x1080`, `2560x1440`, `3840x2160`, and `6016x3384`, querying live `tester.getRect(...)` `RenderBox` bounds, and executing real `tester.tap` / `tester.drag` gestures.
     - `check_40_interactive_gesture_and_hud_non_overlap_gate` ([codebase.rs](file:///Users/perkunas/jail/DEAP01-spec-core/crates/verify-baseline/src/checks/codebase.rs#L607-L660)) **silently passes** when `app_flutter/test/generated/schema_ui_contract_test.g.dart` does not exist (`if contract_path.is_file()`), and `check_39_zero_mocking_test_ast_gate` ([codebase.rs](file:///Users/perkunas/jail/DEAP01-spec-core/crates/verify-baseline/src/checks/codebase.rs#L745-L753)) **violates Check 19** by hardcoding 7 C-UAS method names (`bridge.relockInterlock(`, `bridge.triggerHotlRfJamRetasking(`, etc.).
   - **`#487` (`crates/verify-baseline/src/checks/codebase.rs` — Check 38)**:
     - `check_presentation_line` ([codebase.rs](file:///Users/perkunas/jail/DEAP01-spec-core/crates/verify-baseline/src/checks/codebase.rs#L335-L408)) hardcodes `"12x CLASS-"` (violating Check 19) and only inspects single-line `Text('...')` calls where `Text(` and `)` are on the same line.
     - Empirical execution of `./target/release/verify-baseline /Users/perkunas/jail/DEAP01-cuas-coa-engine --no-domain` proved that Check 38 **missed almost every hardcoded literal reported in `#487`**: `'Asset-Alpha (P1)'`, `'Asset-Bravo (P2)'`, `id: 'SECTOR-ALPHA-01'`, `'2 NODES'`, `'75% Reserve * EW First'` in `sidebar_tree.dart`; `'FOCUSED ASSET: Asset-Alpha'`, `'NORTH-01 v'`, `'RADAR-01: 15km'`, `'EW-JAM-02: ENGAGED'`, `'[AIR-GAP]'` in `topographical_view.dart`; and `'LOW (0.0%)'`, `'2 / 8 INTERCEPTORS'`, `'6 / 8 INTERCEPTORS'`, `'(SOLVER CONF: 99.2%)'`, `'WAVE N+1: PASS'` in `coa_inspector_panel.dart`.
   - **`#488` (`crates/verify-baseline/src/defect_dossier.rs`)**:
     - `validate_defect_dossier_scoped` ([defect_dossier.rs](file:///Users/perkunas/jail/DEAP01-spec-core/crates/verify-baseline/src/defect_dossier.rs#L1143-L1230)) omitted **Gate 13 Check 19 domain-token scanning** inside Section 6 ` ```rust ` blocks (when targeting an upstream crate) and omitted **Gate 14 colliding `Check <N>` number validation**.
   - **`#468` (`crates/reconcile-backlog/src/lib.rs` & `crates/deap-core/src/diagnostics.rs`)**:
     - `crates/reconcile-backlog/src/lib.rs` has **zero code** parsing or executing `<!-- test-target: [path] -->` / `- **Test-Target**:` before allowing a `bug` issue to transition to `status:fixed-resolved`. It was labeled `status:fixed-resolved` without a single line of `test-target` enforcement being written in `reconcile-backlog`!
   - **`#465`, `#466`, `#467`, `#469`, `#470`, `#471`, `#472`, `#481`**:
     - Left `OPEN` on GitHub; additionally, `#465`'s `docstring.rs` generator emits repetitive mechanical boilerplate (`Time Complexity: O(N(arg)) bounded over input/member dimension of fn`) across crates, and `#470`/`#471`'s `parse_sysml_tree` still returns `PackageDef { name: "SysML_Model", ... }` when `root.packages.len() > 1` unless consumers inspect `is_synthetic_forest_root`.

---

## 1. Work Package Breakdown (100% Subagent-Delegated After `PROCEED`)

### Work Package 1: Fix `#488` Completely in `crates/verify-baseline/src/defect_dossier.rs`
- **Target Files**:
  - `crates/verify-baseline/src/defect_dossier.rs`
  - `crates/verify-baseline/tests/defect_dossier_tests.rs`
- **Changes**:
  1. **Gate 13 (Section 6 Check 34 + Check 19 Enforcement)**:
     - Keep the deterministic linear scanner rejecting `regex::`, `Regex::new`, `RegexSet`, and `use regex` inside Section 6 ` ```rust ` blocks.
     - Add Check 19 domain-cleanliness enforcement on Section 6 ` ```rust ` blocks whenever `test-target` or `- **File**:` targets one of the 6 upstream compiler crates (`crates/{deap-core,compile-sysml,ingest-sysml,verify-baseline,assemble-conops,reconcile-backlog}/`), rejecting hardcoded customer domain literals (`ASSET-`, `SECTOR-`, `INTERCEPTORS`, `RADAR-`, `EW-JAM-`, `cuas`, `alphaCovOffset`, etc.) via `domain_cleanliness_gate` rules.
  2. **Gate 14 (Upstream `test-target` Grounding + Colliding `Check <N>` Detection)**:
     - Enforce that in an `UPSTREAM_SPEC_CORE_COMPILER` workspace, `test-target` must belong to one of the 6 upstream crates and its crate directory must exist.
     - Enforce that if a dossier proposes a new `Check <N>` (e.g., `Check 34: <Title>`), `<N>` must not collide with an already-assigned canonical check number (`Checks 1..=37`) in `runner.rs` with a different title.
  3. **TDD Verification**:
     - Write failing unit tests in `crates/verify-baseline/tests/defect_dossier_tests.rs` first (RED), implement Gates 13 and 14 (GREEN), and run `cargo test -p verify-baseline --test defect_dossier_tests`.

---

### Work Package 2: Fix `#468` Completely in `crates/reconcile-backlog/src/lib.rs` (plus Verify `#465`–`#467`, `#469`–`#472`, `#481`)
- **Target Files**:
  - `crates/reconcile-backlog/src/lib.rs`
  - `crates/reconcile-backlog/src/main.rs`
- **Changes**:
  1. **Implement `#468` `test-target` Reconciliation Contract in `crates/reconcile-backlog/src/lib.rs`**:
     - Add deterministic linear extraction `extract_defect_test_target(issue_body: &str) -> Option<String>` that parses `<!-- test-target: <path> -->` and `- **Test-Target**: \`<path>\`` from `bug` issue bodies and verifies both match.
     - Add `verify_defect_test_target(repo_root: &Path, test_target: &str) -> Result<(), String>` that verifies the `test-target` file exists on disk under `crates/` and executes `cargo test` for that target crate/test binary, blocking transition to `status:fixed-resolved` (and issue closure) if `test-target` is missing or `cargo test` fails.
  2. **Verify & Harden `#465`–`#467`, `#469`–`#472`, `#481`**:
     - Verify `Check 33` (`#469`), `Check 36` (`#465`), `adversarial-code-auditor/SKILL.md` (`#466`, `#467`), `parse_sysml_tree` (`#470`, `#471`), `ingest-sysml` (`#472`), and `rules/verification-required.md` (`#481`) with dedicated unit tests across the workspace.

---

### Work Package 3: Replace Hollow Codegen Stubs in `crates/compile-sysml/src/flutter_codegen.rs` (`#483`, `#484`, `#485`, `#486`, `#489`)
- **Target Files**:
  - `crates/compile-sysml/src/flutter_codegen.rs`
  - `crates/compile-sysml/tests/flutter_codegen_tests.rs`
- **Changes**:
  1. **Fix `compute_schema_hash` & Disk Topology Loading (`#485`, `#489`)**:
     - When `emit_flutter_contracts` loads `app_flutter/assets/topology_data.json` and `.pipeline/schema.sysml` from disk, compute `DEAP_SCHEMA_HASH: u64` over the full canonical `.pipeline/schema.sysml` serialization + full `topology_data.json` bytes (never `""`).
     - In `generate_domain_seed_dart`, emit `type_definitions` rows for **both** `model.parts` and every distinct `node.type_name` present in `topology_data.json` so SQLite `FOREIGN KEY (type_name) REFERENCES type_definitions(type_name)` never fails at runtime.
  2. **Replace `generate_solver_bridge_dart` Stub with Full 6-Layer C-ABI Codegen (`#483`, `#484`)**:
     - Generate `app_flutter/lib/data/generated/solver_bridge.g.dart` containing:
       - `DEAP_ABI_VERSION = 1` and `DEAP_SCHEMA_HASH = 0x...` constants.
       - Typed Dart DTO classes for every `item def` and `port def` in `SysmlModel` with strongly-typed fields mapped from `ItemDef.attributes` and `CalcDef` parameters (`Real` -> `double`, `Integer` -> `int`, `Boolean` -> `bool`, `String` -> `String`), `toJson()`, and `fromJson()`.
       - `dart:ffi` `#[repr(C)]` `final class DeapByteBuffer extends Struct` (`Pointer<Uint8> ptr`, `@Size() int len`, `@Size() int cap`, `@Uint32() int crc32`, `@Uint32() int status`) and `DeapFfiStatus` enum (`ok(0)`, `invalidArgument(1)`, `schemaVersionMismatch(2)`, `crcMismatch(3)`, `payloadTooLarge(4)`, `internalPanic(5)`).
       - `DynamicLibrary` FFI bindings (`deap_get_abi_version`, `deap_get_schema_hash`, `deap_evaluate_port`, `deap_free_buffer`), pure-Dart IEEE 802.3 `computeCrc32(Uint8List bytes)`, `MAX_FFI_PAYLOAD_BYTES = 16777216`, `try / finally { deapFreeBuffer(buf); }` copy-out barrier, `NativeFinalizer` safety net, and `Isolate.run` background worker isolate dispatch.
     - Also emit `include/deap_solver_abi.h` (or Rust FFI contract module `generated_solver_ffi_contract.rs` when `crates/` contains a domain solver crate) implementing the `#[repr(C)] pub struct DeapByteBuffer`, `DeapFfiStatus`, and `std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| ...))` C-ABI entry points.
  3. **Replace `generate_ui_contract_test_dart` Fake Rectangles with Real 4-Viewport `RenderBox` & Gesture Contract Tests (`#486`, `#489`)**:
     - Remove the fake `const Rect.fromLTWH(...)` constants, commented-out `// await tester.tap(...)`, and literal `'{}'` bug.
     - Emit executable `testWidgets` loops across all 4 mandatory viewports (`Size(1920, 1080)`, `Size(2560, 1440)`, `Size(3840, 2160)`, `Size(6016, 3384)`) that:
       - Bind `tester.view.physicalSize = viewport; tester.view.devicePixelRatio = 1.0;`.
       - Query live `RenderBox` geometry via `tester.getRect(find.byKey(...))` on every discovered `ValueKey('deap_hud_...')` and `ValueKey('deap_entity_...')` widget, asserting `hudA.intersect(hudB).isEmpty` and `!hudRect.overlaps(tester.getCenter(entityFinder))`.
       - Execute real `await tester.tap(...)` and `await tester.pump()` on interactive schema controls (`SchemaWidgetKeys.actionButton(actionName)` / `SchemaWidgetKeys.entityRow(id)`) and assert state/notifier progression.

---

### Work Package 4: Make Checks 38, 39, 40, and 41 in `crates/verify-baseline/src/checks/codebase.rs` 100% Domain-Agnostic (Check 19) and Fail-Closed (`#484`, `#485`, `#486`, `#487`)
- **Target Files**:
  - `crates/verify-baseline/src/checks/codebase.rs`
  - `crates/verify-baseline/tests/codebase_checks_tests.rs`
- **Changes**:
  1. **Zero Hardcoded C-UAS Tokens in `codebase.rs` (Strict Check 19 Compliance)**:
     - Remove every hardcoded C-UAS string from `crates/verify-baseline/src/checks/codebase.rs` (`"12x CLASS-"`, `bridge.relockInterlock(`, `bridge.triggerHotlRfJamRetasking(`, `_buildCandidates`, `alphaCovOffset`, `reserveBoostPct`, `extraDensityPenalty`, `paretoScore`, `violationCount: 0`, etc.).
  2. **Check 38 (`check_38_presentation_anti_hardcoding_gate` — `#487`)**:
     - Scan multi-line and single-line `Text(...)`, widget constructors, and collection literals inside `app_flutter/lib/features/**/*.dart` and `app_flutter/lib/presentation/**/*.dart`.
     - Reject any string literal containing:
       - Any instance ID or label dynamically loaded from `app_flutter/assets/topology_data.json` or `.pipeline/schema.sysml` (including substring matches such as `'Asset-Alpha (P1)'`, `'FOCUSED ASSET: Asset-Alpha'`, `'SECTOR-ALPHA-01'`).
       - Hardcoded telemetry/unit patterns (`<number>%`, `<number>km`, `<number> / <number>`, `(<number>%)`, `T-<digits>:<digits>`, `<number>x `) inside presentation literals.
       - Hardcoded selection/status literals (`isActive: true`, `isActive: false`, `id: '...'`) inside presentation widget trees.
     - Verify empirically that running Check 38 against `/Users/perkunas/jail/DEAP01-cuas-coa-engine` catches **every single** violation listed in `#487` (`sidebar_tree.dart`, `topographical_view.dart`, `coa_inspector_panel.dart`).
  3. **Check 39 (`check_39_zero_mocking_test_ast_gate` — `#486`)**:
     - Reject direct controller/bridge state-mutation calls inside `app_flutter/test/**/*_test.dart` whenever `testWidgets` does not invoke `tester.tap`, `tester.drag`, `tester.longPress`, `tester.enterText`, or `tester.sendKeyEvent` prior to `expect(...)`, using generic AST/token structural detection (e.g., calling mutator methods on `*Bridge` / `*Controller` / `*Notifier` in `testWidgets` without preceding `tester.` gesture dispatch) instead of hardcoded C-UAS method names.
  4. **Check 40 (`check_40_interactive_gesture_and_hud_non_overlap_gate` — `#486`)**:
     - Make Check 40 **fail-closed**: if `app_flutter/lib/` and `.pipeline/schema.sysml` exist in a downstream workspace, `app_flutter/test/generated/schema_ui_contract_test.g.dart` **must** exist on disk (missing file is a hard `Check 40` error, not a silent `Ok(())`), must contain real `tester.getRect(` and `tester.tap(` calls (not comments or `Rect.fromLTWH` constants), and must test all 4 viewports (`1920`, `2560`, `3840`, `6016`).
  5. **Check 41 (`check_41_solver_ssot_and_sqlite_seed_parity_gate` — `#484`, `#485`)**:
     - **Solver Bridge SSOT Delegation (`#484`)**:
       - Dynamically extract `calc def` and `action def` identifiers and arithmetic formulas from `.pipeline/schema.sysml` and solver functions from `crates/*/src/**/*.rs`.
       - Scan all `app_flutter/lib/**/*bridge*.dart` files: require `app_flutter/lib/data/generated/solver_bridge.g.dart` to exist when a domain solver crate is present, and reject any hand-written `*bridge*.dart` file that exceeds the thin FFI binding threshold or performs arithmetic synthesis (`+`, `-`, `*`, `/`, `.clamp(`) inside candidate/COA/telemetry synthesis methods without invoking `dart:ffi` (`DynamicLibrary` / `lookupFunction`).
     - **Bijective SQLite Seed Parity (`#485`)**:
       - Remove the silent `if !seeds_dir.is_dir() { return; }` bypass!
       - Whenever `app_flutter/assets/topology_data.json` exists, scan `app_flutter/lib/data/seeds/generated_domain_seed.g.dart` **and** `app_flutter/lib/data/domain_seed_strategy.dart`:
         - Require every entity `id` in `topology_data.json` to be seeded into SQLite `instances`.
         - Reject legacy telecom placeholder seed rows (`NetworkElement`, `BtsSite`, `CoreRouter`, `CellSector`) when those types are absent from `.pipeline/schema.sysml`.

---

### Work Package 5: Downstream C-UAS Verification & Full Tracker Closure (`#465`–`#472`, `#481`, `#483`–`#489`)
- **Deliverables**:
  1. Run `cargo test --workspace` and `./target/release/verify-baseline . --no-domain` in `DEAP01-spec-core` (`Checks 10..=41` green).
  2. Clean up the untracked `.pipeline/defects/` file created during our diagnostic check in `/Users/perkunas/jail/DEAP01-cuas-coa-engine` and empirically verify that running `DEAP01-spec-core`'s `./target/release/verify-baseline` against `/Users/perkunas/jail/DEAP01-cuas-coa-engine` deterministically catches every unfixed downstream violation (`#484` `cuas_coa_solver_bridge.dart`, `#485` `domain_seed_strategy.dart`, `#486` `cuas_tactical_cockpit_test.dart`, `#487` `sidebar_tree.dart` / `topographical_view.dart` / `coa_inspector_panel.dart`), and that running `./target/release/compile-sysml --emit-flutter-contracts` generates the valid contract files.
  3. Commit and push all upstream fixes to `origin/main` in `gintatkinson/DEAP01-spec-core` (`git diff origin/main` empty).
  4. Run `./target/release/reconcile-backlog` and close (`gh issue close`) all verified upstream issues `#465`, `#466`, `#467`, `#468`, `#469`, `#470`, `#471`, `#472`, `#481`, `#483`, `#484`, `#485`, `#486`, `#487`, `#488`, and `#489` with empirical verification evidence comments, verifying via `gh issue view` that their state is `CLOSED` and `status:fixed-resolved`.
