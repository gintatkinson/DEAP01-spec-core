| Attribute | Specification Detail |
| :--- | :--- |
| **Issue ID** | #41 |
| **Title** | Feature 32: [Formal Safety Traceability] BipartiteGraphBuilder |
| **Type** | feature |
| **Parent Epic** | [EPIC-09-FormalSafetyTraceability](../epics/EPIC-09-FormalSafetyTraceability.md) |
| **Interface Type** | Physical / Structural / Internal Bus |
| **Schema Containers** | `FormalSafetyTraceability::BipartiteGraphBuilder` |
| **Generation Mode** | subagent |
| **Specification Source** | [schema/subsystems/formal_safety_traceability/architecture.sysml](../../schema/subsystems/formal_safety_traceability/architecture.sysml) |

# Feature 32: [Formal Safety Traceability] BipartiteGraphBuilder

## UML Class Diagram

```mermaid
classDiagram
    class BipartiteGraphBuilder {
        +ASTStreamPort ast_stream (in)
        +SafetyPort safety_out (out)
    }
```

## Interface Requirements
### 1. Test Data Shape
```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "title": "BipartiteGraphBuilderPayloadSchema",
  "type": "object",
  "properties": {
    "part": {
      "type": "string",
      "const": "BipartiteGraphBuilder"
    },
    "status": {
      "type": "string",
      "enum": ["nominal", "degraded", "fault"]
    },
    "ast_stream": {
      "type": "object",
      "properties": {
        "direction": { "type": "string", "const": "in" },
        "port_type": { "type": "string", "const": "ASTStreamPort" }
      },
      "required": ["direction", "port_type"]
    },
    "safety_out": {
      "type": "object",
      "properties": {
        "direction": { "type": "string", "const": "out" },
        "port_type": { "type": "string", "const": "SafetyPort" }
      },
      "required": ["direction", "port_type"]
    }
  },
  "required": [
    "part",
    "status",
    "ast_stream",
    "safety_out"
  ],
  "additionalProperties": false
}
```

### 2. Validation & Constraints
Formal constraints and invariant guarantees enforced by BipartiteGraphBuilder.

### 3. Visual Layout & Arrangement
Logical layout, viewport containment, and container structure for BipartiteGraphBuilder conforming to platform design tokens. Container boundaries enforce @container isolation rules.

### 4. Interactive Flow & States
Operational lifecycle events and discrete state transitions for BipartiteGraphBuilder.

### Layer 1: Domain State & Signal Model
Domain state vector representing BipartiteGraphBuilder status, AST references, and typed signal models.

### Layer 2: Logic & Safety State Management
Deterministic state machine solver and safety monitor logic for BipartiteGraphBuilder.

### Layer 3: Presentation & Actuator Interface Binding
Presentation interface binding and diagnostic event routing for BipartiteGraphBuilder.

## Acceptance Criteria (BDD)
- [ ] AC-01: Given nominal BipartiteGraphBuilder domain state, When command or input signals are received, Then updates internal state and signal outputs within declared bounds.
- [ ] AC-02: Given operational state, When BipartiteGraphBuilder evaluates constraint 'operational_invariants', Then asserts invariant satisfaction without violation.
- [ ] AC-03: Given malformed input signals or boundary condition violation, When BipartiteGraphBuilder error handling is invoked, Then transitions safely to failsafe fallback state without corruption.

## Source References
Formal component specifications and normative systems engineering requirements:
- Subsystem Requirements: `schema/subsystems/formal_safety_traceability/requirements.sysml`
- Subsystem Architecture: `schema/subsystems/formal_safety_traceability/architecture.sysml`
- Normative Systems Engineering Standard: ISO/IEC/IEEE 15288:2023 §6.4.3 Architecture Definition Process

Component behavior and interface definitions derive from `schema/model.sysml` pursuant to ISO/IEC/IEEE 15288.
