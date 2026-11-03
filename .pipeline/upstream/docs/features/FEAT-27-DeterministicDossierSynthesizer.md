| Attribute | Specification Detail |
| :--- | :--- |
| **Issue ID** | #681 |
| **Title** | Feature 27: [BP 06 Deterministic Static 5 Pillar Code Auditor and Defect Dossier Synthesizer Blueprint] DeterministicDossierSynthesizer |
| **Type** | feature |
| **Parent Epic** | [EPIC-06-BP_06_Deterministic_Static_5_Pillar_Code_Auditor_and_Defect_Dossier_Synthesizer_Blueprint](../epics/EPIC-06-BP_06_Deterministic_Static_5_Pillar_Code_Auditor_and_Defect_Dossier_Synthesizer_Blueprint.md) |
| **Interface Type** | Physical / Structural / Internal Bus |
| **Schema Containers** | `BP_06_Deterministic_Static_5_Pillar_Code_Auditor_and_Defect_Dossier_Synthesizer_Blueprint::DeterministicDossierSynthesizer` |
| **Generation Mode** | subagent |
| **Specification Source** | [schema/model.sysml](../../schema/model.sysml) |

# Feature 27: [BP 06 Deterministic Static 5 Pillar Code Auditor and Defect Dossier Synthesizer Blueprint] DeterministicDossierSynthesizer

## UML Class Diagram

```mermaid
classDiagram
    class DeterministicDossierSynthesizer {
        +String findingInPort (in)
        +String dossierMarkdownOutPort (out)
        +Integer mandatorySectionCount
        +Integer whysChainDepth
    }
```

## Interface Requirements
### 1. Test Data Shape
```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "title": "DeterministicDossierSynthesizerPayloadSchema",
  "type": "object",
  "properties": {
    "part": {
      "type": "string",
      "const": "DeterministicDossierSynthesizer"
    },
    "status": {
      "type": "string",
      "enum": ["nominal", "degraded", "fault"]
    },
    "mandatorySectionCount": {
      "type": "integer",
      "x-sysml-type": "Integer",
      "description": "Declared attribute mandatorySectionCount (Integer)",
      "default": 7
    },
    "whysChainDepth": {
      "type": "integer",
      "x-sysml-type": "Integer",
      "description": "Enforces 5 numbered causal `Why ...? Because ...` root-cause steps in Section 2. [unit: steps]",
      "default": 5
    },
    "findingInPort": {
      "type": "object",
      "properties": {
        "direction": { "type": "string", "const": "in" },
        "port_type": { "type": "string", "const": "String" }
      },
      "required": ["direction", "port_type"]
    },
    "dossierMarkdownOutPort": {
      "type": "object",
      "properties": {
        "direction": { "type": "string", "const": "out" },
        "port_type": { "type": "string", "const": "String" }
      },
      "required": ["direction", "port_type"]
    }
  },
  "required": [
    "part",
    "status",
    "findingInPort",
    "dossierMarkdownOutPort"
  ],
  "additionalProperties": false
}
```

### 2. Validation & Constraints
Declared structural attributes, boundary ports, invariants, and allocated requirements for classifier `DeterministicDossierSynthesizer`:

| AST Member Kind | Identifier | Declared Type / Direction | Default / Invariant Contract |
| :--- | :--- | :--- | :--- |
| Attribute | `mandatorySectionCount` | `Integer` | Default `7` |
| Attribute | `whysChainDepth` | `Integer` | Default `5` |
| Port | `findingInPort` | `in String` | Boundary interface contract |
| Port | `dossierMarkdownOutPort` | `out String` | Boundary interface contract |
| Constraint | `Invariant_REQ_0606_SC_01` | Invariant (`REQ-0606 (REQ_0606_BP_06_Deterministic_Static_5_Pillar_Code_Auditor_and_Defect_Dossier_Synthesizer_Blueprint)`) | Enforces `mandatorySectionCount == 7 && whysChainDepth == 5 && dossier_schema_check_count == 12` |
| Constraint | `Invariant_REQ_0606_SC_02` | Invariant (`REQ-0606 (REQ_0606_BP_06_Deterministic_Static_5_Pillar_Code_Auditor_and_Defect_Dossier_Synthesizer_Blueprint)`) | Enforces `closed_loop_mismatch_bytes == 0 && zeroRegexMode == true` |
| Requirement | `REQ-0606 (REQ_0606_BP_06_Deterministic_Static_5_Pillar_Code_Auditor_and_Defect_Dossier_Synthesizer_Blueprint)` | Allocated Requirement | Synthesizes deterministic 7-section defect dossiers (`./.pipeline/defects/<NAME>.md`) from `ra_ap_syntax` typed AST, Stage-2 `./lints/deap-dylint-mir/` `rustc_middle::mir` (`TyCtxt`), and `deap_core::dart_cst` (`logos` + `rowan`) findings under Contract A and Contract B |

### 3. Visual Layout & Arrangement
Structural composition and boundary topology for classifier `DeterministicDossierSynthesizer` organize no nested subparts declared in schema paired with boundary ports `in findingInPort : String`, `out dossierMarkdownOutPort : String`. Enforces Contract A (100% normative OMG SysML v2.0 + KerML 1.0 grammar coverage with zero 16-construct caps) and Contract B (`pulldown-cmark` / `pulldown-cmark-to-cmark` lossless Markdown AST generation from `ra_ap_syntax`, Stage-2 `./lints/deap-dylint-mir/` `rustc_middle::mir` (`TyCtxt`), and `deap_core::dart_cst` (`logos` + `rowan`) spans, with complete deletion of `source_ast.rs` and zero hand-rolled scanners).

