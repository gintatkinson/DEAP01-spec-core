| Attribute | Specification Detail |
| :--- | :--- |
| **Issue ID** | #28 |
| **Title** | Feature 19: [Sysml Grammar Lowering] GrammarLoweringEngine |
| **Type** | feature |
| **Parent Epic** | [EPIC-06-SysmlGrammarLowering](../epics/EPIC-06-SysmlGrammarLowering.md) |
| **Interface Type** | Physical / Structural / Internal Bus |
| **Schema Containers** | `SysmlGrammarLowering::GrammarLoweringEngine` |
| **Generation Mode** | subagent |
| **Specification Source** | [schema/subsystems/sysml_grammar_lowering/architecture.sysml](../../schema/subsystems/sysml_grammar_lowering/architecture.sysml) |

# Feature 19: [Sysml Grammar Lowering] GrammarLoweringEngine

## UML Class Diagram

```mermaid
classDiagram
    class GrammarLoweringEngine {
        +ASTGraphPort ast_in (in)
        +ASTStreamPort ast_out (out)
    }
    GrammarLoweringEngine --> KerMLTokenStream : tokenStream
    GrammarLoweringEngine --> SysMLv2GrammarParser : grammarParser
    GrammarLoweringEngine --> ASTLoweringEngine : loweringEngine
```

## Interface Requirements
### 1. Test Data Shape
```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "title": "GrammarLoweringEnginePayloadSchema",
  "type": "object",
  "properties": {
    "part": {
      "type": "string",
      "const": "GrammarLoweringEngine"
    },
    "status": {
      "type": "string",
      "enum": ["nominal", "degraded", "fault"]
    },
    "ast_in": {
      "type": "object",
      "properties": {
        "direction": { "type": "string", "const": "in" },
        "port_type": { "type": "string", "const": "ASTGraphPort" }
      },
      "required": ["direction", "port_type"]
    },
    "ast_out": {
      "type": "object",
      "properties": {
        "direction": { "type": "string", "const": "out" },
        "port_type": { "type": "string", "const": "ASTStreamPort" }
      },
      "required": ["direction", "port_type"]
    }
  },
  "required": [
    "part",
    "status",
    "ast_in",
    "ast_out"
  ],
  "additionalProperties": false
}
```

### 2. Validation & Constraints
Formal constraints and invariant guarantees enforced by GrammarLoweringEngine.

### 3. Visual Layout & Arrangement
Logical layout, viewport containment, and container structure for GrammarLoweringEngine conforming to platform design tokens. Container boundaries enforce @container isolation rules.

### 4. Interactive Flow & States
Operational lifecycle events and discrete state transitions for GrammarLoweringEngine.

### Layer 1: Domain State & Signal Model
Domain state vector representing GrammarLoweringEngine status, AST references, and typed signal models.

### Layer 2: Logic & Safety State Management
Deterministic state machine solver and safety monitor logic for GrammarLoweringEngine.

### Layer 3: Presentation & Actuator Interface Binding
Presentation interface binding and diagnostic event routing for GrammarLoweringEngine.

## Acceptance Criteria (BDD)
- [ ] AC-01: Given nominal GrammarLoweringEngine domain state, When command or input signals are received, Then updates internal state and signal outputs within declared bounds.
- [ ] AC-02: Given operational state, When GrammarLoweringEngine evaluates constraint 'Invariant_REQ_0054_FormalInvariant', Then asserts invariant satisfaction without violation.
- [ ] AC-03: Given operational state, When GrammarLoweringEngine evaluates constraint 'Invariant_REQ_0054_Every_element_is_an_immutable_tuple', Then asserts invariant satisfaction without violation.
- [ ] AC-04: Given operational state, When GrammarLoweringEngine evaluates constraint 'Invariant_REQ_0054_Relationship_endpoint_invariants', Then asserts invariant satisfaction without violation.
- [ ] AC-05: Given operational state, When GrammarLoweringEngine evaluates constraint 'Invariant_REQ_0054_Ownership_forest_and_acyclicity_invariants', Then asserts invariant satisfaction without violation.
- [ ] AC-06: Given operational state, When GrammarLoweringEngine evaluates constraint 'Invariant_REQ_0054_FormalInvariant_2', Then asserts invariant satisfaction without violation.
- [ ] AC-07: Given operational state, When GrammarLoweringEngine evaluates constraint 'Invariant_REQ_0054_Annotation_non_interference', Then asserts invariant satisfaction without violation.
- [ ] AC-08: Given operational state, When GrammarLoweringEngine evaluates constraint 'Invariant_REQ_0054_Diagnostic_trigger_mappings_per_REQ_0191', Then asserts invariant satisfaction without violation.
- [ ] AC-09: Given operational state, When GrammarLoweringEngine evaluates constraint 'Invariant_REQ_0055_FormalInvariant', Then asserts invariant satisfaction without violation.
- [ ] AC-10: Given malformed input signals or boundary condition violation, When GrammarLoweringEngine error handling is invoked, Then transitions safely to failsafe fallback state without corruption.

## Source References
Formal component specifications and normative systems engineering requirements:
- Subsystem Requirements: `schema/subsystems/sysml_grammar_lowering/requirements.sysml`
- Subsystem Architecture: `schema/subsystems/sysml_grammar_lowering/architecture.sysml`
- Normative Systems Engineering Standard: ISO/IEC/IEEE 15288:2023 §6.4.3 Architecture Definition Process

Component behavior and interface definitions derive from `schema/model.sysml` pursuant to ISO/IEC/IEEE 15288.
