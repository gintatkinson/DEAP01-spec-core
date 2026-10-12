| Attribute | Specification Detail |
| :--- | :--- |
| **Issue ID** | #59 |
| **Title** | Feature 50: [Diagnostic Error Catalog] SourceLocationMapper |
| **Type** | feature |
| **Parent Epic** | [EPIC-13-DiagnosticErrorCatalog](../epics/EPIC-13-DiagnosticErrorCatalog.md) |
| **Interface Type** | Physical / Structural / Internal Bus |
| **Schema Containers** | `DiagnosticErrorCatalog::SourceLocationMapper` |
| **Generation Mode** | subagent |
| **Specification Source** | [schema/subsystems/diagnostic_error_catalog/architecture.sysml](../../schema/subsystems/diagnostic_error_catalog/architecture.sysml) |

# Feature 50: [Diagnostic Error Catalog] SourceLocationMapper

## UML Class Diagram

```mermaid
classDiagram
    class SourceLocationMapper {
        +DiagnosticPort diag_in (in)
        +DiagnosticReportPort diag_out (out)
    }
```

## Interface Requirements
### 1. Test Data Shape
```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "title": "SourceLocationMapperPayloadSchema",
  "type": "object",
  "properties": {
    "part": {
      "type": "string",
      "const": "SourceLocationMapper"
    },
    "status": {
      "type": "string",
      "enum": ["nominal", "degraded", "fault"]
    },
    "diag_in": {
      "type": "object",
      "properties": {
        "direction": { "type": "string", "const": "in" },
        "port_type": { "type": "string", "const": "DiagnosticPort" }
      },
      "required": ["direction", "port_type"]
    },
    "diag_out": {
      "type": "object",
      "properties": {
        "direction": { "type": "string", "const": "out" },
        "port_type": { "type": "string", "const": "DiagnosticReportPort" }
      },
      "required": ["direction", "port_type"]
    }
  },
  "required": [
    "part",
    "status",
    "diag_in",
    "diag_out"
  ],
  "additionalProperties": false
}
```

### 2. Validation & Constraints
Formal constraints and invariant guarantees enforced by SourceLocationMapper.

### 3. Visual Layout & Arrangement
Logical layout, viewport containment, and container structure for SourceLocationMapper conforming to platform design tokens. Container boundaries enforce @container isolation rules.

### 4. Interactive Flow & States
Operational lifecycle events and discrete state transitions for SourceLocationMapper.

### Layer 1: Domain State & Signal Model
Domain state vector representing SourceLocationMapper status, AST references, and typed signal models.

### Layer 2: Logic & Safety State Management
Deterministic state machine solver and safety monitor logic for SourceLocationMapper.

### Layer 3: Presentation & Actuator Interface Binding
Presentation interface binding and diagnostic event routing for SourceLocationMapper.

## Acceptance Criteria (BDD)
- [ ] AC-01: Given nominal SourceLocationMapper domain state, When command or input signals are received, Then updates internal state and signal outputs within declared bounds.
- [ ] AC-02: Given operational state, When SourceLocationMapper evaluates constraint 'operational_invariants', Then asserts invariant satisfaction without violation.
- [ ] AC-03: Given malformed input signals or boundary condition violation, When SourceLocationMapper error handling is invoked, Then transitions safely to failsafe fallback state without corruption.

## Source References
Formal component specifications and normative systems engineering requirements:
- Subsystem Requirements: `schema/subsystems/diagnostic_error_catalog/requirements.sysml`
- Subsystem Architecture: `schema/subsystems/diagnostic_error_catalog/architecture.sysml`
- Normative Systems Engineering Standard: ISO/IEC/IEEE 15288:2023 §6.4.3 Architecture Definition Process

Component behavior and interface definitions derive from `schema/model.sysml` pursuant to ISO/IEC/IEEE 15288.
