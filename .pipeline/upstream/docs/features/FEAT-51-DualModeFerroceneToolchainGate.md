| Attribute | Specification Detail |
| :--- | :--- |
| **Issue ID** | #757 |
| **Title** | Feature 51: [BP 11 Ferrocene Qualified Toolchain FLS Traceability Structural Coverage and 3 Ring Rust_Flutter DO 178 C DO 330 Certification Architecture] DualModeFerroceneToolchainGate |
| **Type** | feature |
| **Parent Epic** | [EPIC-11-BP_11_Ferrocene_Qualified_Toolchain_FLS_Traceability_Structural_Coverage_and_3_Ring_Rust_Flutter_DO_178C_DO_330_Certification_Architecture](../epics/EPIC-11-BP_11_Ferrocene_Qualified_Toolchain_FLS_Traceability_Structural_Coverage_and_3_Ring_Rust_Flutter_DO_178C_DO_330_Certification_Architecture.md) |
| **Interface Type** | Physical / Structural / Internal Bus |
| **Schema Containers** | `BP_11_Ferrocene_Qualified_Toolchain_FLS_Traceability_Structural_Coverage_and_3_Ring_Rust_Flutter_DO_178C_DO_330_Certification_Architecture::DualModeFerroceneToolchainGate` |
| **Generation Mode** | subagent |
| **Specification Source** | [schema/model.sysml](../../schema/model.sysml) |

# Feature 51: [BP 11 Ferrocene Qualified Toolchain FLS Traceability Structural Coverage and 3 Ring Rust_Flutter DO 178 C DO 330 Certification Architecture] DualModeFerroceneToolchainGate

## UML Class Diagram

```mermaid
classDiagram
    class DualModeFerroceneToolchainGate {
        +String toolchain_manifests_in (in)
        +String qualification_dossier_out (out)
        +String toolchain_diag_out (out)
        +Integer max_toolchain_version_drift
        +Integer stage_fixed_point_mismatch_files
        +Boolean reproducible_ferrocene_lock
        +Boolean require_formal_toolchains
        +String deterministic_uuid
    }
```

## Interface Requirements
### 1. Test Data Shape
```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "title": "DualModeFerroceneToolchainGatePayloadSchema",
  "type": "object",
  "properties": {
    "part": {
      "type": "string",
      "const": "DualModeFerroceneToolchainGate"
    },
    "status": {
      "type": "string",
      "enum": ["nominal", "degraded", "fault"]
    },
    "max_toolchain_version_drift": {
      "type": "integer",
      "x-sysml-type": "Integer",
      "description": "Declared attribute max_toolchain_version_drift (Integer)",
      "default": 0
    },
    "stage_fixed_point_mismatch_files": {
      "type": "integer",
      "x-sysml-type": "Integer",
      "description": "Permitted Stage-1 versus Stage-2 SHA-256 digest mismatch count under Ferrocene and standard `rustc` bootstrap (`E0705`). [unit: files]",
      "default": 0
    },
    "reproducible_ferrocene_lock": {
      "type": "boolean",
      "x-sysml-type": "Boolean",
      "description": "Enforces `--locked --offline` deterministic compilation and independent `verify-baseline` TQL-5 cross-checking. [unit: flag]",
      "default": true
    },
    "require_formal_toolchains": {
      "type": "boolean",
      "x-sysml-type": "Boolean",
      "description": "When asserted in qualification CI, fails closed if Ferrocene criticalup or toolchains are absent. [unit: flag]",
      "default": false
    },
    "deterministic_uuid": {
      "type": "string",
      "x-sysml-type": "String",
      "description": "Canonical deterministic blueprint component identifier (`DEAP-BP-DET-11`). [unit: uuidv5]",
      "default": "6ba7b810-9dad-11d1-80b4-00c04fd430cb"
    },
    "toolchain_manifests_in": {
      "type": "object",
      "properties": {
        "direction": { "type": "string", "const": "in" },
        "port_type": { "type": "string", "const": "String" }
      },
      "required": ["direction", "port_type"]
    },
    "qualification_dossier_out": {
      "type": "object",
      "properties": {
        "direction": { "type": "string", "const": "out" },
        "port_type": { "type": "string", "const": "String" }
      },
      "required": ["direction", "port_type"]
    },
    "toolchain_diag_out": {
      "type": "object",
      "properties": {
        "direction": { "type": "string", "const": "out" },
        "port_type": { "type": "string", "const": "String" }
      },
      "required": ["direction", "port_type"]
    }
  },
  "required": [
    "part",
    "status",
    "toolchain_manifests_in",
    "qualification_dossier_out",
    "toolchain_diag_out"
  ],
  "additionalProperties": false
}
```

