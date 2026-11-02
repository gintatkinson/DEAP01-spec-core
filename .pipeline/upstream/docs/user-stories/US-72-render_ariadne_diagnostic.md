| Attribute | Specification Detail |
| :--- | :--- |
| **Issue ID** | #821 |
| **Title** | User Story 72: render ariadne diagnostic |
| **Type** | user-story |
| **Parent Epic** | [EPIC-10-BP_10_Incremental_Lossless_SysML_v2_and_Polyglot_Compiler_Engine](../epics/EPIC-10-BP_10_Incremental_Lossless_SysML_v2_and_Polyglot_Compiler_Engine.md) |
| **SysML Interaction** | `BP_10_Incremental_Lossless_SysML_v2_and_Polyglot_Compiler_Engine_SequenceInteraction` |
| **SysML Test Case** | `Test_render_ariadne_diagnostic` |
| **Generation Mode** | subagent |
| **Specification Source** | [schema/model.sysml](../../schema/model.sysml) |

# User Story 72: render ariadne diagnostic

## User Story
As a `TypedPolyglotIngestionFrontend`, I want to invoke `render_ariadne_diagnostic()` on `SalsaIncrementalCompilerDatabase`, so that `render_ariadne_diagnostic(input, diag)` renders deterministic ANSI-free `ariadne` UTF-8 byte-span reports paired with single-line `--message-format=json` NDJSON objects (`DEAP-E1001..E1005`) across all SysML v2, KerML 1.0, and 6 polyglot CST/AST frontends.

## Architectural Realization (3-Layer Definition of Done)

### Layer 1: Domain State & Data Model
- Domain state attributes on `SalsaIncrementalCompilerDatabase`: `query_layer_count: Integer`, `trivia_edit_downstream_recomputations: Integer`, `pure_deterministic_btreemap: Boolean`, `cycle_recovery_fail_closed: Boolean`, `Blueprint_ID: String`, `Requirement_ID: String`, `Title: String`, `Status: String`, `Target_Crates: String`, `Tracked_Issues: String`, `Generation_Mode: String`, `Specification_Source: String`, `Layer_1_Domain_State_Data_Model: String`, `Layer_2_Logic_State_Management: String`, `Layer_3_Presentation_Actuator_Interface_Binding: String`, `max_ast_depth: Real`, `untouched_trivia_mutation_bytes: Real`, `stage_fixed_point_mismatch_files: Real`, `query_layer_count: Real`, `trivia_edit_downstream_recomputations: Real`, `polyglot_frontend_count: Real`, `rustc_private_feature_count: Real`.
- Action `render_ariadne_diagnostic` invocation message envelope bound to `SalsaIncrementalCompilerDatabase` (`SysmlDiagnostic` / `Utf8TextRange`) typed domain schema and Contract A (100% normative OMG SysML v2.0 + KerML 1.0 Definition and Usage symmetry across namespaces, `#hazard`/`#risk`/`#capability`/`#rt_loop`/`#dal`/`#fls` `metadata def` plus backward-compatible legacy `hazard def`/`risk def`/`capability def`, structural, behavioral, state, calculation, constraint, requirement, use-case, verification, view, and KerML specialization/expression clauses with zero 16-construct caps).
- Declared lifecycle state model: `BP_10_Incremental_Lossless_SysML_v2_and_Polyglot_Compiler_Engine_LifecycleState`, `Idle`, `NormalizingSourceRevision`, `LexingAndBuildingRowanCst`, `EmittingAriadneAndJsonDiagnostics`, `LoweringTriviaErasedHir`, `EarlyCutoffShortCircuited`, `EvaluatingSafetyAndKaniQueries`, `SplicingFormattingPreservingSync`, `VerifiedIncrementalClean`.

