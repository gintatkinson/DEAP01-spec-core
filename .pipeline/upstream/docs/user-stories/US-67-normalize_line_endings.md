| Attribute | Specification Detail |
| :--- | :--- |
| **Issue ID** | #817 |
| **Title** | User Story 67: normalize line endings |
| **Type** | user-story |
| **Parent Epic** | [EPIC-10-BP_10_Incremental_Lossless_SysML_v2_and_Polyglot_Compiler_Engine](../epics/EPIC-10-BP_10_Incremental_Lossless_SysML_v2_and_Polyglot_Compiler_Engine.md) |
| **SysML Interaction** | `BP_10_Incremental_Lossless_SysML_v2_and_Polyglot_Compiler_Engine_SequenceInteraction` |
| **SysML Test Case** | `Test_normalize_line_endings` |
| **Generation Mode** | subagent |
| **Specification Source** | [schema/model.sysml](../../schema/model.sysml) |

# User Story 67: normalize line endings

## User Story
As a `TypedPolyglotIngestionFrontend`, I want to invoke `normalize_line_endings()` on `SalsaIncrementalCompilerDatabase`, so that `normalize_line_endings(raw)` deterministically normalizes `\r\n` (CRLF) to canonical `\n` (LF) across macOS, Linux, and Windows (`camino::Utf8Path`) prior to `logos` DFA lexing and `rowan::GreenNode` construction across 100% of normative OMG SysML v2.0 + KerML 1.0 grammar constructs.

## Architectural Realization (3-Layer Definition of Done)

### Layer 1: Domain State & Data Model
- Domain state attributes on `SalsaIncrementalCompilerDatabase`: `query_layer_count: Integer`, `trivia_edit_downstream_recomputations: Integer`, `pure_deterministic_btreemap: Boolean`, `cycle_recovery_fail_closed: Boolean`, `Blueprint_ID: String`, `Requirement_ID: String`, `Title: String`, `Status: String`, `Target_Crates: String`, `Tracked_Issues: String`, `Generation_Mode: String`, `Specification_Source: String`, `Layer_1_Domain_State_Data_Model: String`, `Layer_2_Logic_State_Management: String`, `Layer_3_Presentation_Actuator_Interface_Binding: String`, `max_ast_depth: Real`, `untouched_trivia_mutation_bytes: Real`, `stage_fixed_point_mismatch_files: Real`, `query_layer_count: Real`, `trivia_edit_downstream_recomputations: Real`, `polyglot_frontend_count: Real`, `rustc_private_feature_count: Real`.
- Action `normalize_line_endings` invocation message envelope bound to `SalsaIncrementalCompilerDatabase` (`SourceDatabase`) typed domain schema and Contract A (100% normative OMG SysML v2.0 + KerML 1.0 Definition and Usage symmetry across namespaces, `#hazard`/`#risk`/`#capability`/`#rt_loop`/`#dal`/`#fls` `metadata def` and backward-compatible legacy `hazard def`/`risk def`/`capability def`, structural, behavioral, state, calculation, constraint, requirement, use-case, verification, view, and KerML specialization/expression clauses with zero 16-construct caps).
- Declared lifecycle state model: `BP_10_Incremental_Lossless_SysML_v2_and_Polyglot_Compiler_Engine_LifecycleState`, `Idle`, `NormalizingSourceRevision`, `LexingAndBuildingRowanCst`, `EmittingAriadneAndJsonDiagnostics`, `LoweringTriviaErasedHir`, `EarlyCutoffShortCircuited`, `EvaluatingSafetyAndKaniQueries`, `SplicingFormattingPreservingSync`, `VerifiedIncrementalClean`.

