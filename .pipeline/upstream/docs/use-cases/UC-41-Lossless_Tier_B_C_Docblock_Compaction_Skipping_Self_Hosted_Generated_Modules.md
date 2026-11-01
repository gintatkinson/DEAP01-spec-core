| Attribute | Specification Detail |
| :--- | :--- |
| **Issue ID** | TBD |
| **Title** | Use Case 41: Lossless Tier B C Docblock Compaction Skipping Self Hosted Generated Modules |
| **Type** | use-case |
| **Primary Actor** | Operator |
| **Scope** | AirGappedCargoLockGovernor |
| **Level** | User Goal |
| **Generation Mode** | subagent |
| **Specification Source** | [schema/model.sysml](../../schema/model.sysml) |

# Use Case 41: Lossless Tier B C Docblock Compaction Skipping Self Hosted Generated Modules

## UML Diagrams

```mermaid
graph TD
    subgraph "AirGappedCargoLockGovernor System Boundary"
        UC_Main(["Lossless Tier B C Docblock Compaction Skipping Self Hoste..."])
        UC_Step01(["compile-sysml --compact-tier-bc <DIR> executes across the..."])
        UC_Step02(["Legacy 28-line docblocks on DocTier::RustTierBC items out..."])
    end
    Operator(("Operator")) --- UC_Main
    UC_Main -. "<<include>>" .-> UC_Step01
    UC_Main -. "<<include>>" .-> UC_Step02
```

## 1. Actors
- `Operator`

## 2. Preconditions
- A Rust directory tree containing DocTier::RustTierA public items with 7-section docstrings, DocTier::RustTierBC private/pub(crate) items carrying legacy 28-line 7-section docblocks with Realises: [REQ-*] bindings, and a crates/compile-sysml/src/modular_codegen/generated/ subtree.

## 3. Trigger
- compile-sysml --compact-tier-bc <DIR> executes across the directory tree.

## 4. Main Success Scenario
1. compile-sysml --compact-tier-bc <DIR> executes across the directory tree.
2. Legacy 28-line docblocks on DocTier::RustTierBC items outside generated/ are losslessly compacted to 2-line /// <summary> + /// Realises: [REQ-*] contracts (**C-CST-06**) without mutating executable tokens.

## 5. Alternate and Exception Flows
- Precondition guard failure in `AirGappedCargoLockGovernor`:
  - `AirGappedCargoLockGovernor` rejects the `Lossless Tier B C Docblock Compaction Skipping Self Hosted Generated Modules` request without partial state mutation and reports diagnostic status to `Operator`.
- Invariant violation while enforcing `Lossless Tier B C Docblock Compaction Skipping Self Hosted Generated Modules`:
  - `AirGappedCargoLockGovernor` aborts `Lossless Tier B C Docblock Compaction Skipping Self Hosted Generated Modules` execution, preserves prior verified state, and notifies `Operator`.

## 6. Postconditions
- Legacy 28-line docblocks on DocTier::RustTierBC items outside generated/ are losslessly compacted to 2-line /// <summary> + /// Realises: [REQ-*] contracts (**C-CST-06**) without mutating executable tokens.

## 8. Realization Matrix
| Specification Item | Type | Link / Reference |
| :--- | :--- | :--- |
| **TBD** | User Story | [User Story 54: verify Stage Fixed Point](../user-stories/US-54-verifyStageFixedPoint.md) |
| **TBD** | Feature | [Feature 41: [BP 08 Lossless Rust CST Codemod Engine and Unified rustc Compatible JSON Diagnostics] AirGappedCargoLockGovernor](../features/FEAT-41-AirGappedCargoLockGovernor.md) |

## Source References
Use case operational flows and lifecycle activity references:
- System Architecture Model: `schema/model.sysml`
- Normative Systems Engineering Standard: ISO/IEC/IEEE 29148:2018 §6.4.2 Concept of Operations

Operational use case realization and traceability links derive from `schema/model.sysml` pursuant to ISO/IEC/IEEE 29148.
