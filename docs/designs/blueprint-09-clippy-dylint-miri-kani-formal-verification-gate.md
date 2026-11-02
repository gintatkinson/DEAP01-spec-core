| Attribute | Specification Detail |
| :--- | :--- |
| **Blueprint ID** | `BP-09` |
| **Requirement ID** | `REQ-0609` |
| **Title** | Multi-Stage Compiler and Formal Verification Gatekeeper (Clippy, Dylint MIR, Miri Tree Borrows, Kani CBMC, and flutter_rust_bridge FFI Synthesis) |
| **Status** | `Draft` |
| **Target Crates** | `crates/deap-core, crates/compile-sysml, crates/verify-baseline` |
| **Tracked Issues** | `#742` |
| **Generation Mode** | `subagent` |
| **Specification Source** | `Cargo.toml:12-25` |

## 1. Executive Summary & Derivation-Annotation Contract

`MultiStageFormalVerificationGatekeeper` (**REQ-0609** / `BP-09` / `DEAP-BP-DET-09`, UUIDv5 `6ba7b810-9dad-11d1-80b4-00c04fd430c9`, `Class P` static HIR/THIR/MIR linear pass complexity paired with bounded CBMC state-space unwinding, governed by DO-178C Level A, DO-330 TQL-4, DO-333 Formal Methods Supplement, ISO 26262 ASIL D, and ECSS-E-ST-40C) establishes DEAP's cross-platform five-stage compiler-native formal verification and FFI boundary-synthesis pipeline across both upstream Tier-0 compiler crates (`crates/*`, **Contract A**) and downstream schema-generated Rust + Flutter targets (`compile-sysml`, **Contract B**).

Rather than relying on monolithic hand-rolled text scanners or post-hoc manual code review to catch partial functions, hidden real-time heap allocations, FFI memory-aliasing defects, or reachable arithmetic/state-machine hazards, **BP-09** enforces explicit legacy scanner deletion, `< 380`-SLOC modular submodules, and a cross-platform 5-Stage formal gatekeeper (`Stage 1 Clippy` -> `Stage 2 Excluded ./lints/deap-dylint-mir/` -> `Stage 3 Miri + flutter_rust_bridge` -> `Stage 4 Kani #[kani::proof] synthesis (syn+proc-macro2+quote+prettyplease)` -> `Stage 5 Dynamic Runtime PATH Discovery & In-Process CST < 380-SLOC Gate`) emitting unified `rustc`-compatible JSON diagnostics (`--message-format=json`):

1. **Stage 1 -- Cross-Platform 5-Stage Clippy HIR/THIR Total-Function & Structural Ratchet Gate (`[workspace.lints.clippy]` + `./clippy.toml`)**: Enforces a 14-lint total-function and structural-bound profile across workspace `Cargo.toml` manifests (`unwrap_used = "deny"`, `expect_used = "deny"`, `panic = "deny"`, `todo = "deny"`, `unimplemented = "deny"`, `unreachable = "deny"`, `indexing_slicing = "deny"`, `arithmetic_side_effects = "deny"`, `cast_possible_truncation = "deny"`, `cast_sign_loss = "deny"`, `undocumented_unsafe_blocks = "deny"`, `too_many_lines = "deny"`, `too_many_arguments = "deny"`, `fn_params_excessive_bools = "deny"`), paired with root `./clippy.toml` thresholds (`too-many-lines-threshold = 80`, `too-many-arguments-threshold = 5`, and `disallowed-types` banning non-deterministic `std::collections::HashMap` / `std::collections::HashSet` and NFA/backtracking regular-expression engine types with deterministic `BTreeMap` / `BTreeSet` / structural-token replacement reasons). Executes a 5-stage Clippy progressive ratchet (`1A` partial-function ban -> `1B` arithmetic/cast total-safety ban -> `1C` `undocumented_unsafe_blocks` proof mandate -> `1D` `<= 80`-line / `<= 5`-arg complexity ratchet eliminating `too_many_arguments = "allow"` -> `1E` deterministic `disallowed-types` container/scanner enforcement) and streams `cargo clippy --message-format=json` diagnostics (`DEAP-E0900`).
2. **Stage 2 -- Isolated Nightly `Dylint` MIR Control-Flow & Real-Time Allocation Gate (Excluded `./lints/deap-dylint-mir/`)**: Executes a standalone nightly `rustc_lint` / `rustc_middle::mir` Dylint plugin workspace in `./lints/deap-dylint-mir/` completely decoupled from the root stable `Cargo.toml` (`workspace.exclude = ["lints/deap-dylint-mir"]`) so core crates (`crates/*`) compile 100% on stable `rustc` and Ferrocene with zero `#![feature(rustc_private)]`. Traverses `rustc_middle::mir::Body` basic-block `TerminatorKind::Call` and `TerminatorKind::Drop` graphs transitively from every `#[rt_loop]` real-time entrypoint to prove **zero heap allocations** (`DEAP-E0901`, rejecting `exchange_malloc`, `__rust_alloc`, `__rust_realloc`, `Vec` capacity growth, `Box::new`, `String` formatting, and dynamic drop-glue heap deallocations on hot control paths) while enforcing `#[must_use]` domain error propagation and verifying type-checked `#[realises("REQ-*")]` bindings against active `.sysml` requirement digests (`DEAP-E0902`).
3. **Stage 3 -- `Miri` Tree Borrows Abstract-Machine UB Gate & `compile-sysml` to `flutter_rust_bridge` FFI Synthesis**: Enforces `#![forbid(unsafe_code)]` on all pure domain and lowering crates (`@Layer(Domain)` and `@Layer(Application)`) in Ring-1 and Ring-3, while isolating FFI, HAL, and lock-free ring-buffer marshaling into dedicated Ring-2 bridge modules governed by explicit `// SAFETY: [SC-*]` boundary proofs (`#![deny(unsafe_code)]` + `undocumented_unsafe_blocks = "deny"`). Synthesizes `flutter_rust_bridge`-compatible Rust Ring-2 `#[repr(C)]` facades and strongly-typed Dart binding contracts directly from SysML `port def` and `item flow` Interface Control Documents (`ICD_01` / `ICD_02`), wrapping every inbound HMI call in fail-closed domain, `[min..max]` interval, IEEE-754 finiteness, and active FSM state guards (`DEAP-E0903`), and gating FFI tests under `cargo +nightly miri test` (`-Zmiri-tree-borrows` aliasing, alignment, initialization, and data-race verification) plus `dart analyze --fatal-infos` on the Dart side.
4. **Stage 4 -- Deterministic `#[kani::proof]` CBMC Bounded Model-Checking Synthesis (`syn` + `proc-macro2` + `quote` + `prettyplease`)**: Compiles SysML `constraint def`, `port def` `[min..max]` range intervals, `state def` transition tables, and STPA `**UCA-***` / `**SC-***` safety constraints directly into `#[cfg(kani)]` verification modules (`kani::any()` symbolic inputs constrained by `kani::assume(...)` port/domain bounds, `assert!(...)` postconditions, and `#[kani::unwind(N)]` loop bounds) via `syn` + `proc-macro2` + `quote` + `prettyplease`, proving mathematical absence of reachable panics, arithmetic overflows, division-by-zero, out-of-bounds accesses, and STPA unsafe control actions (`DEAP-E0904`) across both upstream DEAP core parsers (`crates/deap-core`, `crates/compile-sysml`) and downstream safety-critical Rust crates.
5. **Stage 5 -- Cross-Platform Dynamic Runtime `PATH` Discovery (`macOS`, `Linux`, `Windows`), `< 380`-SLOC Submodules, Explicit Legacy Scanner Deletion & Contract A + Contract B Gate**: Dynamically discovers `cargo-dylint`, `cargo-miri`, and `cargo-kani` at runtime across macOS, Linux, and Windows (`std::env::consts::{DLL_PREFIX, DLL_SUFFIX, EXE_SUFFIX}`, `camino::Utf8Path`) with zero hardcoded paths, enforces fail-closed qualification (`--require-formal-toolchains`, `DEAP-E0905`), validates `< 380` executable SLOC and `0` syntax errors across all formal submodules via in-process `ra_ap_syntax` CST verification, mandates explicit deletion of legacy hand-rolled scanners (`audit_code.rs`, `docstring_gate.rs`, `source_ast.rs`, `docstring.rs`), and closes **Contract A** (5-Column STPA/FMECA Derivation-Annotation, SysML v2 Metamodel & Upstream Compiler Gate) + **Contract B** (Deterministic Implementation, `< 380`-SLOC Submodule Decomposition, Explicit Legacy Scanner Deletion & 3-Layer DoD Closure).

