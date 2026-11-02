| Attribute | Specification Detail |
| :--- | :--- |
| **Issue ID** | #782 |
| **Title** | Use Case 54: Formatting Preserving Bidirectional CST Splicing Across Forward and Reverse Sync |
| **Type** | use-case |
| **Primary Actor** | Operator |
| **Scope** | FormattingPreservingSyncSplicer |
| **Level** | User Goal |
| **Generation Mode** | subagent |
| **Specification Source** | [schema/model.sysml](../../schema/model.sysml) |

# Use Case 54: Formatting Preserving Bidirectional CST Splicing Across Forward and Reverse Sync

## UML Diagrams

```mermaid
graph TD
    subgraph "FormattingPreservingSyncSplicer System Boundary"
        UC_Main(["Formatting Preserving Bidirectional CST Splicing Across Forward and Reverse Sync"])
        UC_Step01(["Execute apply_formatting_preserving_splices across disjoint Utf8TextRange spans"])
        UC_Step02(["Preserve 100 percent of untouched comments and whitespace and reject overlapping spans"])
    end
    Operator(("Operator")) --- UC_Main
    UC_Main -. "&lt;&lt;include&gt;&gt;" .-> UC_Step01
    UC_Main -. "&lt;&lt;include&gt;&gt;" .-> UC_Step02
```

## 1. Actors
- `Operator`

## 2. Preconditions
- An existing `.sysml` schema containing human-authored architectural comments, custom indentation, and three `part def` subsystems across the normative OMG SysML v2.0 + KerML 1.0 grammar (Contract A: zero 16-construct caps) alongside synchronized `./docs/features/` and `./docs/epics/` Markdown specifications (`pulldown-cmark` + `pulldown-cmark-to-cmark`), where only one `part def` port signature is updated during reverse sync, plus a negative splice request containing overlapping `Utf8TextRange` intervals.

## 3. Trigger
- `apply_formatting_preserving_splices` in `deap_core::sysml_cst::printer` (Contract B: `logos` + `rowan` + `salsa` + `ariadne`, `pulldown-cmark` + `pulldown-cmark-to-cmark`, `syn` + `proc-macro2` + `quote` + `prettyplease`, zero hand-rolled scanners) executes forward/reverse splice plans and evaluates the negative overlapping splice plan.

## 4. Main Success Scenario
1. `Operator` triggers forward or reverse synchronization on `FormattingPreservingSyncSplicer` (`untouched_trivia_mutation_bytes = 0`, `atomic_disjoint_splice = true`, `stage_fixed_point_mismatch_files = 0`) via `sync_delta_in`.
2. `FormattingPreservingSyncSplicer` locates the modified `part def` port declaration's exact `Utf8TextRange` in `HirSourceMap` inside `< 380` exec SLOC submodules (`./crates/deap-core/src/sysml_cst/printer.rs`), with legacy hand-rolled scanners (`./crates/deap-core/src/sysml_ast/mod.rs` `-2,400+` SLOC, `./crates/compile-sysml/src/parser.rs` `-1,100+` SLOC, `./crates/ingest-sysml/src/extractors.rs` `-900+` SLOC) retired.
3. `FormattingPreservingSyncSplicer` invokes `apply_formatting_preserving_splices` to verify pairwise-disjoint `Utf8TextRange` intervals and splices only the modified declaration's `rowan` CST subtree in reverse byte order.
4. `FormattingPreservingSyncSplicer` re-parses the spliced `.sysml` and Markdown buffers via `parse_lossless_sysml_cst` and `pulldown-cmark` to verify zero syntax errors and confirms `untouched_trivia_mutation_bytes == 0` (`C_INC_02`) across all unmodified comments, whitespace, and sibling declarations under `Invariant_REQ_0610_SC_03`.
5. `FormattingPreservingSyncSplicer` commits the spliced artifacts atomically on `spliced_artifact_out` while preserving `BP-03` bijective roundtrip and `BP-07` Stage-1 == Stage-2 fixed-point identity (`C_INC_03`).