### 2. Validation & Constraints
Declared structural attributes, boundary ports, invariants, and allocated requirements for classifier `DualModeFerroceneToolchainGate`:

| AST Member Kind | Identifier | Declared Type / Direction | Default / Invariant Contract |
| :--- | :--- | :--- | :--- |
| Attribute | `max_toolchain_version_drift` | `Integer` | Default `0` (`C_CERT_01`: `max_toolchain_version_drift >= 0 and max_toolchain_version_drift <= 0`; enforces exact `criticalup.toml` + `rust-toolchain.toml` semantic version and target-triple parity via `toml_edit` and `cargo-platform` with `DEAP-E1101`) |
| Attribute | `stage_fixed_point_mismatch_files` | `Integer` | Default `0` (`C_CERT_02`: `stage_fixed_point_mismatch_files >= 0 and stage_fixed_point_mismatch_files <= 0`; enforces Stage-1 == Stage-2 self-hosted modular codegen SHA-256 fixed-point identity with `E0705`) |
| Attribute | `reproducible_ferrocene_lock` | `Boolean` | Default `true` (enforces `--locked --offline` deterministic compilation and DO-330 TQL-1 / TQL-5 cross-checking) |
| Attribute | `require_formal_toolchains` | `Boolean` | Default `false` (when `true`, fails closed with `DEAP-E1101` if Ferrocene `criticalup` or qualified toolchains are absent across runtime `PATH` discovery probes `C_CERT_08`) |
| Attribute | `deterministic_uuid` | `String` | Default `"6ba7b810-9dad-11d1-80b4-00c04fd430cb"` (`DEAP-BP-DET-11`) |
| Port | `toolchain_manifests_in` | `in String` | Boundary interface contract ingesting `criticalup.toml` and `rust-toolchain.toml` manifests |
| Port | `qualification_dossier_out` | `out String` | Boundary interface contract emitting DO-330 TQL-1 / TQL-5 qualification receipts |
| Port | `toolchain_diag_out` | `out String` | Boundary interface contract emitting single-line NDJSON diagnostics (`DEAP-E1101`, `E0705`) |
| Constraint | `C_CERT_01` | `max_toolchain_version_drift >= 0 and max_toolchain_version_drift <= 0` | Enforces zero semantic `rustc` version or qualified target-triple drift between `criticalup.toml` and `rust-toolchain.toml` |
| Constraint | `C_CERT_02` | `stage_fixed_point_mismatch_files >= 0 and stage_fixed_point_mismatch_files <= 0` | Enforces zero Stage-1 vs Stage-2 SHA-256 codegen file mismatches under Ferrocene and upstream `rustc` |
| Constraint | `C_CERT_07` | `max_module_exec_sloc >= 1 and max_module_exec_sloc <= 380` | Bounds submodule implementation inside `< 380` executable SLOC (`crates/verify-baseline/src/ferrocene_gate/`) |
| Constraint | `C_CERT_08` | `min_toolchain_discovery_probes >= 1 and min_toolchain_discovery_probes <= 3` | Enforces dynamic cross-platform runtime `PATH` probing across macOS, Linux, and Windows (`std::env::consts::{DLL_PREFIX, DLL_SUFFIX, EXE_SUFFIX}`, `camino::Utf8Path`) |
| Requirement | `REQ-0611 (REQ_0611_BP_11_Ferrocene_Qualified_Toolchain_FLS_Traceability_Structural_Coverage_and_3_Ring_Rust_Flutter_DO_178C)` | Allocated Requirement | Enforces `Invariant_REQ_0611_SC_01` and `Invariant_REQ_0611_SC_05` under RTCA DO-178C DAL-A..E and RTCA DO-330 TQL-1 / TQL-5 |

