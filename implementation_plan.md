# Implementation Plan: Zero-Trust Rust C-ABI Blueprint, Option 1C Full Solution Blueprint, Rust Adversarial Auditor Upgrade & Rust Compiler/Verifier Implementation (`#483`–`#487`)

## 1. Objective & Complete Scope

This implementation plan preserves **100% of the originally planned Work Packages 1, 2, and 3** and integrates the **Rust Adversarial Auditor upgrades (`defect_dossier.rs` + `#484`–`#487`)** and **Native Rust Codegen & AST Gate implementations (`crates/compile-sysml` + `crates/verify-baseline`)** so that Rust performs the heavy lifting deterministically in milliseconds.

**All execution across every work package is delegated 100% to context-isolated single-item subagents running strictly inside the standard sandbox (`BypassSandbox` is strictly forbidden, guaranteeing zero IDE permission popups).**

### Planned Execution Sequence
1. **Work Package 0 -- Upgrade Adversarial Auditor in Rust (`defect_dossier.rs`), File Auditor Defect & Sanitize `#484`–`#487`**:
   - Move adversarial dossier validation rules into Rust (`crates/verify-baseline/src/defect_dossier.rs`) so `verify-baseline check-defect` mechanically enforces:
     - **Gate 13 (Section 6 Rust AST & Invariant Validator)**: Rejects forbidden `regex` crate usage (`regex::`, `Regex::new`, `RegexSet`, `use regex` -- Check 34) and hardcoded customer domain strings (Check 19) inside Section 6 ` ```rust ` blocks before any dossier can be persisted or published.
     - **Gate 14 (Target-Repo Crate & Check-Number Collision Validator)**: Rejects downstream `test-target` crate paths (such as `crates/cuas-coa-engine/src/lib.rs`) when targeting `UPSTREAM_SPEC_CORE_COMPILER` (`DEAP01-spec-core`), requiring one of the 6 upstream crates (`deap-core`, `compile-sysml`, `ingest-sysml`, `verify-baseline`, `assemble-conops`, `reconcile-backlog`), and rejects colliding `Check <N>` numbers in proposed corrections.
   - File `.pipeline/defects/DEFECT_AUDITOR_SECTION6_ZERO_REGEX_AND_UPSTREAM_TARGET.md` via `./target/release/verify-baseline check-defect`, update `.agents/skills/adversarial-code-auditor/SKILL.md` (removing `BypassSandbox: true`), and sanitize live GitHub issues `#484`, `#485`, `#486`, and `#487` so their Section 6 Rust code is 100% zero-regex, domain-agnostic, and aligned to `Checks 38..=41`.
2. **Work Package 1 (First Priority) -- Author `docs/designs/zero-trust-deterministic-rust-c-abi-blueprint.md` (`DEAP-BLUEPRINT-CABI-001`) & Sync GitHub Feature Issue `#483` (Addressing `#484`)**:
   - Author the complete `DEAP-BLUEPRINT-CABI-001` blueprint detailing the 6-layer Zero-Trust Deterministic Rust C-ABI Solver Bridge Architecture (`#![forbid(unsafe_code)]` on `solver.rs`, `catch_unwind` on `ffi.rs`, `DEAP_SCHEMA_HASH` SHA-256 handshake, IEEE 802.3 CRC32 `DeapByteBuffer`, `NativeFinalizer` guard, `DeapNativeSolverIsolate` worker isolate, and `compile-sysml` generation of `app_flutter/lib/data/generated/solver_bridge.g.dart` from SysML `port def`/`item def`/`calc def` AST nodes to eliminate the 5,971-line hand-written Dart bridge in `#484`), and synchronize GitHub Feature Issue `#483`.
