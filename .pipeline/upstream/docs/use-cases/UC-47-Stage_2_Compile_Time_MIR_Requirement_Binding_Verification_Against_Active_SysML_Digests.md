| Attribute | Specification Detail |
| :--- | :--- |
| **Issue ID** | TBD |
| **Title** | Use Case 47: Stage 2 Compile Time MIR Requirement Binding Verification Against Active SysML Digests |
| **Type** | use-case |
| **Primary Actor** | Operator |
| **Scope** | FrbIcdBridgeAndMiriUbGate |
| **Level** | User Goal |
| **Generation Mode** | subagent |
| **Specification Source** | [schema/model.sysml](../../schema/model.sysml) |

# Use Case 47: Stage 2 Compile Time MIR Requirement Binding Verification Against Active SysML Digests

## UML Diagrams

```mermaid
graph TD
    subgraph "FrbIcdBridgeAndMiriUbGate System Boundary"
        UC_Main(["Stage 2 Compile Time MIR Requirement Binding Verification..."])
        UC_Step01(["IsolatedDylintMirAllocationGate cross-checks extracted HI..."])
        UC_Step02(["Valid REQ-0609 bindings pass cleanly, whereas the danglin..."])
    end
    Operator(("Operator")) --- UC_Main
    UC_Main -. "<<include>>" .-> UC_Step01
    UC_Main -. "<<include>>" .-> UC_Step02
```

## 1. Actors
- `Operator`

## 2. Preconditions
- Rust items annotated with #[realises(\"REQ-0609\")] or /// Realises: [REQ-0609] verified against an active .digest.json compiled from .sysml, plus a negative fixture referencing a non-existent or retired identifier REQ-9999.

## 3. Trigger
- IsolatedDylintMirAllocationGate cross-checks extracted HIR/MIR DefId requirement bindings against the active SysML requirement digest inventory.

## 4. Main Success Scenario
1. IsolatedDylintMirAllocationGate cross-checks extracted HIR/MIR DefId requirement bindings against the active SysML requirement digest inventory.
2. Valid REQ-0609 bindings pass cleanly, whereas the dangling REQ-9999 annotation fails closed with non-zero exit status and single-line JSON diagnostic DEAP-E0902.

## 5. Alternate and Exception Flows
- Precondition guard failure in `FrbIcdBridgeAndMiriUbGate`:
  - `FrbIcdBridgeAndMiriUbGate` rejects the `Stage 2 Compile Time MIR Requirement Binding Verification Against Active SysML Digests` request without partial state mutation and reports diagnostic status to `Operator`.
- Invariant violation while enforcing `Stage 2 Compile Time MIR Requirement Binding Verification Against Active SysML Digests`:
  - `FrbIcdBridgeAndMiriUbGate` aborts `Stage 2 Compile Time MIR Requirement Binding Verification Against Active SysML Digests` execution, preserves prior verified state, and notifies `Operator`.

## 6. Postconditions
- Valid REQ-0609 bindings pass cleanly, whereas the dangling REQ-9999 annotation fails closed with non-zero exit status and single-line JSON diagnostic DEAP-E0902.

## 8. Realization Matrix
| Specification Item | Type | Link / Reference |
| :--- | :--- | :--- |
| **TBD** | User Story | [User Story 59: synthesize Frb Icd Facade](../user-stories/US-59-synthesizeFrbIcdFacade.md) |
| **TBD** | Feature | [Feature 46: [BP 09 Multi Stage Compiler and Formal Verification Gatekeeper] FrbIcdBridgeAndMiriUbGate](../features/FEAT-46-FrbIcdBridgeAndMiriUbGate.md) |

## Source References
Use case operational flows and lifecycle activity references:
- System Architecture Model: `schema/model.sysml`
- Normative Systems Engineering Standard: ISO/IEC/IEEE 29148:2018 §6.4.2 Concept of Operations

Operational use case realization and traceability links derive from `schema/model.sysml` pursuant to ISO/IEC/IEEE 29148.