### 3. Visual Layout & Arrangement
Structural composition and boundary topology for classifier `DualModeFerroceneToolchainGate` organize no nested subparts declared in schema paired with boundary ports `in toolchain_manifests_in : String`, `out qualification_dossier_out : String`, `out toolchain_diag_out : String`. Under Contract A, all SysML v2.0 and KerML 1.0 package, requirement, constraint, and `#dal`/`#fls` metadata constructs are ingested losslessly via `logos` + `rowan` + `salsa` + `ariadne` (`deap_core::sysml_cst`) with zero 16-construct caps.

### 4. Interactive Flow & States
Operational lifecycle events and discrete state transitions for `DualModeFerroceneToolchainGate` follow `BP_11_Ferrocene_Qualified_Toolchain_FLS_Traceability_Structural_Coverage_and_3_Ring_Rust_Flutter_DO_178C_DO_330_Certification_Architecture_LifecycleState` (`Idle -> ProbingCrossPlatformToolchain` on `probeToolchainPath`; `ProbingCrossPlatformToolchain -> VerifyingDualToolchainParity` on `toolchainDiscovered` or `RejectedFailClosed` on `rejectMissingToolchainE1101`; `VerifyingDualToolchainParity -> SynthesizingThreeRingPartitions` on `toolchainAndStage2ParityLocked` or `RejectedFailClosed` on `rejectToolchainDriftE1101`).

### Layer 1: Domain State & Signal Model
Classifier `DualModeFerroceneToolchainGate` encapsulates typed domain state attributes `max_toolchain_version_drift : Integer`, `stage_fixed_point_mismatch_files : Integer`, `reproducible_ferrocene_lock : Boolean`, `require_formal_toolchains : Boolean`, `deterministic_uuid : String` alongside operational lifecycle status and typed manifest representations parsed via `toml_edit`, `cargo-platform`, `cargo_metadata`, `cargo-lock`, and `cargo-util-schemas` (Contract B, zero hand-rolled scanners).

### Layer 2: Logic & Safety State Management
Classifier `DualModeFerroceneToolchainGate` governs deterministic structural transition evaluation (`probe_criticalup_toolchain(config)` and `verify_dual_toolchain_parity(spec)`) enforcing invariant obligations `C_CERT_01`, `C_CERT_02`, `C_CERT_07`, `C_CERT_08`, `Invariant_REQ_0611_SC_01`, and `Invariant_REQ_0611_SC_05`. Dynamic cross-platform runtime `PATH` discovery (`std::env::consts::{DLL_PREFIX, DLL_SUFFIX, EXE_SUFFIX}`, `camino::Utf8Path`, zero hardcoded OS/user/machine paths) and dual-manifest parity / Stage-1 == Stage-2 SHA-256 fixed-point verification execute inside `< 380`-SLOC submodules (`crates/verify-baseline/src/ferrocene_gate/`, `max_module_exec_sloc <= 380`, `C_CERT_07`) with explicit deletion of legacy hand-rolled line/string toolchain manifest scanners.

### Layer 3: Presentation & Actuator Interface Binding
Classifier `DualModeFerroceneToolchainGate` binds external and behavioral interfaces across port `in toolchain_manifests_in : String`, port `out qualification_dossier_out : String`, port `out toolchain_diag_out : String`, emitting DO-330 TQL-1 / TQL-5 qualification dossiers and single-line NDJSON diagnostics (`DEAP-E1101`, `E0705`).

### Associated Use Cases & User Stories

#### Associated Use Cases
- [ ] #787 - [Use Case 59: Dual Mode criticalup toml and rust toolchain toml Parity Cross Platform Discovery and DO 330 Fixed Point Qualification](../use-cases/UC-59-Dual_Mode_criticalup_toml_and_rust_toolchain_toml_Parity_Cross_Platform_Discovery_and_DO_330_Fixed_Point_Qualification.md)
- [ ] #793 - [Use Case 65: Cross Platform Execution Submodule Layout Conformance and Twelve Gate Blueprint Verification Gate](../use-cases/UC-65-Cross_Platform_Execution_Submodule_Layout_Conformance_and_Twelve_Gate_Blueprint_Verification_Gate.md)

