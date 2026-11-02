| Attribute | Specification Detail |
| :--- | :--- |
| **Issue ID** | #768 |
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
1. `Operator` invokes `--extract-rust-module` or `--apply-machine-fixes` on `ZeroTokenRustSpanCodemodEngine` (`codemod_command_in`) or `UnifiedRustcJsonDiagnosticEmitter` (`machine_fix_inout`) with a candidate `SpanSpliceEdit` transaction.
2. `ZeroTokenRustSpanCodemodEngine` parses the target `.rs` file via `LosslessRustCstAnalyzer` (`ra_ap_syntax::SourceFile::parse`) inside `< 380` exec SLOC submodules (`crates/deap-core/src/rust_cst/`) and resolves requested item identifiers and `TextRange` byte intervals.
3. `ZeroTokenRustSpanCodemodEngine` evaluates pairwise interval disjointness (`b_i <= a_j` or `b_j <= a_i`) and UTF-8 character boundary alignment across all proposed `SpanSpliceEdit` entries under `Invariant_REQ_0608_SC_02`.
4. Upon detecting a missing AST symbol, an overlapping `TextRange` pair, or a post-splice `ra_ap_syntax` syntax error in the candidate buffer, `ZeroTokenRustSpanCodemodEngine` aborts the in-memory transaction before touching the filesystem.
5. `UnifiedRustcJsonDiagnosticEmitter` invokes `format_ndjson_line` to emit a single-line `rustc`-compatible JSON diagnostic (`DEAP-E0801` missing item, `DEAP-E0802` overlapping span collision, or `DEAP-E0803` post-splice syntax error) on `ndjson_out`, leaving all workspace files bit-for-bit unchanged and exiting with code 1.

## 5. Alternate and Exception Flows
- 3a. Requested AST item identifier is absent from `RustItemInventory` (`Diagnostic: DEAP-E0801`):
  - `ZeroTokenRustSpanCodemodEngine` rejects the command, emits single-line NDJSON diagnostic `DEAP-E0801` on `codemod_error_out`, and exits fail-closed with code 1.
- 4a. Two or more `SpanSpliceEdit` intervals overlap in UTF-8 byte range (`Diagnostic: DEAP-E0802`):
  - `ZeroTokenRustSpanCodemodEngine` aborts the splice batch without mutating any file, emits `DEAP-E0802` with both colliding `TextRange` spans, and terminates fail-closed with exit code 1.
- 5a. Candidate spliced buffer introduces a `ra_ap_syntax` CST parse error (`Diagnostic: DEAP-E0803`):
  - `ZeroTokenRustSpanCodemodEngine` discards the in-memory candidate buffer, emits `DEAP-E0803` on `codemod_error_out`, preserves prior verified state, and exits with code 1.

## 6. Postconditions
- Execution fails closed with exit code 1, emits a single-line `rustc`-compatible JSON diagnostic (`DEAP-E0801`, `DEAP-E0802`, or `DEAP-E0803`), and leaves all workspace files bit-for-bit unchanged.

## 8. Realization Matrix
| Specification Item | Type | Link / Reference |
| :--- | :--- | :--- |
| **TBD** | User Story | [User Story 55: extract rust module](../user-stories/US-55-extract_rust_module.md) |
| **TBD** | Feature | [Feature 36: [BP 08 Lossless Rust CST Codemod Engine and Unified rustc Compatible JSON Diagnostics] AirGappedCargoLockGovernor](../features/FEAT-36-AirGappedCargoLockGovernor.md) |

## Source References
Use case operational flows and lifecycle activity references:
- System Architecture Model: `schema/model.sysml`
- Normative Systems Engineering Standard: ISO/IEC/IEEE 29148:2018 §6.4.2 Concept of Operations

Operational use case realization and traceability links derive from `schema/model.sysml` pursuant to ISO/IEC/IEEE 29148.
