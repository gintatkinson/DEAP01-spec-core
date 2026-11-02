| Attribute | Specification Detail |
| :--- | :--- |
| **Issue ID** | #715 |
| **Title** | User Story 26: Reconcile Backlog Reconcile Docs Dir Docs |
| **Type** | user-story |
| **Parent Epic** | [EPIC-04-BP_04_Deterministic_Specification_ICD_and_WBS_Forward_Compiler_Blueprint](../epics/EPIC-04-BP_04_Deterministic_Specification_ICD_and_WBS_Forward_Compiler_Blueprint.md) |
| **SysML Interaction** | `BP_04_Deterministic_Specification_ICD_and_WBS_Forward_Compiler_Blueprint_SequenceInteraction` |
| **SysML Test Case** | `Test_ReconcileBacklogReconcileDocsDirDocs` |
| **Generation Mode** | subagent |
| **Specification Source** | [schema/model.sysml](../../schema/model.sysml) |

# User Story 26: Reconcile Backlog Reconcile Docs Dir Docs

## User Story

As a `Operator_Or_CI_Runner`, I want to invoke `ReconcileBacklogReconcileDocsDirDocs()` on `BidirectionalBacklogReconciler`, so that `reconcile-backlog reconcile --docs-dir docs` deterministically reconciles all Epic, Feature, User Story, and Use Case frontmatter tables and checklists via `pulldown-cmark` + `pulldown-cmark-to-cmark` AST splicing and `salsa` cross-reference queries with zero hand-rolled scanners.

## Architectural Realization (3-Layer Definition of Done)

### Layer 1: Domain State & Data Model
- Domain state attributes on `BidirectionalBacklogReconciler`: `trackHtmlCommentState: Boolean`, `parentInclusiveEpics: Boolean`.
- **Contract A (Full OMG SysML v2.0 + KerML 1.0 Grammar Coverage -- Zero 16-Construct Caps)**: Action `ReconcileBacklogReconcileDocsDirDocs` message envelope binds to the full OMG SysML v2.0 + KerML 1.0 `package`, `part def`/`part`, `item def`/`item`, `action def`/`action`, `state def`/`state`, `interaction def`/`interaction`, `use case def`/`use case`, `verification def`/`verification`, `test case def`/`test case`, and `#capability` CST/AST (`deap_core::sysml_cst`, `logos` + `rowan` + `salsa`) with zero 16-construct caps.
- **Contract B (`pulldown-cmark` + `pulldown-cmark-to-cmark` Lossless Markdown AST + `toml_edit` + `salsa` Across All 6 Crates -- Zero Hand-Rolled Scanners)**: Parses and splices specification Markdown documents as typed `pulldown-cmark::Event` AST streams and `toml_edit::DocumentMut` manifests across `deap-core`, `compile-sysml`, `ingest-sysml`, `reconcile-backlog`, `verify-baseline`, and `assemble-conops`.
- Declared lifecycle state model: `BP_04_Deterministic_Specification_ICD_and_WBS_Forward_Compiler_Blueprint_LifecycleState` (`AstLoaded`, `ForwardProjected`, `IcdSynthesized`, `WbsExported`, `BacklogReconciled`, `ReverseSynced`, `Rejected`, `VerifiedRoundtrip`).

### Layer 2: Logic & State Management
- Behavioral execution contract: invokes `ReconcileBacklogReconcileDocsDirDocs()` on `BidirectionalBacklogReconciler` via `pulldown-cmark` + `pulldown-cmark-to-cmark` and `salsa` with atomic precondition verification and zero hand-rolled line or string scanners.
- State transition management: `AstLoaded -> ForwardProjected` (trigger `forwardSyncSpecs`) and `WbsExported -> BacklogReconciled` (trigger `reconcileBacklog`).
- Formal constraints and requirements enforced: `Invariant_REQ_0604_SC_01`, `Invariant_REQ_0604_SC_02`, and `REQ-0604 (REQ_0604_BP_04_Deterministic_Specification_ICD_and_WBS_Forward_Compiler_Blueprint)`.
- Modularity & explicit legacy scanner deletion target: bounds implementation inside `< 380` executable SLOC submodules (`./crates/compile-sysml/src/forward_sync/`, `./crates/compile-sysml/src/icd_wbs/`, `./crates/reconcile-backlog/src/table_cst/`) while enforcing explicit deletion of legacy hand-rolled Markdown/table/checklist line scanners (`-1,100+` SLOC net-negative ratchet).

### Layer 3: Presentation & Actuator Interface Binding
- Component port bindings on `BidirectionalBacklogReconciler`: `specDocsInOutPort: String (inout)`, `subpackageSyncOutPort: String (out)`.
- Interaction binding (`BP_04_Deterministic_Specification_ICD_and_WBS_Forward_Compiler_Blueprint_SequenceInteraction`): routes `ReconcileBacklogReconcileDocsDirDocs` invocation from `Operator_Or_CI_Runner` to `BidirectionalBacklogReconciler` and returns `ResolveParentEpicByPkgName`.
- Diagnostic fault surface: reports deterministic `ariadne` / `miette` rejection diagnostics (`E0300`) to `Operator_Or_CI_Runner` across the declared interface boundary when guard or invariant checks fail.