3. **Work Package 2 (Second Priority) -- Author `docs/designs/option-1c-rust-app-glue-ui-contracts-and-anti-fabrication-blueprint.md` (`DEAP-BLUEPRINT-APP-GLUE-001`) & Register GitHub Feature Issue (Addressing `#485`–`#487`)**:
   - Author the complete `DEAP-BLUEPRINT-APP-GLUE-001` blueprint detailing:
     - **Option 1C Universal 5-Table SQLite Seed & Typed `InstanceRecord` Views (`#485`)**: `compile-sysml` deterministic generation of `app_flutter/lib/data/seeds/generated_domain_seed.g.dart` (seeding `type_definitions`, `type_attributes`, `type_relations`, `properties`, and `instances` from `.pipeline/schema.sysml` + `topology_data.json` with zero legacy telecom rows) and `app_flutter/lib/models/generated_schema_models.g.dart`.
     - **Multi-Resolution (`1920x1080` through `6K` `6016x3384`) UI & Spatial Contract Test Generator (`#486`)**: `compile-sysml` deterministic generation of `app_flutter/test/generated/schema_ui_contract_test.g.dart` sweeping all 6 canonical viewports (`FHD_1080P_1X`, `QHD_1440P_1X`, `UHD_4K_1X`, `UHD_4K_2X`, `XDR_6K_1X`, `XDR_6K_2X`) and enforcing the 5 mathematical invariants (`RenderFlex` overflow trap, viewport containment, $O(K^2)$ pairwise 2D `Rect.overlaps == false`, proportional split ratios / $\ge 44 \times 44\text{ dp}$ hit targets, and exhaustive topology node `tester.tap` SQLite transitions with `tearDownAll` disposal).
     - **`verify-baseline` Native Rust Adversarial & Anti-Fabrication AST Gates (`Checks 38..=41` in `crates/verify-baseline/src/checks/codebase.rs`)**: Moving the 4 adversarial code audit checks into native Rust so `verify-baseline` executes them automatically in `< 50 ms`.
   - Register the tracked GitHub Feature Issue for `DEAP-BLUEPRINT-APP-GLUE-001` via `./target/release/reconcile-backlog create-issue`.
4. **Work Package 3 -- Implement Native Rust Codegen (`crates/compile-sysml`) & Native Rust Adversarial AST Gates Checks 38–41 (`crates/verify-baseline`)**:
   - **Subagent 3A (`Rust Verification Engineer -- Checks 38-41 in crates/verify-baseline`)**: Implements `crates/verify-baseline/src/checks/codebase.rs` and wires **Check 38** (`#487` Presentation Anti-Hardcoding AST Gate), **Check 39** (`#486` Zero-Mocking Test AST & SQLite Disposal Gate), **Check 40** (`#486` Interactive Gesture, `1080p`–`6K` Viewport & Pairwise 2D HUD Non-Overlap Gate), and **Check 41** (`#483`/`#484`/`#485` Track B C-ABI Solver Bridge SSOT & 5-Table SQLite Seed Parity Gate) into `crates/verify-baseline/src/runner.rs` with full TDD RED-GREEN unit tests.
   - **Subagent 3B (`Rust Compiler Engineer -- Flutter & C-ABI Codegen in crates/compile-sysml`)**: Implements `crates/compile-sysml/src/flutter_codegen.rs` (wired into `crates/compile-sysml/src/lib.rs` and `main.rs` via `--emit-flutter-contracts`) to deterministically emit `generated_domain_seed.g.dart`, `generated_schema_models.g.dart`, `solver_bridge.g.dart` (plus C-ABI `ffi.rs`), and the 6-viewport (`1920x1080` through `6016x3384`) `schema_ui_contract_test.g.dart` with full TDD RED-GREEN unit tests.
