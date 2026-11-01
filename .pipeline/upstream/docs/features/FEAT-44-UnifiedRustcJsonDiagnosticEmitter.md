| Attribute | Specification Detail |
| :--- | :--- |
| **Issue ID** | TBD |
| **Title** | Feature 44: [BP 08 Lossless Rust CST Codemod Engine and Unified rustc Compatible JSON Diagnostics] UnifiedRustcJsonDiagnosticEmitter |
| **Type** | feature |
| **Parent Epic** | [EPIC-08-BP_08_Lossless_Rust_CST_Codemod_Engine_and_Unified_rustc_Compatible_JSON_Diagnostics](../epics/EPIC-08-BP_08_Lossless_Rust_CST_Codemod_Engine_and_Unified_rustc_Compatible_JSON_Diagnostics.md) |
| **Interface Type** | Physical / Structural / Internal Bus |
| **Schema Containers** | `BP_08_Lossless_Rust_CST_Codemod_Engine_and_Unified_rustc_Compatible_JSON_Diagnostics::UnifiedRustcJsonDiagnosticEmitter` |
| **Generation Mode** | subagent |
| **Specification Source** | [schema/model.sysml](../../schema/model.sysml) |

# Feature 44: [BP 08 Lossless Rust CST Codemod Engine and Unified rustc Compatible JSON Diagnostics] UnifiedRustcJsonDiagnosticEmitter

## UML Class Diagram

```mermaid
classDiagram
    class UnifiedRustcJsonDiagnosticEmitter {
        +String finding_in (in)
        +String ndjson_out (out)
        +String machine_fix_inout (inout)
        +Integer cli_binary_count
        +Boolean zero_ansi_in_json_mode
        +Boolean machine_applicable_only
        +String envelope_reason
    }
```

