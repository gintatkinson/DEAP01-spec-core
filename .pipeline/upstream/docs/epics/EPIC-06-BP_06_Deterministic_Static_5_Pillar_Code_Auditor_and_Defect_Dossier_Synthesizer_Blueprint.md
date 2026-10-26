| Attribute | Specification Detail |
| :--- | :--- |
| **Issue ID** | #690 |
| **Title** | Epic 06: BP 06 Deterministic Static 5 Pillar Code Auditor and Defect Dossier Synthesizer Blueprint |
| **Type** | epic |
| **Package** | BP_06_Deterministic_Static_5_Pillar_Code_Auditor_and_Defect_Dossier_Synthesizer_Blueprint |
| **Subsystem** | BP 06 Deterministic Static 5 Pillar Code Auditor and Defect Dossier Synthesizer Blueprint |
| **Generation Mode** | subagent |
| **Specification Source** | [schema/model.sysml](../../schema/model.sysml) |

# Epic 06: BP 06 Deterministic Static 5 Pillar Code Auditor and Defect Dossier Synthesizer Blueprint

## 1. Context
Subsystem package for BP_06_Deterministic_Static_5_Pillar_Code_Auditor_and_Defect_Dossier_Synthesizer_Blueprint

## 2. Requirements & Checklist
- [ ] #680 - [Feature 26: [BP 06 Deterministic Static 5 Pillar Code Auditor and Defect Dossier Synthesizer Blueprint] ClosedLoopDefectPublisherGate](../features/FEAT-26-ClosedLoopDefectPublisherGate.md)
- [ ] #681 - [Feature 27: [BP 06 Deterministic Static 5 Pillar Code Auditor and Defect Dossier Synthesizer Blueprint] DeterministicDossierSynthesizer](../features/FEAT-27-DeterministicDossierSynthesizer.md)
- [ ] #682 - [Feature 28: [BP 06 Deterministic Static 5 Pillar Code Auditor and Defect Dossier Synthesizer Blueprint] DeterministicStaticDefectAuditor](../features/FEAT-28-DeterministicStaticDefectAuditor.md)
- [ ] #683 - [Feature 29: [BP 06 Deterministic Static 5 Pillar Code Auditor and Defect Dossier Synthesizer Blueprint] FivePillarStaticAstAnalyzer](../features/FEAT-29-FivePillarStaticAstAnalyzer.md)
- [ ] #684 - [Feature 30: [BP 06 Deterministic Static 5 Pillar Code Auditor and Defect Dossier Synthesizer Blueprint] MermaidUmlDiagramGenerator](../features/FEAT-30-MermaidUmlDiagramGenerator.md)

### Associated Use Cases & User Stories

#### Associated Use Cases
- [ ] #649 - [Use Case 26: Static Five Pillar AST Defect Detection](../use-cases/UC-26-Static_Five_Pillar_AST_Defect_Detection.md)
- [ ] #650 - [Use Case 27: Deterministic Seven Section Dossier and Mermaid Synthesis](../use-cases/UC-27-Deterministic_Seven_Section_Dossier_and_Mermaid_Synthesis.md)
- [ ] #651 - [Use Case 28: Idempotent Local and Remote SSOT Deduplication](../use-cases/UC-28-Idempotent_Local_and_Remote_SSOT_Deduplication.md)
- [ ] #652 - [Use Case 29: Closed Loop Git Hub SSOT Publishing and Byte Parity Verification](../use-cases/UC-29-Closed_Loop_GitHub_SSOT_Publishing_and_Byte_Parity_Verification.md)
- [ ] #653 - [Use Case 30: Fail Closed Rejection on Malformed Dossier Schema Ephemeral Paths or Payload Divergence](../use-cases/UC-30-Fail_Closed_Rejection_on_Malformed_Dossier_Schema_Ephemeral_Paths_or_Payload_Divergence.md)
- [ ] #654 - [Use Case 31: Dogfood Sys ML v2 Markdown Ingestion and Blueprint Gate Conformance](../use-cases/UC-31-Dogfood_SysML_v2_Markdown_Ingestion_and_Blueprint_Gate_Conformance.md)