### Layer 2: Logic & State Management
- Behavioral execution contract: invokes `normalize_line_endings()` on `SalsaIncrementalCompilerDatabase` (`deap_core::sysml_cst::normalize_line_endings`) with atomic precondition verification under Contract B (`logos` + `rowan` + `salsa` + `ariadne`, `ra_ap_syntax`, `deap_core::dart_cst`, `serde_json`, `ingest_sysml::cst::{idl_cst, yang_cst}`, `pulldown-cmark`, and `camino::Utf8Path` with zero hand-rolled string/line/char-by-char scanners).
- State transition management: `Idle -> NormalizingSourceRevision` (trigger `normalize_line_endings`).
- Formal constraints and requirements enforced: `C_INC_01 (max_ast_depth >= 1 and max_ast_depth <= 64)`, `C_INC_02 (untouched_trivia_mutation_bytes >= 0 and untouched_trivia_mutation_bytes <= 0)`, `C_INC_03 (stage_fixed_point_mismatch_files >= 0 and stage_fixed_point_mismatch_files <= 0)`, `C_INC_04 (query_layer_count >= 4 and query_layer_count <= 4)`, `C_INC_05 (trivia_edit_downstream_recomputations >= 0 and trivia_edit_downstream_recomputations <= 0)`, `C_INC_06 (polyglot_frontend_count >= 6 and polyglot_frontend_count <= 6)`, `C_INC_07 (rustc_private_feature_count >= 0 and rustc_private_feature_count <= 0)`, `REQ-0610 (REQ_0610_BP_10_Incremental_Lossless_SysML_v2_and_Polyglot_Compiler_Engine)`.
- Modularity & net-negative ratchet target: bounds implementation inside `< 380` exec SLOC submodules (`./crates/deap-core/src/sysml_cst/`, `./crates/compile-sysml/src/salsa_db/`, `./crates/ingest-sysml/src/cst/`, `./crates/ingest-sysml/src/polyglot_cst/`) while explicitly deleting legacy hand-rolled scanners in `./crates/deap-core/src/sysml_ast/mod.rs` (`-2,400+` SLOC), `./crates/compile-sysml/src/parser.rs` (`-1,100+` SLOC), and `./crates/ingest-sysml/src/extractors.rs` (`-900+` SLOC).

### Layer 3: Presentation & Actuator Interface Binding
- Component port bindings on `SalsaIncrementalCompilerDatabase`: `source_revision_in: String (in)`, `hir_model_out: String (out)`, `safety_verification_out: String (out)`.
- Interaction binding (`BP_10_Incremental_Lossless_SysML_v2_and_Polyglot_Compiler_Engine_SequenceInteraction`): routes `normalize_line_endings` invocation from `TypedPolyglotIngestionFrontend` to `SalsaIncrementalCompilerDatabase` and returns `normalize_line_endings`.
- Diagnostic fault surface: reports deterministic ANSI-free `ariadne` byte-span diagnostics and single-line `--message-format=json` NDJSON error reports (`DEAP-E1001`) to `TypedPolyglotIngestionFrontend` across the declared interface boundary when guard or invariant checks fail.

## UML Sequence Diagram

```mermaid
sequenceDiagram
    autonumber
    actor Operator
    participant TypedPolyglotIngestionFrontend
    participant SalsaIncrementalCompilerDatabase
    Operator->>TypedPolyglotIngestionFrontend: normalize_line_endings()
    TypedPolyglotIngestionFrontend->>SalsaIncrementalCompilerDatabase: normalize_line_endings()
    SalsaIncrementalCompilerDatabase-->>TypedPolyglotIngestionFrontend: normalize_line_endings
    TypedPolyglotIngestionFrontend-->>Operator: StatusReport
```

## Formal SysML Test Case & Verification Binding
```sysml
test case def Test_normalize_line_endings {
    subject SalsaIncrementalCompilerDatabase;
    verify requirement REQ_0610_BP_10_Incremental_Lossless_SysML_v2_and_Polyglot_Compiler_Engine;
    objective "Verify cross-platform CRLF-to-LF source normalization prior to logos + rowan CST construction across macOS, Linux, and Windows.";
    step normalize_line_endings;
}
```

