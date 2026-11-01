| Attribute | Specification Detail |
| :--- | :--- |
| **Issue ID** | TBD |
| **Title** | Feature 48: [BP 09 Multi Stage Compiler and Formal Verification Gatekeeper] KaniCbmcProofSynthesizer |
| **Type** | feature |
| **Parent Epic** | [EPIC-09-BP_09_Multi_Stage_Compiler_and_Formal_Verification_Gatekeeper](../epics/EPIC-09-BP_09_Multi_Stage_Compiler_and_Formal_Verification_Gatekeeper.md) |
| **Interface Type** | Physical / Structural / Internal Bus |
| **Schema Containers** | `BP_09_Multi_Stage_Compiler_and_Formal_Verification_Gatekeeper::KaniCbmcProofSynthesizer` |
| **Generation Mode** | subagent |
| **Specification Source** | [schema/model.sysml](../../schema/model.sysml) |

# Feature 48: [BP 09 Multi Stage Compiler and Formal Verification Gatekeeper] KaniCbmcProofSynthesizer

## UML Class Diagram

```mermaid
classDiagram
    class KaniCbmcProofSynthesizer {
        +String sysml_safety_ast_in (in)
        +String kani_proof_module_out (out)
        +String cbmc_proof_receipt_out (out)
        +Integer max_kani_counterexamples
        +Integer default_kani_unwind_bound
        +Boolean structured_prettyplease_codegen
    }
```

## Interface Requirements
### 1. Test Data Shape
```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "title": "KaniCbmcProofSynthesizerPayloadSchema",
  "type": "object",
  "properties": {
    "part": {
      "type": "string",
      "const": "KaniCbmcProofSynthesizer"
    },
    "status": {
      "type": "string",
      "enum": ["nominal", "degraded", "fault"]
    },
    "max_kani_counterexamples": {
      "type": "integer",
      "x-sysml-type": "Integer",
      "description": "Declared attribute max_kani_counterexamples (Integer)",
      "default": 0
    },
    "default_kani_unwind_bound": {
      "type": "integer",
      "x-sysml-type": "Integer",
      "description": "Deterministic loop unwinding ceiling emitted into `#[kani::unwind(N)]` when AST loops carry finite static bounds. [unit: iterations]",
      "default": 16
    },
    "structured_prettyplease_codegen": {
      "type": "boolean",
      "x-sysml-type": "Boolean",
      "description": "Emits `#[cfg(kani)]` proof modules via `syn` + `quote` + `prettyplease` without raw string-template concatenation. [unit: flag]",
      "default": true
    },
    "sysml_safety_ast_in": {
      "type": "object",
      "properties": {
        "direction": { "type": "string", "const": "in" },
        "port_type": { "type": "string", "const": "String" }
      },
      "required": ["direction", "port_type"]
    },
    "kani_proof_module_out": {
      "type": "object",
      "properties": {
        "direction": { "type": "string", "const": "out" },
        "port_type": { "type": "string", "const": "String" }
      },
      "required": ["direction", "port_type"]
    },
    "cbmc_proof_receipt_out": {
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
    "sysml_safety_ast_in",
    "kani_proof_module_out",
    "cbmc_proof_receipt_out"
  ],
  "additionalProperties": false
}
```

### 2. Validation & Constraints
Declared structural attributes, boundary ports, invariants, and allocated requirements for classifier `KaniCbmcProofSynthesizer`:

| AST Member Kind | Identifier | Declared Type / Direction | Default / Invariant Contract |
| :--- | :--- | :--- | :--- |
| Attribute | `max_kani_counterexamples` | `Integer` | Default `0` |
| Attribute | `default_kani_unwind_bound` | `Integer` | Default `16` |
| Attribute | `structured_prettyplease_codegen` | `Boolean` | Default `true` |
| Port | `sysml_safety_ast_in` | `in String` | Boundary interface contract |
| Port | `kani_proof_module_out` | `out String` | Boundary interface contract |
| Port | `cbmc_proof_receipt_out` | `out String` | Boundary interface contract |

### 3. Visual Layout & Arrangement
Structural composition and boundary topology for classifier `KaniCbmcProofSynthesizer` organize no nested subparts declared in schema paired with boundary ports `in sysml_safety_ast_in : String`, `out kani_proof_module_out : String`, `out cbmc_proof_receipt_out : String`.

### 4. Interactive Flow & States
Operational lifecycle events and discrete state transitions for KaniCbmcProofSynthesizer.

### Layer 1: Domain State & Signal Model
Classifier `KaniCbmcProofSynthesizer` encapsulates typed domain state attributes `max_kani_counterexamples : Integer`, `default_kani_unwind_bound : Integer`, `structured_prettyplease_codegen : Boolean` alongside operational lifecycle status.

### Layer 2: Logic & Safety State Management
Classifier `KaniCbmcProofSynthesizer` governs stateless structural transition evaluation enforcing no additional invariant guards declared in schema.

### Layer 3: Presentation & Actuator Interface Binding
Classifier `KaniCbmcProofSynthesizer` binds external and behavioral interfaces across port `in sysml_safety_ast_in : String`, port `out kani_proof_module_out : String`, port `out cbmc_proof_receipt_out : String`.

## Acceptance Criteria (BDD)
- [ ] AC-01: Given valid `KaniCbmcProofSynthesizer` initialization inputs for `max_kani_counterexamples` (`Integer`), `default_kani_unwind_bound` (`Integer`), `structured_prettyplease_codegen` (`Boolean`) (no additional constraints specified in schema), When `KaniCbmcProofSynthesizer` commits typed domain state, Then declared state attributes hold verified values.
- [ ] AC-02: Given malformed or out-of-domain input values for `max_kani_counterexamples` (`Integer`), `default_kani_unwind_bound` (`Integer`), `structured_prettyplease_codegen` (`Boolean`), When `KaniCbmcProofSynthesizer` validates incoming state updates, Then `KaniCbmcProofSynthesizer` rejects the invalid state update and emits a diagnostic via diagnostic port `kani_proof_module_out`.
- [ ] AC-03: Given nominal signal arrival across `sysml_safety_ast_in` (`in String`), `kani_proof_module_out` (`out String`), `cbmc_proof_receipt_out` (`out String`), When `KaniCbmcProofSynthesizer` processes the declared interface transaction, Then output state conforms to the structural classifier contract.
- [ ] AC-04: Given an invalid or unrecognized interface frame across `sysml_safety_ast_in` (`in String`), `kani_proof_module_out` (`out String`), `cbmc_proof_receipt_out` (`out String`), When `KaniCbmcProofSynthesizer` boundary validation executes, Then `KaniCbmcProofSynthesizer` refuses the malformed frame via diagnostic port `kani_proof_module_out` and preserves prior verified state.

## Source References
Formal component specifications and normative systems engineering requirements:
- System Architecture Model: `schema/model.sysml`
- Normative Systems Engineering Standard: ISO/IEC/IEEE 15288:2023 §6.4.3 Architecture Definition Process

Component behavior and interface definitions derive from `schema/model.sysml` pursuant to ISO/IEC/IEEE 15288.
