| Attribute | Specification Detail |
| :--- | :--- |
| **Issue ID** | #649 |
| **Title** | Use Case 26: Static Five Pillar AST Defect Detection |
| **Type** | use-case |
| **Primary Actor** | Operator_Or_CI_Pipeline |
| **Scope** | ClosedLoopDefectPublisherGate |
| **Level** | User Goal |
| **Generation Mode** | subagent |
| **Specification Source** | [schema/model.sysml](../../schema/model.sysml) |

# Use Case 26: Static Five Pillar AST Defect Detection

## UML Diagrams

```mermaid
graph TD
    subgraph "ClosedLoopDefectPublisherGate System Boundary"
        UC_Main(["Static Five Pillar AST Defect Detection"])
        UC_Step01(["Operator_Or_CI_Pipeline invokes Static Five Pillar AST De..."])
        UC_Step02(["ClosedLoopDefectPublisherGate verifies boundary precondit..."])
        UC_Step03(["ClosedLoopDefectPublisherGate records verified Static Fiv..."])
    end
    Operator_Or_CI_Pipeline(("Operator_Or_CI_Pipeline")) --- UC_Main
    UC_Main -. "<<include>>" .-> UC_Step01
    UC_Main -. "<<include>>" .-> UC_Step02
    UC_Main -. "<<include>>" .-> UC_Step03
```

## 1. Actors
- `Operator_Or_CI_Pipeline`

## 2. Preconditions
- `ClosedLoopDefectPublisherGate` is initialized and ready to enforce `Static Five Pillar AST Defect Detection`.
- `Operator_Or_CI_Pipeline` holds authorized operational access to `ClosedLoopDefectPublisherGate`.

## 3. Trigger
- Static Five Pillar AST Defect Detection

## 4. Main Success Scenario
1. `Operator_Or_CI_Pipeline` invokes `VerifyBaselineAuditCodePathCrates` (`verify-baseline audit-code --path crates/`) on `DeterministicStaticDefectAuditor` with `auditor_subagent_count == 0`, transitioning `BP_06_Deterministic_Static_5_Pillar_Code_Auditor_and_Defect_Dossier_Synthesizer_Blueprint_LifecycleState` from `Idle` to `IngestingRustAndDartCst`.
2. `DeterministicStaticDefectAuditor` invokes `ParseRustCstAndParseDartCst` via `deap_core_rust_and_dart_cst` (`ra_ap_syntax`, `logos`, `rowan`, and `syn`) on `TargetRustCstInputPort` to produce a deterministic slice of `RustCstItem` nodes.
3. `FivePillarStaticAstAnalyzer` receives the parsed AST stream on `astStreamInPort` (`zeroRegexMode = true`, `activePillarCount = 5`) and transitions to `EvaluatingFivePillars`.
4. `FivePillarStaticAstAnalyzer` executes `EvaluateSourceFilePillarsAcross5` across all `pillar_coverage_count = 5` correctness risk pillars (`E0601` through `E0605`) in linear single-pass `O(N)` complexity under `Invariant_REQ_0606_SC_02`.
5. `FivePillarStaticAstAnalyzer` emits line-exact `AuditFinding` records with diagnostic code `E0601` via `findingStreamOutPort` to `DeterministicDossierSynthesizer` (or transitions to `Verified` on `zero_findings_detected`) and reports status to `Operator_Or_CI_Pipeline`.

## 5. Alternate and Exception Flows
- 3a. Unreadable source file or CST parse error on `TargetRustCstInputPort` (`Diagnostic: E0601`):
  - `DeterministicStaticDefectAuditor` transitions to `Rejected`, emits diagnostic `E0601` via `ariadne` / `miette`, and terminates fail-closed with exit code 1.
- 4a. Pillar coverage count falls below `pillar_coverage_count == 5` under `Invariant_REQ_0606_SC_01`:
  - `FivePillarStaticAstAnalyzer` returns `Result::Err`, aborts the static audit without partial state mutation, and exits fail-closed with code 1.
- 5a. Invariant violation while enforcing `Invariant_REQ_0606_SC_01` or `Invariant_REQ_0606_SC_02` in `ClosedLoopDefectPublisherGate`:
  - `ClosedLoopDefectPublisherGate` aborts execution, preserves prior verified state, and notifies `Operator_Or_CI_Pipeline` with fail-closed exit status 1.

## 6. Postconditions
- `ClosedLoopDefectPublisherGate` satisfies declared objective `Static Five Pillar AST Defect Detection` with all subsystem invariants intact.
- Verified `Static Five Pillar AST Defect Detection` completion status is recorded for `Operator_Or_CI_Pipeline`.

## 8. Realization Matrix
| Specification Item | Type | Link / Reference |
| :--- | :--- | :--- |
| **#726** | User Story | [User Story 37: Verify Baseline Audit Code Path Crates](../user-stories/US-37-VerifyBaselineAuditCodePathCrates.md) |
| **#680** | Feature | [Feature 26: [BP 06 Deterministic Static 5 Pillar Code Auditor and Defect Dossier Synthesizer Blueprint] ClosedLoopDefectPublisherGate](../features/FEAT-26-ClosedLoopDefectPublisherGate.md) |

## Source References
Use case operational flows and lifecycle activity references:
- System Architecture Model: `schema/model.sysml`
- Normative Systems Engineering Standard: ISO/IEC/IEEE 29148:2018 §6.4.2 Concept of Operations

Operational use case realization and traceability links derive from `schema/model.sysml` pursuant to ISO/IEC/IEEE 29148.