5. **Work Package 4 -- Synchronize Canonical Blueprint Cross-References, Master Skills, Verify Baseline & Push to `origin/main`**:
   - Synchronize canonical blueprint cross-references in `DEAP_POLY_REPO_BINARY_DISTRIBUTION_ARCHITECTURE.md`, `RUNTIME_METADATA_ENGINE.md`, `PERSISTENCE_ARCHITECTURE.md`, and `DEAP_CUAS_COA_FLUTTER_BLUEPRINT.md`, plus `.agents/skills/adversarial-code-auditor/SKILL.md` and `.agents/skills/feature-driven-implementation/SKILL.md`.
   - Build release binaries (`cargo build --release`), run `./target/release/verify-baseline . --no-domain` and `cargo test --workspace`, commit atomically, push to `origin/main`, and verify `git diff origin/main` is empty.

---

## 2. Detailed Work Package Specifications (100% Subagent Delegated)

### Work Package 0: Upgrade Adversarial Auditor in Rust (`defect_dossier.rs`), File Auditor Defect & Sanitize `#484`–`#487`
- **Assigned Subagent Role**: `Rust Adversarial Auditor Engineer -- Dossier AST Gate & Issue Sanitization`
- **Active Skill**: `/Users/perkunas/jail/DEAP01-spec-core/.agents/skills/adversarial-code-auditor/SKILL.md` and `/Users/perkunas/jail/DEAP01-spec-core/.agents/skills/debug-protocol/SKILL.md`
- **Target Files**:
  - `.pipeline/defects/DEFECT_AUDITOR_SECTION6_ZERO_REGEX_AND_UPSTREAM_TARGET.md`
  - `crates/verify-baseline/src/defect_dossier.rs`
  - `crates/verify-baseline/tests/defect_dossier_tests.rs`
  - `.agents/skills/adversarial-code-auditor/SKILL.md`
- **Detailed Specifications**:
  1. Author `.pipeline/defects/DEFECT_AUDITOR_SECTION6_ZERO_REGEX_AND_UPSTREAM_TARGET.md` (7-section dossier with `test-target: crates/verify-baseline/tests/defect_dossier_tests.rs`) and publish it via `./target/release/verify-baseline check-defect ... --publish` inside the standard sandbox.
  2. Write failing RED tests in `crates/verify-baseline/tests/defect_dossier_tests.rs` and implement in `crates/verify-baseline/src/defect_dossier.rs`:
     - **Section 6 Rust Zero-Regex Gate**: Scan every ` ```rust ` code block in `## 6. Proposed Correction` and reject `regex::`, `Regex::new`, `RegexSet`, or `use regex` (Check 34 enforcement at dossier validation time).
     - **Upstream `test-target` Crate Grounding Gate**: When validating against `UPSTREAM_SPEC_CORE_COMPILER` (`DEAP01-spec-core`), verify that `test-target` starts with one of the 6 valid upstream crate paths (`crates/deap-core/`, `crates/compile-sysml/`, `crates/ingest-sysml/`, `crates/verify-baseline/`, `crates/assemble-conops/`, `crates/reconcile-backlog/`), rejecting downstream crate paths such as `crates/cuas-coa-engine/src/lib.rs`.
  3. Update `.agents/skills/adversarial-code-auditor/SKILL.md` to document the new Rust gates and remove `BypassSandbox: true` so subagents never trigger IDE permission popups.
  4. Update GitHub Issues `#484`, `#485`, `#486`, and `#487` via `./target/release/verify-baseline check-defect ... --issue <ID> --publish` so their `test-target` paths reference `crates/verify-baseline/src/runner.rs`, their check numbers are `Check 38` (`#487`), `Check 39` & `Check 40` (`#486`), and `Check 41` (`#484` & `#485`), and their Section 6 Rust snippets are 100% zero-regex and domain-agnostic.

---

### Work Package 1 (First Priority): Author Zero-Trust Deterministic Rust C-ABI Blueprint (`DEAP-BLUEPRINT-CABI-001`) & Sync Issue `#483`
- **Assigned Subagent Role**: `Systems Safety Architect -- Zero-Trust Rust C-ABI Blueprint`
- **Active Skill**: `/Users/perkunas/jail/DEAP01-spec-core/.agents/skills/schema-specification-engineering/SKILL.md`
- **Target File**:
  - `docs/designs/zero-trust-deterministic-rust-c-abi-blueprint.md`