### Layer 2: Logic & State Management
- Behavioral execution contract: invokes `render_ariadne_diagnostic()` on `SalsaIncrementalCompilerDatabase` (`deap_core::sysml_cst::diagnostics::render_ariadne_diagnostic`) with atomic precondition verification under Contract B (`logos` + `rowan` + `salsa` + `ariadne`, `ra_ap_syntax`, `deap_core::dart_cst`, `serde_json`, `ingest_sysml::cst::{idl_cst, yang_cst}`, and `pulldown-cmark` with zero hand-rolled string/line/char-by-char scanners).
- State transition management: `Idle -> NormalizingSourceRevision` (trigger `normalize_line_endings`) and `LexingAndBuildingRowanCst / LoweringTriviaErasedHir / SplicingFormattingPreservingSync -> EmittingAriadneAndJsonDiagnostics` (triggers `rejectMaxAstDepthOverflowE1002`, `rejectCyclicSalsaQueryE1004`, `rejectOverlappingSpliceE1003`).
- Formal constraints and requirements enforced: `C_INC_01 (max_ast_depth >= 1 and max_ast_depth <= 64)`, `C_INC_02 (untouched_trivia_mutation_bytes >= 0 and untouched_trivia_mutation_bytes <= 0)`, `C_INC_03 (stage_fixed_point_mismatch_files >= 0 and stage_fixed_point_mismatch_files <= 0)`, `C_INC_04 (query_layer_count >= 4 and query_layer_count <= 4)`, `C_INC_05 (trivia_edit_downstream_recomputations >= 0 and trivia_edit_downstream_recomputations <= 0)`, `C_INC_06 (polyglot_frontend_count >= 6 and polyglot_frontend_count <= 6)`, `C_INC_07 (rustc_private_feature_count >= 0 and rustc_private_feature_count <= 0)`, `REQ-0610 (REQ_0610_BP_10_Incremental_Lossless_SysML_v2_and_Polyglot_Compiler_Engine)`.
- Modularity & net-negative ratchet target: bounds implementation inside `< 380` exec SLOC submodules (`./crates/deap-core/src/sysml_cst/`, `./crates/compile-sysml/src/salsa_db/`, `./crates/ingest-sysml/src/cst/`, `./crates/ingest-sysml/src/polyglot_cst/`) while explicitly deleting legacy hand-rolled scanners in `./crates/deap-core/src/sysml_ast/mod.rs` (`-2,400+` SLOC), `./crates/compile-sysml/src/parser.rs` (`-1,100+` SLOC), and `./crates/ingest-sysml/src/extractors.rs` (`-900+` SLOC).

### Layer 3: Presentation & Actuator Interface Binding
- Component port bindings on `SalsaIncrementalCompilerDatabase`: `source_revision_in: String (in)`, `hir_model_out: String (out)`, `safety_verification_out: String (out)`.
- Interaction binding (`BP_10_Incremental_Lossless_SysML_v2_and_Polyglot_Compiler_Engine_SequenceInteraction`): routes `render_ariadne_diagnostic` invocation from `TypedPolyglotIngestionFrontend` to `SalsaIncrementalCompilerDatabase` and returns `normalize_line_endings`.
- Diagnostic fault surface: reports deterministic ANSI-free (`Config::with_color(false)`) `ariadne` byte-span reports and single-line `--message-format=json` NDJSON diagnostics (`DEAP-E1001..E1005`) to `TypedPolyglotIngestionFrontend` across the declared interface boundary when guard or invariant checks fail.

## UML Sequence Diagram

```mermaid
sequenceDiagram
    autonumber
    actor Operator
    participant TypedPolyglotIngestionFrontend
    participant SalsaIncrementalCompilerDatabase
    Operator->>TypedPolyglotIngestionFrontend: render_ariadne_diagnostic()
    TypedPolyglotIngestionFrontend->>SalsaIncrementalCompilerDatabase: render_ariadne_diagnostic()
    SalsaIncrementalCompilerDatabase-->>TypedPolyglotIngestionFrontend: normalize_line_endings
    TypedPolyglotIngestionFrontend-->>Operator: StatusReport
```

## Formal SysML Test Case & Verification Binding
```sysml
test case def Test_render_ariadne_diagnostic {
    subject SalsaIncrementalCompilerDatabase;
    verify requirement REQ_0610_BP_10_Incremental_Lossless_SysML_v2_and_Polyglot_Compiler_Engine;
    objective "Verify deterministic ANSI-free ariadne byte-span rendering and single-line --message-format=json NDJSON diagnostics (DEAP-E1001..E1005).";
    step render_ariadne_diagnostic;
}
```