#### Associated User Stories
- [ ] #822 - [User Story 73: probe criticalup toolchain](../user-stories/US-73-probe_criticalup_toolchain.md)
- [ ] #823 - [User Story 74: verify dual toolchain parity](../user-stories/US-74-verify_dual_toolchain_parity.md)
- [ ] #824 - [User Story 75: synthesize three ring workspace](../user-stories/US-75-synthesize_three_ring_workspace.md)
- [ ] #825 - [User Story 76: verify fls traceability chain](../user-stories/US-76-verify_fls_traceability_chain.md)
- [ ] #826 - [User Story 77: verify certified core and table a7 coverage](../user-stories/US-77-verify_certified_core_and_table_a7_coverage.md)
- [ ] #827 - [User Story 78: verify three ring edge and monitor readback](../user-stories/US-78-verify_three_ring_edge_and_monitor_readback.md)

## Acceptance Criteria (BDD)
- [ ] AC-01: Given valid `DualModeFerroceneToolchainGate` initialization inputs for `max_toolchain_version_drift` (`Integer`), `stage_fixed_point_mismatch_files` (`Integer`), `reproducible_ferrocene_lock` (`Boolean`), `require_formal_toolchains` (`Boolean`), `deterministic_uuid` (`String`) satisfying `C_CERT_01`, `C_CERT_02`, `C_CERT_07`, and `C_CERT_08` (`Invariant_REQ_0611_SC_01`, `Invariant_REQ_0611_SC_05`), When `DualModeFerroceneToolchainGate` commits typed domain state via `toml_edit` and `cargo-platform`, Then declared state attributes hold verified values with zero semantic version or target-triple drift.
- [ ] AC-02: Given malformed or out-of-domain input values for `max_toolchain_version_drift` (`Integer`), `stage_fixed_point_mismatch_files` (`Integer`), `reproducible_ferrocene_lock` (`Boolean`), `require_formal_toolchains` (`Boolean`), `deterministic_uuid` (`String`) or drift between `criticalup.toml` and `rust-toolchain.toml` (`max_toolchain_version_drift > 0` or `stage_fixed_point_mismatch_files > 0`), When `DualModeFerroceneToolchainGate` validates incoming state updates, Then `DualModeFerroceneToolchainGate` rejects the invalid state update fail-closed and emits diagnostic `DEAP-E1101` or `E0705` via diagnostic port `qualification_dossier_out` and `toolchain_diag_out`.
- [ ] AC-03: Given nominal signal arrival across `toolchain_manifests_in` (`in String`), `qualification_dossier_out` (`out String`), `toolchain_diag_out` (`out String`) and dynamic cross-platform runtime `PATH` discovery across macOS, Linux, and Windows (`min_toolchain_discovery_probes` in `1..3`), When `DualModeFerroceneToolchainGate` processes the declared interface transaction inside `< 380`-SLOC submodules (`C_CERT_07`), Then output state conforms to the structural classifier contract.
- [ ] AC-04: Given an invalid or unrecognized interface frame across `toolchain_manifests_in` (`in String`), `qualification_dossier_out` (`out String`), `toolchain_diag_out` (`out String`), or missing Ferrocene toolchain when `require_formal_toolchains == true`, When `DualModeFerroceneToolchainGate` boundary validation executes, Then `DualModeFerroceneToolchainGate` refuses the malformed frame via diagnostic port `qualification_dossier_out` and `toolchain_diag_out` (`DEAP-E1101`) and preserves prior verified state.

## Source References
Formal component specifications and normative systems engineering requirements:
- System Architecture Model: `schema/model.sysml`
- Normative Systems Engineering Standard: ISO/IEC/IEEE 15288:2023 §6.4.3 Architecture Definition Process

Component behavior and interface definitions derive from `schema/model.sysml` pursuant to ISO/IEC/IEEE 15288.
