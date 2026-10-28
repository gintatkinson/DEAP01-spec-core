| Attribute | Specification Detail |
| :--- | :--- |
| **Issue ID** | #730 |
| **Title** | User Story 43: Gh Issue Create Or Gh Issue |
| **Type** | user-story |
| **Parent Epic** | [EPIC-06-BP_06_Deterministic_Static_5_Pillar_Code_Auditor_and_Defect_Dossier_Synthesizer_Blueprint](../epics/EPIC-06-BP_06_Deterministic_Static_5_Pillar_Code_Auditor_and_Defect_Dossier_Synthesizer_Blueprint.md) |
| **SysML Interaction** | `BP_06_Deterministic_Static_5_Pillar_Code_Auditor_and_Defect_Dossier_Synthesizer_Blueprint_SequenceInteraction` |
| **SysML Test Case** | `Test_GhIssueCreateOrGhIssue` |
| **Generation Mode** | subagent |
| **Specification Source** | [schema/model.sysml](../../schema/model.sysml) |

# User Story 43: Gh Issue Create Or Gh Issue

## User Story

As a `Operator_Or_CI_Pipeline`, I want to invoke `GhIssueCreateOrGhIssue()` on `DeterministicDossierSynthesizer`, so that gh issue create or gh issue edit with body-file.

## Architectural Realization (3-Layer Definition of Done)

### Layer 1: Domain State & Data Model
- Domain state attributes on `DeterministicDossierSynthesizer`: `mandatorySectionCount: Integer`, `whysChainDepth: Integer`.
- Action `GhIssueCreateOrGhIssue` invocation message envelope bound to `DeterministicDossierSynthesizer` typed domain schema.
- Declared lifecycle state model: `BP_06_Deterministic_Static_5_Pillar_Code_Auditor_and_Defect_Dossier_Synthesizer_Blueprint_LifecycleState`, `Idle`, `IngestingSourceAst`, `EvaluatingFivePillars`, `SynthesizingDossier`, `Verified`, `Validating12Gates`, `Rejected`, `DeduplicatingSsot`, `PublishingClosedLoop`.

### Layer 2: Logic & State Management
- Behavioral execution contract: invokes `GhIssueCreateOrGhIssue()` on `DeterministicDossierSynthesizer` with atomic precondition verification.
- State transition management: `Idle -> IngestingSourceAst` (trigger `verify_baseline_audit_code_path`).
- Formal constraints and requirements enforced: `REQ-0606 (REQ_0606_BP_06_Deterministic_Static_5_Pillar_Code_Auditor_and_Defect_Dossier_Synthesizer_Blueprint)`.

### Layer 3: Presentation & Actuator Interface Binding
- Component port bindings on `DeterministicDossierSynthesizer`: `findingInPort: String (in)`, `dossierMarkdownOutPort: String (out)`.
- Interaction binding (`BP_06_Deterministic_Static_5_Pillar_Code_Auditor_and_Defect_Dossier_Synthesizer_Blueprint_SequenceInteraction`): routes `GhIssueCreateOrGhIssue` invocation from `Operator_Or_CI_Pipeline` to `DeterministicDossierSynthesizer` and returns `ParseRustSourceAstAndParse`.
- Diagnostic fault surface: reports deterministic rejection diagnostics to `Operator_Or_CI_Pipeline` across the declared interface boundary when guard or invariant checks fail.

## UML Sequence Diagram

```mermaid
sequenceDiagram
    autonumber
    actor Operator_Or_CI_Pipeline
    participant ClosedLoopDefectPublisherGate
    participant DeterministicDossierSynthesizer
    Operator_Or_CI_Pipeline->>ClosedLoopDefectPublisherGate: GhIssueCreateOrGhIssue()
    ClosedLoopDefectPublisherGate->>DeterministicDossierSynthesizer: GhIssueCreateOrGhIssue()
    DeterministicDossierSynthesizer-->>ClosedLoopDefectPublisherGate: ParseRustSourceAstAndParse
    ClosedLoopDefectPublisherGate-->>Operator_Or_CI_Pipeline: StatusReport
```

## Acceptance Criteria (BDD)
- [ ] AC-US-43-01: Given `DeterministicDossierSynthesizer` is in declared state `Idle` with domain state attributes (`mandatorySectionCount: Integer`, `whysChainDepth: Integer`), When `Operator_Or_CI_Pipeline` invokes `GhIssueCreateOrGhIssue` on `DeterministicDossierSynthesizer`, Then `DeterministicDossierSynthesizer` validates and binds the request against `DeterministicDossierSynthesizer` domain state (`mandatorySectionCount: Integer`, `whysChainDepth: Integer`) and transitions to `IngestingSourceAst`.
- [ ] AC-US-43-02: Given `DeterministicDossierSynthesizer` active state management during `GhIssueCreateOrGhIssue`, When `DeterministicDossierSynthesizer` evaluates formal constraint `Invariant_REQ_0606_SC_01`, Then `DeterministicDossierSynthesizer` verifies constraint satisfaction before committing state updates.
- [ ] AC-US-43-03: Given `DeterministicDossierSynthesizer` active state management during `GhIssueCreateOrGhIssue`, When `DeterministicDossierSynthesizer` evaluates formal constraint `Invariant_REQ_0606_SC_02`, Then `DeterministicDossierSynthesizer` verifies constraint satisfaction before committing state updates.
- [ ] AC-US-43-04: Given `DeterministicDossierSynthesizer` requirement allocation for `GhIssueCreateOrGhIssue`, When `DeterministicDossierSynthesizer` verifies formal requirement `REQ-0606 (REQ_0606_BP_06_Deterministic_Static_5_Pillar_Code_Auditor_and_Defect_Dossier_Synthesizer_Blueprint)`, Then `DeterministicDossierSynthesizer` confirms normative requirement satisfaction across execution.
- [ ] AC-US-43-05: Given `DeterministicDossierSynthesizer` ingress port binding (`findingInPort: String`), When `DeterministicDossierSynthesizer` completes `GhIssueCreateOrGhIssue`, Then `DeterministicDossierSynthesizer` delivers `ParseRustSourceAstAndParse` across egress port (`dossierMarkdownOutPort: String`) to `Operator_Or_CI_Pipeline`.
- [ ] AC-US-43-06: Given violated formal constraint `Invariant_REQ_0606_SC_01`, When `Operator_Or_CI_Pipeline` invokes `GhIssueCreateOrGhIssue` on `DeterministicDossierSynthesizer`, Then `DeterministicDossierSynthesizer` rejects the request without partial state mutation, preserves prior verified state, and emits a deterministic fault diagnostic across the interface binding.

## Required Features
- [ ] #680 - [Feature 26: [BP 06 Deterministic Static 5 Pillar Code Auditor and Defect Dossier Synthesizer Blueprint] ClosedLoopDefectPublisherGate](../features/FEAT-26-ClosedLoopDefectPublisherGate.md)
- [ ] #681 - [Feature 27: [BP 06 Deterministic Static 5 Pillar Code Auditor and Defect Dossier Synthesizer Blueprint] DeterministicDossierSynthesizer](../features/FEAT-27-DeterministicDossierSynthesizer.md)

## Source References
Operational concept definitions and system sequence interaction models:
- System Architecture Model: `schema/model.sysml`
- Normative Systems Engineering Standard: ISO/IEC/IEEE 29148:2018 §6.4.2 Concept of Operations

Authoritative user story flows and interaction lifelines derive from `schema/model.sysml` pursuant to ISO/IEC/IEEE 29148.