## Acceptance Criteria (BDD)
- [ ] AC-US-72-01: Given `SalsaIncrementalCompilerDatabase` is in declared state `Idle` with domain state attributes (`query_layer_count: Integer`, `trivia_edit_downstream_recomputations: Integer`, `pure_deterministic_btreemap: Boolean`, `cycle_recovery_fail_closed: Boolean`), When `TypedPolyglotIngestionFrontend` invokes `render_ariadne_diagnostic` on `SalsaIncrementalCompilerDatabase`, Then `SalsaIncrementalCompilerDatabase` validates and binds the request against `SalsaIncrementalCompilerDatabase` domain state (`query_layer_count: Integer`, `trivia_edit_downstream_recomputations: Integer`, `pure_deterministic_btreemap: Boolean`, `cycle_recovery_fail_closed: Boolean`) inside `< 380`-SLOC submodules (`./crates/deap-core/src/sysml_cst/diagnostics.rs`) and transitions to `NormalizingSourceRevision` / `EmittingAriadneAndJsonDiagnostics`.
- [ ] AC-US-72-02: Given `SalsaIncrementalCompilerDatabase` active state management during `render_ariadne_diagnostic`, When `SalsaIncrementalCompilerDatabase` evaluates formal constraint `C_INC_01 (max_ast_depth >= 1 and max_ast_depth <= 64)`, Then `SalsaIncrementalCompilerDatabase` verifies constraint satisfaction before committing state updates.
- [ ] AC-US-72-03: Given `SalsaIncrementalCompilerDatabase` active state management during `render_ariadne_diagnostic`, When `SalsaIncrementalCompilerDatabase` evaluates formal constraint `C_INC_02 (untouched_trivia_mutation_bytes >= 0 and untouched_trivia_mutation_bytes <= 0)`, Then `SalsaIncrementalCompilerDatabase` verifies constraint satisfaction before committing state updates.
- [ ] AC-US-72-04: Given `SalsaIncrementalCompilerDatabase` active state management during `render_ariadne_diagnostic`, When `SalsaIncrementalCompilerDatabase` evaluates formal constraint `C_INC_03 (stage_fixed_point_mismatch_files >= 0 and stage_fixed_point_mismatch_files <= 0)`, Then `SalsaIncrementalCompilerDatabase` verifies constraint satisfaction before committing state updates.
- [ ] AC-US-72-05: Given `SalsaIncrementalCompilerDatabase` active state management during `render_ariadne_diagnostic`, When `SalsaIncrementalCompilerDatabase` evaluates formal constraint `C_INC_04 (query_layer_count >= 4 and query_layer_count <= 4)`, Then `SalsaIncrementalCompilerDatabase` verifies constraint satisfaction before committing state updates.
- [ ] AC-US-72-06: Given `SalsaIncrementalCompilerDatabase` active state management during `render_ariadne_diagnostic`, When `SalsaIncrementalCompilerDatabase` evaluates formal constraint `C_INC_05 (trivia_edit_downstream_recomputations >= 0 and trivia_edit_downstream_recomputations <= 0)`, Then `SalsaIncrementalCompilerDatabase` verifies constraint satisfaction before committing state updates.
- [ ] AC-US-72-07: Given `SalsaIncrementalCompilerDatabase` ingress port binding (`source_revision_in: String`), When `SalsaIncrementalCompilerDatabase` completes `render_ariadne_diagnostic`, Then `SalsaIncrementalCompilerDatabase` delivers `normalize_line_endings` across egress port (`hir_model_out: String`, `safety_verification_out: String`) to `TypedPolyglotIngestionFrontend`.
- [ ] AC-US-72-08: Given violated formal constraint `C_INC_01 (max_ast_depth >= 1 and max_ast_depth <= 64)`, When `TypedPolyglotIngestionFrontend` invokes `render_ariadne_diagnostic` on `SalsaIncrementalCompilerDatabase`, Then `SalsaIncrementalCompilerDatabase` rejects the request without partial state mutation, preserves prior verified state, and emits a deterministic ANSI-free `ariadne` + NDJSON (`DEAP-E1001..E1005`) fault diagnostic across the interface binding.

## Required Features
- [ ] #755 - [Feature 49: [BP 10 Incremental Lossless SysML v2 and Polyglot Compiler Engine] TypedPolyglotIngestionFrontend](../features/FEAT-49-TypedPolyglotIngestionFrontend.md) (emits exact byte-span `DEAP-E1005` polyglot diagnostics)
- [ ] #754 - [Feature 48: [BP 10 Incremental Lossless SysML v2 and Polyglot Compiler Engine] SalsaIncrementalCompilerDatabase](../features/FEAT-48-SalsaIncrementalCompilerDatabase.md) (routes `DEAP-E1004` cycle diagnostics and verification reports)
- [ ] #753 - [Feature 47: [BP 10 Incremental Lossless SysML v2 and Polyglot Compiler Engine] LogosRowanSysmlCstEngine](../features/FEAT-47-LogosRowanSysmlCstEngine.md) (provides `render_ariadne_diagnostic` over `Utf8TextRange` CST spans)

## Source References
Operational concept definitions and system sequence interaction models:
- System Architecture Model: `schema/model.sysml`
- Normative Systems Engineering Standard: ISO/IEC/IEEE 29148:2018 §6.4.2 Concept of Operations

Authoritative user story flows and interaction lifelines derive from `schema/model.sysml` pursuant to ISO/IEC/IEEE 29148.
