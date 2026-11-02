| Attribute | Specification Detail |
| :--- | :--- |
| **Issue ID** | #684 |
| **Title** | Feature 30: [BP 06 Deterministic Static 5 Pillar Code Auditor and Defect Dossier Synthesizer Blueprint] MermaidUmlDiagramGenerator |
| **Type** | feature |
| **Parent Epic** | [EPIC-06-BP_06_Deterministic_Static_5_Pillar_Code_Auditor_and_Defect_Dossier_Synthesizer_Blueprint](../epics/EPIC-06-BP_06_Deterministic_Static_5_Pillar_Code_Auditor_and_Defect_Dossier_Synthesizer_Blueprint.md) |
| **Interface Type** | Physical / Structural / Internal Bus |
| **Schema Containers** | `BP_06_Deterministic_Static_5_Pillar_Code_Auditor_and_Defect_Dossier_Synthesizer_Blueprint::MermaidUmlDiagramGenerator` |
| **Generation Mode** | subagent |
| **Specification Source** | [schema/model.sysml](../../schema/model.sysml) |

# Feature 30: [BP 06 Deterministic Static 5 Pillar Code Auditor and Defect Dossier Synthesizer Blueprint] MermaidUmlDiagramGenerator

## UML Class Diagram

```mermaid
classDiagram
    class MermaidUmlDiagramGenerator {
        +String umlSpecInPort (in)
        +String mermaidBlockOutPort (out)
        +Boolean forbidSemicolons
        +Boolean check13bCompliant
    }
```

## Interface Requirements
### 1. Test Data Shape
```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "title": "MermaidUmlDiagramGeneratorPayloadSchema",
  "type": "object",
  "properties": {
    "part": {
      "type": "string",
      "const": "MermaidUmlDiagramGenerator"
    },
    "status": {
      "type": "string",
      "enum": ["nominal", "degraded", "fault"]
    },
    "forbidSemicolons": {
      "type": "boolean",
      "x-sysml-type": "Boolean",
      "description": "Declared attribute forbidSemicolons (Boolean)",
      "default": true
    },
    "check13bCompliant": {
      "type": "boolean",
      "x-sysml-type": "Boolean",
      "description": "Enforces Rules E1 through E5 across synthesized `sequenceDiagram` and `classDiagram` blocks. [unit: flag]",
      "default": true
    },
    "umlSpecInPort": {
      "type": "object",
      "properties": {
        "direction": { "type": "string", "const": "in" },
        "port_type": { "type": "string", "const": "String" }
      },
      "required": ["direction", "port_type"]
    },
    "mermaidBlockOutPort": {
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
    "umlSpecInPort",
    "mermaidBlockOutPort"
  ],
  "additionalProperties": false
}
```

### 2. Validation & Constraints
Declared structural attributes, boundary ports, invariants, and allocated requirements for classifier `MermaidUmlDiagramGenerator`:

| AST Member Kind | Identifier | Declared Type / Direction | Default / Invariant Contract |
| :--- | :--- | :--- | :--- |
| Attribute | `forbidSemicolons` | `Boolean` | Default `true` |
| Attribute | `check13bCompliant` | `Boolean` | Default `true` |
| Port | `umlSpecInPort` | `in String` | Boundary interface contract |
| Port | `mermaidBlockOutPort` | `out String` | Boundary interface contract |
| Constraint | `Invariant_REQ_0606_SC_01` | Invariant (`REQ-0606 (REQ_0606_BP_06_Deterministic_Static_5_Pillar_Code_Auditor_and_Defect_Dossier_Synthesizer_Blueprint)`) | Enforces Check 13B Rules E1--E5 and `forbidSemicolons == true && check13bCompliant == true` |
| Constraint | `Invariant_REQ_0606_SC_02` | Invariant (`REQ-0606 (REQ_0606_BP_06_Deterministic_Static_5_Pillar_Code_Auditor_and_Defect_Dossier_Synthesizer_Blueprint)`) | Enforces deterministic AST-backed Mermaid synthesis with zero hand-rolled scanners |
| Requirement | `REQ-0606 (REQ_0606_BP_06_Deterministic_Static_5_Pillar_Code_Auditor_and_Defect_Dossier_Synthesizer_Blueprint)` | Allocated Requirement | Synthesizes Section 4 Mermaid `sequenceDiagram` and `classDiagram` blocks from `ra_ap_syntax`, Stage-2 `./lints/deap-dylint-mir/` `rustc_middle::mir` (`TyCtxt`), and `deap_core::dart_cst` (`logos` + `rowan`) call/type graphs under Contract A and Contract B |