- **Detailed Specifications**:
  1. Author `docs/designs/zero-trust-deterministic-rust-c-abi-blueprint.md` (`DEAP-BLUEPRINT-CABI-001`, linked to Feature `#483` and Defect `#484`):
     - **Section 1 (Executive Summary & 3-Tier Poly-Repo Allocation Matrix)**: Exact allocation across Tier 0 (`DEAP01-spec-core`), Tier 1 (`DEAP-profile-flutter-app`), and Tier 2 (`DEAP01-<domain>-engine`), plus coexistence of Tier 1 C++ Rendering FFI (`cesium_native_bridge/src/bridge.cpp`) and Tier 2 Rust Solver C-ABI FFI (`crates/<domain-engine>/src/ffi.rs`).
     - **Section 2 (Hazard Analysis & Threat Model)**: **H-CABI-01** (Cross-FFI Rust Panic Unwinding UB), **H-CABI-02** (Cross-Isolate Pointer Aliasing, UAF & Double-Free), **H-CABI-03** (Schema/ABI Version Skew), and **H-CABI-04** (5,971-Line Hand-Coded Dart Formula Drift & UI Thread Frame Drops from `#484`).
     - **Section 3 (The 6-Layer Zero-Trust Deterministic Rust C-ABI Architecture)**:
       - Layer 1: `#![forbid(unsafe_code)]` on `crates/<domain-engine>/src/solver.rs` and all upstream crates; `unsafe` isolated strictly to the ~80-line deterministic `ffi.rs` shim.
       - Layer 2: `std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| ...))` panic barrier on every `extern "C"` function (`DeapFfiStatus::InternalPanic = 5`).
       - Layer 3: Compile-Time & Runtime `DEAP_SCHEMA_HASH: u64` (SHA-256 of `.pipeline/schema.sysml` + `topology_data.json`) and `DEAP_ABI_VERSION = 1` handshake (`DeapFfiStatus::SchemaVersionMismatch = 2`).
       - Layer 4: Copy-In / Copy-Out flat checksummed `#[repr(C)] DeapByteBuffer { ptr, len, cap, crc32, status }` with IEEE 802.3 CRC32 verification, `MAX_FFI_PAYLOAD_BYTES = 16_777_216` (16 MiB), immediate copy into Dart `Uint8List` inside `try / finally { deap_free_buffer(buf); }`, and Tier 1 `NativeFinalizer` safety net.
       - Layer 5: Dedicated Background Worker Isolate (`DeapNativeSolverIsolate`) using `TransferableTypedData` and deterministic watchdog timer.
       - Layer 6: `compile-sysml --emit-dart-bridge` generating `app_flutter/lib/data/generated/solver_bridge.g.dart` from SysML `port def` / `item def` / `calc def` AST nodes (`CoaBundlePort`, `EngagementPairingPort`, `InterlockAnnunciatorPort`, `FaultInjectionPort`), paired with `verify-baseline` Check 41 (`check_41_solver_bridge_and_sqlite_seed_parity_gate`).
     - **Section 4–7**: Formal C99 header (`include/deap_solver_abi.h`), Rust `ffi.rs` & `solver.rs` reference implementations, Dart `dart:ffi` bindings, Mermaid `sequenceDiagram`, `stateDiagram-v2`, and `classDiagram`, 4-Dimension Failure Mode Matrix, Given-When-Then Acceptance Criteria (**REQ-CABI-01**..**05**), and TDD Fault-Injection Test Matrix (**TEST-CABI-01**..**06**).
  2. Synchronize `docs/designs/zero-trust-deterministic-rust-c-abi-blueprint.md` with GitHub Feature Issue `#483` via `gh issue edit 483 --body-file docs/designs/zero-trust-deterministic-rust-c-abi-blueprint.md` and verify via `gh issue view 483`.

