| Attribute | Specification Detail |
| :--- | :--- |
| **Issue ID** | #766 |
| **Title** | Use Case 38: Lossless Red Green CST Roundtrip and Exact DEAP v2 0 AST Metrics Extraction |
| **Type** | use-case |
| **Primary Actor** | Operator |
| **Scope** | AirGappedCargoLockGovernor |
| **Level** | User Goal |
| **Generation Mode** | subagent |
| **Specification Source** | [schema/model.sysml](../../schema/model.sysml) |

# Use Case 38: Lossless Red Green CST Roundtrip and Exact DEAP v2 0 AST Metrics Extraction

## UML Diagrams

```mermaid
graph TD
    subgraph "AirGappedCargoLockGovernor System Boundary"
        UC_Main(["Lossless Red Green CST Roundtrip and Exact DEAP v2 0 AST ..."])
        UC_Step01(["LosslessRustCstAnalyzer parses both fixtures into ra_ap_s..."])
        UC_Step02(["Root."])
    end
    Operator(("Operator")) --- UC_Main
    UC_Main -. "<<include>>" .-> UC_Step01
    UC_Main -. "<<include>>" .-> UC_Step02
```

## 1. Actors
- `Operator`

## 2. Preconditions
- A Rust `.rs` fixture containing outer/inner docblocks, inline and block comments, raw strings containing braces, outer `rt_loop` and `cfg(test)` modules, methods taking `&self` and `&mut self`, a function with `let mut flag: bool = false`, a 4-clause compound boolean predicate, a `pub fn` returning `Result<usize, String>`, and a `pub(crate) fn` helper (plus a second fixture with an incomplete trailing expression).

## 3. Trigger
- LosslessRustCstAnalyzer parses both fixtures into ra_ap_syntax + rowan Red/Green syntax trees and computes DEAP v2.0 metrics and DocTier classifications.

## 4. Main Success Scenario
1. `Operator` supplies the Rust `.rs` source buffer on `rust_source_in` to `LosslessRustCstAnalyzer` (`lossless_roundtrip_required = true`, `max_fn_exec_sloc = 50`, `max_fn_param_count = 5`, `max_mut_bool_flags = 0`, `max_predicate_arity = 3`).
2. `LosslessRustCstAnalyzer` parses the UTF-8 buffer via `ra_ap_syntax::SourceFile::parse` into a `rowan` Red/Green syntax tree inside `< 380` exec SLOC submodules (`crates/deap-core/src/rust_cst/`) and verifies bit-for-bit root `GreenNode` roundtrip identity under `Invariant_REQ_0608_SC_01`.
3. `LosslessRustCstAnalyzer` executes `list_rust_items` to classify top-level and `impl`-associated items into `DocTier::RustTierA` (`pub` items outside test/rt_loop) and `DocTier::RustTierBC` (`pub(crate)` and private helpers) with exact `TextRange` spans emitted on `item_inventory_out`.
4. `LosslessRustCstAnalyzer` invokes `analyze_metrics` to compute `production_exec_sloc` (excluding comments, docblocks, attributes, and `cfg(test)` modules), per-function `exec_sloc`, formal parameter counts (excluding `self`, `&self`, and `&mut self`), mutable boolean flag counts (`let mut flag: bool`), and compound boolean predicate clause arity.
5. `LosslessRustCstAnalyzer` emits the `DeapV2AstMetrics` report on `metrics_report_out` (and for the second fixture with an incomplete trailing expression, preserves 100% of tokens losslessly while recording exact syntax error byte spans).

## 5. Alternate and Exception Flows
- 3a. Root `rowan::GreenNode` byte projection diverges from the input `.rs` buffer under `Invariant_REQ_0608_SC_01`:
  - `LosslessRustCstAnalyzer` rejects the CST parse, emits structured JSON diagnostic `DEAP-E0803` via `UnifiedRustcJsonDiagnosticEmitter`, and exits fail-closed with code 1.
- 4a. Audited function violates `C_CST_01` (`exec_sloc > 50`), `C_CST_02` (`param_count > 5`), or `C_CST_03` (`mut_bool_flags > 0` or `predicate_arity > 3`):
  - `LosslessRustCstAnalyzer` records the exact `TextRange` span violation in `metrics_report_out` and signals `UnifiedRustcJsonDiagnosticEmitter` to fail closed with exit code 1.
- 5a. Precondition or syntax tree invariant violation in `AirGappedCargoLockGovernor`:
  - `AirGappedCargoLockGovernor` aborts execution without partial state mutation, preserves prior verified state, and reports diagnostic status to `Operator`.

## 6. Postconditions
- Root `rowan::GreenNode` text equals the input `.rs` buffer bit-for-bit, and `DeapV2AstMetrics` accurately reports `DocTier` classifications, `production_exec_sloc`, parameter counts, mutable bool flags, and predicate arity.

## 8. Realization Matrix
| Specification Item | Type | Link / Reference |
| :--- | :--- | :--- |
| **TBD** | User Story | [User Story 53: verify offline locked build](../user-stories/US-53-verify_offline_locked_build.md) |
| **TBD** | Feature | [Feature 36: [BP 08 Lossless Rust CST Codemod Engine and Unified rustc Compatible JSON Diagnostics] AirGappedCargoLockGovernor](../features/FEAT-36-AirGappedCargoLockGovernor.md) |

## Source References
Use case operational flows and lifecycle activity references:
- System Architecture Model: `schema/model.sysml`
- Normative Systems Engineering Standard: ISO/IEC/IEEE 29148:2018 §6.4.2 Concept of Operations

Operational use case realization and traceability links derive from `schema/model.sysml` pursuant to ISO/IEC/IEEE 29148.
