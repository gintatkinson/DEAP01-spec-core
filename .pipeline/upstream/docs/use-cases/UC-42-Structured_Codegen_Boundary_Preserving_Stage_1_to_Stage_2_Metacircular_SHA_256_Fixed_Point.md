| Attribute | Specification Detail |
| :--- | :--- |
| **Issue ID** | TBD |
| **Title** | Use Case 42: Structured Codegen Boundary Preserving Stage 1 to Stage 2 Metacircular SHA 256 Fixed Point |
| **Type** | use-case |
| **Primary Actor** | Operator |
| **Scope** | AirGappedCargoLockGovernor |
| **Level** | User Goal |
| **Generation Mode** | subagent |
| **Specification Source** | [schema/model.sysml](../../schema/model.sysml) |

# Use Case 42: Structured Codegen Boundary Preserving Stage 1 to Stage 2 Metacircular SHA 256 Fixed Point

## UML Diagrams

```mermaid
graph TD
    subgraph "AirGappedCargoLockGovernor System Boundary"
        UC_Main(["Structured Codegen Boundary Preserving Stage 1 to Stage 2..."])
        UC_Step01(["Stage-1 generated sources under crates/compile-sysml/src/..."])
        UC_Step02(["SHA-256(Stage 1) == SHA-256(Stage 2) holds across all gen..."])
    end
    Operator(("Operator")) --- UC_Main
    UC_Main -. "<<include>>" .-> UC_Step01
    UC_Main -. "<<include>>" .-> UC_Step02
```

## 1. Actors
- `Operator`

## 2. Preconditions
- The self-hosting modular codegen schema .pipeline/upstream/schema/modular_codegen.sysml lowered through syn + proc-macro2 + quote + prettyplease inside crates/compile-sysml/src/modular_codegen/.

## 3. Trigger
- Stage-1 generated sources under crates/compile-sysml/src/modular_codegen/generated/ are re-emitted into an isolated Stage-2 directory and verified for SHA-256 tree parity.

## 4. Main Success Scenario
1. Stage-1 generated sources under crates/compile-sysml/src/modular_codegen/generated/ are re-emitted into an isolated Stage-2 directory and verified for SHA-256 tree parity.
2. SHA-256(Stage 1) == SHA-256(Stage 2) holds across all generated relative paths (**C-CST-04** stage_fixed_point_mismatch_files == 0), whereas any injected formatting or AST divergence fails closed.

## 5. Alternate and Exception Flows
- Precondition guard failure in `AirGappedCargoLockGovernor`:
  - `AirGappedCargoLockGovernor` rejects the `Structured Codegen Boundary Preserving Stage 1 to Stage 2 Metacircular SHA 256 Fixed Point` request without partial state mutation and reports diagnostic status to `Operator`.
- Invariant violation while enforcing `Structured Codegen Boundary Preserving Stage 1 to Stage 2 Metacircular SHA 256 Fixed Point`:
  - `AirGappedCargoLockGovernor` aborts `Structured Codegen Boundary Preserving Stage 1 to Stage 2 Metacircular SHA 256 Fixed Point` execution, preserves prior verified state, and notifies `Operator`.

## 6. Postconditions
- SHA-256(Stage 1) == SHA-256(Stage 2) holds across all generated relative paths (**C-CST-04** stage_fixed_point_mismatch_files == 0), whereas any injected formatting or AST divergence fails closed.

## 8. Realization Matrix
| Specification Item | Type | Link / Reference |
| :--- | :--- | :--- |
| **TBD** | User Story | [User Story 55: format Ndjson Line](../user-stories/US-55-formatNdjsonLine.md) |
| **TBD** | Feature | [Feature 41: [BP 08 Lossless Rust CST Codemod Engine and Unified rustc Compatible JSON Diagnostics] AirGappedCargoLockGovernor](../features/FEAT-41-AirGappedCargoLockGovernor.md) |

## Source References
Use case operational flows and lifecycle activity references:
- System Architecture Model: `schema/model.sysml`
- Normative Systems Engineering Standard: ISO/IEC/IEEE 29148:2018 §6.4.2 Concept of Operations

Operational use case realization and traceability links derive from `schema/model.sysml` pursuant to ISO/IEC/IEEE 29148.