---

### Work Package 2 (Second Priority): Author Complete Option 1C Solution Architecture Blueprint (`DEAP-BLUEPRINT-APP-GLUE-001`) & Register GitHub Feature Issue
- **Assigned Subagent Role**: `Compiler & UI Verification Architect -- Option 1C Solution Blueprint`
- **Active Skill**: `/Users/perkunas/jail/DEAP01-spec-core/.agents/skills/schema-specification-engineering/SKILL.md`
- **Target File**:
  - `docs/designs/option-1c-rust-app-glue-ui-contracts-and-anti-fabrication-blueprint.md`
- **Detailed Specifications**:
  1. Author `docs/designs/option-1c-rust-app-glue-ui-contracts-and-anti-fabrication-blueprint.md` (`DEAP-BLUEPRINT-APP-GLUE-001`, linked to `#483` and addressing `#484`, `#485`, `#486`, `#487`):
     - **Section 1 (Executive Summary & Option 1C Architectural Resolution)**: Full comparison of Option 1A vs. Option 1B vs. Option 1C and the 3-Tier Poly-Repo Allocation Matrix.
     - **Section 2 (Part I -- Option 1C Universal 5-Table SQLite Seed & Typed `InstanceRecord` Views in `crates/compile-sysml`, addressing `#485`)**:
       - `app_flutter/lib/data/seeds/generated_domain_seed.g.dart` + `assets/properties_db.db.gz`: Deterministic synthesis from `.pipeline/schema.sysml` + `topology_data.json` populating all 5 canonical Tier 1 SQLite tables (`type_definitions`, `type_attributes`, `type_relations`, `properties`, `instances`) in `< 2 ms` for `:memory:` tests and production boot, eliminating missing nodes (`ASSET-RADAR-MAST-04`, `ASSET-COMM-GATEWAY-05`), parameter drift, and legacy telecom bloat.
       - `app_flutter/lib/models/generated_schema_models.g.dart`: Immutable typed Dart `extension type` / view projections over `InstanceRecord` + `TypeDescriptor`, SysML `inv` `[min..max]` validators, `SchemaEntityIds`, and `SchemaWidgetKeys`.
     - **Section 3 (Part II -- Multi-Resolution `1920x1080` through `6K` `6016x3384` Immutable UI & Spatial Contract Test Suite in `crates/compile-sysml`, addressing `#486`)**:
       - `app_flutter/test/generated/schema_ui_contract_test.g.dart`: Loops across all 6 canonical viewports:
         1. `FHD_1080P_1X` (`1920 x 1080 @ dpr 1.0`, logical `1920 x 1080`)
         2. `QHD_1440P_1X` (`2560 x 1440 @ dpr 1.0`, logical `2560 x 1440`)
         3. `UHD_4K_1X` (`3840 x 2160 @ dpr 1.0`, logical `3840 x 2160`)
         4. `UHD_4K_2X` (`3840 x 2160 @ dpr 2.0`, logical `1920 x 1080`)
         5. `XDR_6K_1X` (`6016 x 3384 @ dpr 1.0`, logical `6016 x 3384`)
         6. `XDR_6K_2X` (`6016 x 3384 @ dpr 2.0`, logical `3008 x 1692`)
       - Enforces the 5 mathematical contract invariants: (1) zero `RenderFlex`/`FlutterError` overflow across all 6 viewports and resize sweeps, (2) full viewport containment, (3) $O(K^2)$ pairwise 2D `rectA.overlaps(rectB) == false` across all HUD overlays and split panes, (4) proportional split ratios (`defaultRatio` +/- 2% unless clamped by `minPaneSize`) and MIL-STD-1472H $\ge 44 \times 44\text{ dp}$ tactile hit targets, and (5) exhaustive topology entity `tester.tap(...)` SQLite-to-Inspector transitions with deterministic `tearDownAll` SQLite handle disposal.
     - **Section 4 (Part III -- `verify-baseline` Native Rust Adversarial & Anti-Fabrication AST Gates Checks 38–41 in `crates/verify-baseline/src/checks/codebase.rs`)**:
       - **Check 38 (`#487`)**: `check_38_presentation_anti_hardcoding_ast_gate` (zero-regex, dynamic schema/topology-driven).
       - **Check 39 (`#486`)**: `check_39_zero_mocking_test_ast_gate` (zero-regex test AST scanner for mock/fake classes, empty stubs, and missing `tearDownAll` disposal).
       - **Check 40 (`#486`)**: `check_40_interactive_gesture_and_hud_non_overlap_gate` (enforcing `schema_ui_contract_test.g.dart` 6-viewport parity, banning shallow `probeWidgetKey` and direct `bridge.*` mutations inside `testWidgets`, and enforcing `assertPairwiseHudNonOverlap`).
       - **Check 41 (`#483`, `#484`, `#485`)**: `check_41_solver_bridge_and_sqlite_seed_parity_gate` (enforcing thin C-ABI solver bridge delegation and 100% bijective topology-to-SQLite 5-table seed parity).
     - **Section 5–6**: Mermaid `flowchart TD`, `sequenceDiagram`, and `classDiagram`, 4-Dimension Failure Mode Matrix, Given-When-Then Acceptance Criteria, and TDD Test Matrix.
  2. Register the GitHub Feature Issue via `./target/release/reconcile-backlog create-issue --body-file docs/designs/option-1c-rust-app-glue-ui-contracts-and-anti-fabrication-blueprint.md --label feature --title "[FEATURE] Option 1C Rust-Compiled SQLite Seed, Typed InstanceRecord Views, 1080p-6K Contract Tests & Anti-Fabrication Gates"` and verify via `gh issue view`.