### 4. Interactive Flow & States
Operational lifecycle events and discrete state transitions for `DeterministicDossierSynthesizer` transitioning from `EvaluatingFivePillars` to `SynthesizingDossier` on `AuditFinding_emitted` and from `SynthesizingDossier` to `Validating12Gates` on `synthesize_defect_dossier_markdown`.

### Layer 1: Domain State & Signal Model
Classifier `DeterministicDossierSynthesizer` encapsulates typed domain state attributes `mandatorySectionCount : Integer` (default `7`) and `whysChainDepth : Integer` (default `5`), typed `AuditFinding` records with workspace-relative paths and exact byte/line spans from `ra_ap_syntax`, `rustc_middle::mir` (`TyCtxt`), and `deap_core::dart_cst`, and `pulldown-cmark` AST nodes for all 7 canonical sections (`## 1. Context and References` through Section 7 and the `## Audit Source` footer).

### Layer 2: Logic & Safety State Management
Classifier `DeterministicDossierSynthesizer` governs deterministic 7-section Markdown synthesis via `pulldown-cmark-to-cmark`, constructing a 5-step causal `Why ...? Because ...` root-cause chain in Section 2 and integrating Check 13B compliant Mermaid blocks in Section 4 while enforcing `Invariant_REQ_0606_SC_01` and `Invariant_REQ_0606_SC_02` (zero Unicode em dashes and zero hardcoded OS/user/machine paths). Implemented inside `< 380`-SLOC submodules under `./crates/verify-baseline/src/defect_dossier/` with `./crates/deap-core/src/source_ast.rs` (`source_ast.rs`) and all legacy hand-rolled string/regex scanners permanently deleted.

### Layer 3: Presentation & Actuator Interface Binding
Classifier `DeterministicDossierSynthesizer` binds external and behavioral interfaces across port `in findingInPort : String` and port `out dossierMarkdownOutPort : String`, delivering synthesized 7-section defect dossier payloads to `ClosedLoopDefectPublisherGate` and emitting `ariadne` / `miette` diagnostics (`E0602`) on synthesis invariant violations.

### Associated Use Cases & User Stories

#### Associated Use Cases
- [ ] #650 - [Use Case 27: Deterministic Seven Section Dossier and Mermaid Synthesis](../use-cases/UC-27-Deterministic_Seven_Section_Dossier_and_Mermaid_Synthesis.md)
- [ ] #653 - [Use Case 30: Fail Closed Rejection on Malformed Dossier Schema Ephemeral Paths or Payload Divergence](../use-cases/UC-30-Fail_Closed_Rejection_on_Malformed_Dossier_Schema_Ephemeral_Paths_or_Payload_Divergence.md)

#### Associated User Stories
- [ ] #726 - [User Story 37: Verify Baseline Audit Code Path Crates](../user-stories/US-37-VerifyBaselineAuditCodePathCrates.md)
- [ ] #727 - [User Story 38: Parse Rust Cst And Parse Dart Cst](../user-stories/US-38-ParseRustCstAndParseDartCst.md)
- [ ] #728 - [User Story 39: Evaluate Source File Pillars Across5](../user-stories/US-39-EvaluateSourceFilePillarsAcross5.md)
- [ ] #615 - [User Story 40: Synthesize Defect Dossier Markdown](../user-stories/US-40-synthesize_defect_dossier_markdown.md)
- [ ] #617 - [User Story 41: Run Check Defect](../user-stories/US-41-run_check_defect.md)
- [ ] #729 - [User Story 42: Execute12 Check Validate Defect Dossier](../user-stories/US-42-Execute12CheckValidateDefectDossier.md)
- [ ] #730 - [User Story 43: Gh Issue Create Or Gh Issue](../user-stories/US-43-GhIssueCreateOrGhIssue.md)
- [ ] #731 - [User Story 44: Gh Issue View Json Title Body](../user-stories/US-44-GhIssueViewJsonTitleBody.md)

## Acceptance Criteria (BDD)
- [ ] AC-01: Given valid `DeterministicDossierSynthesizer` initialization inputs for `mandatorySectionCount` (`Integer`), `whysChainDepth` (`Integer`) (no additional constraints specified in schema), When `DeterministicDossierSynthesizer` commits typed domain state, Then declared state attributes hold verified values.
- [ ] AC-02: Given malformed or out-of-domain input values for `mandatorySectionCount` (`Integer`), `whysChainDepth` (`Integer`), When `DeterministicDossierSynthesizer` validates incoming state updates, Then `DeterministicDossierSynthesizer` rejects the invalid state update and emits a diagnostic via diagnostic port `dossierMarkdownOutPort`.
- [ ] AC-03: Given nominal signal arrival across `findingInPort` (`in String`), `dossierMarkdownOutPort` (`out String`), When `DeterministicDossierSynthesizer` processes the declared interface transaction, Then output state conforms to the structural classifier contract.
- [ ] AC-04: Given an invalid or unrecognized interface frame across `findingInPort` (`in String`), `dossierMarkdownOutPort` (`out String`), When `DeterministicDossierSynthesizer` boundary validation executes, Then `DeterministicDossierSynthesizer` refuses the malformed frame via diagnostic port `dossierMarkdownOutPort` and preserves prior verified state.

## Source References
Formal component specifications and normative systems engineering requirements:
- System Architecture Model: `schema/model.sysml`
- Normative Systems Engineering Standard: ISO/IEC/IEEE 15288:2023 §6.4.3 Architecture Definition Process

Component behavior and interface definitions derive from `schema/model.sysml` pursuant to ISO/IEC/IEEE 15288.
