# SysML v2 Grammar & Abstract Syntax Tree (AST) Specification

## 1. Overview & Type System Architecture

The SysML v2 AST model provides strongly-typed, schema-driven representations of all KerML and SysML v2 engineering entities.
All types derive `Debug`, `Clone`, `PartialEq`, `Serialize`, and `Deserialize` to guarantee seamless JSON serialization, round-trip fidelity, and compatibility with the DEAP pipeline digest format (`.pipeline/schema-digest.json`).

---

## 2. Core AST Data Structures

### 2.1 PackageDef (`SysMLPackage`)
Top-level container for SysML v2 models and sub-packages:
- `name: String`
- `doc: Option<String>`
- `parent_package: Option<String>`
- `packages: Vec<PackageDef>`
- `part_defs: Vec<PartDef>`
- `port_defs: Vec<PortDef>`
- `attribute_defs: Vec<AttributeDef>`
- `action_defs: Vec<ActionDef>`
- `operation_defs: Vec<OperationDef>`
- `capability_defs: Vec<CapabilityDef>`
- `interaction_defs: Vec<InteractionDef>`
- `constraint_defs: Vec<ConstraintDef>`
- `test_case_defs: Vec<TestCaseDef>`
- `requirement_defs: Vec<RequirementDef>`
- `state_defs: Vec<StateDef>`
- `use_case_defs: Vec<UseCaseDef>`
- `item_defs: Vec<ItemDef>`
- `hazard_defs: Vec<HazardDef>`
- `risk_defs: Vec<RiskDef>`
- `connection_defs: Vec<ConnectionDef>`

### 2.2 PartDef (`SysMLPart`)
Structural components (blocks/parts) and usages:
- `name: String`
- `doc: Option<String>`
- `is_def: bool` (true for `part def`, false for part usage `part`)
- `type_name: Option<String>` (specialization or usage type)
- Nested collections matching `PackageDef` capabilities (attributes, ports, actions, constraints, requirements, states, subparts, connections, items, hazards, risks).

### 2.3 PortDef (`SysMLPortDef`)
Interface interaction points:
- `name: String`
- `type_name: String`
- `direction: String` (`in`, `out`, `inout`)
- `is_conjugated: bool` (`~` prefix)
- `doc: Option<String>`
- `port_category: String`
- `protocol_family: String`
- `electrical_attributes: BTreeMap<String, Value>`
- `item_flows: Vec<FlowDef>`

### 2.4 AttributeDef
Typed primitive or structured values:
- `name: String`
- `type_name: String`
- `default_value: Option<String>`
- `doc: Option<String>`

### 2.5 ActionDef & OperationDef
Behavioral execution specifications:
- `name: String`
- `doc: Option<String>`
- `in_params: Vec<AttributeDef>`
- `out_params: Vec<AttributeDef>`
- `parameters: Vec<AttributeDef>`
- `performer: Option<String>`
- `steps: Vec<String>`
- `is_def: bool`

### 2.6 StateDef & TransitionDef
Real-time statechart FSM models:
- StateDef: `name`, `doc`, `entry_action`, `do_action`, `exit_action`, `transitions: Vec<TransitionDef>`
- TransitionDef: `name`, `source`, `target`, `trigger`, `guard`, `effect`

### 2.7 ConstraintDef & CalculationDef
Formal mathematical invariants, KaTeX expressions, bounds, and assertions:
- `name: String`
- `expression: String`
- `parameters: Vec<String>`
- `is_assertion: bool` (`assert constraint` vs `constraint def`)
- `doc: Option<String>`

### 2.8 RequirementDef
Formal safety, regulatory, and system requirements:
- `name: String`
- `req_id: String`
- `text: String`
- `doc: Option<String>`
- `assumes: Vec<String>`
- `requires: Vec<String>`
- `verified_by: Vec<String>`
- `satisfied_by: Vec<String>`

### 2.9 ConnectionDef & FlowDef
Topological links, bus routing, and message flows:
- `name: String`
- `source_port: String`
- `target_port: String`
- `source_part: Option<String>`
- `target_part: Option<String>`
- `is_flow: bool`
- `protocol: Option<String>`
- `latency_ms: Option<f64>`
- `severity: i32`
- `item_flow_ref: Option<String>`
- `doc: Option<String>`

### 2.10 ItemDef
Data payloads and telemetry packets:
- `name: String`
- `doc: Option<String>`
- `attributes: Vec<AttributeDef>`

---

## 3. Recursive Descent Parsing Strategy

1. **Top-Down Block and Statement Splitting**:
   - The scanner emits tokens preserving nesting depth for `{ ... }` blocks and `;` terminators.
2. **Context-Sensitive Dispatch**:
   - Headers determine construct type (`package`, `part def`, `port`, `action`, etc.).
   - Body declarations are parsed recursively into typed containers.
3. **Expression Grammar**:
   - Infix expression parser handles operator precedence (`+`, `-`, `*`, `/`, `<=`, `>=`, `==`, `!=`, `and`, `or`).
   - Supports LaTeX / KaTeX mathematical bounds inside `constraint def` bodies.