---

### Work Package 3: Implement Native Rust Codegen (`crates/compile-sysml`) & Native Rust Adversarial AST Gates Checks 38–41 (`crates/verify-baseline`)
- **Subagent 3A (`Rust Verification Engineer -- Checks 38-41 in crates/verify-baseline`)**:
  - **Active Skill**: `/Users/perkunas/jail/DEAP01-spec-core/.agents/skills/feature-driven-implementation/SKILL.md`
  - **Target Files**:
    - `crates/verify-baseline/src/checks/codebase.rs`
    - `crates/verify-baseline/src/checks/mod.rs`
    - `crates/verify-baseline/src/runner.rs`
  - **Deliverables**:
    1. Write failing RED unit tests in `crates/verify-baseline/src/checks/codebase.rs` (using `tempfile::TempDir` fixtures) for Checks 38, 39, 40, and 41 covering every failure mode in `#484`–`#487`.
    2. Implement `check_38_presentation_anti_hardcoding_ast_gate`, `check_39_zero_mocking_test_ast_gate`, `check_40_interactive_gesture_and_hud_non_overlap_gate`, and `check_41_solver_bridge_and_sqlite_seed_parity_gate` in `crates/verify-baseline/src/checks/codebase.rs` using 100% zero-regex, domain-agnostic Rust with 7-section safety docstrings, re-export in `crates/verify-baseline/src/checks/mod.rs`, and wire Checks 38–41 into `crates/verify-baseline/src/runner.rs`.
    3. Verify all positive and negative unit tests pass (`cargo test -p verify-baseline`).
