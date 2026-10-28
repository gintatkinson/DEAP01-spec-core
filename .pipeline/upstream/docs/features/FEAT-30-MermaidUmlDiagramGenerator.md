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

### 3. Visual Layout & Arrangement
Structural composition and boundary topology for classifier `MermaidUmlDiagramGenerator` organize no nested subparts declared in schema paired with boundary ports `in umlSpecInPort : String`, `out mermaidBlockOutPort : String`.

### 4. Interactive Flow & States
Operational lifecycle events and discrete state transitions for MermaidUmlDiagramGenerator.

### Layer 1: Domain State & Signal Model
Classifier `MermaidUmlDiagramGenerator` encapsulates typed domain state attributes `forbidSemicolons : Boolean`, `check13bCompliant : Boolean` alongside operational lifecycle status.

### Layer 2: Logic & Safety State Management
Classifier `MermaidUmlDiagramGenerator` governs stateless structural transition evaluation enforcing no additional invariant guards declared in schema.

### Layer 3: Presentation & Actuator Interface Binding
Classifier `MermaidUmlDiagramGenerator` binds external and behavioral interfaces across port `in umlSpecInPort : String`, port `out mermaidBlockOutPort : String`.

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
