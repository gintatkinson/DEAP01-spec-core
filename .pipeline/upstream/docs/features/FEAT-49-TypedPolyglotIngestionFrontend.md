| Attribute | Specification Detail |
| :--- | :--- |
| **Issue ID** | #755 |
| **Title** | Feature 49: [BP 10 Incremental Lossless SysML v2 and Polyglot Compiler Engine] TypedPolyglotIngestionFrontend |
| **Type** | feature |
| **Parent Epic** | [EPIC-10-BP_10_Incremental_Lossless_SysML_v2_and_Polyglot_Compiler_Engine](../epics/EPIC-10-BP_10_Incremental_Lossless_SysML_v2_and_Polyglot_Compiler_Engine.md) |
| **Interface Type** | Physical / Structural / Internal Bus |
| **Schema Containers** | `BP_10_Incremental_Lossless_SysML_v2_and_Polyglot_Compiler_Engine::TypedPolyglotIngestionFrontend` |
| **Generation Mode** | subagent |
| **Specification Source** | [schema/model.sysml](../../schema/model.sysml) |

# Feature 49: [BP 10 Incremental Lossless SysML v2 and Polyglot Compiler Engine] TypedPolyglotIngestionFrontend

## UML Class Diagram

```mermaid
classDiagram
    class TypedPolyglotIngestionFrontend {
        +String polyglot_source_in (in)
        +String lowered_hir_out (out)
        +String ingest_diag_out (out)
        +Integer polyglot_frontend_count
        +Integer rustc_private_feature_count
        +Boolean json_and_ariadne_span_diagnostics
        +Boolean zero_backtracking_lexers
    }
```