- **Subagent 3B (`Rust Compiler Engineer -- Flutter & C-ABI Codegen in crates/compile-sysml`)**:
  - **Active Skill**: `/Users/perkunas/jail/DEAP01-spec-core/.agents/skills/feature-driven-implementation/SKILL.md`
  - **Target Files**:
    - `crates/compile-sysml/src/flutter_codegen.rs`
    - `crates/compile-sysml/src/lib.rs`
    - `crates/compile-sysml/src/main.rs`
  - **Deliverables**:
    1. Write failing RED unit tests in `crates/compile-sysml/src/flutter_codegen.rs` testing deterministic generation of `generated_domain_seed.g.dart` (5 canonical SQLite tables), `generated_schema_models.g.dart` (typed `InstanceRecord` views + `DEAP_SCHEMA_HASH`), `solver_bridge.g.dart` (C-ABI + SysML port/item DTOs), and `schema_ui_contract_test.g.dart` (6-viewport `1920x1080` through `6016x3384` contract suite).
    2. Implement `crates/compile-sysml/src/flutter_codegen.rs` with 100% 7-section safety docstrings and zero regex, expose in `crates/compile-sysml/src/lib.rs`, and wire `--emit-flutter-contracts <DIR>` into `crates/compile-sysml/src/main.rs`.
    3. Verify `cargo test -p compile-sysml` passes with zero failures.

---

### Work Package 4: Synchronize Canonical Blueprints, Rebuild Release Binaries, Verify Baseline & Push to `origin/main`
- **Assigned Subagent Role**: `Governance & Verification Engineer -- Blueprint Grounding & Remote Sync`
- **Active Skill**: `/Users/perkunas/jail/DEAP01-spec-core/.agents/skills/project-constitution/SKILL.md`
- **Target Files**:
  - `docs/architecture/blueprints/DEAP_POLY_REPO_BINARY_DISTRIBUTION_ARCHITECTURE.md`
  - `docs/architecture/blueprints/RUNTIME_METADATA_ENGINE.md`
  - `docs/architecture/blueprints/PERSISTENCE_ARCHITECTURE.md`
  - `docs/architecture/blueprints/DEAP_CUAS_COA_FLUTTER_BLUEPRINT.md`
  - `.agents/skills/feature-driven-implementation/SKILL.md`
- **Deliverables**:
  1. Synchronize canonical blueprint cross-references to `docs/designs/zero-trust-deterministic-rust-c-abi-blueprint.md` (`#483`), `docs/designs/option-1c-rust-app-glue-ui-contracts-and-anti-fabrication-blueprint.md`, `crates/compile-sysml/src/flutter_codegen.rs`, and `crates/verify-baseline/src/checks/codebase.rs` (strictly satisfying Check 37 `check_blueprint_grounding` now that those `.rs` files exist on disk!).
  2. Build release binaries (`cargo build --release`), run `./target/release/verify-baseline . --no-domain` and `cargo test --workspace` (verifying 100% pass across Checks 10 through 41), commit atomically, push to `origin/main`, and verify `git diff origin/main` is empty.

---

## 3. Verification & Acceptance Criteria

- [ ] **WP0**: `defect_dossier.rs` enforces Section 6 Rust zero-regex and upstream `test-target` crate grounding; new defect dossier filed; `#484`–`#487` sanitized on GitHub; `BypassSandbox: true` removed from `adversarial-code-auditor/SKILL.md`.
- [ ] **WP1**: `docs/designs/zero-trust-deterministic-rust-c-abi-blueprint.md` (`DEAP-BLUEPRINT-CABI-001`) authored and synced with GitHub Feature Issue `#483`.
- [ ] **WP2**: `docs/designs/option-1c-rust-app-glue-ui-contracts-and-anti-fabrication-blueprint.md` (`DEAP-BLUEPRINT-APP-GLUE-001`) authored and registered as a GitHub Feature Issue via `./target/release/reconcile-backlog create-issue`.
- [ ] **WP3**: Native Rust `crates/verify-baseline/src/checks/codebase.rs` (Checks 38–41) and `crates/compile-sysml/src/flutter_codegen.rs` (`--emit-flutter-contracts`) implemented with passing RED-GREEN unit tests.
- [ ] **WP4**: Canonical blueprints updated, `./target/release/verify-baseline . --no-domain` and `cargo test --workspace` pass with 0 errors, and `git diff origin/main` is empty.
