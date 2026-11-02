| Attribute | Specification Detail |
| :--- | :--- |
| **Issue ID** | #774 |
| **Title** | Use Case 46: Stage 2 Isolated Nightly Dylint MIR Zero Allocation Proof on Real Time Control Loops |
| **Type** | use-case |
| **Primary Actor** | Operator |
| **Scope** | FrbIcdBridgeAndMiriUbGate |
| **Level** | User Goal |
| **Generation Mode** | subagent |
| **Specification Source** | [schema/model.sysml](../../schema/model.sysml) |

# Use Case 46: Stage 2 Isolated Nightly Dylint MIR Zero Allocation Proof on Real Time Control Loops

## UML Diagrams

```mermaid
graph TD
    subgraph "FrbIcdBridgeAndMiriUbGate System Boundary"
        UC_Main(["Stage 2 Isolated Nightly Dylint MIR Zero Allocation Proof..."])
        UC_Step01(["IsolatedDylintMirAllocationGate traverses rustc_middle::m..."])
    end
    Operator(("Operator")) --- UC_Main
    UC_Main -. "<<include>>" .-> UC_Step01
```

## 1. Actors
- `Operator`

## 2. Preconditions
- A standalone nightly Dylint MIR lint workspace isolated from root stable `Cargo.toml`, evaluating two outer `rt_loop` control functions: one pure stack-only total function and one negative fixture that transitively invokes `Box::new`, `Vec::push` growth, `format!` heap allocation, or dynamic drop-glue deallocation through an indirect helper function.

## 3. Trigger
- IsolatedDylintMirAllocationGate traverses rustc_middle::mir::Body basic-block call and drop terminators across the outer rt_loop call graph while stable cargo check --workspace --offline --locked runs independently on the root workspace.

## 4. Main Success Scenario
1. `MultiStageVerificationOrchestrator` transitions from `RunningStage1Clippy` to `RunningStage2DylintMir` on `stage1TotalFunctionsVerified` and invokes `evaluate_stage2_dylint_mir(input)` on `IsolatedDylintMirAllocationGate` (`max_rt_heap_allocations = 0`, `isolated_nightly_workspace = true`).
2. `IsolatedDylintMirAllocationGate` verifies via `verify_in_process_cst_syntax(module_path, max_exec_sloc)` that the isolated nightly plugin workspace is decoupled from root stable `Cargo.toml` and bounded within `< 380` exec SLOC (`C_VER_07`).
3. `IsolatedDylintMirAllocationGate` receives lowered `rustc_middle::mir::Body` basic-block graphs on `mir_body_graph_in` and computes the transitive call and drop terminator closure reachable from every outer `rt_loop` entrypoint under `Invariant_REQ_0609_SC_02`.
4. For the pure stack-only control function, `IsolatedDylintMirAllocationGate` confirms `max_rt_heap_allocations == 0` (`C_VER_03`) while stable `cargo check --workspace --offline --locked` succeeds independently on the root workspace, transitioning to `RunningStage3FrbAndMiri`.
5. For the negative fixture, `IsolatedDylintMirAllocationGate` detects the transitive `Box::new` / `Vec::push` / `format!` / drop-glue call chain, transitions to `FailedClosedJsonDiagnostic`, and emits single-line JSON diagnostic `DEAP-E0901` on `dylint_ndjson_out` with exit code 1.

## 5. Alternate and Exception Flows
- 3a. Transitive heap allocation (`__rust_alloc`, `exchange_malloc`, `Vec::push`, `format!`) or dynamic drop terminator reachable from an outer `rt_loop` function (`Diagnostic: DEAP-E0901`):
  - `IsolatedDylintMirAllocationGate` transitions to `FailedClosedJsonDiagnostic`, emits single-line NDJSON diagnostic `DEAP-E0901` on `dylint_ndjson_out` with the full call chain, and exits fail-closed with code 1.
- 4a. Nightly `rustc_private` dependency leaks into root stable `Cargo.toml` (`isolated_nightly_workspace == false`):
  - `verify_in_process_cst_syntax` rejects the manifest configuration, emits `DEAP-E0901`, and terminates fail-closed with exit code 1.
- 5a. Invariant violation while enforcing `C_VER_03` or `C_VER_07` (`max_module_exec_sloc <= 380`):
  - `FrbIcdBridgeAndMiriUbGate` aborts execution without partial state mutation, preserves prior verified state, and notifies `Operator`.

## 6. Postconditions
- `FrbIcdBridgeAndMiriUbGate` satisfies declared objective `Stage 2 Isolated Nightly Dylint MIR Zero Allocation Proof on Real Time Control Loops` with all subsystem invariants intact.
- Verified `Stage 2 Isolated Nightly Dylint MIR Zero Allocation Proof on Real Time Control Loops` completion status is recorded for `Operator`.

## 8. Realization Matrix
| Specification Item | Type | Link / Reference |
| :--- | :--- | :--- |
| **TBD** | User Story | [User Story 60: verify clippy policy](../user-stories/US-60-verify_clippy_policy.md) |
| **TBD** | Feature | [Feature 41: [BP 09 Multi Stage Compiler and Formal Verification Gatekeeper] FrbIcdBridgeAndMiriUbGate](../features/FEAT-41-FrbIcdBridgeAndMiriUbGate.md) |

## Source References
Use case operational flows and lifecycle activity references:
- System Architecture Model: `schema/model.sysml`
- Normative Systems Engineering Standard: ISO/IEC/IEEE 29148:2018 §6.4.2 Concept of Operations

Operational use case realization and traceability links derive from `schema/model.sysml` pursuant to ISO/IEC/IEEE 29148.
