| Attribute | Specification Detail |
| :--- | :--- |
| **Issue ID** | TBD |
| **Title** | Use Case 40: Fail Closed Rejection on Span Collisions Missing Symbols or Post Splice Syntax Errors |
| **Type** | use-case |
| **Primary Actor** | Operator |
| **Scope** | AirGappedCargoLockGovernor |
| **Level** | User Goal |
| **Generation Mode** | subagent |
| **Specification Source** | [schema/model.sysml](../../schema/model.sysml) |

# Use Case 40: Fail Closed Rejection on Span Collisions Missing Symbols or Post Splice Syntax Errors

## UML Diagrams

```mermaid
graph TD
    subgraph "AirGappedCargoLockGovernor System Boundary"
        UC_Main(["Fail Closed Rejection on Span Collisions Missing Symbols ..."])
        UC_Step01(["ZeroTokenRustSpanCodemodEngine or UnifiedRustcJsonDiagnos..."])
        UC_Step02(["Execution fails closed with exit code 1, emits a single-l..."])
    end
    Operator(("Operator")) --- UC_Main
    UC_Main -. "<<include>>" .-> UC_Step01
    UC_Main -. "<<include>>" .-> UC_Step02
```

## 1. Actors
- `Operator`

## 2. Preconditions
- An --extract-rust-module or --apply-machine-fixes invocation targeting either an overlapping pair of TextRange byte intervals, a non-existent AST item identifier, or a replacement payload that introduces a CST syntax error.

## 3. Trigger
- ZeroTokenRustSpanCodemodEngine or UnifiedRustcJsonDiagnosticEmitter evaluates the proposed SpanSpliceEdit transaction.

## 4. Main Success Scenario
1. ZeroTokenRustSpanCodemodEngine or UnifiedRustcJsonDiagnosticEmitter evaluates the proposed SpanSpliceEdit transaction.
2. Execution fails closed with exit code 1, emits a single-line rustc-compatible JSON diagnostic (DEAP-E0801 missing item, DEAP-E0802 overlapping span collision, or DEAP-E0803 post-splice syntax error), and leaves all.

## 5. Alternate and Exception Flows
- Precondition guard failure in `AirGappedCargoLockGovernor`:
  - `AirGappedCargoLockGovernor` rejects the `Fail Closed Rejection on Span Collisions Missing Symbols or Post Splice Syntax Errors` request without partial state mutation and reports diagnostic status to `Operator`.
- Invariant violation while enforcing `Fail Closed Rejection on Span Collisions Missing Symbols or Post Splice Syntax Errors`:
  - `AirGappedCargoLockGovernor` aborts `Fail Closed Rejection on Span Collisions Missing Symbols or Post Splice Syntax Errors` execution, preserves prior verified state, and notifies `Operator`.

## 6. Postconditions
- Execution fails closed with exit code 1, emits a single-line rustc-compatible JSON diagnostic (DEAP-E0801 missing item, DEAP-E0802 overlapping span collision, or DEAP-E0803 post-splice syntax error), and leaves all.

## 8. Realization Matrix
| Specification Item | Type | Link / Reference |
| :--- | :--- | :--- |
| **TBD** | User Story | [User Story 53: extract Rust Module](../user-stories/US-53-extractRustModule.md) |
| **TBD** | Feature | [Feature 41: [BP 08 Lossless Rust CST Codemod Engine and Unified rustc Compatible JSON Diagnostics] AirGappedCargoLockGovernor](../features/FEAT-41-AirGappedCargoLockGovernor.md) |

## Source References
Use case operational flows and lifecycle activity references:
- System Architecture Model: `schema/model.sysml`
- Normative Systems Engineering Standard: ISO/IEC/IEEE 29148:2018 §6.4.2 Concept of Operations

Operational use case realization and traceability links derive from `schema/model.sysml` pursuant to ISO/IEC/IEEE 29148.
