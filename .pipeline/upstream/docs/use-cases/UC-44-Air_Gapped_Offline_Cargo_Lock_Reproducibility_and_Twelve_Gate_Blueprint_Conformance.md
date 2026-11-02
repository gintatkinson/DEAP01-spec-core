| Attribute | Specification Detail |
| :--- | :--- |
| **Issue ID** | #772 |
| **Title** | Use Case 44: Air Gapped Offline Cargo Lock Reproducibility and Twelve Gate Blueprint Conformance |
| **Type** | use-case |
| **Primary Actor** | Operator |
| **Scope** | AirGappedCargoLockGovernor |
| **Level** | User Goal |
| **Generation Mode** | subagent |
| **Specification Source** | [schema/model.sysml](../../schema/model.sysml) |

# Use Case 44: Air Gapped Offline Cargo Lock Reproducibility and Twelve Gate Blueprint Conformance

## UML Diagrams

### Use Case Diagram
```mermaid
graph TD
    subgraph "AirGappedCargoLockGovernor System Boundary"
        UC_Main(["Air Gapped Offline Cargo Lock Reproducibility and Twelve Gate Blueprint Conformance"])
        UC_Step01(["Validate Air Gapped Offline Cargo Lock Checksums via cargo_lock and cargo_metadata"])
        UC_Step02(["Execute Twelve Gate Blueprint Conformance Verification and Bijective Roundtrip"])
    end
    Operator(("Operator")) --- UC_Main
    UC_Main -. "<<include>>" .-> UC_Step01
    UC_Main -. "<<include>>" .-> UC_Step02
```

### State Machine Diagram
```mermaid
stateDiagram-v2
    [*] --> Idle
    Idle --> ParsingLosslessCst : "parseRustCst"
    ParsingLosslessCst --> VerifyingSelfHostFixedPoint : "verifyOfflineLockedAndTwelveGates"
    VerifyingSelfHostFixedPoint --> EmittingJsonDiagnostics : "rejectLockDriftOrBlueprintGateFailure"
    VerifyingSelfHostFixedPoint --> VerifiedClean : "offlineLockedAndTwelveGatesVerified"
    EmittingJsonDiagnostics --> [*]
    VerifiedClean --> [*]
```

## 1. Actors
- `Operator`

## 2. Preconditions
- Workspace Cargo.toml and Cargo.lock populated via cargo fetch alongside docs/designs/blueprint-08-rust-analyzer-cst-refactoring-and-json-diagnostics.md.

## 3. Trigger
- cargo check --workspace --offline --locked and ./target/release/verify-baseline check-blueprint docs/designs/blueprint-08-rust-analyzer-cst-refactoring-and-json-diagnostics.md are executed.

## 4. Main Success Scenario
1. `Operator` invokes `verify_offline_locked_build` (`cargo check --workspace --offline --locked`) on `AirGappedCargoLockGovernor` (`offline_locked_enforced = true`, `vendor_bundle_supported = true`) via `manifest_lock_in`.
2. `AirGappedCargoLockGovernor` validates workspace `Cargo.toml` and `Cargo.lock` dependency versions and SHA-256 package checksums without network access (`cargo vendor`-compatible) and emits `SupplyChainReceipt` on `lock_receipt_out`.
3. `Operator` executes `./target/release/verify-baseline check-blueprint docs/designs/blueprint-08-rust-analyzer-cst-refactoring-and-json-diagnostics.md` to validate all 12 mechanical blueprint verification gates.
4. `ingest-sysml` and `compile-sysml` extract and compile `BP_08_Lossless_Rust_CST_Codemod_Engine_and_Unified_rustc_Compatible_JSON_Diagnostics` (`REQ-0608`, `C_CST_01..07`), verifying bijective SysML v2 AST roundtrip parity.
5. Offline locked compilation and all 12 blueprint verification gates succeed with exit code 0, persisting `.sysml` and `.digest.json` artifacts and recording verified supply-chain and blueprint conformance for `Operator`.

## 5. Alternate and Exception Flows
- 3a. `Cargo.lock` is out of sync with `Cargo.toml` or requires network access under `--offline --locked` (`Diagnostic: DEAP-E0804`):
  - `AirGappedCargoLockGovernor` transitions to `RejectedFailClosed`, emits single-line JSON diagnostic `DEAP-E0804`, and exits fail-closed with code 1.
- 4a. Any of the 12 mechanical blueprint verification gates fails on `blueprint-08-rust-analyzer-cst-refactoring-and-json-diagnostics.md`:
  - `verify-baseline check-blueprint` aborts artifact persistence, reports the exact gate failure, and terminates fail-closed with exit code 1.
- 5a. Invariant violation while enforcing `C_CST_01` through `C_CST_07` in `AirGappedCargoLockGovernor`:
  - `AirGappedCargoLockGovernor` aborts execution without partial state mutation, preserves prior verified state, and notifies `Operator`.