## UML Sequence Diagram

```mermaid
sequenceDiagram
    autonumber
    actor Operator_Or_CI_Runner
    participant AgileSpecForwardProjector
    participant BidirectionalBacklogReconciler
    Operator_Or_CI_Runner->>AgileSpecForwardProjector: ReconcileBacklogReconcileDocsDirDocs()
    AgileSpecForwardProjector->>BidirectionalBacklogReconciler: ReconcileBacklogReconcileDocsDirDocs()
    BidirectionalBacklogReconciler-->>AgileSpecForwardProjector: ResolveParentEpicByPkgName
    AgileSpecForwardProjector-->>Operator_Or_CI_Runner: StatusReport
```

## Formal SysML Test Case & Verification Binding
```sysml
test case def Test_ReconcileBacklogReconcileDocsDirDocs {
    subject BidirectionalBacklogReconciler;
    verify requirement REQ_0604_BP_04_Deterministic_Specification_ICD_and_WBS_Forward_Compiler_Blueprint;
    objective "Verify deterministic backlog reconciliation across ./docs/ via pulldown-cmark and pulldown-cmark-to-cmark";
}
```

## Acceptance Criteria (BDD)
- [ ] AC-US-26-01: Given `BidirectionalBacklogReconciler` is in declared state `AstLoaded` with domain state attributes (`trackHtmlCommentState: Boolean`, `parentInclusiveEpics: Boolean`), When `Operator_Or_CI_Runner` invokes `ReconcileBacklogReconcileDocsDirDocs` on `BidirectionalBacklogReconciler`, Then `BidirectionalBacklogReconciler` validates and binds the request against `BidirectionalBacklogReconciler` domain state (`trackHtmlCommentState: Boolean`, `parentInclusiveEpics: Boolean`) and transitions to `ForwardProjected`.
- [ ] AC-US-26-02: Given `BidirectionalBacklogReconciler` active state management during `ReconcileBacklogReconcileDocsDirDocs`, When `BidirectionalBacklogReconciler` evaluates formal constraint `Invariant_REQ_0604_SC_01`, Then `BidirectionalBacklogReconciler` verifies constraint satisfaction before committing state updates.
- [ ] AC-US-26-03: Given `BidirectionalBacklogReconciler` active state management during `ReconcileBacklogReconcileDocsDirDocs`, When `BidirectionalBacklogReconciler` evaluates formal constraint `Invariant_REQ_0604_SC_02`, Then `BidirectionalBacklogReconciler` verifies constraint satisfaction before committing state updates.
- [ ] AC-US-26-04: Given `BidirectionalBacklogReconciler` requirement allocation for `ReconcileBacklogReconcileDocsDirDocs`, When `BidirectionalBacklogReconciler` verifies formal requirement `REQ-0604 (REQ_0604_BP_04_Deterministic_Specification_ICD_and_WBS_Forward_Compiler_Blueprint)` across the full OMG SysML v2.0 + KerML 1.0 grammar (Contract A, zero 16-construct caps) and `pulldown-cmark` + `pulldown-cmark-to-cmark` + `toml_edit` + `salsa` stack (Contract B, zero hand-rolled scanners) inside `< 380`-SLOC submodules with legacy scanners deleted, Then `BidirectionalBacklogReconciler` confirms normative requirement satisfaction across execution.
- [ ] AC-US-26-05: Given `BidirectionalBacklogReconciler` ingress port binding (`specDocsInOutPort: String`), When `BidirectionalBacklogReconciler` completes `ReconcileBacklogReconcileDocsDirDocs`, Then `BidirectionalBacklogReconciler` delivers `ResolveParentEpicByPkgName` across egress port (`specDocsInOutPort: String`, `subpackageSyncOutPort: String`) to `Operator_Or_CI_Runner`.
- [ ] AC-US-26-06: Given violated formal constraint `Invariant_REQ_0604_SC_01`, When `Operator_Or_CI_Runner` invokes `ReconcileBacklogReconcileDocsDirDocs` on `BidirectionalBacklogReconciler`, Then `BidirectionalBacklogReconciler` rejects the request without partial state mutation, preserves prior verified state, and emits a deterministic fault diagnostic (`E0300`) across the interface binding.

## Required Features
- [ ] #670 - [Feature 16: [BP 04 Deterministic Specification ICD and WBS Forward Compiler Blueprint] AgileSpecForwardProjector](../features/FEAT-16-AgileSpecForwardProjector.md) (projects canonical CommonMark specification suites)
- [ ] #671 - [Feature 17: [BP 04 Deterministic Specification ICD and WBS Forward Compiler Blueprint] BidirectionalBacklogReconciler](../features/FEAT-17-BidirectionalBacklogReconciler.md) (reconciles frontmatter tables and checklists via pulldown-cmark-to-cmark)

## Source References
Operational concept definitions and system sequence interaction models:
- System Architecture Model: `schema/model.sysml`
- Normative Systems Engineering Standard: ISO/IEC/IEEE 29148:2018 §6.4.2 Concept of Operations

Authoritative user story flows and interaction lifelines derive from `schema/model.sysml` pursuant to ISO/IEC/IEEE 29148.