#### Associated User Stories
- [ ] #610 - [User Story 49: verify baseline](../user-stories/US-49-verify_baseline.md)
- [ ] #611 - [User Story 50: parse rust source ast](../user-stories/US-50-parse_rust_source_ast.md)
- [ ] #612 - [User Story 51: Deterministic](../user-stories/US-51-Deterministic.md)
- [ ] #613 - [User Story 52: evaluate source file pillars](../user-stories/US-52-evaluate_source_file_pillars.md)
- [ ] #614 - [User Story 53: Line exact](../user-stories/US-53-Line_exact.md)
- [ ] #615 - [User Story 54: synthesize defect dossier markdown](../user-stories/US-54-synthesize_defect_dossier_markdown.md)
- [ ] #616 - [User Story 55: Compliant](../user-stories/US-55-Compliant.md)
- [ ] #617 - [User Story 56: run check defect](../user-stories/US-56-run_check_defect.md)
- [ ] #618 - [User Story 57: [ClosedLoopDefectPublisherGate] Execute](../user-stories/US-57-Execute.md)
- [ ] #619 - [User Story 58: gh](../user-stories/US-58-gh.md)
- [ ] #620 - [User Story 59: Created](../user-stories/US-59-Created.md)
- [ ] #621 - [User Story 60: Live](../user-stories/US-60-Live.md)
- [ ] #622 - [User Story 61: CLOSED LOOP](../user-stories/US-61-CLOSED_LOOP.md)
- [ ] #623 - [User Story 62: Exit item](../user-stories/US-62-Exit_item.md)

## 3. Architecture
Subsystem structural composition, port allocations, and directional data connectors realized by ClosedLoopDefectPublisherGate.

## 4. Operational Considerations
Deterministic compilation passes, error containment, and provable polynomial complexity execution.

## 5. Security & Governance
Safety-critical invariant satisfaction, formal trace matrix closure, and zero hardcoded domain semantics.

## 6. Source References
- System Architecture Model: `schema/model.sysml`
- Normative Systems Engineering Standard: ISO/IEC/IEEE 15288:2023 §6.4.3 Architecture Definition Process

Subsystem architectural composition and formal invariants derive from `schema/model.sysml` pursuant to ISO/IEC/IEEE 15288.

## System-Level UML Class Diagram

```mermaid
classDiagram
    class ClosedLoopDefectPublisherGate {
        +String dossierCandidateInPort (inout)
        +String verifiedReceiptOutPort (inout)
        +Integer schemaGateCount
        +Boolean requireByteParity
    }
    class DeterministicDossierSynthesizer {
        +String findingInPort (inout)
        +String dossierMarkdownOutPort (inout)
        +Integer mandatorySectionCount
        +Integer whysChainDepth
    }
    class DeterministicStaticDefectAuditor {
        +String TargetSourceAstInputPort (inout)
        +String GitHubIssueSsotQueryPort (inout)
        +String DefectDossierArtifactPort (inout)
        +String ClosedLoopTrackerPublishPort (inout)
        +Integer auditor_subagent_count
        +Integer dossier_schema_check_count
        +Integer pillar_coverage_count
        +Integer closed_loop_mismatch_bytes
    }
    class FivePillarStaticAstAnalyzer {
        +String astStreamInPort (inout)
        +String findingStreamOutPort (inout)
        +Integer activePillarCount
        +Boolean zeroRegexMode
    }
    class MermaidUmlDiagramGenerator {
        +String umlSpecInPort (inout)
        +String mermaidBlockOutPort (inout)
        +Boolean forbidSemicolons
        +Boolean check13bCompliant
    }
    ClosedLoopDefectPublisherGate --> DeterministicDossierSynthesizer : coordinates
    ClosedLoopDefectPublisherGate --> DeterministicStaticDefectAuditor : coordinates
    ClosedLoopDefectPublisherGate --> FivePillarStaticAstAnalyzer : coordinates
    ClosedLoopDefectPublisherGate --> MermaidUmlDiagramGenerator : coordinates
```

## System State Machine Diagram

```mermaid
stateDiagram-v2
    [*] --> Idle
    Idle --> IngestingSourceAst : verify_baseline_audit_code_path
    IngestingSourceAst --> EvaluatingFivePillars : SourceAstItem_slice_parsed
    EvaluatingFivePillars --> SynthesizingDossier : AuditFinding_emitted
    EvaluatingFivePillars --> Verified : zero_findings_detected
    SynthesizingDossier --> Validating12Gates : synthesize_defect_dossier_markdown
    Validating12Gates --> Rejected : schema_or_Mermaid_check_violation
    Validating12Gates --> DeduplicatingSsot : all_12_checks_passed
    DeduplicatingSsot --> Verified : existing_finding_skipped
    DeduplicatingSsot --> PublishingClosedLoop : new_finding_persisted_to_pipeline_defects
    PublishingClosedLoop --> Verified : closed_loop_mismatch_bytes_0
    PublishingClosedLoop --> Rejected : remote_payload_mismatch
    Rejected --> [*]
    Verified --> [*]
```
