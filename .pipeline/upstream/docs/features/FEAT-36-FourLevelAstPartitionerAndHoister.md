| Attribute | Specification Detail |
| :--- | :--- |
| **Issue ID** | TBD |
| **Title** | Feature 36: [BP 07 Self Hosting 4 Level Modular Multi File Code Generator Blueprint] FourLevelAstPartitionerAndHoister |
| **Type** | feature |
| **Parent Epic** | [EPIC-07-BP_07_Self_Hosting_4_Level_Modular_Multi_File_Code_Generator_Blueprint](../epics/EPIC-07-BP_07_Self_Hosting_4_Level_Modular_Multi_File_Code_Generator_Blueprint.md) |
| **Interface Type** | Physical / Structural / Internal Bus |
| **Schema Containers** | `BP_07_Self_Hosting_4_Level_Modular_Multi_File_Code_Generator_Blueprint::FourLevelAstPartitionerAndHoister` |
| **Generation Mode** | subagent |
| **Specification Source** | [schema/model.sysml](../../schema/model.sysml) |

# Feature 36: [BP 07 Self Hosting 4 Level Modular Multi File Code Generator Blueprint] FourLevelAstPartitionerAndHoister

## UML Class Diagram

```mermaid
classDiagram
    class FourLevelAstPartitionerAndHoister {
        +String validated_ast_in (in)
        +String module_graph_out (out)
        +Integer max_action_steps_per_stage
        +Integer max_transitions_per_handler
        +Integer anf_branch_budget
    }
```

## Interface Requirements
### 1. Test Data Shape
```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "title": "FourLevelAstPartitionerAndHoisterPayloadSchema",
  "type": "object",
  "properties": {
    "part": {
      "type": "string",
      "const": "FourLevelAstPartitionerAndHoister"
    },
    "status": {
      "type": "string",
      "enum": ["nominal", "degraded", "fault"]
    },
    "max_action_steps_per_stage": {
      "type": "integer",
      "x-sysml-type": "Integer",
      "description": "Declared attribute max_action_steps_per_stage (Integer)",
      "default": 6
    },
    "max_transitions_per_handler": {
      "type": "integer",
      "x-sysml-type": "Integer",
      "description": "Maximum outgoing `StateDef::transitions` arms per `step_from_()` handler unit. [unit: arms]",
      "default": 6
    },
    "anf_branch_budget": {
      "type": "integer",
      "x-sysml-type": "Integer",
      "description": "Maximum boolean branching operators retained before hoisting `eval__term_` helpers. [unit: branches]",
      "default": 6
    },
    "validated_ast_in": {
      "type": "object",
      "properties": {
        "direction": { "type": "string", "const": "in" },
        "port_type": { "type": "string", "const": "String" }
      },
      "required": ["direction", "port_type"]
    },
    "module_graph_out": {
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
    "validated_ast_in",
    "module_graph_out"
  ],
  "additionalProperties": false
}
```

### 2. Validation & Constraints
Declared structural attributes, boundary ports, invariants, and allocated requirements for classifier `FourLevelAstPartitionerAndHoister`:

| AST Member Kind | Identifier | Declared Type / Direction | Default / Invariant Contract |
| :--- | :--- | :--- | :--- |
| Attribute | `max_action_steps_per_stage` | `Integer` | Default `6` |
| Attribute | `max_transitions_per_handler` | `Integer` | Default `6` |
| Attribute | `anf_branch_budget` | `Integer` | Default `6` |
| Port | `validated_ast_in` | `in String` | Boundary interface contract |
| Port | `module_graph_out` | `out String` | Boundary interface contract |
| Constraint | `Invariant_REQ_0607_SC_02` | Invariant (`REQ-0607 (REQ_0607_BP_07_Self_Hosting_4_Level_Modular_Multi_File_Code_Generator_Blueprint)`) | Declared structural invariant |
| Constraint | `Invariant_REQ_0607_SC_01` | Invariant (`REQ-0607 (REQ_0607_BP_07_Self_Hosting_4_Level_Modular_Multi_File_Code_Generator_Blueprint)`) | Declared structural invariant |
| Constraint | `Invariant_REQ_0607_REQ_0607` | Invariant (`REQ-0607 (REQ_0607_BP_07_Self_Hosting_4_Level_Modular_Multi_File_Code_Generator_Blueprint)`) | Declared structural invariant |
| Constraint | `Invariant_REQ_0607_SC_03` | Invariant (`REQ-0607 (REQ_0607_BP_07_Self_Hosting_4_Level_Modular_Multi_File_Code_Generator_Blueprint)`) | Declared structural invariant |
| Requirement | `REQ-0607 (REQ_0607_BP_07_Self_Hosting_4_Level_Modular_Multi_File_Code_Generator_Blueprint)` | Allocated Requirement | SelfHostingModularCodegenCompiler (**REQ-0607** / BP-07 / DEAP-BP-DET-07, UUIDv5 6ba7b810-9dad-11d1-80b4-00c04fd430c7, Class P linear Kahn DAG and AST traversal complexity, governed by DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C) eliminates monolithic single-file code |

```sysml
constraint Invariant_REQ_0607_SC_02 {}
constraint Invariant_REQ_0607_SC_01 {}
constraint Invariant_REQ_0607_REQ_0607 {}
constraint Invariant_REQ_0607_SC_03 {}
```

### 3. Visual Layout & Arrangement
Structural composition and boundary topology for classifier `FourLevelAstPartitionerAndHoister` organize no nested subparts declared in schema paired with boundary ports `in validated_ast_in : String`, `out module_graph_out : String`.

### 4. Interactive Flow & States
Operational lifecycle events and discrete state transitions for FourLevelAstPartitionerAndHoister.

### Layer 1: Domain State & Signal Model
Classifier `FourLevelAstPartitionerAndHoister` encapsulates typed domain state attributes `max_action_steps_per_stage : Integer`, `max_transitions_per_handler : Integer`, `anf_branch_budget : Integer` alongside operational lifecycle status.

### Layer 2: Logic & Safety State Management
Classifier `FourLevelAstPartitionerAndHoister` governs stateless structural transition evaluation enforcing invariant obligations `Invariant_REQ_0607_SC_02`, `Invariant_REQ_0607_SC_01`, `Invariant_REQ_0607_REQ_0607`, `Invariant_REQ_0607_SC_03`.

### Layer 3: Presentation & Actuator Interface Binding
Classifier `FourLevelAstPartitionerAndHoister` binds external and behavioral interfaces across port `in validated_ast_in : String`, port `out module_graph_out : String`.