## Interface Requirements
### 1. Test Data Shape
```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "title": "TypedPolyglotIngestionFrontendPayloadSchema",
  "type": "object",
  "properties": {
    "part": {
      "type": "string",
      "const": "TypedPolyglotIngestionFrontend"
    },
    "status": {
      "type": "string",
      "enum": ["nominal", "degraded", "fault"]
    },
    "polyglot_frontend_count": {
      "type": "integer",
      "x-sysml-type": "Integer",
      "description": "Declared attribute polyglot_frontend_count (Integer)",
      "default": 6
    },
    "rustc_private_feature_count": {
      "type": "integer",
      "x-sysml-type": "Integer",
      "description": "Enforces `ZERO #![feature(rustc_private)]` occurrences across all workspace crates under `./crates/ *`. [unit: crates]",
      "default": 0
    },
    "json_and_ariadne_span_diagnostics": {
      "type": "boolean",
      "x-sysml-type": "Boolean",
      "description": "Emits deterministic `ariadne` byte-span reports and `--message-format=json` diagnostics (`DEAP-E1005`) across all polyglot formats. [unit: flag]",
      "default": true
    },
    "zero_backtracking_lexers": {
      "type": "boolean",
      "x-sysml-type": "Boolean",
      "description": "Enforces linear-time `logos` DFA / `rowan` CST and typed `serde_json` AST parsing with zero hand-rolled line lexers or NFA backtracking (`Check 34`). [unit: flag]",
      "default": true
    },
    "polyglot_source_in": {
      "type": "object",
      "properties": {
        "direction": { "type": "string", "const": "in" },
        "port_type": { "type": "string", "const": "String" }
      },
      "required": ["direction", "port_type"]
    },
    "lowered_hir_out": {
      "type": "object",
      "properties": {
        "direction": { "type": "string", "const": "out" },
        "port_type": { "type": "string", "const": "String" }
      },
      "required": ["direction", "port_type"]
    },
    "ingest_diag_out": {
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
    "polyglot_source_in",
    "lowered_hir_out",
    "ingest_diag_out"
  ],
  "additionalProperties": false
}
```

### 2. Validation & Constraints
Declared structural attributes, boundary ports, invariants, and allocated requirements for classifier `TypedPolyglotIngestionFrontend`:

| AST Member Kind | Identifier | Declared Type / Direction | Default / Invariant Contract |
| :--- | :--- | :--- | :--- |
| Attribute | `polyglot_frontend_count` | `Integer` | Default `6` |
| Attribute | `rustc_private_feature_count` | `Integer` | Default `0` |
| Attribute | `json_and_ariadne_span_diagnostics` | `Boolean` | Default `true` |
| Attribute | `zero_backtracking_lexers` | `Boolean` | Default `true` |
| Port | `polyglot_source_in` | `in String` | Boundary interface contract |
| Port | `lowered_hir_out` | `out String` | Boundary interface contract |
| Port | `ingest_diag_out` | `out String` | Boundary interface contract |

### 3. Visual Layout & Arrangement
Structural composition and boundary topology for classifier `TypedPolyglotIngestionFrontend` organize no nested subparts declared in schema paired with boundary ports `in polyglot_source_in : String`, `out lowered_hir_out : String`, `out ingest_diag_out : String`.

### 4. Interactive Flow & States
Operational lifecycle events and discrete state transitions for TypedPolyglotIngestionFrontend.

### Layer 1: Domain State & Signal Model
Classifier `TypedPolyglotIngestionFrontend` encapsulates typed domain state attributes `polyglot_frontend_count : Integer`, `rustc_private_feature_count : Integer`, `json_and_ariadne_span_diagnostics : Boolean`, `zero_backtracking_lexers : Boolean` alongside operational lifecycle status.

### Layer 2: Logic & Safety State Management
Classifier `TypedPolyglotIngestionFrontend` governs stateless structural transition evaluation enforcing no additional invariant guards declared in schema.

### Layer 3: Presentation & Actuator Interface Binding
Classifier `TypedPolyglotIngestionFrontend` binds external and behavioral interfaces across port `in polyglot_source_in : String`, port `out lowered_hir_out : String`, port `out ingest_diag_out : String`.

### Associated Use Cases & User Stories

#### Associated Use Cases
- None allocated (structural subsystem component)

#### Associated User Stories
- [ ] #815 - [User Story 66: resolve polyglot backend](../user-stories/US-66-resolve_polyglot_backend.md)
- [ ] #816 - [User Story 67: normalize line endings](../user-stories/US-67-normalize_line_endings.md)
- [ ] #817 - [User Story 68: parse lossless sysml cst](../user-stories/US-68-parse_lossless_sysml_cst.md)
- [ ] #818 - [User Story 69: verify lossless cst invariants](../user-stories/US-69-verify_lossless_cst_invariants.md)
- [ ] #819 - [User Story 70: evaluate hir early cutoff](../user-stories/US-70-evaluate_hir_early_cutoff.md)
- [ ] #820 - [User Story 71: apply formatting preserving splices](../user-stories/US-71-apply_formatting_preserving_splices.md)
- [ ] #821 - [User Story 72: render ariadne diagnostic](../user-stories/US-72-render_ariadne_diagnostic.md)

## Acceptance Criteria (BDD)
- [ ] AC-01: Given valid `TypedPolyglotIngestionFrontend` initialization inputs for `polyglot_frontend_count` (`Integer`), `rustc_private_feature_count` (`Integer`), `json_and_ariadne_span_diagnostics` (`Boolean`), `zero_backtracking_lexers` (`Boolean`) (no additional constraints specified in schema), When `TypedPolyglotIngestionFrontend` commits typed domain state, Then declared state attributes hold verified values.
- [ ] AC-02: Given malformed or out-of-domain input values for `polyglot_frontend_count` (`Integer`), `rustc_private_feature_count` (`Integer`), `json_and_ariadne_span_diagnostics` (`Boolean`), `zero_backtracking_lexers` (`Boolean`), When `TypedPolyglotIngestionFrontend` validates incoming state updates, Then `TypedPolyglotIngestionFrontend` rejects the invalid state update and emits a diagnostic via diagnostic port `lowered_hir_out`.
- [ ] AC-03: Given nominal signal arrival across `polyglot_source_in` (`in String`), `lowered_hir_out` (`out String`), `ingest_diag_out` (`out String`), When `TypedPolyglotIngestionFrontend` processes the declared interface transaction, Then output state conforms to the structural classifier contract.
- [ ] AC-04: Given an invalid or unrecognized interface frame across `polyglot_source_in` (`in String`), `lowered_hir_out` (`out String`), `ingest_diag_out` (`out String`), When `TypedPolyglotIngestionFrontend` boundary validation executes, Then `TypedPolyglotIngestionFrontend` refuses the malformed frame via diagnostic port `lowered_hir_out` and preserves prior verified state.

## Source References
Formal component specifications and normative systems engineering requirements:
- System Architecture Model: `schema/model.sysml`
- Normative Systems Engineering Standard: ISO/IEC/IEEE 15288:2023 §6.4.3 Architecture Definition Process

Component behavior and interface definitions derive from `schema/model.sysml` pursuant to ISO/IEC/IEEE 15288.