## Interface Requirements
### 1. Test Data Shape
```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "title": "UnifiedRustcJsonDiagnosticEmitterPayloadSchema",
  "type": "object",
  "properties": {
    "part": {
      "type": "string",
      "const": "UnifiedRustcJsonDiagnosticEmitter"
    },
    "status": {
      "type": "string",
      "enum": ["nominal", "degraded", "fault"]
    },
    "cli_binary_count": {
      "type": "integer",
      "x-sysml-type": "Integer",
      "description": "Declared attribute cli_binary_count (Integer)",
      "default": 5
    },
    "zero_ansi_in_json_mode": {
      "type": "boolean",
      "x-sysml-type": "Boolean",
      "description": "Suppresses all ANSI escape sequences and multi-line unstructured banner logs when `--message-format=json` is active. [unit: flag]",
      "default": true
    },
    "machine_applicable_only": {
      "type": "boolean",
      "x-sysml-type": "Boolean",
      "description": "Restricts `--apply-machine-fixes` exclusively to disjoint child suggestions tagged `\"MachineApplicable\"`. [unit: flag]",
      "default": true
    },
    "envelope_reason": {
      "type": "string",
      "x-sysml-type": "String",
      "description": "Canonical top-level JSON discriminator matching `rustc` and `cargo check --message-format=json`. [unit: schema]",
      "default": "compiler-message"
    },
    "finding_in": {
      "type": "object",
      "properties": {
        "direction": { "type": "string", "const": "in" },
        "port_type": { "type": "string", "const": "String" }
      },
      "required": ["direction", "port_type"]
    },
    "ndjson_out": {
      "type": "object",
      "properties": {
        "direction": { "type": "string", "const": "out" },
        "port_type": { "type": "string", "const": "String" }
      },
      "required": ["direction", "port_type"]
    },
    "machine_fix_inout": {
      "type": "object",
      "properties": {
        "direction": { "type": "string", "const": "inout" },
        "port_type": { "type": "string", "const": "String" }
      },
      "required": ["direction", "port_type"]
    }
  },
  "required": [
    "part",
    "status",
    "finding_in",
    "ndjson_out",
    "machine_fix_inout"
  ],
  "additionalProperties": false
}
```

### 2. Validation & Constraints
Declared structural attributes, boundary ports, invariants, and allocated requirements for classifier `UnifiedRustcJsonDiagnosticEmitter`:

| AST Member Kind | Identifier | Declared Type / Direction | Default / Invariant Contract |
| :--- | :--- | :--- | :--- |
| Attribute | `cli_binary_count` | `Integer` | Default `5` |
| Attribute | `zero_ansi_in_json_mode` | `Boolean` | Default `true` |
| Attribute | `machine_applicable_only` | `Boolean` | Default `true` |
| Attribute | `envelope_reason` | `String` | Default `"compiler-message"` |
| Port | `finding_in` | `in String` | Boundary interface contract |
| Port | `ndjson_out` | `out String` | Boundary interface contract |
| Port | `machine_fix_inout` | `inout String` | Boundary interface contract |

### 3. Visual Layout & Arrangement
Structural composition and boundary topology for classifier `UnifiedRustcJsonDiagnosticEmitter` organize no nested subparts declared in schema paired with boundary ports `in finding_in : String`, `out ndjson_out : String`, `inout machine_fix_inout : String`.

### 4. Interactive Flow & States
Operational lifecycle events and discrete state transitions for UnifiedRustcJsonDiagnosticEmitter.

### Layer 1: Domain State & Signal Model
Classifier `UnifiedRustcJsonDiagnosticEmitter` encapsulates typed domain state attributes `cli_binary_count : Integer`, `zero_ansi_in_json_mode : Boolean`, `machine_applicable_only : Boolean`, `envelope_reason : String` alongside operational lifecycle status.

### Layer 2: Logic & Safety State Management
Classifier `UnifiedRustcJsonDiagnosticEmitter` governs stateless structural transition evaluation enforcing no additional invariant guards declared in schema.

### Layer 3: Presentation & Actuator Interface Binding
Classifier `UnifiedRustcJsonDiagnosticEmitter` binds external and behavioral interfaces across port `in finding_in : String`, port `out ndjson_out : String`, port `inout machine_fix_inout : String`.

## Acceptance Criteria (BDD)
- [ ] AC-01: Given valid `UnifiedRustcJsonDiagnosticEmitter` initialization inputs for `cli_binary_count` (`Integer`), `zero_ansi_in_json_mode` (`Boolean`), `machine_applicable_only` (`Boolean`), `envelope_reason` (`String`) (no additional constraints specified in schema), When `UnifiedRustcJsonDiagnosticEmitter` commits typed domain state, Then declared state attributes hold verified values.
- [ ] AC-02: Given malformed or out-of-domain input values for `cli_binary_count` (`Integer`), `zero_ansi_in_json_mode` (`Boolean`), `machine_applicable_only` (`Boolean`), `envelope_reason` (`String`), When `UnifiedRustcJsonDiagnosticEmitter` validates incoming state updates, Then `UnifiedRustcJsonDiagnosticEmitter` rejects the invalid state update and emits a diagnostic via diagnostic port `ndjson_out`.
- [ ] AC-03: Given nominal signal arrival across `finding_in` (`in String`), `ndjson_out` (`out String`), `machine_fix_inout` (`inout String`), When `UnifiedRustcJsonDiagnosticEmitter` processes the declared interface transaction, Then output state conforms to the structural classifier contract.
- [ ] AC-04: Given an invalid or unrecognized interface frame across `finding_in` (`in String`), `ndjson_out` (`out String`), `machine_fix_inout` (`inout String`), When `UnifiedRustcJsonDiagnosticEmitter` boundary validation executes, Then `UnifiedRustcJsonDiagnosticEmitter` refuses the malformed frame via diagnostic port `ndjson_out` and preserves prior verified state.

## Source References
Formal component specifications and normative systems engineering requirements:
- System Architecture Model: `schema/model.sysml`
- Normative Systems Engineering Standard: ISO/IEC/IEEE 15288:2023 §6.4.3 Architecture Definition Process

Component behavior and interface definitions derive from `schema/model.sysml` pursuant to ISO/IEC/IEEE 15288.