## 5. Alternate and Exception Flows
- 3a. Negative splice plan contains overlapping or out-of-bounds `Utf8TextRange` intervals (`Diagnostic: DEAP-E1003`):
  1. `apply_formatting_preserving_splices` detects the overlap before mutating any in-memory or disk file.
  2. `FormattingPreservingSyncSplicer` emits `ariadne` and single-line NDJSON diagnostic `DEAP-E1003` on `splice_diag_out` and exits fail-closed with code 1.
- 4a. Spliced candidate buffer introduces a post-splice CST syntax error (`Diagnostic: DEAP-E1003`):
  1. `FormattingPreservingSyncSplicer` discards the candidate buffer in memory and leaves disk files untouched.
  2. `FormattingPreservingSyncSplicer` emits `DEAP-E1003` on `splice_diag_out` and terminates fail-closed with exit code 1.
- 5a. Any untouched comment or whitespace byte outside the target `Utf8TextRange` is mutated (`untouched_trivia_mutation_bytes > 0`, `C_INC_02`):
  1. `FormattingPreservingSyncSplicer` rolls back the splice transaction and preserves prior verified state.
  2. `FormattingPreservingSyncSplicer` notifies `Operator` via `splice_diag_out`.

## 6. Postconditions
- **Success Guarantee:** Only the modified declaration's CST span is spliced while 100% of untouched comments, whitespace, and sibling declarations remain byte-for-byte identical (`untouched_trivia_mutation_bytes == 0`, `C_INC_02`) and Stage-1 == Stage-2 fixed-point identity holds (`stage_fixed_point_mismatch_files == 0`, `C_INC_03`).
- **Failure Guarantee:** Overlapping `Utf8TextRange` intervals or post-splice syntax errors abort atomically with `DEAP-E1003` and zero disk mutation.

## 7. Architectural Realization (3-Layer Definition of Done)
- **Layer 1 (Domain State & Data Model):** Encapsulates `HirSourceMap`, pairwise-disjoint `Utf8TextRange` splice intervals, `rowan::GreenNode` CST subtrees across 100% of normative OMG SysML v2.0 + KerML 1.0 constructs (Contract A: zero 16-construct caps), and `pulldown-cmark` Markdown AST events.
- **Layer 2 (Logic & State Management):** Executes `apply_formatting_preserving_splices` and post-splice CST re-parse verification under `C_INC_02` and `C_INC_03` inside `< 380`-SLOC submodules (`./crates/deap-core/src/sysml_cst/printer.rs`) while explicitly deleting legacy hand-rolled scanners (`./crates/deap-core/src/sysml_ast/mod.rs` `-2,400+` SLOC, `./crates/compile-sysml/src/parser.rs` `-1,100+` SLOC, `./crates/ingest-sysml/src/extractors.rs` `-900+` SLOC).
- **Layer 3 (Presentation & Actuator Interface Binding):** Binds `sync_delta_in`, `spliced_artifact_out`, and `splice_diag_out` to atomic artifact writes and deterministic `ariadne` + `--message-format=json` NDJSON diagnostics (`DEAP-E1003`).

## 8. Realization Matrix
| Specification Item | Type | Link / Reference |
| :--- | :--- | :--- |
| **#820** | User Story | [User Story 71: apply formatting preserving splices](../user-stories/US-71-apply_formatting_preserving_splices.md) |
| **#817** | User Story | [User Story 68: parse lossless sysml cst](../user-stories/US-68-parse_lossless_sysml_cst.md) |
| **#752** | Feature | [Feature 46: [BP 10 Incremental Lossless SysML v2 and Polyglot Compiler Engine] FormattingPreservingSyncSplicer](../features/FEAT-46-FormattingPreservingSyncSplicer.md) |
| **#753** | Feature | [Feature 47: [BP 10 Incremental Lossless SysML v2 and Polyglot Compiler Engine] LogosRowanSysmlCstEngine](../features/FEAT-47-LogosRowanSysmlCstEngine.md) |

## Source References
Use case operational flows and lifecycle activity references:
- System Architecture Model: `schema/model.sysml`
- Normative Systems Engineering Standard: ISO/IEC/IEEE 29148:2018 §6.4.2 Concept of Operations

Operational use case realization and traceability links derive from `schema/model.sysml` pursuant to ISO/IEC/IEEE 29148.
