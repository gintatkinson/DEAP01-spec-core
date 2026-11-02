| Attribute | Specification Detail |
| :--- | :--- |
| **Issue ID** | #719 |
| **Title** | User Story 30: Merge Extracted Entities Into Owning Subpackages |
| **Type** | user-story |
| **Parent Epic** | [EPIC-04-BP_04_Deterministic_Specification_ICD_and_WBS_Forward_Compiler_Blueprint](../epics/EPIC-04-BP_04_Deterministic_Specification_ICD_and_WBS_Forward_Compiler_Blueprint.md) |
| **SysML Interaction** | `BP_04_Deterministic_Specification_ICD_and_WBS_Forward_Compiler_Blueprint_SequenceInteraction` |
| **SysML Test Case** | `Test_MergeExtractedEntitiesIntoOwningSubpackages` |
| **Generation Mode** | subagent |
| **Specification Source** | [schema/model.sysml](../../schema/model.sysml) |

# User Story 30: Merge Extracted Entities Into Owning Subpackages

## User Story

As a `Operator_Or_CI_Runner`, I want to invoke `MergeExtractedEntitiesIntoOwningSubpackages()` on `BidirectionalBacklogReconciler`, so that all entities extracted from `./docs/` via `pulldown-cmark` AST streams are merged into their owning OMG SysML v2.0 + KerML 1.0 `PackageDef` subpackages via `logos` + `rowan` + `salsa` with zero root AST namespace pollution (`forward_reverse_drift_count == 0`) and zero hand-rolled scanners.

## Architectural Realization (3-Layer Definition of Done)

### Layer 1: Domain State & Data Model
- Domain state attributes on `BidirectionalBacklogReconciler` (`trackHtmlCommentState: Boolean`, `parentInclusiveEpics: Boolean`) and `DeterministicSpecIcdWbsForwardCompiler` (`forward_reverse_drift_count: Integer`).
- **Contract A (Full OMG SysML v2.0 + KerML 1.0 Grammar Coverage -- Zero 16-Construct Caps)**: Action `MergeExtractedEntitiesIntoOwningSubpackages` message envelope binds to the complete OMG SysML v2.0 + KerML 1.0 `package`/`library package`/`standard library package`, `part def`/`part`, `item def`/`item`, `attribute def`/`attribute`, `enum def`/`enum`, `port def`/`port` (`in`, `out`, `inout`, `~`), `connection def`/`connection`, `flow def`/`flow`, `interface def`/`interface`, `allocation def`/`allocation`, `occurrence def`/`occurrence`, `action def`/`action`, `state def`/`state`, `calc def`/`calc`, `constraint def`/`constraint`, `requirement def`/`requirement`, `use case def`/`use case`, `interaction def`/`interaction`, `verification def`/`verification`, `test case def`/`test case`, and `#capability`/`#hazard`/`#risk`/`#dal`/`#fls` CST/AST (`deap_core::sysml_cst`, `logos` + `rowan` + `salsa`) with zero 16-construct caps.
- **Contract B (`pulldown-cmark` + `pulldown-cmark-to-cmark` Lossless Markdown AST + `toml_edit` + `salsa` Across All 6 Crates -- Zero Hand-Rolled Scanners)**: Resolves `Schema Containers` and `Package` metadata via `pulldown-cmark` AST tables and performs format-preserving `rowan` CST splicing into owning `PackageDef` subpackages across `deap-core`, `compile-sysml`, `ingest-sysml`, `reconcile-backlog`, `verify-baseline`, and `assemble-conops`.
- Declared lifecycle state model: `BP_04_Deterministic_Specification_ICD_and_WBS_Forward_Compiler_Blueprint_LifecycleState` (`AstLoaded`, `ForwardProjected`, `IcdSynthesized`, `WbsExported`, `BacklogReconciled`, `ReverseSynced`, `Rejected`, `VerifiedRoundtrip`).