## 6. Postconditions
- **Success Guarantee:** Offline locked dependency resolution succeeds without network access (`cargo vendor`-compatible via `cargo-lock`, `cargo_metadata`, `cargo-util-schemas`, `cargo-platform`, and `toml_edit`), and all 12 mechanical blueprint verification gates pass with exit code 0, persisting `.sysml` and `.digest.json` artifacts.
- **Failure Guarantee:** Any `Cargo.lock` drift (`DEAP-E0804`) or blueprint verification failure aborts artifact persistence, preserves prior verified state, and exits fail-closed with code 1.

## 7. Operational Context
- **Contract A (100% Normative OMG SysML v2.0 + KerML 1.0 Grammar Coverage)**: Verifies full Definition and Usage symmetry across the OMG SysML v2.0 and KerML 1.0 textual grammar with zero 16-construct caps during 12-gate blueprint ingestion and bijective roundtrip compilation.
- **Contract B (Official Rust Compiler, Cargo, `rust-analyzer` & Markdown AST Crate Stack)**: Uses `cargo-lock`, `cargo_metadata`, `cargo-util-schemas`, `cargo-platform`, `toml_edit`, `ra_ap_syntax`, `ra_ap_vfs`, `ra_ap_paths`, `ra_ap_line_index`, `ra_ap_text_edit`, `rowan`, `camino`, `ignore`, `pulldown-cmark`, and `pulldown-cmark-to-cmark` with explicit net-negative deletion of ~19,700+ lines of hand-rolled scanners (retiring `crates/deap-core/src/workspace.rs` (356 lines), `crates/deap-core/src/rust_cst.rs` (599 lines), `crates/compile-sysml/src/docstring.rs` (549 lines), `crates/verify-baseline/src/checks/docstring_tier_rules.rs` (729 lines), `crates/verify-baseline/src/checks/structural.rs` (980 lines), and `crates/verify-baseline/src/ratchet_table.rs`).
- **3-Layer Definition of Done (DoD)**:
  - **Layer 1 (Domain State & Data Model)**: `cargo_lock::Lockfile`, `cargo_metadata::Metadata`, `toml_edit::DocumentMut`, and `SupplyChainReceipt`.
  - **Layer 2 (Logic & State Management)**: Enforces `offline_locked_enforced = true`, `vendor_bundle_supported = true`, `C_CST_01..C_CST_06`, and `Invariant_REQ_0608_SC_01..04` inside `< 380`-SLOC submodules (`crates/deap-core/src/rust_cst/`, `crates/deap-core/src/json_diag/`, `crates/compile-sysml/src/codemod/`).
  - **Layer 3 (Presentation & Actuator Interface Binding)**: Binds `manifest_lock_in` and `lock_receipt_out` on `AirGappedCargoLockGovernor`.

## 8. Realization Matrix
| Specification Item | Type | Link / Reference |
| :--- | :--- | :--- |
| **#802** | User Story | [User Story 53: verify offline locked build](../user-stories/US-53-verify_offline_locked_build.md) |
| **#742** | Feature | [Feature 36: [BP 08 Lossless Rust CST Codemod Engine and Unified rustc Compatible JSON Diagnostics] AirGappedCargoLockGovernor](../features/FEAT-36-AirGappedCargoLockGovernor.md) |
| **#745** | Feature | [Feature 39: [BP 08 Lossless Rust CST Codemod Engine and Unified rustc Compatible JSON Diagnostics] UnifiedRustcJsonDiagnosticEmitter](../features/FEAT-39-UnifiedRustcJsonDiagnosticEmitter.md) |

### Required User Stories
- [ ] #802 - [User Story 53: verify offline locked build](../user-stories/US-53-verify_offline_locked_build.md) (validates `--offline --locked` workspace builds and SHA-256 package checksums via `cargo-lock` and `cargo_metadata`)

### Required Features
- [ ] #742 - [Feature 36: [BP 08 Lossless Rust CST Codemod Engine and Unified rustc Compatible JSON Diagnostics] AirGappedCargoLockGovernor](../features/FEAT-36-AirGappedCargoLockGovernor.md) (governs air-gapped Cargo lockfile verification and `SupplyChainReceipt` emission)
- [ ] #745 - [Feature 39: [BP 08 Lossless Rust CST Codemod Engine and Unified rustc Compatible JSON Diagnostics] UnifiedRustcJsonDiagnosticEmitter](../features/FEAT-39-UnifiedRustcJsonDiagnosticEmitter.md) (emits fail-closed `DEAP-E0804` single-line NDJSON diagnostics on lockfile drift)

## Source References
Use case operational flows and lifecycle activity references:
- System Architecture Model: `schema/model.sysml`
- Normative Systems Engineering Standard: ISO/IEC/IEEE 29148:2018 §6.4.2 Concept of Operations

Operational use case realization and traceability links derive from `schema/model.sysml` pursuant to ISO/IEC/IEEE 29148.
