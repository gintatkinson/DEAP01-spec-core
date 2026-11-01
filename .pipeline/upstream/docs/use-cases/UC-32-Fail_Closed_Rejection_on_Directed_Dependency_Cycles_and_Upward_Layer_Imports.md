| Attribute | Specification Detail |
| :--- | :--- |
| **Issue ID** | TBD |
| **Title** | Use Case 32: Fail Closed Rejection on Directed Dependency Cycles and Upward Layer Imports |
| **Type** | use-case |
| **Primary Actor** | Operator |
| **Scope** | FourLevelAstPartitionerAndHoister |
| **Level** | User Goal |
| **Generation Mode** | subagent |
| **Specification Source** | [schema/model.sysml](../../schema/model.sysml) |

# Use Case 32: Fail Closed Rejection on Directed Dependency Cycles and Upward Layer Imports

## UML Diagrams

```mermaid
graph TD
    subgraph "FourLevelAstPartitionerAndHoister System Boundary"
        UC_Main(["Fail Closed Rejection on Directed Dependency Cycles and U..."])
        UC_Step01(["compile-sysml --emit-modular-rust <OUT_DIR> --schema <FIX..."])
        UC_Step02(["TopologicalLayerDagVerifier aborts transactionally with e..."])
    end
    Operator(("Operator")) --- UC_Main
    UC_Main -. "<<include>>" .-> UC_Step01
    UC_Main -. "<<include>>" .-> UC_Step02
```

## 1. Actors
- `Operator`

## 2. Preconditions
- A SysML v2 PackageDef fixture containing either a directed dependency cycle across packages/classifiers or a @Layer(Domain) classifier referencing a @Layer(Presentation) or @Layer(Application) classifier.

## 3. Trigger
- compile-sysml --emit-modular-rust <OUT_DIR> --schema <FIXTURE> is executed.

## 4. Main Success Scenario
1. compile-sysml --emit-modular-rust <OUT_DIR> --schema <FIXTURE> is executed.
2. TopologicalLayerDagVerifier aborts transactionally with exit code 1, emits structured diagnostic E0702 (cycle) or E0703 (illegal upward layer dependency), and leaves <OUT_DIR> with zero generated files written.

## 5. Alternate and Exception Flows
- Precondition guard failure in `FourLevelAstPartitionerAndHoister`:
  - `FourLevelAstPartitionerAndHoister` rejects the `Fail Closed Rejection on Directed Dependency Cycles and Upward Layer Imports` request without partial state mutation and reports diagnostic status to `Operator`.
- Invariant violation while enforcing `Fail Closed Rejection on Directed Dependency Cycles and Upward Layer Imports`:
  - `FourLevelAstPartitionerAndHoister` aborts `Fail Closed Rejection on Directed Dependency Cycles and Upward Layer Imports` execution, preserves prior verified state, and notifies `Operator`.

## 6. Postconditions
- TopologicalLayerDagVerifier aborts transactionally with exit code 1, emits structured diagnostic E0702 (cycle) or E0703 (illegal upward layer dependency), and leaves <OUT_DIR> with zero generated files written.

## 8. Realization Matrix
| Specification Item | Type | Link / Reference |
| :--- | :--- | :--- |
| **TBD** | User Story | [User Story 45: verify Dag And Layers](../user-stories/US-45-verifyDagAndLayers.md) |
| **TBD** | Feature | [Feature 36: [BP 07 Self Hosting 4 Level Modular Multi File Code Generator Blueprint] FourLevelAstPartitionerAndHoister](../features/FEAT-36-FourLevelAstPartitionerAndHoister.md) |

## Source References
Use case operational flows and lifecycle activity references:
- System Architecture Model: `schema/model.sysml`
- Normative Systems Engineering Standard: ISO/IEC/IEEE 29148:2018 §6.4.2 Concept of Operations

Operational use case realization and traceability links derive from `schema/model.sysml` pursuant to ISO/IEC/IEEE 29148.
