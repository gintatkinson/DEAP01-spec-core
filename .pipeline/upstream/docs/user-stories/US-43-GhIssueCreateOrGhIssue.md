| Attribute | Specification Detail |
| :--- | :--- |
| **Issue ID** | #730 |
| **Title** | User Story 43: Gh Issue Create Or Gh Issue |
| **Type** | user-story |
| **Parent Epic** | [EPIC-06-BP_06_Deterministic_Static_5_Pillar_Code_Auditor_and_Defect_Dossier_Synthesizer_Blueprint](../epics/EPIC-06-BP_06_Deterministic_Static_5_Pillar_Code_Auditor_and_Defect_Dossier_Synthesizer_Blueprint.md) (#690) |
| **SysML Interaction** | `BP_06_Deterministic_Static_5_Pillar_Code_Auditor_and_Defect_Dossier_Synthesizer_Blueprint_SequenceInteraction` |
| **SysML Test Case** | `Test_GhIssueCreateOrGhIssue` |
| **Generation Mode** | subagent |
| **Specification Source** | [schema/model.sysml](../../schema/model.sysml) |

# User Story 43: Gh Issue Create Or Gh Issue

## User Story

As a `Operator_Or_CI_Pipeline`, I want to invoke `GhIssueCreateOrGhIssue()` on `ClosedLoopDefectPublisherGate`, `DeterministicStaticDefectAuditor`, and `DeterministicDossierSynthesizer`, so that validated 7-section defect dossiers in `./.pipeline/defects/*.md` are published idempotently to the GitHub issue tracker via `gh issue create --body-file` or `gh issue edit --body-file` across `ClosedLoopTrackerPublishPort`.

## Architectural Realization (3-Layer Definition of Done)

### Layer 1: Domain State & Data Model
- Domain state attributes on `ClosedLoopDefectPublisherGate` (`schemaGateCount: Integer = 12`, `requireByteParity: Boolean = true`), `DeterministicStaticDefectAuditor` (`closed_loop_mismatch_bytes: Integer = 0`, `auditor_subagent_count: Integer = 0`), and `DeterministicDossierSynthesizer` (`mandatorySectionCount: Integer = 7`, `whysChainDepth: Integer = 5`).
- Enforces **Contract A** (100% normative OMG SysML v2.0 + KerML 1.0 textual grammar coverage with full Definition and Usage symmetry and zero 16-construct artificial caps) and **Contract B** (`ra_ap_syntax` typed AST + Stage-2 `./lints/deap-dylint-mir/` `rustc_middle::mir` (`TyCtxt`) 5-pillar defect dossier auditor + `deap_core::dart_cst` (`logos` + `rowan`) isolation + `pulldown-cmark` / `pulldown-cmark-to-cmark` + complete deletion of `source_ast.rs`, zero hand-rolled scanners).
- Action `GhIssueCreateOrGhIssue` invocation message envelope bound to `ClosedLoopDefectPublisherGate` and `DeterministicStaticDefectAuditor` typed domain schemas.
- Declared lifecycle state model: `BP_06_Deterministic_Static_5_Pillar_Code_Auditor_and_Defect_Dossier_Synthesizer_Blueprint_LifecycleState` (`Idle`, `IngestingRustAndDartCst`, `EvaluatingFivePillars`, `SynthesizingDossier`, `Verified`, `Validating12Gates`, `Rejected`, `DeduplicatingSsot`, `PublishingClosedLoop`).

### Layer 2: Logic & State Management
- Behavioral execution contract: invokes `GhIssueCreateOrGhIssue()` after SSOT deduplication (`DeduplicatingSsot -> PublishingClosedLoop` on `new_finding_persisted_to_pipeline_defects`), publishing the local `./.pipeline/defects/<NAME>.md` file via `--body-file`.
- State transition management: `DeduplicatingSsot -> PublishingClosedLoop` (trigger `new_finding_persisted_to_pipeline_defects`).
- Formal constraints and requirements enforced: **REQ-0606** (`REQ_0606_BP_06_Deterministic_Static_5_Pillar_Code_Auditor_and_Defect_Dossier_Synthesizer_Blueprint`), `Invariant_REQ_0606_SC_01`, and `Invariant_REQ_0606_SC_02`.
- Modularity & legacy scanner deletion target: bounds implementation inside `< 380`-SLOC submodules (`./crates/verify-baseline/src/defect_dossier/` and `./crates/verify-baseline/src/defect_auditor/`) while permanently deleting `./crates/deap-core/src/source_ast.rs` (`source_ast.rs`) and retiring all hand-rolled regex/substring defect detectors.

### Layer 3: Presentation & Actuator Interface Binding
- Component port bindings: `DefectDossierArtifactPort: String (out)`, `ClosedLoopTrackerPublishPort: String (out)`, `dossierCandidateInPort: String (in)`, `verifiedReceiptOutPort: String (out)`.
- Interaction binding (`BP_06_Deterministic_Static_5_Pillar_Code_Auditor_and_Defect_Dossier_Synthesizer_Blueprint_SequenceInteraction`): routes `GhIssueCreateOrGhIssue` across `ClosedLoopDefectPublisherGate`, `DeterministicStaticDefectAuditor`, and `DeterministicDossierSynthesizer`.
- Diagnostic fault surface: reports deterministic `ariadne` / `miette` rejection diagnostics (`E0603`, `E0604`) to `Operator_Or_CI_Pipeline` across the declared interface boundary when tracker publication or invariant checks fail.

## UML Sequence Diagram

```mermaid
sequenceDiagram
    autonumber
    actor Operator_Or_CI_Pipeline
    participant ClosedLoopDefectPublisherGate
    participant DeterministicStaticDefectAuditor
    participant DeterministicDossierSynthesizer
    Operator_Or_CI_Pipeline->>ClosedLoopDefectPublisherGate: GhIssueCreateOrGhIssue()
    ClosedLoopDefectPublisherGate->>DeterministicStaticDefectAuditor: GhIssueCreateOrGhIssue()
    DeterministicStaticDefectAuditor->>DeterministicDossierSynthesizer: GhIssueCreateOrGhIssue()
    DeterministicDossierSynthesizer-->>ClosedLoopDefectPublisherGate: ParseRustCstAndParseDartCst
    ClosedLoopDefectPublisherGate-->>Operator_Or_CI_Pipeline: StatusReport
```

## Acceptance Criteria (BDD)
- [ ] AC-US-43-01: Given `ClosedLoopDefectPublisherGate` and `DeterministicStaticDefectAuditor` are in declared state `DeduplicatingSsot` after all 12 dossier checks pass (`schemaGateCount == 12`, `mandatorySectionCount == 7`, `whysChainDepth == 5`), When `Operator_Or_CI_Pipeline` invokes `GhIssueCreateOrGhIssue`, Then the validated dossier is persisted to `./.pipeline/defects/`, published via `gh issue create --body-file` or `gh issue edit --body-file`, and the state machine transitions to `PublishingClosedLoop`.
- [ ] AC-US-43-02: Given active state management during `GhIssueCreateOrGhIssue`, When formal constraint `Invariant_REQ_0606_SC_01` (`dossier_schema_check_count == 12 && auditor_subagent_count == 0`) is evaluated, Then constraint satisfaction is verified before committing state updates.
- [ ] AC-US-43-03: Given active state management during `GhIssueCreateOrGhIssue`, When formal constraint `Invariant_REQ_0606_SC_02` (`closed_loop_mismatch_bytes == 0 && zeroRegexMode == true`) is evaluated, Then constraint satisfaction is verified before committing state updates.
- [ ] AC-US-43-04: Given requirement allocation for `GhIssueCreateOrGhIssue`, When formal requirement **REQ-0606** (`REQ_0606_BP_06_Deterministic_Static_5_Pillar_Code_Auditor_and_Defect_Dossier_Synthesizer_Blueprint`) is verified, Then normative Contract A and Contract B satisfaction is confirmed across `< 380`-SLOC submodules with `source_ast.rs` deleted.
- [ ] AC-US-43-05: Given ingress port binding (`dossierCandidateInPort: String`, `findingInPort: String`), When `GhIssueCreateOrGhIssue` completes, Then the published tracker reference is routed across `ClosedLoopTrackerPublishPort: String` for closed-loop readback verification.
- [ ] AC-US-43-06: Given a `gh` subprocess failure or violated formal constraint `Invariant_REQ_0606_SC_01`, When `Operator_Or_CI_Pipeline` invokes `GhIssueCreateOrGhIssue`, Then `ClosedLoopDefectPublisherGate` transitions to `Rejected`, preserves prior verified state, and emits a deterministic `ariadne` / `miette` fault diagnostic (`E0604`).

## Required Features
- [ ] #680 - [Feature 26: [BP 06 Deterministic Static 5 Pillar Code Auditor and Defect Dossier Synthesizer Blueprint] ClosedLoopDefectPublisherGate](../features/FEAT-26-ClosedLoopDefectPublisherGate.md)
- [ ] #681 - [Feature 27: [BP 06 Deterministic Static 5 Pillar Code Auditor and Defect Dossier Synthesizer Blueprint] DeterministicDossierSynthesizer](../features/FEAT-27-DeterministicDossierSynthesizer.md)
- [ ] #682 - [Feature 28: [BP 06 Deterministic Static 5 Pillar Code Auditor and Defect Dossier Synthesizer Blueprint] DeterministicStaticDefectAuditor](../features/FEAT-28-DeterministicStaticDefectAuditor.md)

## Source References
Operational concept definitions and system sequence interaction models:
- System Architecture Model: `schema/model.sysml`
- Normative Systems Engineering Standard: ISO/IEC/IEEE 29148:2018 §6.4.2 Concept of Operations

Authoritative user story flows and interaction lifelines derive from `schema/model.sysml` pursuant to ISO/IEC/IEEE 29148.