### Layer 2: Logic & State Management
- Behavioral execution contract: invokes `MergeExtractedEntitiesIntoOwningSubpackages()` on `BidirectionalBacklogReconciler` with atomic subpackage scope verification, rejecting any orphaned entity or root AST namespace pollution fail-closed (`E0400`).
- State transition management: `AstLoaded -> ForwardProjected` (trigger `forwardSyncSpecs`), `BacklogReconciled -> ReverseSynced` (trigger `reverseSyncToSubpackages`), and `ReverseSynced -> Rejected` (trigger `rootAstPollutionOrDriftDetected`).
- Formal constraints and requirements enforced: `Invariant_REQ_0604_SC_01`, `Invariant_REQ_0604_SC_02`, and `REQ-0604 (REQ_0604_BP_04_Deterministic_Specification_ICD_and_WBS_Forward_Compiler_Blueprint)`.
- Modularity & explicit legacy scanner deletion target: bounds implementation inside `< 380` executable SLOC submodules (`./crates/compile-sysml/src/forward_sync/`, `./crates/compile-sysml/src/icd_wbs/`, `./crates/reconcile-backlog/src/table_cst/`) while enforcing explicit deletion of legacy hand-rolled subpackage/Markdown string scanners (`-1,100+` SLOC net-negative ratchet).

### Layer 3: Presentation & Actuator Interface Binding
- Component port bindings on `BidirectionalBacklogReconciler`: `specDocsInOutPort: String (inout)`, `subpackageSyncOutPort: String (out)`.
- Interaction binding (`BP_04_Deterministic_Specification_ICD_and_WBS_Forward_Compiler_Blueprint_SequenceInteraction`): routes `MergeExtractedEntitiesIntoOwningSubpackages` invocation from `Operator_Or_CI_Runner` to `BidirectionalBacklogReconciler` and returns `ResolveParentEpicByPkgName`.
- Diagnostic fault surface: reports deterministic `ariadne` / `miette` rejection diagnostics (`E0400`) to `Operator_Or_CI_Runner` across the declared interface boundary when guard or invariant checks fail.

## UML Sequence Diagram

```mermaid
sequenceDiagram
    autonumber
    actor Operator_Or_CI_Runner
    participant AgileSpecForwardProjector
    participant BidirectionalBacklogReconciler
    Operator_Or_CI_Runner->>AgileSpecForwardProjector: MergeExtractedEntitiesIntoOwningSubpackages()
    AgileSpecForwardProjector->>BidirectionalBacklogReconciler: MergeExtractedEntitiesIntoOwningSubpackages()
    BidirectionalBacklogReconciler-->>AgileSpecForwardProjector: ResolveParentEpicByPkgName
    AgileSpecForwardProjector-->>Operator_Or_CI_Runner: StatusReport
```

## Formal SysML Test Case & Verification Binding
```sysml
test case def Test_MergeExtractedEntitiesIntoOwningSubpackages {
    subject BidirectionalBacklogReconciler;
    verify requirement REQ_0604_BP_04_Deterministic_Specification_ICD_and_WBS_Forward_Compiler_Blueprint;
    objective "Verify subpackage-scoped entity merging via rowan and salsa with zero root AST namespace pollution";
}
```

