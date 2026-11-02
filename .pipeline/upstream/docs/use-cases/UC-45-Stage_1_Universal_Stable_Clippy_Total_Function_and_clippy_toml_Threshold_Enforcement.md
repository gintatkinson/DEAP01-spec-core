| Attribute | Specification Detail |
| :--- | :--- |
| **Issue ID** | #773 |
| **Title** | Use Case 45: Stage 1 Universal Stable Clippy Total Function and clippy toml Threshold Enforcement |
| **Type** | use-case |
| **Primary Actor** | Operator |
| **Scope** | FrbIcdBridgeAndMiriUbGate |
| **Level** | User Goal |
| **Generation Mode** | subagent |
| **Specification Source** | [schema/model.sysml](../../schema/model.sysml) |

# Use Case 45: Stage 1 Universal Stable Clippy Total Function and clippy toml Threshold Enforcement

## UML Diagrams

```mermaid
graph TD
    subgraph "FrbIcdBridgeAndMiriUbGate System Boundary"
        UC_Main(["Stage 1 Universal Stable Clippy Total Function and clippy..."])
        UC_Step01(["StableClippyTotalFunctionGate executes."])
    end
    Operator(("Operator")) --- UC_Main
    UC_Main -. "<<include>>" .-> UC_Step01
```

## 1. Actors
- `Operator`

## 2. Preconditions
- An upstream or generated downstream Rust workspace configured with `workspace.lints.clippy` enforcing all 14 total-function, arithmetic, cast, unsafe-doc, and complexity denials plus root `clippy.toml` (`too-many-lines-threshold = 80`, `too-many-arguments-threshold = 5`, and `disallowed-types` banning non-deterministic `HashMap` / `HashSet` and backtracking pattern-matching types), alongside a negative fixture containing unchecked `+` arithmetic, direct slice indexing, an 81-line function, a 6-argument function, or `std::collections::HashMap`.

## 3. Trigger
- StableClippyTotalFunctionGate executes.

## 4. Main Success Scenario
1. `Operator` invokes `MultiStageVerificationOrchestrator` (`verification_stage_count = 4`, `fail_closed_short_circuit = true`), which transitions `BP_09_Multi_Stage_Compiler_and_Formal_Verification_Gatekeeper_LifecycleState` to `RunningStage1Clippy` and invokes `verify_clippy_policy(policy, manifest_lints)` on `StableClippyTotalFunctionGate`.
2. `StableClippyTotalFunctionGate` loads `Cargo.toml` and `clippy.toml` on `workspace_manifest_in` inside `< 380` exec SLOC submodules (`crates/verify-baseline/src/formal_gates/`) and verifies all `denied_clippy_lint_count = 14` total-function lints, `too_many_lines_threshold <= 80` (`C_VER_01`), `too_many_arguments_threshold <= 5` (`C_VER_02`), and `disallowed_types_count >= 3`.
3. `StableClippyTotalFunctionGate` executes `cargo clippy --workspace --all-targets --message-format=json` under `Invariant_REQ_0609_SC_01` and streams NDJSON results across `clippy_ndjson_out` (`BP_09_ClippyToOrchestratorConnection`).
4. For the compliant workspace, `StableClippyTotalFunctionGate` confirms zero total-function, arithmetic, cast, or complexity violations and transitions `BP_09_Multi_Stage_Compiler_and_Formal_Verification_Gatekeeper_LifecycleState` to `RunningStage2DylintMir`.
5. For the negative fixture, `StableClippyTotalFunctionGate` intercepts the lint failure, transitions to `FailedClosedJsonDiagnostic`, short-circuits Stage 2 promotion, and emits a single-line NDJSON diagnostic (`DEAP-E0900`) on `unified_gate_ndjson_out` with exit code 1.

## 5. Alternate and Exception Flows
- 3a. Root `Cargo.toml` or `clippy.toml` omits a required total-function denial or exceeds `C_VER_01` (`80` lines) / `C_VER_02` (`5` arguments) (`Diagnostic: DEAP-E0900`):
  - `StableClippyTotalFunctionGate` transitions to `FailedClosedJsonDiagnostic`, emits single-line NDJSON diagnostic `DEAP-E0900` on `clippy_ndjson_out`, and exits fail-closed with code 1.
- 4a. Target crate introduces unchecked arithmetic, direct slice indexing, or banned `std::collections::HashMap` under `Invariant_REQ_0609_SC_01`:
  - `StableClippyTotalFunctionGate` fails closed with non-zero exit status, halts downstream Stage 2..4 execution (`fail_closed_short_circuit = true`), and streams the exact span diagnostic to `Operator`.
- 5a. Precondition or manifest parse error in `FrbIcdBridgeAndMiriUbGate`:
  - `FrbIcdBridgeAndMiriUbGate` aborts execution without partial state mutation, preserves prior verified state, and notifies `Operator`.

## 6. Postconditions
- `FrbIcdBridgeAndMiriUbGate` satisfies declared objective `Stage 1 Universal Stable Clippy Total Function and clippy toml Threshold Enforcement` with all subsystem invariants intact.
- Verified `Stage 1 Universal Stable Clippy Total Function and clippy toml Threshold Enforcement` completion status is recorded for `Operator`.

## 8. Realization Matrix
| Specification Item | Type | Link / Reference |
| :--- | :--- | :--- |
| **TBD** | User Story | [User Story 59: probe formal toolchain](../user-stories/US-59-probe_formal_toolchain.md) |
| **TBD** | Feature | [Feature 41: [BP 09 Multi Stage Compiler and Formal Verification Gatekeeper] FrbIcdBridgeAndMiriUbGate](../features/FEAT-41-FrbIcdBridgeAndMiriUbGate.md) |

## Source References
Use case operational flows and lifecycle activity references:
- System Architecture Model: `schema/model.sysml`
- Normative Systems Engineering Standard: ISO/IEC/IEEE 29148:2018 §6.4.2 Concept of Operations

Operational use case realization and traceability links derive from `schema/model.sysml` pursuant to ISO/IEC/IEEE 29148.