## Acceptance Criteria (BDD)
- [ ] AC-01: Given valid `FourLevelAstPartitionerAndHoister` inputs satisfying `Invariant_REQ_0607_SC_02` [Req `REQ-0607 (REQ_0607_BP_07_Self_Hosting_4_Level_Modular_Multi_File_Code_Generator_Blueprint)`], When `FourLevelAstPartitionerAndHoister` evaluates invariant `Invariant_REQ_0607_SC_02`, Then the invariant holds and verified state is committed.
- [ ] AC-02: Given boundary inputs that violate `Invariant_REQ_0607_SC_02` [Req `REQ-0607 (REQ_0607_BP_07_Self_Hosting_4_Level_Modular_Multi_File_Code_Generator_Blueprint)`], When `FourLevelAstPartitionerAndHoister` evaluates invariant `Invariant_REQ_0607_SC_02`, Then `FourLevelAstPartitionerAndHoister` rejects the invalid transition and routes a diagnostic via diagnostic port `module_graph_out`.
- [ ] AC-03: Given valid `FourLevelAstPartitionerAndHoister` inputs satisfying `Invariant_REQ_0607_SC_01` [Req `REQ-0607 (REQ_0607_BP_07_Self_Hosting_4_Level_Modular_Multi_File_Code_Generator_Blueprint)`], When `FourLevelAstPartitionerAndHoister` evaluates invariant `Invariant_REQ_0607_SC_01`, Then the invariant holds and verified state is committed.
- [ ] AC-04: Given boundary inputs that violate `Invariant_REQ_0607_SC_01` [Req `REQ-0607 (REQ_0607_BP_07_Self_Hosting_4_Level_Modular_Multi_File_Code_Generator_Blueprint)`], When `FourLevelAstPartitionerAndHoister` evaluates invariant `Invariant_REQ_0607_SC_01`, Then `FourLevelAstPartitionerAndHoister` rejects the invalid transition and routes a diagnostic via diagnostic port `module_graph_out`.
- [ ] AC-05: Given valid `FourLevelAstPartitionerAndHoister` inputs satisfying `Invariant_REQ_0607_REQ_0607` [Req `REQ-0607 (REQ_0607_BP_07_Self_Hosting_4_Level_Modular_Multi_File_Code_Generator_Blueprint)`], When `FourLevelAstPartitionerAndHoister` evaluates invariant `Invariant_REQ_0607_REQ_0607`, Then the invariant holds and verified state is committed.
- [ ] AC-06: Given boundary inputs that violate `Invariant_REQ_0607_REQ_0607` [Req `REQ-0607 (REQ_0607_BP_07_Self_Hosting_4_Level_Modular_Multi_File_Code_Generator_Blueprint)`], When `FourLevelAstPartitionerAndHoister` evaluates invariant `Invariant_REQ_0607_REQ_0607`, Then `FourLevelAstPartitionerAndHoister` rejects the invalid transition and routes a diagnostic via diagnostic port `module_graph_out`.
- [ ] AC-07: Given valid `FourLevelAstPartitionerAndHoister` inputs satisfying `Invariant_REQ_0607_SC_03` [Req `REQ-0607 (REQ_0607_BP_07_Self_Hosting_4_Level_Modular_Multi_File_Code_Generator_Blueprint)`], When `FourLevelAstPartitionerAndHoister` evaluates invariant `Invariant_REQ_0607_SC_03`, Then the invariant holds and verified state is committed.
- [ ] AC-08: Given boundary inputs that violate `Invariant_REQ_0607_SC_03` [Req `REQ-0607 (REQ_0607_BP_07_Self_Hosting_4_Level_Modular_Multi_File_Code_Generator_Blueprint)`], When `FourLevelAstPartitionerAndHoister` evaluates invariant `Invariant_REQ_0607_SC_03`, Then `FourLevelAstPartitionerAndHoister` rejects the invalid transition and routes a diagnostic via diagnostic port `module_graph_out`.
- [ ] AC-09: Given nominal typed values for `max_action_steps_per_stage` (`Integer`), `max_transitions_per_handler` (`Integer`), `anf_branch_budget` (`Integer`) across `validated_ast_in` (`in String`), `module_graph_out` (`out String`), When `FourLevelAstPartitionerAndHoister` executes structural interface processing, Then state and output ports conform to declared schema types.
- [ ] AC-10: Given malformed input frames across `max_action_steps_per_stage` (`Integer`), `max_transitions_per_handler` (`Integer`), `anf_branch_budget` (`Integer`) or `validated_ast_in` (`in String`), `module_graph_out` (`out String`), When `FourLevelAstPartitionerAndHoister` boundary validation executes, Then `FourLevelAstPartitionerAndHoister` rejects the malformed input via diagnostic port `module_graph_out`.

## Source References
Formal component specifications and normative systems engineering requirements:
- System Architecture Model: `schema/model.sysml`
- Normative Systems Engineering Standard: ISO/IEC/IEEE 15288:2023 §6.4.3 Architecture Definition Process

Component behavior and interface definitions derive from `schema/model.sysml` pursuant to ISO/IEC/IEEE 15288.