### 3. Visual Layout & Arrangement
Structural composition and boundary topology for classifier `MermaidUmlDiagramGenerator` organize no nested subparts declared in schema paired with boundary ports `in umlSpecInPort : String`, `out mermaidBlockOutPort : String`. Enforces Contract A (100% normative OMG SysML v2.0 + KerML 1.0 grammar coverage with zero 16-construct caps) and Contract B (typed AST/MIR call-graph and classifier projection from `ra_ap_syntax`, Stage-2 `./lints/deap-dylint-mir/` `rustc_middle::mir` (`TyCtxt`), and `deap_core::dart_cst` (`logos` + `rowan`), with complete deletion of `source_ast.rs` and zero hand-rolled scanners).

### 4. Interactive Flow & States
Operational lifecycle events and discrete state transitions for `MermaidUmlDiagramGenerator` during `SynthesizingDossier` and `Validating12Gates` within `BP_06_Deterministic_Static_5_Pillar_Code_Auditor_and_Defect_Dossier_Synthesizer_Blueprint_LifecycleState`.

### Layer 1: Domain State & Signal Model
Classifier `MermaidUmlDiagramGenerator` encapsulates typed domain state attributes `forbidSemicolons : Boolean` (default `true`) and `check13bCompliant : Boolean` (default `true`), along with structured UML lifeline, message, and class descriptors derived from `ra_ap_syntax` and `rustc_middle::mir` (`TyCtxt`) findings.

### Layer 2: Logic & Safety State Management
Classifier `MermaidUmlDiagramGenerator` governs deterministic Section 4 Mermaid diagram synthesis enforcing Check 13B Rules E1 through E5 (valid diagram header, newline closing fence, zero semicolons, zero curly braces or secondary colons in class members, double-quoted special labels, and open return arrows `-->`) and invariant obligations `Invariant_REQ_0606_SC_01` and `Invariant_REQ_0606_SC_02`. Implemented inside `< 380`-SLOC submodules under `./crates/verify-baseline/src/defect_dossier/` with `./crates/deap-core/src/source_ast.rs` (`source_ast.rs`) and all legacy hand-rolled scanners permanently deleted.

### Layer 3: Presentation & Actuator Interface Binding
Classifier `MermaidUmlDiagramGenerator` binds external and behavioral interfaces across port `in umlSpecInPort : String` and port `out mermaidBlockOutPort : String`, delivering Check 13B verified Mermaid code blocks to `DeterministicDossierSynthesizer` and emitting `ariadne` / `miette` diagnostics (`E0602`) on syntax or rule violations.

### Associated Use Cases & User Stories

#### Associated Use Cases
- [ ] #650 - [Use Case 27: Deterministic Seven Section Dossier and Mermaid Synthesis](../use-cases/UC-27-Deterministic_Seven_Section_Dossier_and_Mermaid_Synthesis.md)
- [ ] #653 - [Use Case 30: Fail Closed Rejection on Malformed Dossier Schema Ephemeral Paths or Payload Divergence](../use-cases/UC-30-Fail_Closed_Rejection_on_Malformed_Dossier_Schema_Ephemeral_Paths_or_Payload_Divergence.md)

#### Associated User Stories
- [ ] #615 - [User Story 40: synthesize defect dossier markdown](../user-stories/US-40-synthesize_defect_dossier_markdown.md)
- [ ] #729 - [User Story 42: Execute12 Check Validate Defect Dossier](../user-stories/US-42-Execute12CheckValidateDefectDossier.md)

## Acceptance Criteria (BDD)
- [ ] AC-01: Given valid `MermaidUmlDiagramGenerator` initialization inputs for `forbidSemicolons` (`Boolean`), `check13bCompliant` (`Boolean`) (no additional constraints specified in schema), When `MermaidUmlDiagramGenerator` commits typed domain state, Then declared state attributes hold verified values.
- [ ] AC-02: Given malformed or out-of-domain input values for `forbidSemicolons` (`Boolean`), `check13bCompliant` (`Boolean`), When `MermaidUmlDiagramGenerator` validates incoming state updates, Then `MermaidUmlDiagramGenerator` rejects the invalid state update and emits a diagnostic via diagnostic port `mermaidBlockOutPort`.
- [ ] AC-03: Given nominal signal arrival across `umlSpecInPort` (`in String`), `mermaidBlockOutPort` (`out String`), When `MermaidUmlDiagramGenerator` processes the declared interface transaction, Then output state conforms to the structural classifier contract.
- [ ] AC-04: Given an invalid or unrecognized interface frame across `umlSpecInPort` (`in String`), `mermaidBlockOutPort` (`out String`), When `MermaidUmlDiagramGenerator` boundary validation executes, Then `MermaidUmlDiagramGenerator` refuses the malformed frame via diagnostic port `mermaidBlockOutPort` and preserves prior verified state.

## Source References
Formal component specifications and normative systems engineering requirements:
- System Architecture Model: `schema/model.sysml`
- Normative Systems Engineering Standard: ISO/IEC/IEEE 15288:2023 §6.4.3 Architecture Definition Process

Component behavior and interface definitions derive from `schema/model.sysml` pursuant to ISO/IEC/IEEE 15288.
