<!-- Copyright Gint Atkinson, gint.atkinson@gmail.com -->

# Rule: SysML v2 Model-as-SSOT & Non-Drifting Elaboration Invariant

**ALWAYS enforce:** SysML v2 is the 100% Single Source of Truth (SSOT) for all system architecture, behavior, interface definitions, and safety invariants in the Digital Engineering Autonomous Pipeline (DEAP). All downstream engineering artifacts and backlog specifications must be formally declared in and synchronized with the SysML v2 Abstract Syntax Tree (AST).

## Scope and Normative Authority

**This file is the single normative home for SysML v2 Single Source of Truth (SSOT) completeness and non-drifting elaboration invariants.**

The SysML v2 model (`.pipeline/schema.sysml` and underlying AST definitions) is the authoritative, mathematically grounded system model. Downstream specifications (Epics, Features, User Stories, and Use Cases), verification plans, and code generation targets are formal projections and elaborations of this model.

## Primary Commercial Toolchain Integration Context

This project explicitly declares **MATLAB / Simulink / Stateflow / Embedded Coder** as the Primary Tier-1 Commercial Toolchain Integration Context.

- **Model-Based Design (MBD)**: High-level architectural blocks and dynamic plant/controller interactions defined in SysML v2 map directly to Simulink block diagrams and subsystem hierarchies.
- **Control Law Synthesis & State Machines**: SysML v2 statecharts, action sequences, and operational modes drive Stateflow truth tables, state transition diagrams, and discrete-event supervisors.
- **High-Integrity Code Generation**: Structural definitions, typed ports, and algorithmic actions feed Embedded Coder for DO-178C / DO-331 qualified C and SPARK Ada safety-critical embedded software synthesis.
- **Verification & Validation**: Simulink Test, Simulink Design Verifier (SLDV), and Polyspace static analysis enforce formal property checking against SysML v2 requirement definitions and STPA/FMECA invariants.

## Formal Specification Declaration Invariants

Every downstream engineering specification artifact MUST be formally rooted in and mapped to corresponding SysML v2 AST elements:

1. **Epics & Features -> Structural Definitions (`package`, `part def`, `item def`)**:
   - Every Epic must map to a top-level or architectural `package` defining subsystem boundaries.
   - Every Feature specification must map to at least one `part def` (structural component/block) or `item def` (data definition/message payload).
   - Component hierarchies, composition relationships, and data item schemas must be formally defined in SysML v2 before Feature elaboration.

2. **User Stories -> Behavioral & Interface Definitions (`action def`, `state def`, `port def`)**:
   - Every User Story must map to formal behavioral elements: discrete computational operations (`action def`), lifecycle and operational modes (`state def`), or interface boundaries (`port def`).
   - Given-When-Then BDD scenarios must execute against the actions, states, and ports declared on the parent component's SysML definition.
   - Parameter inputs (`in`), outputs (`out`), and flow directions must be explicitly typed.

3. **Use Cases -> Formal Use Case Definitions (`use case def`)**:
   - Every Use Case specification must be declared as a formal `use case def` in the SysML v2 model.
   - Each `use case def` MUST formally specify:
     - `subject`: The system or subsystem part under design (`part def`) providing the capability.
     - `actor`: Primary initiating actors and secondary participating actors (bound via typed ports/interfaces).
     - `objective`: Formal operational goal and success criteria.
     - `include` / `extend`: Formal relationships connecting modular or conditional sub-use-cases.

4. **Safety Invariants -> Formal Requirement Definitions (`requirement def`)**:
   - All safety constraints derived from STPA (System-Theoretic Process Analysis: Unsafe Control Actions, Loss Scenarios) and FMECA (Failure Mode, Effects, and Criticality Analysis) MUST be modeled as formal `requirement def` nodes.
   - Requirements must define unique identifiers, textual statements, formal constraint expressions (`assume` / `require`), and risk/hazard IDs.
   - Every `requirement def` must declare explicit `satisfy` or `verify` dependencies linking to the implementing `part def`, `action def`, or `state def`.

## Zero Model Drift & Bidirectional Parity Mandate

SysML v2 model parity is non-negotiable across all lifecycle phases:

- **Tandem Elaboration**: When downstream specifications (Epics, Features, User Stories, Use Cases) are created, refined, or modified during pipeline execution, the SysML v2 model (`.pipeline/schema.sysml`) MUST be updated in tandem.
- **100% Bidirectional Parity**: No specification element, state transition, port interface, or action signature may exist in backlog Markdown without a corresponding SysML v2 AST declaration, and no SysML v2 model element may remain unextracted/untraced in specifications.
- **Re-Generation & Verification Gate**: Any modification to functional requirements or behavioral flows must regenerate the SysML v2 AST digest (`.pipeline/schema-digest.json`) and pass parity auditor validation before commits are accepted.

## Prohibition of Heuristic Prose Parsing

- **Formal AST Priority**: Tools, linters, validators, and code generators MUST operate directly on the structured SysML v2 Abstract Syntax Tree (AST) nodes.
- **Zero Heuristic Parsing**: Inferring system structure, types, ports, or message signatures from unstructured natural language Markdown prose is strictly prohibited when formal SysML v2 AST nodes exist.
- **Deterministic Toolchain Ingestion**: Downstream synthesis tools (including MATLAB/Simulink bridges, state machine generators, and interface validators) must consume the validated AST representation to ensure mathematical determinism and prevent specification ambiguity.

## Pure Schema-Driven & Domain-Agnostic Compilation

- **Generic AST Invariant**: SysML v2 parsing, AST extraction, and verification gates in DEAP operate purely on generic AST tokens (`package`, `part def`, `item def`, `action def`, `state def`, `port def`, `requirement def`, `use case def`, and their typed relationships) without domain-specific heuristics, assumptions, or hardcoded entity names.
- **Zero Domain-Biasing**: The MBSE compiler and verification tools MUST NOT encode domain-specific rules (e.g., flight controllers, automotive sensors, medical infusion logic) into parsing or checking algorithms. All semantics are derived purely and deterministically from user-provided schema definitions in `schema/`.
- **Mathematical Determinism**: Universal token-based processing ensures absolute mathematical determinism and cross-domain portability across aerospace, automotive, defense, medical, and industrial automation engineering domains.

## Automated Closed-Loop Reverse Synchronization

To guarantee zero model drift between agile specification backlogs and the architectural model, DEAP mandates the **Automated Closed-Loop Reverse Synchronization standard**:

- **Reverse Sync Compilation Mandate**: Whenever subagents or engineers elaborate, refine, or add new structural elements, state transitions, port interfaces, action signatures, use cases, or safety constraints in markdown specifications (`docs/epics`, `docs/features`, `docs/user-stories`, `docs/use-cases`), the reverse synchronization compiler MUST be executed:
  ```bash
  python3 scripts/compile_sysml.py --reverse-sync
  ```
- **Automated AST Extraction & Elaboration**: The `--reverse-sync` engine mechanically extracts:
  1. *Structural Blocks & Items*: Ingests YAML frontmatter metadata and Mermaid class diagrams, compiling newly declared components and payload schemas into `part def` and `item def` nodes.
  2. *Behavioral Statecharts & Actions*: Ingests Mermaid state diagrams and Given-When-Then BDD scenarios, compiling state transitions, guards, events, and action signatures with typed parameters into `state def` and `action def` nodes.
  3. *Use Cases & Interaction Flows*: Ingests Use Case realization tables, actor/subject bindings, and include/extend trees, compiling them into formal `use case def` nodes.
  4. *Safety & RTA Invariants*: Ingests STPA UCA and FMECA tables, compiling safety rules into formal `requirement def`, `constraint def`, and `assert constraint` nodes.
- **Non-Destructive Semantic Merge & Digest Regeneration**: The AST merge engine merges extracted AST deltas into `.pipeline/schema.sysml` without destroying existing formal invariants, serializes the updated model via canonical `to_sysml()` emission, and regenerates the SHA-256 cryptographic digest in `.pipeline/schema-digest.json`.
- **Pre-Commit Verification Lock**: All reverse-synchronized models MUST immediately undergo verification via `verify_model_coverage.py` across all 22 parity gates before commits or pull requests are merged.

## Why

Treating textual specifications and models as separate entities inevitably causes specification drift, where documentation diverges from the architectural model and code generation toolchains. Enforcing SysML v2 as the 100% Single Source of Truth guarantees model integrity, enables automated bidirectional validation, and provides an unbroken digital thread from high-level safety invariants to generated DO-178C C/SPARK Ada flight code.
