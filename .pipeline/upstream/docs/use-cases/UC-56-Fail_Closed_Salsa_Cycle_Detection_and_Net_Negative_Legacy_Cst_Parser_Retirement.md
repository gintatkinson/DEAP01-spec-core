| Attribute | Specification Detail |
| :--- | :--- |
| **Issue ID** | #784 |
| **Title** | Use Case 56: Fail Closed Salsa Cycle Detection and Net Negative Legacy Cst Parser Retirement |
| **Type** | use-case |
| **Primary Actor** | Operator |
| **Scope** | FormattingPreservingSyncSplicer |
| **Level** | User Goal |
| **Generation Mode** | subagent |
| **Specification Source** | [schema/model.sysml](../../schema/model.sysml) |

# Use Case 56: Fail Closed Salsa Cycle Detection and Net Negative Legacy Cst Parser Retirement

## UML Diagrams

```mermaid
graph TD
    subgraph "FormattingPreservingSyncSplicer System Boundary"
        UC_Main(["Fail Closed Salsa Cycle Detection and Net Negative Legacy..."])
        UC_Step01(["compile_sysml::salsa_db evaluates scoped symbol resolutio..."])
        UC_Step02(["Cycle recovery intercepts the cycle deterministically wit..."])
    end
    Operator(("Operator")) --- UC_Main
    UC_Main -. "<<include>>" .-> UC_Step01
    UC_Main -. "<<include>>" .-> UC_Step02
```

## 1. Actors
- `Operator`

## 2. Preconditions
- A negative multi-file .sysml fixture introducing a cyclic import / cyclic part def :> BasePart specialization loop across two packages, alongside workspace SLOC and module-size audits.

## 3. Trigger
- compile_sysml::salsa_db evaluates scoped symbol resolution and verify-baseline audits legacy cst_parser deletion targets.

## 4. Main Success Scenario
1. `Operator` submits the multi-file `.sysml` fixture containing a cyclic `import` or cyclic `part def :> BasePart` specialization loop to `SalsaIncrementalCompilerDatabase` (`cycle_recovery_fail_closed = true`) on `source_revision_in`.
2. `SalsaIncrementalCompilerDatabase` evaluates scoped symbol and specialization resolution across `HirDatabase` inside `< 380` exec SLOC submodules (`crates/compile-sysml/src/salsa_db/`).
3. `Salsa` cycle recovery intercepts the dependency cycle deterministically without infinite recursion, stack overflow, or thread deadlock under `Invariant_REQ_0610_SC_04`.
4. `SalsaIncrementalCompilerDatabase` invokes `render_ariadne_diagnostic` and emits fail-closed diagnostic `DEAP-E1004` (`ariadne` byte-span report + single-line NDJSON) enumerating every cycle participant.
5. `verify-baseline` audits workspace modularity and confirms that legacy hand-rolled parsers in `crates/deap-core/src/sysml_ast/mod.rs` (`-2,400+` SLOC), `crates/compile-sysml/src/parser.rs` (`-1,100+` SLOC), and `crates/ingest-sysml/src/extractors.rs` (`-900+` SLOC) are retired in favor of `< 380` exec SLOC CST/Salsa submodules.

## 5. Alternate and Exception Flows
- 3a. Cyclic `import` or `part def :> BasePart` specialization loop detected in `HirDatabase` (`Diagnostic: DEAP-E1004`):
  - `SalsaIncrementalCompilerDatabase` invokes its cycle-recovery handler, emits `DEAP-E1004` via `render_ariadne_diagnostic` and NDJSON, and exits fail-closed with code 1.
- 4a. Legacy hand-rolled recursive-descent parser or line-oriented extractor remains in `deap-core`, `compile-sysml`, or `ingest-sysml`:
  - `verify-baseline` Check 34 and modularity gates fail closed with exit code 1 and report the unretired monolith.
- 5a. Invariant violation while enforcing `Invariant_REQ_0610_SC_04` in `FormattingPreservingSyncSplicer`:
  - `FormattingPreservingSyncSplicer` aborts execution without partial state mutation, preserves prior verified state, and notifies `Operator`.

## 6. Postconditions
- Cycle recovery intercepts the cycle deterministically without infinite recursion or thread deadlock and fails closed with `DEAP-E1004` (`ariadne` + NDJSON), while legacy hand-rolled parsers (`-4,400+` combined SLOC) are retired into `< 380` exec SLOC submodules.

## 8. Realization Matrix
| Specification Item | Type | Link / Reference |
| :--- | :--- | :--- |
| **TBD** | User Story | [User Story 70: evaluate hir early cutoff](../user-stories/US-70-evaluate_hir_early_cutoff.md) |
| **TBD** | Feature | [Feature 46: [BP 10 Incremental Lossless SysML v2 and Polyglot Compiler Engine] FormattingPreservingSyncSplicer](../features/FEAT-46-FormattingPreservingSyncSplicer.md) |

## Source References
Use case operational flows and lifecycle activity references:
- System Architecture Model: `schema/model.sysml`
- Normative Systems Engineering Standard: ISO/IEC/IEEE 29148:2018 §6.4.2 Concept of Operations

Operational use case realization and traceability links derive from `schema/model.sysml` pursuant to ISO/IEC/IEEE 29148.