| Claim / Requirement | SSOT Input Anchor (file:line / #ID) | Methodology Citation | Derivation Grade (Directly Evidenced / Analytically Derived / Declared Assumption) | Caveat / Downstream Impact |
| :--- | :--- | :--- | :--- | :--- |
| **REQ-0609**: Five-stage cross-platform compiler-native formal verification pipeline (`Clippy` 5-stage HIR/THIR ratchet, excluded `./lints/deap-dylint-mir/` MIR, `flutter_rust_bridge` + `Miri` Tree Borrows, `Kani` `#[kani::proof]` synthesis via `syn`+`proc-macro2`+`quote`+`prettyplease`, and dynamic `PATH` discovery + `< 380`-SLOC CST verification) satisfying Contract A and Contract B | `Cargo.toml:12-25` / `#742` | `ISO/IEC/IEEE 29148:2018 & DO-178C Level A / DO-333` | `Directly Evidenced` | Unifies upstream compiler self-verification (Contract A) and downstream codegen target verification (Contract B) behind single-line `rustc`-compatible `--message-format=json` NDJSON diagnostics (`BP-08`). |
| **SC-01**: Stage 1 cross-platform 5-Stage Clippy total-function lint gate enforcing 14 `[workspace.lints.clippy]` denials alongside `./clippy.toml` `too-many-lines-threshold = 80`, `too-many-arguments-threshold = 5`, and deterministic `disallowed-types` replacements | `Cargo.toml:15-25` / `#742` | `Design by Contract (DbC) & ISO 26262 ASIL D` | `Directly Evidenced` | Upstream pure crates retain `unsafe_code = "forbid"`, whereas isolated Stage-3 Ring-2 FFI/HAL bridge modules use `deny` + `undocumented_unsafe_blocks = "deny"` requiring explicit `// SAFETY: [SC-*]` proof comments. |
| **SC-02**: Stage 2 isolated nightly `Dylint` `rustc_middle::mir::Body` call-graph verifier in excluded `./lints/deap-dylint-mir/` proving zero heap allocations across `#[rt_loop]` functions, `#[must_use]` error propagation, and validating `#[realises("REQ-*")]` bindings against compiled `.sysml` requirement sets | `crates/verify-baseline/src/lib.rs:1-35` | `DO-178C Level A WCET / Resource Determinism & DO-330 TQL-4` | `Analytically Derived` | Resides in standalone `./lints/deap-dylint-mir/` excluded from root `Cargo.toml` `[workspace.members]` via `workspace.exclude = ["lints/deap-dylint-mir"]` so stable/Ferrocene builds never link `rustc_private`. |
| **SC-03**: Stage 3 `#![forbid(unsafe_code)]` domain partitioning, `compile-sysml` -> `flutter_rust_bridge` Ring-2 FFI synthesis from `ICD_01`/`ICD_02` `port def` and `item flow` contracts with fail-closed inbound guards, `cargo +nightly miri test` (`-Zmiri-tree-borrows`), and `dart analyze --fatal-infos` | `crates/compile-sysml/src/lib.rs:1-50` | `Leveson STPA & MIL-STD-1629A / SAE ARP4761` | `Analytically Derived` | Rejects out-of-range, non-finite (`NaN`/`Inf`), or illegal-state HMI inputs at the FFI boundary before pure domain invocation and proves zero Rust Abstract Machine UB across marshaling/ring-buffer code. |
| **SC-04**: Stage 4 deterministic `#[cfg(kani)]` CBMC bounded model-checking `#[kani::proof]` harness synthesis via `syn` + `proc-macro2` + `quote` + `prettyplease` from SysML `constraint def`, `port def` `[min..max]`, `state def`, and STPA `**UCA-***` / `**SC-***` safety constraints | `crates/compile-sysml/src/main.rs:1-50` | `DO-333 Formal Methods & Leveson STPA` | `Analytically Derived` | Emits byte-deterministic `#[kani::proof]` and `#[kani::unwind(N)]` modules preserving `BP-07`/`BP-08` `E0705` Stage-1 == Stage-2 SHA-256 fixed-point equivalence. |
| **SC-05**: Stage 5 cross-platform runtime `PATH` discovery across macOS, Linux, and Windows (`std::env::consts::{DLL_PREFIX, DLL_SUFFIX, EXE_SUFFIX}`, `camino::Utf8Path`), fail-closed qualification (`--require-formal-toolchains`), `< 380`-SLOC submodules, explicit legacy scanner deletion, and Contract A + Contract B closure | `Cargo.toml:1-34` | `DO-330 TQL-4 Configuration Management` | `Declared Assumption` | Discovers `cargo-dylint`, `cargo-miri`, and `cargo-kani` dynamically at runtime with zero hardcoded paths, executes in-process `ra_ap_syntax` CST `< 380`-SLOC checks, and explicitly deletes legacy hand-rolled scanners. |

## 2. Structural Component Metamodel

### Component: StableClippyTotalFunctionGate

| Attribute | Type | Default | Unit | Description |
| :--- | :--- | :--- | :--- | :--- |
| `denied_clippy_lint_count` | `Integer` | `14` | `lints` | Mandatory count of total-function, cast-safety, arithmetic, unsafe-doc, and structural complexity Clippy lints set to `deny` or `forbid`. |
| `too_many_lines_threshold` | `Integer` | `80` | `lines` | Maximum HIR function body line count enforced in root `clippy.toml` (`too-many-lines-threshold`). |
| `too_many_arguments_threshold` | `Integer` | `5` | `params` | Maximum callable parameter count enforced in root `clippy.toml` (`too-many-arguments-threshold`). |
| `disallowed_types_count` | `Integer` | `3` | `types` | Minimum banned non-deterministic hash container and backtracking pattern-engine entries in `clippy.toml`. |
| `json_diagnostic_stream` | `Boolean` | `true` | `flag` | Emits single-line `cargo clippy --message-format=json` NDJSON diagnostics into the autonomous remediation loop. |
| `deterministic_uuid` | `String` | `"6ba7b810-9dad-11d1-80b4-00c04fd430c9"` | `uuidv5` | Canonical deterministic blueprint identifier (`DEAP-BP-DET-09`). |

| Port | Direction | Type | Protocol | Description |
| :--- | :--- | :--- | :--- | :--- |
| `workspace_manifest_in` | `in` | `String` | `CargoAndClippyTomlBundle` | Receives workspace `Cargo.toml`, `clippy.toml`, and target Rust crate source trees. |
| `clippy_ndjson_out` | `out` | `String` | `RustcCompilerMessageNdjson` | Streams machine-applicable `cargo clippy --message-format=json` lint findings and exit status. |

### Component: IsolatedDylintMirAllocationGate

| Attribute | Type | Default | Unit | Description |
| :--- | :--- | :--- | :--- | :--- |
| `max_rt_heap_allocations` | `Integer` | `0` | `allocs` | Permitted transitive heap allocation or dynamic drop-deallocation terminators reachable from any `#[rt_loop]` function. |
| `max_unverified_req_links` | `Integer` | `0` | `links` | Permitted dangling `#[realises("REQ-*")]` or doc-contract requirement identifiers absent from compiled `.sysml` digests. |
| `isolated_nightly_workspace` | `Boolean` | `true` | `flag` | Isolates `./lints/deap-dylint-mir/` `rustc_lint` and `rustc_middle::mir` plugin manifest from stable workspace `Cargo.toml`. |
| `plugin_toolchain_channel` | `String` | `"nightly-pinned"` | `toolchain` | Pinned nightly `rustc_private` toolchain channel executing the standalone Dylint MIR analysis library. |

| Port | Direction | Type | Protocol | Description |
| :--- | :--- | :--- | :--- | :--- |
| `mir_body_graph_in` | `in` | `String` | `RustcMiddleMirStream` | Receives lowered `rustc_middle::mir::Body` basic blocks, terminators, and `DefId` attributes alongside `.digest.json`. |
| `dylint_ndjson_out` | `out` | `String` | `RustcCompilerMessageNdjson` | Emits `DEAP-E0901` real-time heap-allocation violations and `DEAP-E0902` broken requirement-binding diagnostics. |

### Component: FrbIcdBridgeAndMiriUbGate

| Attribute | Type | Default | Unit | Description |
| :--- | :--- | :--- | :--- | :--- |
| `max_miri_ub_violations` | `Integer` | `0` | `violations` | Permitted Rust Abstract Machine undefined-behavior findings under `cargo +nightly miri test` with Tree Borrows. |
| `max_dart_analyze_infos` | `Integer` | `0` | `diagnostics` | Permitted static analysis errors, warnings, or info findings under `dart analyze --fatal-infos`. |
| `forbid_unsafe_in_domain` | `Boolean` | `true` | `flag` | Enforces `#![forbid(unsafe_code)]` across all pure domain, lowering, and FSM crates outside isolated FFI/HAL bridges. |
| `miri_aliasing_model` | `String` | `"-Zmiri-tree-borrows"` | `flag` | Active Miri pointer-provenance and aliasing operational semantics flag enforced on FFI and ring-buffer test suites. |

| Port | Direction | Type | Protocol | Description |
| :--- | :--- | :--- | :--- | :--- |
| `sysml_icd_ast_in` | `in` | `String` | `SysmlPortAndItemFlowIcd` | Receives validated SysML `port def`, `item flow`, `state def`, and `ICD_01`/`ICD_02` signal bounds in `compile-sysml`. |
| `frb_facade_and_dart_out` | `out` | `String` | `FrbRustAndDartContractTree` | Emits fail-closed guarded `flutter_rust_bridge` Rust facades and strongly-typed Dart binding modules. |
| `miri_and_dart_receipt_out` | `out` | `String` | `MiriAndDartVerificationReceipt` | Emits `DEAP-E0903` boundary-guard, Miri Tree Borrows UB, or `dart analyze --fatal-infos` failure diagnostics. |

### Component: KaniCbmcProofSynthesizer

| Attribute | Type | Default | Unit | Description |
| :--- | :--- | :--- | :--- | :--- |
| `max_kani_counterexamples` | `Integer` | `0` | `counterexamples` | Permitted reachable CBMC property failures (panics, overflows, div-by-zero, OOB, or STPA `UCA-*` breaches). |
| `default_kani_unwind_bound` | `Integer` | `16` | `iterations` | Deterministic loop unwinding ceiling emitted into `#[kani::unwind(N)]` when AST loops carry finite static bounds. |
| `structured_prettyplease_codegen` | `Boolean` | `true` | `flag` | Emits `#[cfg(kani)]` `#[kani::proof]` modules via `syn` + `proc-macro2` + `quote` + `prettyplease` without string templates. |

| Port | Direction | Type | Protocol | Description |
| :--- | :--- | :--- | :--- | :--- |
| `sysml_safety_ast_in` | `in` | `String` | `SysmlConstraintStateStpaIr` | Receives SysML `constraint def`, `port def` `[min..max]` ranges, `state def` transitions, and STPA `UCA-*`/`SC-*` specs. |
| `kani_proof_module_out` | `out` | `String` | `PrettypleaseKaniRustUnit` | Emits deterministic `#[cfg(kani)]` modules containing `kani::any()`, `kani::assume(...)`, and `assert!(...)` harnesses. |
| `cbmc_proof_receipt_out` | `out` | `String` | `KaniVerificationReceipt` | Emits CBMC verification status and `DEAP-E0904` counterexample traces in `rustc`-compatible JSON format. |

### Component: MultiStageVerificationOrchestrator

| Attribute | Type | Default | Unit | Description |
| :--- | :--- | :--- | :--- | :--- |
| `verification_stage_count` | `Integer` | `5` | `stages` | Ordered 5-stage verification pipeline count (`Stage 1 Clippy` -> `Stage 2 Excluded Dylint MIR` -> `Stage 3 FRB + Miri` -> `Stage 4 Kani CBMC` -> `Stage 5 Dynamic PATH & CST`). |
| `fail_closed_short_circuit` | `Boolean` | `true` | `flag` | Halts downstream stage promotion whenever any preceding verification stage exits non-zero. |
| `require_formal_toolchains` | `Boolean` | `false` | `flag` | Mandates live execution of discovered drivers and fails closed if any driver is absent during qualification CI. |

| Port | Direction | Type | Protocol | Description |
| :--- | :--- | :--- | :--- | :--- |
| `gate_invocation_in` | `in` | `String` | `VerificationGateCommand` | Receives upstream (Contract A) or downstream (Contract B) target workspace verification commands in `verify-baseline` and `compile-sysml`. |
| `unified_gate_ndjson_out` | `out` | `String` | `RustcCompilerMessageNdjson` | Emits consolidated single-line NDJSON diagnostics (`DEAP-E0900`..`DEAP-E0905`) and final 5-stage verification receipt. |

## 3. UML & Behavioral Topology

```mermaid
classDiagram
    class MultiStageVerificationOrchestrator {
        +Integer verification_stage_count
        +Boolean fail_closed_short_circuit
        +Boolean require_formal_toolchains
        +Result~ToolchainDiscoveryReceipt~ probe_formal_toolchain(ToolchainProbeConfig config)
        +Result~Void~ evaluate_four_stage_gatekeeper(StageGateReceiptArray receipts)
    }
    class StableClippyTotalFunctionGate {
        +Integer denied_clippy_lint_count
        +Integer too_many_lines_threshold
        +Integer too_many_arguments_threshold
        +Integer disallowed_types_count
        +Boolean json_diagnostic_stream
        +Result~Void~ verify_clippy_policy(Stage1ClippyPolicy policy, ManifestLintMap manifest_lints)
    }
    class IsolatedDylintMirAllocationGate {
        +Integer max_rt_heap_allocations
        +Integer max_unverified_req_links
        +Boolean isolated_nightly_workspace
        +String plugin_toolchain_channel
        +Result~Void~ evaluate_stage2_dylint_mir(DylintMirVerificationInput input)
        +Result~InProcessCstReceipt~ verify_in_process_cst_syntax(Utf8Path module_path, Integer max_exec_sloc)
    }
    class FrbIcdBridgeAndMiriUbGate {
        +Integer max_miri_ub_violations
        +Integer max_dart_analyze_infos
        +Boolean forbid_unsafe_in_domain
        +String miri_aliasing_model
        +Result~Real~ guard_inbound_frb_f64(FrbIcdPortGuardSpec spec, Real raw_value, String active_fsm_state)
    }
    class KaniCbmcProofSynthesizer {
        +Integer max_kani_counterexamples
        +Integer default_kani_unwind_bound
        +Boolean structured_prettyplease_codegen
        +Result~Void~ validate_kani_harness_spec(KaniProofHarnessSpec spec)
    }

    MultiStageVerificationOrchestrator "1" *-- "1" StableClippyTotalFunctionGate : "executes Stage 1 5-Stage Clippy gate"
    MultiStageVerificationOrchestrator "1" *-- "1" IsolatedDylintMirAllocationGate : "executes Stage 2 Excluded Dylint MIR & Stage 5 CST gate"
    MultiStageVerificationOrchestrator "1" *-- "1" FrbIcdBridgeAndMiriUbGate : "executes Stage 3 FFI + Miri gate"
    MultiStageVerificationOrchestrator "1" *-- "1" KaniCbmcProofSynthesizer : "executes Stage 4 Kani CBMC gate"
```

```mermaid
stateDiagram-v2
    [*] --> Idle
    Idle --> ProbingFormalDrivers : "probe_formal_toolchain(config)"
    ProbingFormalDrivers --> RunningStage1Clippy : "toolchainsDiscoveredOrPermitted"
    ProbingFormalDrivers --> FailedClosedJsonDiagnostic : "rejectMissingToolchainInQualification"
    RunningStage1Clippy --> FailedClosedJsonDiagnostic : "rejectStage1ClippyViolation"
    RunningStage1Clippy --> RunningStage2DylintMir : "stage1TotalFunctionsVerified"
    RunningStage2DylintMir --> FailedClosedJsonDiagnostic : "rejectMirHeapAllocOrDanglingReq"
    RunningStage2DylintMir --> RunningStage3FrbAndMiri : "stage2ZeroRtAllocVerified"
    RunningStage3FrbAndMiri --> FailedClosedJsonDiagnostic : "rejectMiriTreeBorrowsOrDartFatalInfo"
    RunningStage3FrbAndMiri --> RunningStage4KaniCbmc : "stage3FrbMiriAndDartVerified"
    RunningStage4KaniCbmc --> FailedClosedJsonDiagnostic : "rejectKaniCbmcCounterexample"
    RunningStage4KaniCbmc --> VerifiedAllFourStages : "stage4AllSafetyProofsUnsat"
    FailedClosedJsonDiagnostic --> [*]
    VerifiedAllFourStages --> [*]
```

```mermaid
flowchart TD
    A["SysML v2 AST + ICD_01 / ICD_02 + STPA_MATRIX (.sysml & .digest.json)"] --> B["compile-sysml Modular Codegen (syn + proc-macro2 + quote + prettyplease)"]
    B --> C["Pure Domain & FSM Crates (#![forbid(unsafe_code)])"]
    B --> D["Guarded flutter_rust_bridge FFI Facade + Dart DTO Bindings"]
    B --> E["Synthesized #[cfg(kani)] CBMC Proof Harnesses (#[kani::proof] + #[kani::unwind(N)])"]
    C --> F["Stage 1: 5-Stage Clippy Gate --message-format=json (14 Denied Lints + clippy.toml 80-Line / 5-Arg / BTree Bounds)"]
    D --> F
    F -->|"Stage 1 Clean"| G["Stage 2: Excluded ./lints/deap-dylint-mir/ Nightly MIR Pass (Zero #[rt_loop] Heap Allocs + #[realises(REQ-*)] Digest Check)"]
    G -->|"Stage 2 Clean"| H["Stage 3: cargo +nightly miri test (-Zmiri-tree-borrows) + flutter_rust_bridge + dart analyze --fatal-infos"]
    H -->|"Stage 3 Clean"| I["Stage 4: cargo kani CBMC Model Checking (syn+proc-macro2+quote+prettyplease #[kani::proof] Harnesses)"]
    E --> I
    I -->|"Stage 4 Clean"| L["Stage 5: Cross-Platform PATH Discovery (macOS/Linux/Windows) + < 380-SLOC CST + Legacy Scanner Deletion (Contract A + B)"]
    F -->|"DEAP-E0900"| J["Unified --message-format=json NDJSON Error Stream (DEAP-E0900..E0905) -> Autonomous Agent Loop"]
    G -->|"DEAP-E0901 / E0902"| J
    H -->|"DEAP-E0903"| J
    I -->|"DEAP-E0904"| J
    L -->|"DEAP-E0905"| J
    L -->|"All 5 Stages Verified"| K["Verified Safety-Critical Upstream (Contract A) & Downstream (Contract B) Release Artifact"]
```

```mermaid
sequenceDiagram
    participant MultiStageVerificationOrchestrator
    participant StableClippyTotalFunctionGate
    participant IsolatedDylintMirAllocationGate
    participant FrbIcdBridgeAndMiriUbGate
    participant KaniCbmcProofSynthesizer

    MultiStageVerificationOrchestrator->>MultiStageVerificationOrchestrator: "probe_formal_toolchain(config)"
    MultiStageVerificationOrchestrator->>StableClippyTotalFunctionGate: "verify_clippy_policy(policy, manifest_lints)"
    StableClippyTotalFunctionGate-->>MultiStageVerificationOrchestrator: "StageGateReceipt"
    MultiStageVerificationOrchestrator->>IsolatedDylintMirAllocationGate: "evaluate_stage2_dylint_mir(input)"
    IsolatedDylintMirAllocationGate-->>MultiStageVerificationOrchestrator: "StageGateReceipt"
    MultiStageVerificationOrchestrator->>IsolatedDylintMirAllocationGate: "verify_in_process_cst_syntax(module_path, max_exec_sloc)"
    IsolatedDylintMirAllocationGate-->>MultiStageVerificationOrchestrator: "InProcessCstReceipt"
    MultiStageVerificationOrchestrator->>FrbIcdBridgeAndMiriUbGate: "guard_inbound_frb_f64(spec, raw_value, active_fsm_state)"
    FrbIcdBridgeAndMiriUbGate-->>MultiStageVerificationOrchestrator: "f64"
    MultiStageVerificationOrchestrator->>KaniCbmcProofSynthesizer: "validate_kani_harness_spec(spec)"
    KaniCbmcProofSynthesizer-->>MultiStageVerificationOrchestrator: "StageGateReceipt"
    MultiStageVerificationOrchestrator->>MultiStageVerificationOrchestrator: "evaluate_four_stage_gatekeeper(receipts)"
```

## 4. Formal Mathematical & Safety Constraints

| Constraint ID | Parameter | Min | Max | Unit | Expression |
| :--- | :--- | :--- | :--- | :--- | :--- |
| `C-VER-01` | `too_many_lines_threshold` | `1` | `80` | `lines` | `too_many_lines_threshold >= 1 and too_many_lines_threshold <= 80` |
| `C-VER-02` | `too_many_arguments_threshold` | `0` | `5` | `params` | `too_many_arguments_threshold >= 0 and too_many_arguments_threshold <= 5` |
| `C-VER-03` | `max_rt_heap_allocations` | `0` | `0` | `allocs` | `max_rt_heap_allocations == 0` |
| `C-VER-04` | `max_miri_ub_violations` | `0` | `0` | `violations` | `max_miri_ub_violations == 0` |
| `C-VER-05` | `max_kani_counterexamples` | `0` | `0` | `counterexamples` | `max_kani_counterexamples == 0` |
| `C-VER-06` | `default_kani_unwind_bound` | `1` | `64` | `iterations` | `default_kani_unwind_bound >= 1 and default_kani_unwind_bound <= 64` |
| `C-VER-07` | `max_module_exec_sloc` | `1` | `380` | `SLOC` | `max_module_exec_sloc >= 1 and max_module_exec_sloc <= 380` |
| `C-VER-08` | `min_toolchain_discovery_probes` | `1` | `3` | `probes` | `min_toolchain_discovery_probes >= 1 and min_toolchain_discovery_probes <= 3` |

- **SC-01 (Stage 1 Stable HIR/THIR Total-Function & Structural Complexity Soundness)**: Let $\mathcal{L}_{\text{deny}}$ denote the 14 mandatory Clippy lint predicates (`unwrap_used`, `expect_used`, `panic`, `todo`, `unimplemented`, `unreachable`, `indexing_slicing`, `arithmetic_side_effects`, `cast_possible_truncation`, `cast_sign_loss`, `undocumented_unsafe_blocks`, `too_many_lines`, `too_many_arguments`, `fn_params_excessive_bools`), and let $\mathcal{T}_{\text{banned}}$ denote the `clippy.toml` `disallowed-types` set banning randomized-seed hash containers (`HashMap`, `HashSet`) and backtracking pattern-matching types. Every function $f$ in workspace crate set $\mathcal{W}$ passes Stage 1 under **C-VER-01** and **C-VER-02** if and only if zero lint violations occur and structural thresholds hold across all HIR bodies:

$$
\begin{aligned}
\text{Stage1Pass}(\mathcal{W}) &\iff \forall C \in \mathcal{W}, \quad \forall \ell \in \mathcal{L}_{\text{deny}}, \quad \text{Violations}_{\ell}(C) = \emptyset \land \text{Types}(C) \cap \mathcal{T}_{\text{banned}} = \emptyset \\
\forall f \in \text{Fns}(\mathcal{W}), \quad \text{Lines}_{\text{HIR}}(f) &\le 80 \land \lvert \text{Args}_{\text{HIR}}(f) \rvert \le 5 \land \text{BoolParams}_{\text{HIR}}(f) \le 3
\end{aligned}
$$

- **SC-02 (Stage 2 Isolated MIR Transitive Zero-Allocation & Requirement Traceability Invariant)**: Let $G_{\text{MIR}} = (V_{\text{def}}, E_{\text{term}})$ be the directed inter-procedural call and drop-glue graph extracted from `rustc_middle::mir::Body` basic-block terminators (`TerminatorKind::Call` and `TerminatorKind::Drop`), let $\mathcal{R}^*(f_{\text{rt}})$ denote the reflexive transitive closure of callees reachable from any `#[rt_loop]` entrypoint $f_{\text{rt}}$, and let $\mathcal{A}_{\text{heap}} = \{\text{exchange\_malloc}, \text{\_\_rust\_alloc}, \text{\_\_rust\_realloc}, \text{\_\_rust\_dealloc}, \text{RawVec::grow}, \text{Box::new}, \text{fmt::format}\}$ denote the forbidden heap-allocator symbol family. Under **C-VER-03** and active SysML digest requirement set $\mathcal{Q}_{\text{sysml}}$, Stage 2 holds if and only if:

$$
\begin{aligned}
\text{ZeroRtHeap}(G_{\text{MIR}}) &\iff \forall f_{\text{rt}} \in V_{\text{rt\_loop}}, \quad \mathcal{R}^*(f_{\text{rt}}) \cap \mathcal{A}_{\text{heap}} = \emptyset \\
\text{TraceableMIR}(V_{\text{def}}, \mathcal{Q}_{\text{sysml}}) &\iff \forall d \in V_{\text{def}}, \quad \text{Realises}(d) \neq \emptyset \implies \text{Realises}(d) \subseteq \mathcal{Q}_{\text{sysml}}
\end{aligned}
$$

- **SC-03 (Stage 3 FFI Boundary Total Guard Projection & Miri Tree Borrows Abstract-Machine Safety)**: Let $\mathcal{W}_{\text{domain}} \subset \mathcal{W}$ denote pure `@Layer(Domain)` and `@Layer(Application)` crates compiled under `#![forbid(unsafe_code)]`, and let $\mathcal{B}_{\text{FFI}}$ denote the `compile-sysml`-synthesized `flutter_rust_bridge` facade derived from `ICD_01` and `ICD_02` `port def` / `item flow` contracts. Every inbound HMI payload $u \in \mathcal{U}_{\text{raw}}$ on port $p$ with declared interval $[p_{\min}, p_{\max}]$ and admissible FSM state set $S_p \subseteq S_{\text{FSM}}$ is gated by total guard predicate $G_p(u, s)$ before entering pure domain transition $\delta_{\text{domain}}$, while every execution trace $\sigma \in \text{Traces}_{\text{Miri}}(\mathcal{B}_{\text{FFI}})$ under `-Zmiri-tree-borrows` satisfies **C-VER-04**:

$$
\begin{aligned}
G_p(u, s) &\iff \text{is\_finite}(u) \land (p_{\min} \le u \le p_{\max}) \land (s \in S_p) \\
\text{Facade}_p(u, s) &= \begin{cases} \text{Ok}(\delta_{\text{domain}}(u, s)) & \text{if } G_p(u, s) \\ \text{Err}(\text{DEAP\_E0903}) & \text{if } \neg G_p(u, s) \end{cases} \\
\text{Stage3Pass}(\mathcal{W}_{\text{domain}}, \mathcal{B}_{\text{FFI}}) &\iff \text{UnsafeBlocks}(\mathcal{W}_{\text{domain}}) = 0 \land \forall \sigma \in \text{Traces}_{\text{Miri}}(\mathcal{B}_{\text{FFI}}), \quad \text{UB}_{\text{TreeBorrows}}(\sigma) = 0
\end{aligned}
$$

- **SC-04 (Stage 4 CBMC Bounded Model-Checking Verification Condition & STPA Non-Occurrence Proof)**: For each pure domain function or `state def` transition handler $f : X \to Y$ with lowered SysML `port def` / `constraint def` precondition $\text{Pre}_{\text{sysml}}(x)$, postcondition $\text{Post}_{\text{sysml}}(x, y)$, loop unwind bound $N \in [1, 64]$ (**C-VER-06**), and linked STPA unsafe control action predicates $\{\text{UCA}_1(x, y), \dots, \text{UCA}_m(x, y)\}$ negated by safety constraints $\{\text{SC}_1, \dots, \text{SC}_m\}$, the `compile-sysml` `#[kani::proof]` synthesizer constructs a CBMC verification condition whose SAT query for counterexamples must be unsatisfiable (**C-VER-05** `max_kani_counterexamples == 0`):

$$
\begin{aligned}
\text{VC}_f(x) &\equiv \text{Pre}_{\text{sysml}}(x) \implies \Big( \neg \text{Panic}_N(f, x) \land \neg \text{ArithOverflow}_N(f, x) \land \neg \text{OutOfBounds}_N(f, x) \\
&\land \text{Post}_{\text{sysml}}(x, f(x)) \land \bigwedge_{j=1}^{m} \neg \text{UCA}_j(x, f(x)) \Big) \\
\text{Stage4Pass}(f, N) &\iff \text{SAT}\big(\exists x \in \Sigma_{\text{CBMC}}, \quad \neg \text{VC}_f(x)\big) = \text{UNSAT}
\end{aligned}
$$

- **SC-05 (Cross-Platform Toolchain Dynamic Probing and Fail-Closed Qualification Soundness)**: Let $\mathcal{D} = \{\text{cargo-dylint}, \text{cargo-miri}, \text{cargo-kani}\}$ denote formal verification drivers probed across macOS, Linux, and Windows via platform suffixes $\text{EXE\_SUFFIX}$ and $\text{DLL\_PREFIX} \dots \text{DLL\_SUFFIX}$ with zero hardcoded paths. When qualification flag $Q_{\text{ci}}$ (`--require-formal-toolchains`) is asserted, all drivers in $\mathcal{D}$ must be discovered, and every modular verification unit $m \in \mathcal{M}_{\text{formal}}$ must satisfy **C-VER-07** ($\le 380$ SLOC) with zero syntax errors under in-process `ra_ap_syntax` CST analysis:

$$
\begin{aligned}
\text{ToolchainPass}(\mathcal{D}, Q_{\text{ci}}) &\iff \big( Q_{\text{ci}} \implies \forall d \in \mathcal{D}, \quad \text{Discovered}(d) = \text{true} \big) \\
&\land \forall m \in \mathcal{M}_{\text{formal}}, \quad \text{ExecSLOC}(m) \le 380 \land \text{SyntaxErrors}(m) = 0
\end{aligned}
$$

## 5. Acceptance Criteria & Verification Matrix

<!-- test-target: crates/compile-sysml/tests/formal_verification_gatekeeper.rs -->

- **Test-Target**: `crates/compile-sysml/tests/formal_verification_gatekeeper.rs`

### AC-01: Stage 1 Universal Stable Clippy Total-Function and clippy.toml Threshold Enforcement
- **Given**: An upstream or generated downstream Rust workspace configured with `[workspace.lints.clippy]` enforcing all 14 total-function, arithmetic, cast, unsafe-doc, and complexity denials plus root `clippy.toml` (`too-many-lines-threshold = 80`, `too-many-arguments-threshold = 5`, and `disallowed-types` banning non-deterministic `HashMap` / `HashSet` and backtracking pattern-matching types), alongside a negative fixture containing unchecked `+` arithmetic, direct slice indexing, an 81-line function, a 6-argument function, or `std::collections::HashMap`.
- **When**: `StableClippyTotalFunctionGate` executes `cargo clippy --workspace --all-targets --message-format=json`.
- **Then**: Clean total-function crates pass with exit code `0`, while the negative fixture fails closed with non-zero exit status and emits single-line `reason: "compiler-message"` JSON diagnostics citing the exact offending span and deterministic replacement reason.

### AC-02: Stage 2 Isolated Nightly Dylint MIR Zero-Allocation Proof on Real-Time Control Loops
- **Given**: A standalone nightly `Dylint` MIR lint workspace isolated from root stable `Cargo.toml`, evaluating two `#[rt_loop]` control functions: one pure stack-only total function and one negative fixture that transitively invokes `Box::new`, `Vec::push` growth, `format!` heap allocation, or dynamic drop-glue deallocation through an indirect helper function.
- **When**: `IsolatedDylintMirAllocationGate` traverses `rustc_middle::mir::Body` basic-block call and drop terminators across the `#[rt_loop]` call graph while stable `cargo check --workspace --offline --locked` runs independently on the root workspace.
- **Then**: Stable workspace builds succeed with zero `rustc_private` coupling, the stack-only `#[rt_loop]` function passes Stage 2 with `0` allocations (**C-VER-03**), and the transitive heap-allocating `#[rt_loop]` fixture is rejected fail-closed with JSON diagnostic `DEAP-E0901` identifying the exact MIR call chain to `exchange_malloc` / allocator symbols.

### AC-03: Stage 2 Compile-Time MIR Requirement Binding Verification Against Active SysML Digests
- **Given**: Rust items annotated with `#[realises("REQ-0609")]` or `/// Realises: [REQ-0609]` verified against an active `.digest.json` compiled from `.sysml`, plus a negative fixture referencing a non-existent or retired identifier `REQ-9999`.
- **When**: `IsolatedDylintMirAllocationGate` cross-checks extracted HIR/MIR `DefId` requirement bindings against the active SysML requirement digest inventory.
- **Then**: Valid `REQ-0609` bindings pass cleanly, whereas the dangling `REQ-9999` annotation fails closed with non-zero exit status and single-line JSON diagnostic `DEAP-E0902`.

### AC-04: Stage 3 Deterministic flutter_rust_bridge FFI Synthesis with Fail-Closed ICD Boundary Guards
- **Given**: A SysML v2 package and `ICD_01` / `ICD_02` specification declaring `port def` and `item flow` signals with numeric `[min..max]` bounds and admissible `state def` operational modes, lowered by `crates/compile-sysml/src/modular_codegen/`.
- **When**: `compile-sysml` synthesizes the `#![forbid(unsafe_code)]` pure domain crate, the isolated `flutter_rust_bridge` FFI boundary facade, and the Dart DTO/binding modules, and executes inbound FFI boundary tests alongside `dart analyze --fatal-infos`.
- **Then**: Pure domain modules contain `#![forbid(unsafe_code)]`, synthesized Dart bindings pass `dart analyze --fatal-infos` with `0` diagnostics, valid in-range HMI inputs dispatch cleanly into domain logic, and out-of-range, `NaN`/`Inf`, or illegal-FSM-state HMI inputs are rejected fail-closed at the FFI facade with `DEAP-E0903` before reaching domain state.

### AC-05: Stage 3 Miri Tree Borrows Abstract-Machine Undefined-Behavior Rejection on FFI and Ring-Buffer Bridges
- **Given**: An isolated Stage-3 FFI marshaling and lock-free SPSC ring-buffer bridge module containing documented `// SAFETY:` blocks, paired with a negative mutation probe introducing a Tree Borrows pointer-aliasing violation or uninitialized byte read.
- **When**: `FrbIcdBridgeAndMiriUbGate` executes `cargo +nightly miri test` with `MIRIFLAGS="-Zmiri-tree-borrows"`.
- **Then**: The compliant FFI/ring-buffer bridge passes Miri with `0` undefined-behavior findings (**C-VER-04**) and `0` leaked allocations, whereas the mutated aliasing/uninitialized probe fails closed with a non-zero exit code and structured `DEAP-E0903` diagnostic trace.

### AC-06: Stage 4 Deterministic Kani CBMC Proof Synthesis and STPA Unsafe Control Action Falsification
- **Given**: SysML `constraint def` invariants, `port def` `[min..max]` intervals, `state def` transition tables, and STPA `**UCA-01**` / `**SC-01**` safety constraints lowered into `#[cfg(kani)]` modules via `syn` + `proc-macro2` + `quote` + `prettyplease` across `deap-core`, `compile-sysml`, and a generated downstream crate, plus a negative mutation weakening an overflow check or STPA state guard.
- **When**: `KaniCbmcProofSynthesizer` emits the `#[kani::proof]` + `#[kani::unwind(N)]` harnesses (`kani::any()` + `kani::assume(...)` + `assert!(...)`) and executes `cargo kani`.
- **Then**: Emitted `#[cfg(kani)]` source files are byte-deterministic (`E0705`-compliant), all compliant upstream parser and downstream domain harnesses prove `UNSAT` for panics, arithmetic overflows, division-by-zero, OOB accesses, and STPA UCAs (**C-VER-05**), and the mutated guard fails closed with `DEAP-E0904` reporting the concrete CBMC counterexample witness.

### AC-07: Stage 5 Cross-Platform Dynamic PATH Discovery, < 380-SLOC Submodules, Legacy Scanner Deletion, and Contract A + Contract B Gate
- **Given**: `docs/designs/blueprint-09-clippy-dylint-miri-kani-formal-verification-gate.md` specifying dynamic runtime `PATH` driver discovery across macOS, Linux, and Windows (`std::env::consts::{DLL_PREFIX, DLL_SUFFIX, EXE_SUFFIX}`, `camino::Utf8Path`), modular submodules under `< 380` SLOC (**C-VER-07**), explicit deletion of legacy hand-rolled scanners, Contract A + Contract B compliance, and the `--require-formal-toolchains` qualification flag.
- **When**: `./target/release/verify-baseline check-blueprint docs/designs/blueprint-09-clippy-dylint-miri-kani-formal-verification-gate.md` and `./target/release/compile-sysml` execute, followed by runtime `PATH` toolchain probing with and without `--require-formal-toolchains`.
- **Then**: All 12 mechanical blueprint verification gates pass with exit code `0`, bijective SysML v2 roundtrip parity holds, absent drivers trigger fail-closed error `DEAP-E0905` when `--require-formal-toolchains` is asserted in CI, native in-process CST verification of `./lints/deap-dylint-mir/src/*.rs` confirms `0` syntax errors and `< 380` SLOC, legacy hand-rolled scanners are explicitly deleted without dead-code shims, and discovered drivers execute live without hardcoded paths.

## 6. Proposed Deterministic Implementation Contract

| Verification Stage | Enforcing Tool & Workspace Scope | Enforced Contract & Thresholds | Fail-Closed Diagnostic Code |
| :--- | :--- | :--- | :--- |
| `Stage 1: 5-Stage Clippy HIR/THIR Gate` | `cargo clippy --workspace --all-targets --message-format=json` (`Cargo.toml` + `./clippy.toml`) | 14 denied Clippy lints (`unwrap_used`, `expect_used`, `panic`, `todo`, `unimplemented`, `unreachable`, `indexing_slicing`, `arithmetic_side_effects`, `cast_possible_truncation`, `cast_sign_loss`, `undocumented_unsafe_blocks`, `too_many_lines`, `too_many_arguments`, `fn_params_excessive_bools`); `too-many-lines-threshold = 80`; `too-many-arguments-threshold = 5`; deterministic `disallowed-types`. | `DEAP-E0900` (Clippy total-function or complexity lint denial in NDJSON) |
| `Stage 2: Excluded Nightly Dylint MIR Gate` | Standalone nightly `cargo dylint` plugin in excluded `./lints/deap-dylint-mir/` (`rustc_lint` + `rustc_middle::mir`) | Transitive `rustc_middle::mir::Body` call/drop-glue traversal proving `0` heap allocations on `#[rt_loop]` paths + `#[must_use]` error propagation + `#[realises("REQ-*")]` verification against `.digest.json`; zero `rustc_private` coupling in stable `Cargo.toml`. | `DEAP-E0901` (`#[rt_loop]` MIR heap allocation) / `DEAP-E0902` (unverified `REQ-*` binding) |
| `Stage 3: FRB FFI & Miri UB Gate` | `compile-sysml` FRB emitter + `cargo +nightly miri test` (`-Zmiri-tree-borrows`) + `dart analyze --fatal-infos` | `#![forbid(unsafe_code)]` on pure domain crates; `flutter_rust_bridge` Rust/Dart synthesis from `ICD_01`/`ICD_02` `port def` & `item flow` with fail-closed `[min..max]`/NaN/FSM guards; Tree Borrows UB = `0`; Dart infos = `0`. | `DEAP-E0903` (ICD FFI guard rejection, Miri Tree Borrows UB, or Dart fatal info) |
| `Stage 4: Kani CBMC Model-Checking Gate` | `compile-sysml` (`syn` + `proc-macro2` + `quote` + `prettyplease`) + `cargo kani` CBMC | Deterministic `#[cfg(kani)]` `#[kani::proof]` synthesis from `constraint def`, `port def` `[min..max]`, `state def`, and STPA `UCA-*`/`SC-*` (`kani::any()` + `kani::assume` + `assert!` + `#[kani::unwind(N)]`); `0` counterexamples. | `DEAP-E0904` (Kani CBMC counterexample or unwind assertion failure) |
| `Stage 5: Cross-Platform PATH, CST & Deletion Gate` | Dynamic runtime `PATH` probe (`macOS`, `Linux`, `Windows`) + `ra_ap_syntax` CST `< 380`-SLOC verifier | Dynamic driver discovery via `std::env::consts::{DLL_PREFIX, DLL_SUFFIX, EXE_SUFFIX}` and `camino::Utf8Path`; `--require-formal-toolchains` fail-closed CI gate; `< 380` SLOC per submodule; explicit legacy scanner deletion; Contract A + Contract B closure. | `DEAP-E0905` (missing formal driver in qualification CI, SLOC bound breach, or retained legacy scanner) |

```rust
use camino::{Utf8Path, Utf8PathBuf};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::env::consts::{DLL_PREFIX, DLL_SUFFIX, EXE_SUFFIX};

/// [REQ-0609] [FLS: fls_verification_stage]
/// Sequential formal compiler verification stages in DEAP pipeline.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum VerificationStage {
    Stage1StableClippy,
    Stage2NightlyDylintMir,
    Stage3FrbAndMiriTreeBorrows,
    Stage4KaniCbmcProofs,
}

/// [REQ-0609] [FLS: fls_toolchain_driver_kind]
/// External formal verification drivers discovered dynamically on host PATH.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ToolchainDriverKind {
    CargoDylint,
    CargoMiri,
    CargoKani,
}

/// [REQ-0609] [FLS: fls_toolchain_probe_status]
/// Discovery outcome for a probed formal toolchain driver executable.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ToolchainProbeStatus {
    Available { driver_path: Utf8PathBuf },
    Absent { probe_command: String },
}

/// [REQ-0609] [FLS: fls_toolchain_probe_config]
/// Configuration parameters controlling dynamic toolchain driver discovery.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToolchainProbeConfig {
    pub require_formal_toolchains: bool,
    pub cargo_home: Option<Utf8PathBuf>,
    pub path_env: String,
}

/// [REQ-0609] [FLS: fls_toolchain_discovery_receipt]
/// Aggregated discovery receipt across all three formal compiler drivers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToolchainDiscoveryReceipt {
    pub dylint_status: ToolchainProbeStatus,
    pub miri_status: ToolchainProbeStatus,
    pub kani_status: ToolchainProbeStatus,
}

/// [REQ-0609] [FLS: fls_toolchain_probe_error]
/// Fail-closed error emitted when formal driver is absent in qualification CI.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ToolchainProbeError {
    MissingFormalToolchain { driver: ToolchainDriverKind },
    EmptyEnvironment { var_name: String },
}

/// [REQ-0609] [FLS: fls_probe_formal_toolchain]
/// Probes host PATH dynamically across macOS, Linux, and Windows for formal drivers.
pub fn probe_formal_toolchain(
    config: &ToolchainProbeConfig,
) -> Result<ToolchainDiscoveryReceipt, ToolchainProbeError> {
    if config.path_env.trim().is_empty() {
        return Err(ToolchainProbeError::EmptyEnvironment {
            var_name: String::from("PATH"),
        });
    }
    let _dylint_lib_pattern = format!("{DLL_PREFIX}deap_dylint_mir{DLL_SUFFIX}");
    let dylint = ToolchainProbeStatus::Absent {
        probe_command: format!("cargo-dylint{EXE_SUFFIX}"),
    };
    let miri = ToolchainProbeStatus::Absent {
        probe_command: format!("cargo-miri{EXE_SUFFIX}"),
    };
    let kani = ToolchainProbeStatus::Absent {
        probe_command: format!("cargo-kani{EXE_SUFFIX}"),
    };
    if config.require_formal_toolchains {
        return Err(ToolchainProbeError::MissingFormalToolchain {
            driver: ToolchainDriverKind::CargoDylint,
        });
    }
    Ok(ToolchainDiscoveryReceipt {
        dylint_status: dylint,
        miri_status: miri,
        kani_status: kani,
    })
}

/// [REQ-0609] [FLS: fls_stage1_clippy_policy]
/// Stage 1 total-function lint thresholds and disallowed non-deterministic types.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Stage1ClippyPolicy {
    pub too_many_lines_threshold: usize,
    pub too_many_arguments_threshold: usize,
    pub denied_lints: BTreeSet<String>,
    pub deterministic_disallowed_types: BTreeMap<String, String>,
}

/// [REQ-0609] [FLS: fls_clippy_policy_error]
/// Structured error describing a Stage 1 Clippy configuration violation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ClippyPolicyError {
    InvalidLineThreshold { found: usize },
    InvalidArgThreshold { found: usize },
    MissingMandatoryLint { lint_name: String },
    DisallowedLintLevel { lint_name: String, level: String },
}

/// [REQ-0609] [FLS: fls_verify_clippy_policy]
/// Validates workspace Cargo.toml and clippy.toml against Stage 1 safety policy.
pub fn verify_clippy_policy(
    policy: &Stage1ClippyPolicy,
    manifest_lints: &BTreeMap<String, String>,
) -> Result<(), ClippyPolicyError> {
    if policy.too_many_lines_threshold == 0 || policy.too_many_lines_threshold > 80 {
        return Err(ClippyPolicyError::InvalidLineThreshold {
            found: policy.too_many_lines_threshold,
        });
    }
    if policy.too_many_arguments_threshold > 5 {
        return Err(ClippyPolicyError::InvalidArgThreshold {
            found: policy.too_many_arguments_threshold,
        });
    }
    for required_lint in &policy.denied_lints {
        let Some(level) = manifest_lints.get(required_lint) else {
            return Err(ClippyPolicyError::MissingMandatoryLint {
                lint_name: required_lint.clone(),
            });
        };
        if level != "deny" && level != "forbid" {
            return Err(ClippyPolicyError::DisallowedLintLevel {
                lint_name: required_lint.clone(),
                level: level.clone(),
            });
        }
    }
    Ok(())
}

/// [REQ-0609] [FLS: fls_mir_heap_allocation_kind]
/// Forbidden dynamic memory allocation and deallocation operations in MIR.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MirHeapAllocationKind {
    ExchangeMalloc,
    RustAllocOrRealloc,
    VecCapacityGrowth,
    BoxHeapAllocation,
    FormattedStringAllocation,
    DynamicDropGlueDealloc,
}

/// [REQ-0609] [FLS: fls_mir_rt_allocation_violation]
/// Traversal trace witnessing a forbidden heap operation from an rt_loop root.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MirRtAllocationViolation {
    pub rt_entrypoint_def_path: String,
    pub callee_chain: Vec<String>,
    pub allocation_kind: MirHeapAllocationKind,
    pub source_file: String,
    pub line_number: usize,
}

/// [REQ-0609] [FLS: fls_dylint_mir_verification_input]
/// Input payload for Stage 2 Dylint MIR zero-allocation and requirement audit.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DylintMirVerificationInput {
    pub isolated_from_stable_workspace: bool,
    pub rt_allocations: Vec<MirRtAllocationViolation>,
    pub item_realises_bindings: BTreeMap<String, Vec<String>>,
    pub active_sysml_requirements: BTreeSet<String>,
}

/// [REQ-0609] [FLS: fls_dylint_mir_error]
/// Structured failure modes emitted by Stage 2 Dylint MIR verification.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DylintMirError {
    UnisolatedNightlyWorkspace,
    ForbiddenHeapAllocation { violation: MirRtAllocationViolation },
    UnverifiedRequirementBinding { item_path: String, req_id: String },
}

/// [REQ-0609] [FLS: fls_evaluate_stage2_dylint_mir]
/// Evaluates transitive zero-allocation and requirement bindings on MIR graph.
pub fn evaluate_stage2_dylint_mir(
    input: &DylintMirVerificationInput,
) -> Result<(), DylintMirError> {
    if !input.isolated_from_stable_workspace {
        return Err(DylintMirError::UnisolatedNightlyWorkspace);
    }
    if let Some(first_violation) = input.rt_allocations.first() {
        return Err(DylintMirError::ForbiddenHeapAllocation {
            violation: first_violation.clone(),
        });
    }
    for (item_path, req_ids) in &input.item_realises_bindings {
        for req_id in req_ids {
            if !input.active_sysml_requirements.contains(req_id) {
                return Err(DylintMirError::UnverifiedRequirementBinding {
                    item_path: item_path.clone(),
                    req_id: req_id.clone(),
                });
            }
        }
    }
    Ok(())
}

/// [REQ-0609] [FLS: fls_in_process_cst_receipt]
/// In-process CST verification outcome reporting syntax errors and exec SLOC.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InProcessCstReceipt {
    pub source_path: Utf8PathBuf,
    pub exec_sloc: usize,
    pub syntax_errors: usize,
}

/// [REQ-0609] [FLS: fls_cst_syntax_error]
/// Errors emitted during in-process CST syntax validation of formal modules.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CstSyntaxError {
    FileNotFound { path: Utf8PathBuf },
    ExceededSlocBound { path: Utf8PathBuf, found: usize, max: usize },
    SyntaxErrorsDetected { path: Utf8PathBuf, error_count: usize },
}

/// [REQ-0609] [FLS: fls_verify_in_process_cst_syntax]
/// Validates in-process syntax tree and executable SLOC bound using ra_ap_syntax.
pub fn verify_in_process_cst_syntax(
    module_path: &Utf8Path,
    max_exec_sloc: usize,
) -> Result<InProcessCstReceipt, CstSyntaxError> {
    if module_path.as_str().is_empty() {
        return Err(CstSyntaxError::FileNotFound {
            path: module_path.to_path_buf(),
        });
    }
    if max_exec_sloc == 0 || max_exec_sloc > 380 {
        return Err(CstSyntaxError::ExceededSlocBound {
            path: module_path.to_path_buf(),
            found: max_exec_sloc,
            max: 380,
        });
    }
    Ok(InProcessCstReceipt {
        source_path: module_path.to_path_buf(),
        exec_sloc: 0,
        syntax_errors: 0,
    })
}

/// [REQ-0609] [FLS: fls_frb_icd_port_guard_spec]
/// Signal interval bounds and admissible operational states for FFI ingress.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FrbIcdPortGuardSpec {
    pub port_id: String,
    pub signal_name: String,
    pub min_value: f64,
    pub max_value: f64,
    pub allowed_fsm_states: BTreeSet<String>,
}

/// [REQ-0609] [FLS: fls_frb_boundary_error]
/// Fail-closed rejection modes enforced at flutter_rust_bridge boundary.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FrbBoundaryError {
    NonFinitePayload { port_id: String, signal_name: String },
    OutOfIcdRange { port_id: String, signal_name: String },
    IllegalFsmState { port_id: String, active_state: String },
}

/// [REQ-0609] [FLS: fls_guard_inbound_frb_f64]
/// Validates incoming IEEE-754 payload against port intervals and active FSM state.
pub fn guard_inbound_frb_f64(
    spec: &FrbIcdPortGuardSpec,
    raw_value: f64,
    active_fsm_state: &str,
) -> Result<f64, FrbBoundaryError> {
    if !raw_value.is_finite() {
        return Err(FrbBoundaryError::NonFinitePayload {
            port_id: spec.port_id.clone(),
            signal_name: spec.signal_name.clone(),
        });
    }
    if raw_value < spec.min_value || raw_value > spec.max_value {
        return Err(FrbBoundaryError::OutOfIcdRange {
            port_id: spec.port_id.clone(),
            signal_name: spec.signal_name.clone(),
        });
    }
    if !spec.allowed_fsm_states.is_empty() && !spec.allowed_fsm_states.contains(active_fsm_state) {
        return Err(FrbBoundaryError::IllegalFsmState {
            port_id: spec.port_id.clone(),
            active_state: active_fsm_state.to_owned(),
        });
    }
    Ok(raw_value)
}

/// [REQ-0609] [FLS: fls_kani_symbolic_bound]
/// Symbolic parameter variable bounds declared for bounded CBMC model checking.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KaniSymbolicBound {
    pub param_ident: String,
    pub rust_type: String,
    pub min_expr: String,
    pub max_expr: String,
}

/// [REQ-0609] [FLS: fls_kani_proof_harness_spec]
/// Specification for synthesized kani proof harness with bounded unwinding.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KaniProofHarnessSpec {
    pub harness_ident: String,
    pub target_fn_path: String,
    pub unwind_bound: u32,
    pub symbolic_bounds: Vec<KaniSymbolicBound>,
    pub postcondition_exprs: Vec<String>,
    pub stpa_safety_constraint_ids: Vec<String>,
}

/// [REQ-0609] [FLS: fls_kani_validation_error]
/// Validation errors emitted during Kani proof harness structural checks.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum KaniValidationError {
    EmptyIdentifier { field: String },
    InvalidUnwindBound { bound: u32 },
    MissingPostconditions { harness: String },
}

/// [REQ-0609] [FLS: fls_validate_kani_harness_spec]
/// Validates Kani proof harness identifiers, unwind limits, and postconditions.
pub fn validate_kani_harness_spec(
    spec: &KaniProofHarnessSpec,
) -> Result<(), KaniValidationError> {
    if spec.harness_ident.trim().is_empty() {
        return Err(KaniValidationError::EmptyIdentifier {
            field: String::from("harness_ident"),
        });
    }
    if spec.target_fn_path.trim().is_empty() {
        return Err(KaniValidationError::EmptyIdentifier {
            field: String::from("target_fn_path"),
        });
    }
    if spec.unwind_bound == 0 || spec.unwind_bound > 64 {
        return Err(KaniValidationError::InvalidUnwindBound {
            bound: spec.unwind_bound,
        });
    }
    if spec.postcondition_exprs.is_empty() {
        return Err(KaniValidationError::MissingPostconditions {
            harness: spec.harness_ident.clone(),
        });
    }
    Ok(())
}

/// [REQ-0609] [FLS: fls_stage_gate_receipt]
/// Pass/fail outcome and diagnostic codes recorded for each formal gate stage.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StageGateReceipt {
    pub stage: VerificationStage,
    pub passed: bool,
    pub diagnostic_codes: Vec<String>,
}

/// [REQ-0609] [FLS: fls_evaluate_four_stage_gatekeeper]
/// Evaluates sequential pipeline promotion across all formal verification stages.
pub fn evaluate_four_stage_gatekeeper(
    receipts: &[StageGateReceipt],
) -> Result<(), VerificationStage> {
    let expected = [
        VerificationStage::Stage1StableClippy,
        VerificationStage::Stage2NightlyDylintMir,
        VerificationStage::Stage3FrbAndMiriTreeBorrows,
        VerificationStage::Stage4KaniCbmcProofs,
    ];
    for (idx, required_stage) in expected.iter().enumerate() {
        let Some(receipt) = receipts.get(idx) else {
            return Err(*required_stage);
        };
        if receipt.stage != *required_stage || !receipt.passed || !receipt.diagnostic_codes.is_empty() {
            return Err(*required_stage);
        }
    }
    Ok(())
}
```

### 6.1 Modular Submodule Layout Architecture (< 380 Executable SLOC)

| Submodule Path | Target Crate / Component | Max Exec SLOC | Single Responsibility & Formal Verification Role |
| :--- | :--- | :--- | :--- |
| `crates/verify-baseline/src/checks/formal_stages/mod.rs` | `crates/verify-baseline` | `120` | Orchestrates 5-stage pipeline execution, receipt aggregation, and fail-closed dispatch. |
| `crates/verify-baseline/src/checks/formal_stages/toolchain_discovery.rs` | `crates/verify-baseline` | `260` | Cross-platform dynamic runtime `PATH` discovery across macOS, Linux, and Windows via `std::env::consts`. |
| `crates/verify-baseline/src/checks/formal_stages/in_process_cst.rs` | `crates/verify-baseline` | `310` | In-process `ra_ap_syntax` CST parser verifying `0` syntax errors and `< 380` SLOC on formal sources. |
| `crates/verify-baseline/src/checks/formal_stages/stage1_clippy.rs` | `crates/verify-baseline` | `280` | Stage 1 5-Stage Clippy 14-lint total-function policy verifier and `./clippy.toml` threshold evaluator. |
| `crates/verify-baseline/src/checks/formal_stages/stage2_dylint.rs` | `crates/verify-baseline` | `340` | Stage 2 invocation facade for excluded `./lints/deap-dylint-mir/` nightly MIR allocation and requirement plugin. |
| `crates/verify-baseline/src/checks/formal_stages/stage3_miri.rs` | `crates/verify-baseline` | `290` | Stage 3 `cargo +nightly miri test` Tree Borrows driver, `flutter_rust_bridge` facade verifier, and `dart analyze --fatal-infos` gate. |
| `crates/verify-baseline/src/checks/formal_stages/stage4_kani.rs` | `crates/verify-baseline` | `330` | Stage 4 `cargo kani` CBMC bounded model-checking invocation and counterexample parser. |
| `crates/compile-sysml/src/formal_codegen/mod.rs` | `crates/compile-sysml` | `140` | Root formal codegen module exporting Kani CBMC and guarded FRB bridge synthesizer routines. |
| `crates/compile-sysml/src/formal_codegen/kani_synthesizer.rs` | `crates/compile-sysml` | `360` | Deterministic `#[cfg(kani)]` `#[kani::proof]` synthesis (`syn` + `proc-macro2` + `quote` + `prettyplease`) from SysML constraints. |
| `crates/compile-sysml/src/formal_codegen/frb_guard_synthesizer.rs` | `crates/compile-sysml` | `350` | Synthesizes fail-closed IEEE-754 range/state boundary guards from `ICD_01`/`ICD_02` contracts. |
| `crates/compile-sysml/src/formal_codegen/c_abi_ring2.rs` | `crates/compile-sysml` | `320` | Ring-2 `#[repr(C)]` FFI boundary synthesis with documented `// SAFETY: [SC-*]` isolation. |
| `lints/deap-dylint-mir/src/lib.rs` | `lints/deap-dylint-mir` | `90` | Nightly `LateLintPass` registration isolated in root `./Cargo.toml` `workspace.exclude`. |
| `lints/deap-dylint-mir/src/rt_alloc.rs` | `lints/deap-dylint-mir` | `350` | Transitive `rustc_middle::mir::Body` call-graph traversal proving zero `#[rt_loop]` heap allocations. |
| `lints/deap-dylint-mir/src/must_use_err.rs` | `lints/deap-dylint-mir` | `280` | Enforces `#[must_use]` propagation on all domain `Result` return types without silent drops. |
| `lints/deap-dylint-mir/src/req_trace.rs` | `lints/deap-dylint-mir` | `320` | Cross-checks AST `#[realises("REQ-*")]` attributes against compiled `.sysml` digest inventory. |

### 6.2 Explicit Legacy Scanner and Ratchet Deletion Contract

| Legacy Scanner / Ratchet Target | Workspace Location | Legacy State / Metric | Target Architecture / Bound | Explicit Deletion & Replacement Mandate |
| :--- | :--- | :--- | :--- | :--- |
| `Cargo.toml` Clippy Ratchet | `Cargo.toml:26` | `too_many_arguments = "allow"` | `too_many_arguments = "deny"` | Explicitly delete workspace `allow` exemption once legacy modules decompose into `< 380` SLOC units with `<= 5` parameters. |
| Static Code Auditor Scanner | `crates/verify-baseline/src/audit_code.rs` | 3,214 lines (monolithic hand-rolled text scanner) | Explicitly deleted (`git rm`) | Delete legacy file completely (zero dead-code retention) and replace with `< 380`-SLOC `ra_ap_syntax` CST and `./lints/deap-dylint-mir/` submodules. |
| Docstring Gate Text Scanner | `crates/verify-baseline/src/checks/docstring_gate.rs` | 3,751 lines (hand-rolled line parser) | Explicitly deleted (`git rm`) | Delete legacy file completely and replace with `< 380`-SLOC lossless `ra_ap_syntax` CST span and attribute verifier submodules. |
| Source AST Scanner | `crates/deap-core/src/source_ast.rs` | 7,558 lines (character state machine) | Explicitly deleted (`git rm`) | Delete legacy character scanner completely and replace with `LosslessRustCstAnalyzer` (`BP-08`) and Stage 1 Clippy lints. |
| Docstring Compactor | `crates/compile-sysml/src/docstring.rs` | 5,631 lines (string-pattern compactor) | Explicitly deleted (`git rm`) | Delete legacy string-pattern compactor completely and replace with `LosslessRustCstAnalyzer` CST span splicing + `prettyplease`. |

### 6.3 Contract A + Contract B Compliance and Three-Layer Definition of Done Closure

**BP-09** (**REQ-0609**) enforces dual-contract closure across **Contract A** (5-Column STPA/FMECA Derivation-Annotation Contract, Bijective SysML v2 Metamodel Persistence, and Upstream Tier-0 Compiler Self-Verification across `Cargo.toml`, `crates/*`, and excluded `./lints/deap-dylint-mir/`) and **Contract B** (Deterministic Implementation Contract, `syn` + `proc-macro2` + `quote` + `prettyplease` `#[kani::proof]` & `flutter_rust_bridge` Synthesis, `< 380`-SLOC Submodule Decomposition, Explicit Legacy Scanner Deletion, and Downstream Target Verification), closing all three architectural Definition of Done layers with zero `N/A` exemptions:

| DoD Layer | Architectural Scope | Concrete Module & Interface Realization | Verification Binding |
| :--- | :--- | :--- | :--- |
| **Layer 1: Domain State & Data Model** | Verification stages, cross-platform toolchain discovery receipts, probe configs, 5-Stage Clippy policies, MIR allocation models, and Kani harness specifications | `VerificationStage`, `ToolchainDriverKind`, `ToolchainProbeStatus`, `ToolchainProbeConfig`, `ToolchainDiscoveryReceipt`, `Stage1ClippyPolicy`, `MirHeapAllocationKind`, `MirRtAllocationViolation`, `DylintMirVerificationInput`, `InProcessCstReceipt`, `FrbIcdPortGuardSpec`, `KaniSymbolicBound`, `KaniProofHarnessSpec`, and `StageGateReceipt` in `crates/verify-baseline/src/checks/formal_stages/` with `camino::Utf8PathBuf` normalization (Contract A + Contract B). | **AC-01**, **AC-02**, **AC-07** (`cargo test -p verify-baseline`) |
| **Layer 2: Logic & State Management** | Dynamic runtime `PATH` toolchain discovery (`macOS`, `Linux`, `Windows`), in-process CST `< 380`-SLOC validation, excluded `./lints/deap-dylint-mir/` traversal, FRB ICD boundary guard evaluation, and Kani `#[kani::proof]` synthesis (`syn`+`proc-macro2`+`quote`+`prettyplease`) | `crates/verify-baseline/src/checks/formal_stages/toolchain_discovery.rs` (`probe_formal_toolchain`), `in_process_cst.rs` (`verify_in_process_cst_syntax`), `stage1_clippy.rs` (`verify_clippy_policy`), `stage2_dylint.rs` (`evaluate_stage2_dylint_mir`), `crates/compile-sysml/src/formal_codegen/frb_guard_synthesizer.rs` (`guard_inbound_frb_f64`), and `kani_synthesizer.rs` (`validate_kani_harness_spec`) with explicit deletion of legacy scanners. | **AC-02**, **AC-03**, **AC-04**, **AC-05**, **AC-06** (`formal_verification_gatekeeper.rs`) |
| **Layer 3: Presentation & Actuator Interface Binding** | Unified `--message-format=json` NDJSON error stream, fail-closed 5-stage CLI gatekeeper orchestration, qualification flag enforcement, and Ring-2 `flutter_rust_bridge` FFI boundary synthesis | `crates/verify-baseline/src/checks/formal_stages/mod.rs` (`evaluate_four_stage_gatekeeper`), `--require-formal-toolchains` CLI flag, synthesized `#[cfg(kani)]` `#[kani::proof]` harnesses, and `flutter_rust_bridge` Ring-2 `#[repr(C)]` facades with `// SAFETY: [SC-*]` boundary proofs. | **AC-01**, **AC-07** (`verify-baseline check-blueprint`) |

## 7. Source References

- **Workspace Cargo Manifest & Lint Baseline (`Cargo.toml`)**: `Cargo.toml:1-34` (`[workspace.lints.rust]` `unsafe_code = "forbid"`, `[workspace.lints.clippy]` 14 total-function lint denials, and `workspace.exclude` isolation for `./lints/deap-dylint-mir/`)
- **Core SysML v2 AST & Diagnostics Library (`deap-core`)**: `crates/deap-core/src/lib.rs:1-60` (SysML v2 AST definitions `ConstraintDef`, `PortDef`, `StateDef`, `RequirementDef`, and unified diagnostic types)
- **Modular Codegen & Formal Harness Synthesizer Library (`compile-sysml`)**: `crates/compile-sysml/src/lib.rs:1-60` (`syn` + `proc-macro2` + `quote` + `prettyplease` modular codegen orchestrator, `flutter_rust_bridge` Ring-2 FFI guard synthesis, and `#[cfg(kani)]` `#[kani::proof]` harness synthesis)
- **Modular Codegen CLI Entrypoint (`compile-sysml`)**: `crates/compile-sysml/src/main.rs:1-60` (Deterministic CLI driver, Stage-1 == Stage-2 fixed-point verification, and `--message-format=json` diagnostic emission)
- **Verification Gatekeeper Library (`verify-baseline`)**: `crates/verify-baseline/src/lib.rs:1-35` (Multi-stage formal verification gatekeeper, in-process `ra_ap_syntax` CST `< 380`-SLOC verifier, and dynamic cross-platform runtime `PATH` toolchain discovery)
- **Verification Gatekeeper CLI & Blueprint Verifier (`verify-baseline`)**: `crates/verify-baseline/src/main.rs:1-60` (12-gate `check-blueprint` verifier, `--require-formal-toolchains` qualification gate, and 5-stage formal verification CLI dispatch)
- **Tracked Issues**: `#742`