## Acceptance Criteria (BDD)
- [ ] AC-US-30-01: Given `BidirectionalBacklogReconciler` is in declared state `AstLoaded` with domain state attributes (`trackHtmlCommentState: Boolean`, `parentInclusiveEpics: Boolean`), When `Operator_Or_CI_Runner` invokes `MergeExtractedEntitiesIntoOwningSubpackages` on `BidirectionalBacklogReconciler`, Then `BidirectionalBacklogReconciler` validates and binds the request against `BidirectionalBacklogReconciler` domain state (`trackHtmlCommentState: Boolean`, `parentInclusiveEpics: Boolean`) and transitions to `ForwardProjected`.
- [ ] AC-US-30-02: Given `BidirectionalBacklogReconciler` active state management during `MergeExtractedEntitiesIntoOwningSubpackages`, When `BidirectionalBacklogReconciler` evaluates formal constraint `Invariant_REQ_0604_SC_01`, Then `BidirectionalBacklogReconciler` verifies constraint satisfaction before committing state updates.
- [ ] AC-US-30-03: Given `BidirectionalBacklogReconciler` active state management during `MergeExtractedEntitiesIntoOwningSubpackages`, When `BidirectionalBacklogReconciler` evaluates formal constraint `Invariant_REQ_0604_SC_02`, Then `BidirectionalBacklogReconciler` verifies constraint satisfaction (`forward_reverse_drift_count == 0`) before committing state updates.
- [ ] AC-US-30-04: Given `BidirectionalBacklogReconciler` requirement allocation for `MergeExtractedEntitiesIntoOwningSubpackages`, When `BidirectionalBacklogReconciler` verifies formal requirement `REQ-0604 (REQ_0604_BP_04_Deterministic_Specification_ICD_and_WBS_Forward_Compiler_Blueprint)` across the full OMG SysML v2.0 + KerML 1.0 grammar (Contract A, zero 16-construct caps) and `pulldown-cmark` + `pulldown-cmark-to-cmark` + `toml_edit` + `salsa` stack (Contract B, zero hand-rolled scanners) inside `< 380`-SLOC submodules with legacy scanners deleted, Then `BidirectionalBacklogReconciler` confirms normative requirement satisfaction across execution.
- [ ] AC-US-30-05: Given `BidirectionalBacklogReconciler` ingress port binding (`specDocsInOutPort: String`), When `BidirectionalBacklogReconciler` completes `MergeExtractedEntitiesIntoOwningSubpackages`, Then `BidirectionalBacklogReconciler` delivers `ResolveParentEpicByPkgName` across egress port (`specDocsInOutPort: String`, `subpackageSyncOutPort: String`) to `Operator_Or_CI_Runner`.
- [ ] AC-US-30-06: Given violated formal constraint `Invariant_REQ_0604_SC_01` or `Invariant_REQ_0604_SC_02` (such as root AST namespace pollution), When `Operator_Or_CI_Runner` invokes `MergeExtractedEntitiesIntoOwningSubpackages` on `BidirectionalBacklogReconciler`, Then `BidirectionalBacklogReconciler` rejects the request without partial state mutation, preserves prior verified state, and emits a deterministic fault diagnostic (`E0400`) across the interface binding.

## Required Features
- [ ] #670 - [Feature 16: [BP 04 Deterministic Specification ICD and WBS Forward Compiler Blueprint] AgileSpecForwardProjector](../features/FEAT-16-AgileSpecForwardProjector.md) (emits Schema Containers and Package frontmatter locators)
- [ ] #671 - [Feature 17: [BP 04 Deterministic Specification ICD and WBS Forward Compiler Blueprint] BidirectionalBacklogReconciler](../features/FEAT-17-BidirectionalBacklogReconciler.md) (merges reverse-synced entities into owning PackageDef subpackages via subpackageSyncOutPort)
- [ ] #672 - [Feature 18: [BP 04 Deterministic Specification ICD and WBS Forward Compiler Blueprint] DeterministicSpecIcdWbsForwardCompiler](../features/FEAT-18-DeterministicSpecIcdWbsForwardCompiler.md) (enforces forward_reverse_drift_count == 0)

## Source References
Operational concept definitions and system sequence interaction models:
- System Architecture Model: `schema/model.sysml`
- Normative Systems Engineering Standard: ISO/IEC/IEEE 29148:2018 §6.4.2 Concept of Operations

Authoritative user story flows and interaction lifelines derive from `schema/model.sysml` pursuant to ISO/IEC/IEEE 29148.
