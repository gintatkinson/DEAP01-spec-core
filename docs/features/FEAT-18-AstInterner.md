| Attribute | Specification Detail |
| :--- | :--- |
| **Issue ID** | #27 |
| **Title** | Feature 18: [Core Metamodel Node Arena] AstInterner |
| **Type** | feature |
| **Parent Epic** | [EPIC-05-CoreMetamodelNodeArena](../epics/EPIC-05-CoreMetamodelNodeArena.md) |
| **Interface Type** | Physical / Structural / Internal Bus |
| **Schema Containers** | `CoreMetamodelNodeArena::AstInterner` |
| **Generation Mode** | subagent |
| **Specification Source** | [schema/subsystems/core_metamodel_node_arena/architecture.sysml](../../schema/subsystems/core_metamodel_node_arena/architecture.sysml) |

# Feature 18: [Core Metamodel Node Arena] AstInterner

## UML Class Diagram

```mermaid
classDiagram
    class AstInterner {
        +TokenStreamPort token_in (in)
        +ASTGraphPort ast_graph_out (out)
        +Boolean deduplication_enabled
    }
```

## Interface Requirements
### 1. Test Data Shape
```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "title": "AstInternerPayloadSchema",
  "type": "object",
  "properties": {
    "part": {
      "type": "string",
      "const": "AstInterner"
    },
    "status": {
      "type": "string",
      "enum": ["nominal", "degraded", "fault"]
    },
    "deduplication_enabled": {
      "type": "boolean",
      "description": "Subsystem attribute deduplication_enabled"
    },
    "token_in": {
      "type": "object",
      "properties": {
        "direction": { "type": "string", "const": "in" },
        "port_type": { "type": "string", "const": "TokenStreamPort" }
      },
      "required": ["direction", "port_type"]
    },
    "ast_graph_out": {
      "type": "object",
      "properties": {
        "direction": { "type": "string", "const": "out" },
        "port_type": { "type": "string", "const": "ASTGraphPort" }
      },
      "required": ["direction", "port_type"]
    }
  },
  "required": [
    "part",
    "status",
    "deduplication_enabled",
    "token_in",
    "ast_graph_out"
  ],
  "additionalProperties": false
}
```

### 2. Validation & Constraints
Formal constraints and invariant guarantees enforced by AstInterner.

### 3. Visual Layout & Arrangement
Logical layout, viewport containment, and container structure for AstInterner conforming to platform design tokens. Container boundaries enforce @container isolation rules.

### 4. Interactive Flow & States
Operational lifecycle events and discrete state transitions for AstInterner.

### Layer 1: Domain State & Signal Model
Domain state vector representing AstInterner status, AST references, and typed signal models.

### Layer 2: Logic & Safety State Management
Deterministic state machine solver and safety monitor logic for AstInterner.

### Layer 3: Presentation & Actuator Interface Binding
Presentation interface binding and diagnostic event routing for AstInterner.

## Acceptance Criteria (BDD)
- [ ] AC-01: Given nominal AstInterner domain state, When command or input signals are received, Then updates internal state and signal outputs within declared bounds.
- [ ] AC-02: Given operational state, When AstInterner evaluates constraint 'operational_invariants', Then asserts invariant satisfaction without violation.
- [ ] AC-03: Given malformed input signals or boundary condition violation, When AstInterner error handling is invoked, Then transitions safely to failsafe fallback state without corruption.

## Source References
Formal component specifications and normative systems engineering requirements:
- Subsystem Requirements: `schema/subsystems/core_metamodel_node_arena/requirements.sysml`
- Subsystem Architecture: `schema/subsystems/core_metamodel_node_arena/architecture.sysml`
- Normative Systems Engineering Standard: ISO/IEC/IEEE 15288:2023 §6.4.3 Architecture Definition Process

Component behavior and interface definitions derive from `schema/model.sysml` pursuant to ISO/IEC/IEEE 15288.