## Acceptance Criteria (BDD)
- [ ] AC-US-67-01: Given `SalsaIncrementalCompilerDatabase` is in declared state `Idle` with domain state attributes (`query_layer_count: Integer`, `trivia_edit_downstream_recomputations: Integer`, `pure_deterministic_btreemap: Boolean`, `cycle_recovery_fail_closed: Boolean`), When `TypedPolyglotIngestionFrontend` invokes `normalize_line_endings` on `SalsaIncrementalCompilerDatabase`, Then `SalsaIncrementalCompilerDatabase` validates and binds the request against `SalsaIncrementalCompilerDatabase` domain state (`query_layer_count: Integer`, `trivia_edit_downstream_recomputations: Integer`, `pure_deterministic_btreemap: Boolean`, `cycle_recovery_fail_closed: Boolean`), normalizes CRLF to canonical LF inside `< 380`-SLOC submodules (`./crates/deap-core/src/sysml_cst/`), and transitions to `NormalizingSourceRevision`.
- [ ] AC-US-67-02: Given `SalsaIncrementalCompilerDatabase` active state management during `normalize_line_endings`, When `SalsaIncrementalCompilerDatabase` evaluates formal constraint `C_INC_01 (max_ast_depth >= 1 and max_ast_depth <= 64)`, Then `SalsaIncrementalCompilerDatabase` verifies constraint satisfaction before committing state updates.
- [ ] AC-US-67-03: Given `SalsaIncrementalCompilerDatabase` active state management during `normalize_line_endings`, When `SalsaIncrementalCompilerDatabase` evaluates formal constraint `C_INC_02 (untouched_trivia_mutation_bytes >= 0 and untouched_trivia_mutation_bytes <= 0)`, Then `SalsaIncrementalCompilerDatabase` verifies constraint satisfaction before committing state updates.
- [ ] AC-US-67-04: Given `SalsaIncrementalCompilerDatabase` active state management during `normalize_line_endings`, When `SalsaIncrementalCompilerDatabase` evaluates formal constraint `C_INC_03 (stage_fixed_point_mismatch_files >= 0 and stage_fixed_point_mismatch_files <= 0)`, Then `SalsaIncrementalCompilerDatabase` verifies constraint satisfaction before committing state updates.
- [ ] AC-US-67-05: Given `SalsaIncrementalCompilerDatabase` active state management during `normalize_line_endings`, When `SalsaIncrementalCompilerDatabase` evaluates formal constraint `C_INC_04 (query_layer_count >= 4 and query_layer_count <= 4)`, Then `SalsaIncrementalCompilerDatabase` verifies constraint satisfaction before committing state updates.
- [ ] AC-US-67-06: Given `SalsaIncrementalCompilerDatabase` active state management during `normalize_line_endings`, When `SalsaIncrementalCompilerDatabase` evaluates formal constraint `C_INC_05 (trivia_edit_downstream_recomputations >= 0 and trivia_edit_downstream_recomputations <= 0)`, Then `SalsaIncrementalCompilerDatabase` verifies constraint satisfaction before committing state updates.
- [ ] AC-US-67-07: Given `SalsaIncrementalCompilerDatabase` ingress port binding (`source_revision_in: String`), When `SalsaIncrementalCompilerDatabase` completes `normalize_line_endings`, Then `SalsaIncrementalCompilerDatabase` delivers `normalize_line_endings` across egress port (`hir_model_out: String`, `safety_verification_out: String`) to `TypedPolyglotIngestionFrontend`.
- [ ] AC-US-67-08: Given violated formal constraint `C_INC_01 (max_ast_depth >= 1 and max_ast_depth <= 64)`, When `TypedPolyglotIngestionFrontend` invokes `normalize_line_endings` on `SalsaIncrementalCompilerDatabase`, Then `SalsaIncrementalCompilerDatabase` rejects the request without partial state mutation, preserves prior verified state, and emits a deterministic fault diagnostic across the interface binding.

## Required Features
- [ ] #847 - [Feature 49: [BP 10 Incremental Lossless SysML v2 and Polyglot Compiler Engine] TypedPolyglotIngestionFrontend](../features/FEAT-49-TypedPolyglotIngestionFrontend.md) (supplies normalized polyglot/SysML source buffers)
- [ ] #846 - [Feature 48: [BP 10 Incremental Lossless SysML v2 and Polyglot Compiler Engine] SalsaIncrementalCompilerDatabase](../features/FEAT-48-SalsaIncrementalCompilerDatabase.md) (hosts `SourceDatabase` input revisions)
- [ ] #845 - [Feature 47: [BP 10 Incremental Lossless SysML v2 and Polyglot Compiler Engine] LogosRowanSysmlCstEngine](../features/FEAT-47-LogosRowanSysmlCstEngine.md) (executes `normalize_line_endings` prior to `logos` DFA lexing)

## Source References
Operational concept definitions and system sequence interaction models:
- System Architecture Model: `schema/model.sysml`
- Normative Systems Engineering Standard: ISO/IEC/IEEE 29148:2018 §6.4.2 Concept of Operations

Authoritative user story flows and interaction lifelines derive from `schema/model.sysml` pursuant to ISO/IEC/IEEE 29148.
