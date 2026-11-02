| Attribute | Specification Detail |
| :--- | :--- |
| **Issue ID** | #653 |
| **Title** | Use Case 30: Fail Closed Rejection on Malformed Dossier Schema Ephemeral Paths or Payload Divergence |
| **Type** | use-case |
| **Primary Actor** | Operator_Or_CI_Pipeline |
| **Scope** | ClosedLoopDefectPublisherGate |
| **Level** | User Goal |
| **Generation Mode** | subagent |
| **Specification Source** | [schema/model.sysml](../../schema/model.sysml) |

# Use Case 30: Fail Closed Rejection on Malformed Dossier Schema Ephemeral Paths or Payload Divergence

## UML Diagrams

```mermaid
graph TD
    subgraph "ClosedLoopDefectPublisherGate System Boundary"
        UC_Main(["Fail Closed Rejection on Malformed Dossier Schema Ephemer..."])
        UC_Step01(["Operator_Or_CI_Pipeline invokes Fail Closed Rejection on ..."])
        UC_Step02(["ClosedLoopDefectPublisherGate verifies boundary precondit..."])
        UC_Step03(["ClosedLoopDefectPublisherGate records verified Fail Close..."])
    end
    Operator_Or_CI_Pipeline(("Operator_Or_CI_Pipeline")) --- UC_Main
    UC_Main -. "<<include>>" .-> UC_Step01
    UC_Main -. "<<include>>" .-> UC_Step02
    UC_Main -. "<<include>>" .-> UC_Step03
```

## 1. Actors
- `Operator_Or_CI_Pipeline`

## 2. Preconditions
- `ClosedLoopDefectPublisherGate` is initialized and ready to enforce `Fail Closed Rejection on Malformed Dossier Schema Ephemeral Paths or Payload Divergence`.
- `Operator_Or_CI_Pipeline` holds authorized operational access to `ClosedLoopDefectPublisherGate`.

## 3. Trigger
- Fail Closed Rejection on Malformed Dossier Schema Ephemeral Paths or Payload Divergence

## 4. Main Success Scenario
1. `Operator_Or_CI_Pipeline` invokes `run_check_defect` (`./target/release/verify-baseline check-defect`) on `ClosedLoopDefectPublisherGate` with a candidate dossier payload supplied on `dossierCandidateInPort`.
2. `ClosedLoopDefectPublisherGate` transitions to `Validating12Gates` and executes `Execute12CheckValidateDefectDossier` (`schemaGateCount = 12`, `dossier_schema_check_count = 12`) using `pulldown-cmark` AST validation.
3. `ClosedLoopDefectPublisherGate` inspects the candidate dossier for ephemeral `/tmp/` or machine-specific paths, missing `DossierSection` headers, shallow `Why ...? Because ...` chains (`whysChainDepth < 5`), unquoted Mermaid labels, or Unicode em dashes.
4. Upon detecting a schema violation, ephemeral path, or remote GitHub body divergence (`closed_loop_mismatch_bytes > 0` bytes), `ClosedLoopDefectPublisherGate` transitions to `Rejected` (`Diagnostic: E0605`).
5. `ClosedLoopDefectPublisherGate` emits a structured `ariadne` / `miette` diagnostic identifying the failed check among the 12 gates, blocks publication on `ClosedLoopTrackerPublishPort`, and exits fail-closed with code 1.

## 5. Alternate and Exception Flows
- 3a. Candidate dossier contains hardcoded host or `/tmp/` ephemeral paths (`Diagnostic: E0605`):
  - `ClosedLoopDefectPublisherGate` transitions to `Rejected` on `schema_or_Mermaid_check_violation`, emits diagnostic `E0605` via `miette`, and terminates fail-closed with exit code 1.
- 4a. Candidate dossier fails any of the 12 structural or Mermaid E1-E5 checks in `validate_defect_dossier`:
  - `ClosedLoopDefectPublisherGate` returns `Result::Err` without writing to `DefectDossierArtifactPort` and exits fail-closed with code 1.
- 5a. Remote GitHub issue body diverges from local `.pipeline/defects/` file after LF normalization (`closed_loop_mismatch_bytes != 0`):
  - `ClosedLoopDefectPublisherGate` transitions to `Rejected` on `remote_payload_mismatch`, preserves prior verified state, and exits fail-closed with status code 1.

## 6. Postconditions
- `ClosedLoopDefectPublisherGate` satisfies declared objective `Fail Closed Rejection on Malformed Dossier Schema Ephemeral Paths or Payload Divergence` with all subsystem invariants intact.
- Verified `Fail Closed Rejection on Malformed Dossier Schema Ephemeral Paths or Payload Divergence` completion status is recorded for `Operator_Or_CI_Pipeline`.

## 8. Realization Matrix
| Specification Item | Type | Link / Reference |
| :--- | :--- | :--- |
| **#617** | User Story | [User Story 41: run check defect](../user-stories/US-41-run_check_defect.md) |
| **#680** | Feature | [Feature 26: [BP 06 Deterministic Static 5 Pillar Code Auditor and Defect Dossier Synthesizer Blueprint] ClosedLoopDefectPublisherGate](../features/FEAT-26-ClosedLoopDefectPublisherGate.md) |

## Source References
Use case operational flows and lifecycle activity references:
- System Architecture Model: `schema/model.sysml`
- Normative Systems Engineering Standard: ISO/IEC/IEEE 29148:2018 §6.4.2 Concept of Operations

Operational use case realization and traceability links derive from `schema/model.sysml` pursuant to ISO/IEC/IEEE 29148.
