---
identifier: DEAP-BLUEPRINT-SYSML-003
title: "SysML v2 Ingestion Engine Solution Blueprint (DEAP-BLUEPRINT-SYSML-003)"
version: "1.0.0"
status: "APPROVED / PRODUCTION-GRADE"
created: "2026-08-10"
target_regulatory_frameworks:
  - DO-178C (High-Assurance Software Considerations)
  - DO-254 (Design Assurance Guidance for Complex Electronic Hardware)
  - ARP4754A / ARP4761 (Safety Assessment Process for Complex Integrated Systems)
  - Generic High-Assurance Risk Assessment Profile (SAIL II-VI)
  - ASTM F3269 (Standard Practice for Run-Time Assurance Systems)
---

# SysML v2 Ingestion Engine Solution Blueprint (DEAP-BLUEPRINT-SYSML-003)

## 1. Executive Summary & Architectural Vision

The **Digital Engineering Agentic Pipeline (DEAP)** SysML v2 Ingestion Engine establishes an automated, production-grade Model-Based Systems Engineering (MBSE) ingestion pipeline. It bridges high-level OMG SysML v2 textual and graphical models with the DEAP multi-tier compiler workspace.

```mermaid
flowchart TD
    subgraph "SysML v2 Source Models"
        SysML_Text["SysML v2 Textual Models - .sysml"]
        SysML_Graph["SysML v2 Kernel Metamodel - KerML"]
    end

    subgraph "DEAP Pure Rust Ingestion Engine (deap-ast & deap-core)"
        AST_Parser["LALR(1) / ANTLR4 AST Parser - deap-ast"]
        AST_Cache["Incremental AST Cache - .pipeline/cache/ast/"]
        Semantic_DAG["In-Memory Semantic Graph DAG"]
        AST_Slicer["Context Slicing Engine (<= 4,096 tokens)"]
        Safety_Annotation["Safety Annotation Binding Engine"]
    end

    subgraph "Projection & Synthesis (deap-codegen)"
        Backlog_Engine["Agile Backlog Projection Engine"]
        MATLAB_Adapter["MATLAB / Simulink / Stateflow Tier-1 Adapter"]
        Code_Synthesis["Evolved 3-Layer Code Synthesis Engine"]
    end

    subgraph "Targets & Compliance (deap-harness)"
        Tracker["Issue Tracker Backlog - Epics, Features, User Stories, Use Cases"]
        MBD_Artifacts["Simulink slreq.ReqSet, Stateflow .sfx, Simulink.Bus"]
        Safety_Code["Safety-Critical Code - SPARK Ada 2014, MISRA-C:2012, Rust"]
        Parity_Gate["AST Parity Auditor & Baseline Gate - deap-harness"]
    end

    SysML_Text --> AST_Parser
    SysML_Graph --> AST_Parser
    AST_Parser <--> AST_Cache
    AST_Parser --> Semantic_DAG
    Semantic_DAG --> AST_Slicer
    Semantic_DAG --> Safety_Annotation
    AST_Slicer --> Backlog_Engine
    Safety_Annotation --> Backlog_Engine
    Safety_Annotation --> MATLAB_Adapter
    Safety_Annotation --> Code_Synthesis
    Backlog_Engine --> Tracker
    MATLAB_Adapter --> MBD_Artifacts
    Code_Synthesis --> Safety_Code
    Tracker --> Parity_Gate
    MBD_Artifacts --> Parity_Gate
    Safety_Code --> Parity_Gate
```

### Key Vision & Objectives:
1. **Seamless MBSE Ingestion:** Parse raw OMG SysML v2 textual declarations (`package`, `part def`, `port`, `connection`, `action`, `state`, `attribute`, `requirement`) into a structured in-memory Semantic Graph Directed Acyclic Graph (DAG) using a pure Rust modular architecture.
2. **Incremental AST Caching:** Maintain a deterministic, content-hash keyed parser cache (`.pipeline/cache/ast/`) to eliminate redundant re-parsing of unchanged model units across CI/CD and subagent execution loops.
3. **Strongly-Typed AST Slicing:** Partition monolithic model DAGs into context-isolated, strongly-typed semantic slices strictly bounded at $\le 4,096$ tokens, enabling zero-truncation consumption by specialized LLM synthesis subagents.
4. **Deterministic Backlog Projection:** Project SysML v2 packages and requirements into issue tracker backlog items (Epics, Features, BDD User Stories, and UML Use Cases) with 100% structural fidelity.
5. **Tier-1 Commercial Toolchain Integration:** Interoperability with MATLAB / Simulink / Stateflow / Embedded Coder (`slreq.ReqSet`, Stateflow `.sfx` charts, `Simulink.Bus` objects) as the primary commercial Model-Based Design (MBD) toolchain.
6. **Evolved 3-Layer Code Synthesis:** Synthesize high-integrity, certifiable target source code in **SPARK Ada 2014**, **MISRA-C:2012**, and **High-Assurance Rust** structured across Domain State/Signals (Layer 1), Logic & Safety State Management (Layer 2), and Display/Actuator Interfaces (Layer 3).
7. **Continuous Compliance Verification:** Enforce 100% bi-directional traceability and zero-orphan coverage via automated AST parity gates in `deap-harness` under stringent high-assurance standards (**DO-178C**, **DO-254**, **ARP4754A/4761**, **ASTM F3269**).

---

## 2. Pure Rust Cargo Workspace Architecture & AST Parser Engine

### 2.1 Pure Rust Modular Cargo Workspace Topology

The ingestion engine and downstream compilation pipeline are organized as a pure Rust modular Cargo workspace:

```
deap-compiler/
├── Cargo.toml                  # Workspace root manifest
├── crates/
│   ├── deap-core/              # Core primitives, diagnostics, hashing, and configuration
│   ├── deap-ast/               # SysML v2 AST, grammar parser, slicing engine, incremental cache
│   ├── deap-codegen/           # Backlog projection, MATLAB/Simulink exporters, 3-layer code synthesis
│   ├── deap-harness/           # Parity validation gates, semantic acceptance tests, simulation drivers
│   └── deap-cli/               # Unified CLI entrypoint (`deap ingest`, `deap slice`, `deap verify`)
```

- **`deap-core`**: Provides zero-allocation string interning, diagnostic emission with ANSI/JSON formatters, SHA-256 cryptographic fingerprinting, and workspace configuration loaders.
- **`deap-ast`**: Houses the formal SysML v2 grammar lexer/parser, strongly-typed node hierarchies, incremental caching subsystem, and topological graph slicing engine.
- **`deap-codegen`**: Implements template-free code generation for MBD artifacts, Markdown backlog projections, and multi-language safety implementations.
- **`deap-harness`**: Implements mechanical verification gates, semantic parity checks, and model-in-the-loop validation runners.
- **`deap-cli`**: Exposes high-performance CLI commands for pipeline operators and automated CI/CD runners.

```mermaid
flowchart LR
    CLI["deap-cli"] --> AST["deap-ast"]
    CLI --> CODEGEN["deap-codegen"]
    CLI --> HARNESS["deap-harness"]
    CODEGEN --> AST
    HARNESS --> AST
    AST --> CORE["deap-core"]
    CODEGEN --> CORE
    HARNESS --> CORE
```

### 2.2 AST Grammar & Primitive Extraction
The `deap-ast` parser processes standard SysML v2 structural and behavioral primitives:
- `package`: Top-level namespace containers mapping to functional domains.
- `part def` / `part`: Structural components, subsystems, and physical/logical entities.
- `port`: Dynamic interface points and bus connectors with explicit directionality (`in`, `out`, `inout`).
- `connection`: Dataflow and logical bindings between ports.
- `action` / `action def`: Discrete operational behaviors, activities, and control flows.
- `state` / `state def`: FSM safety states, transition conditions, entry/exit actions.
- `attribute`: Typed data fields with value ranges, units, and multiplicity constraints.
- `requirement`: Formal system requirements with explicit text, IDs, and verification criteria.
- `enum def` / `item def`: Discrete enumeration types and structured data items.

### 2.3 Strongly-Typed AST Data Structures in Rust (`deap-ast`)

```rust
// Strongly-typed AST structures in crates/deap-ast/src/model.rs

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeKind {
    Package,
    PartDef,
    Port,
    Connection,
    Action,
    State,
    Requirement,
    Attribute,
    ItemDef,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnnotationAttribute {
    pub name: String, // e.g., "TriggersHazard", "SafetyRealises", "Layer"
    pub params: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceSpan {
    pub file_path: String,
    pub start_line: usize,
    pub end_line: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SysMLNode {
    pub node_id: String,
    pub kind: NodeKind,
    pub name: String,
    pub qualified_name: String,
    pub docstring: Option<String>,
    pub attributes: HashMap<String, String>,
    pub annotations: Vec<AnnotationAttribute>,
    pub source_ref: Option<SourceSpan>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SysMLEdge {
    pub source_id: String,
    pub target_id: String,
    pub edge_type: String, // "contains", "connects_to", "transition", "realises", "triggers_hazard"
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SemanticGraphDAG {
    pub nodes: HashMap<String, SysMLNode>,
    pub edges: Vec<SysMLEdge>,
    pub package_hierarchy: HashMap<String, Vec<String>>,
}
```

### 2.4 Strongly-Typed AST Slicing Mechanics ($\le 4,096$ Tokens)

To guarantee that autonomous AI subagents operate without context truncation or hallucination, `deap-ast` incorporates a deterministic topological graph slicing engine. Large monolithic models (which can span tens of thousands of tokens) are partitioned into coherent, self-contained subgraphs strictly bounded at $\le 4,096$ tokens.

```mermaid
flowchart TD
    subgraph Monolithic_AST ["Monolithic Model AST (15,000+ Tokens)"]
        Root_DAG["SemanticGraphDAG"]
    end

    subgraph Slicing_Engine ["deap-ast Graph Slicing Engine"]
        Subsystem_Filter["Subsystem Boundary Slicer"]
        Interface_Filter["Port & Connection Slicer"]
        FSM_Filter["Behavioral Statechart Slicer"]
        Token_Budget["Token Budget Allocator (<= 4,096 Tokens)"]
    end

    subgraph Slices ["Context-Isolated AST Slices (<= 4,096 Tokens Each)"]
        Slice_A["Slice A: Subsystem_01 Structure & Interfaces"]
        Slice_B["Slice B: Subsystem_01 FSM & Transition Logic"]
        Slice_C["Slice C: Cross-Subsystem Signal Invariants"]
    end

    Root_DAG --> Subsystem_Filter & Interface_Filter & FSM_Filter
    Subsystem_Filter & Interface_Filter & FSM_Filter --> Token_Budget
    Token_Budget --> Slice_A & Slice_B & Slice_C
```

#### Slicing Algorithms & Invariants:
1. **Subsystem Partition Slicing:** Extracts a single target `part def` along with all direct child parts, attributes, and internal actions. External connections are replaced with strongly-typed port interface stubs.
2. **Interface Boundary Slicing:** Extracts all connected `PortDefNode` instances, item flows, and signal definitions across an N² subsystem boundary, pruning internal logic.
3. **Behavioral State Slicing:** Extracts a single `state def` machine with all transition paths, guard predicates, and entry/exit actions, binding external event triggers to typed stub signals.
4. **Token Budget Guarantee:** The slicing engine tokenizes the serialized JSON/YAML projection of the subgraph using the target tokenizer (e.g. tiktoken cl100k_base). If the token count exceeds 4,096 tokens, the engine recursively subdivides behavioral actions from structural declarations, yielding two complementary sub-slices.

```rust
// AST Slicing Interface in crates/deap-ast/src/slicing.rs

pub struct AstSlice {
    pub slice_id: String,
    pub target_node: String,
    pub token_count: usize,
    pub subgraph: SemanticGraphDAG,
    pub external_interface_stubs: Vec<SysMLNode>,
}

impl SemanticGraphDAG {
    pub fn slice_subsystem(&self, target_part_id: &str, max_tokens: usize) -> Result<Vec<AstSlice>, SlicingError> {
        // Deterministic BFS extraction preserving immediate dependencies
        // Enforces token_count <= max_tokens (default 4,096)
        // Replaces out-of-slice targets with typed stub references
    }
}
```

### 2.5 Incremental AST Parser Caching

To optimize execution velocity and support rapid continuous verification loops, `deap-ast` implements an incremental disk and memory caching subsystem:

1. **Fingerprint Computation:** For every input `.sysml` or schema file, `deap-ast` computes a SHA-256 content digest and combines it with compiler version metadata.
2. **Cache Keying & Layout:** Cached binary AST artifacts are stored under `.pipeline/cache/ast/<sha256>.bin` using bincode serialization for sub-millisecond cold loading.
3. **Delta Dependency Invalidation:** When a package is modified, only the affected AST nodes and their immediate topological dependents in the DAG are invalidated; unaffected packages are pulled directly from cache.

```rust
// Incremental Cache in crates/deap-ast/src/cache.rs

pub struct IncrementalAstCache {
    cache_dir: PathBuf,
}

impl IncrementalAstCache {
    pub fn get_or_parse<F>(&self, file_path: &Path, parse_fn: F) -> Result<SemanticGraphDAG, ParseError>
    where
        F: FnOnce(&Path) -> Result<SemanticGraphDAG, ParseError>,
    {
        let digest = deap_core::hash_file_sha256(file_path)?;
        let cache_file = self.cache_dir.join(format!("{}.bin", digest));
        if cache_file.exists() {
            return deap_core::deserialize_bincode(&cache_file);
        }
        let ast = parse_fn(file_path)?;
        deap_core::serialize_bincode(&cache_file, &ast)?;
        Ok(ast)
    }
}
```

### 2.6 Custom Safety Annotations
The synthesizer extracts custom DEAP safety annotations embedded within SysML v2 textual comments or metadata tags:

| Annotation Syntax | Parameters | Extraction & Semantic Linkage |
| :--- | :--- | :--- |
| `@TriggersHazard(id="HAZ-NNN", severity="SeverityLevel")` | `id`: Hazard ID string, `severity`: `Catastrophic`, `Hazardous`, `Major`, `Minor` | Binds element to system hazard analysis matrix; forces creation of BDD negative scenario. |
| `@SafetyRealises(requirement="REQ-NNN")` | `requirement`: Requirement ID string | Establishes bi-directional traceability between structural/behavioral elements and upstream requirements. |
| `@Layer(tier="Layer1\|Layer2\|Layer3")` | `tier`: Target layer designation | Binds the element explicitly to Layer 1 (State/Signals), Layer 2 (Statecharts/FSM), or Layer 3 (Display/Actuators). |

### 2.7 Dual-LLM Air-Gapped Agent Dispatch Protocol

To guarantee mathematical determinism, total intellectual property containment, and complete immunity to cloud rate-limiting or content censorship, DEAP utilizes a dual-LLM air-gapped agent dispatch architecture orchestrated by `deap-harness` and `deap-cli`:

1. **Reasoning Slot (DeepSeek-R1)**:
   - Dedicated Chain-of-Thought (CoT) reasoning model executing on sovereign air-gapped workstation hardware.
   - Ingests strongly-typed AST slices (<= 4,096 tokens) from `deap-ast` to derive formal semantics, invariant boundaries, and state reachability matrices.
   - Streams formal mathematical proofs, reachability derivations, and safety invariant checks directly into `.pipeline/diagnostics/cot_audit_log.json`.
   - Never generates raw code directly; outputs verified formal derivations and boundary assertions.

2. **Execution Slot (Qwen-2.5-Coder)**:
   - High-throughput deterministic code and specification synthesis model.
   - Ingests verified reasoning derivations from `.pipeline/diagnostics/cot_audit_log.json` alongside the strongly-typed AST slice.
   - Generates verified SysML v2 specifications, formal verification contracts, and multi-toolchain synthesis artifacts.
   - All generated code and model artifacts are validated through `deap-harness` mechanical gates before committing to the repository.

---

## 3. Agile Backlog Projection Engine (`deap-codegen`)

The Agile Backlog Projection Engine deterministically maps the in-memory `SemanticGraphDAG` into issue tracker backlog artifacts.

```mermaid
flowchart TD
    subgraph "SysML v2 Model Nodes"
        Pkg["package: SubsystemController_01"]
        Req["requirement: REQ-SIG-001 - Signal Validation"]
        Part["part def: SensorProcessingUnit_01"]
        Act["action: ProcessSensorSample"]
        Haz["TriggersHazard - id='HAZ-SIG-01', severity='Hazardous'"]
    end

    subgraph "Backlog Projection Rules"
        Rule_Epic["Package to Epic Mapping"]
        Rule_Feat["Requirement or Part to Feature Mapping"]
        Rule_Story["Action or State or Hazard to BDD User Story"]
        Rule_UC["System Interaction to System Use Case"]
    end

    subgraph "Issue Tracker Backlog"
        Epic_Issue["Epic #101: Subsystem Controller"]
        Feat_Issue["Feature #102: Process Sensor Sampling"]
        Story_Issue["User Story #103: Handle Invalid Sample Failsafe"]
        UC_Issue["Use Case #104: Monitor Signal State"]
    end

    Pkg --> Rule_Epic --> Epic_Issue
    Req --> Rule_Feat
    Part --> Rule_Feat --> Feat_Issue
    Act --> Rule_Story
    Haz --> Rule_Story --> Story_Issue
    Act --> Rule_UC --> UC_Issue
    Feat_Issue -->|Child of| Epic_Issue
    Story_Issue -->|Validates| Feat_Issue
    UC_Issue -->|Realizes| Story_Issue
```

### 3.1 Projection Transformation Rules
1. **SysML `package` -> Issue Tracker Epic:**
   - Title format: `[Module/Domain]: [Functional Area]` (e.g., `[CoreSubsystem]: Signal Processing & State Estimation`).
   - Frontmatter tags: `type: epic`, `sysml_package: <qualified_name>`.
2. **SysML `requirement` & `part def` -> Issue Tracker Feature:**
   - Title format: `[Verb] [Object] [Qualifier]` (e.g., `Sample Sensor Data Stream`).
   - Frontmatter tags: `type: feature`, `epic_ref: #<Epic_ID>`, `sysml_element: <qualified_name>`.
   - Acceptance Criteria: Derived from `attribute` constraints, `port` boundaries, and `@SafetyRealises` tags.
3. **SysML `action`, `state`, & `@TriggersHazard` -> BDD User Story:**
   - Title format: `As a [Actor/Subsystem], I want to [Action], so that [Outcome/Safety Constraint]`.
   - Canonical BDD Scenarios (Given-When-Then):
     - **Pattern A (Display System Interfaces):** `Given [Parameter Buffer State], When [Binary Command Received], Then [Widget State & Display Kernel Render Updated]`.
     - **Pattern B (Real-Time Safety Statecharts / Controller):** `Given [Subsystem State Vector / Discrete Event], When [Safety FSM Transition Triggered], Then [Actuator Command / Output Signal Rendered]`.
     - **Pattern C (Decoupled Operator Console):** `Given [Console Domain Model State], When [Operator Action Initiated], Then [ViewModel State & Component Binding Updated]`.
4. **SysML `use case` & System Interaction -> UML System Use Case:**
   - Formal specification containing Primary Actors, Preconditions, Main Success Scenario, Alternate Flows, and a complete Realization Matrix linking Use Cases to User Stories and Features.

---

## 4. MATLAB / Simulink / Stateflow Tier-1 Integration Adapter

DEAP establishes **MATLAB / Simulink / Stateflow / Embedded Coder** as the Primary Tier-1 Commercial Toolchain Integration for Model-Based Design (MBD), control law synthesis, and DO-178C C/SPARK Ada code generation.

```mermaid
flowchart TD
    subgraph "SysML v2 Ingested Model"
        SysML_Reqs["SysML v2 Requirements"]
        SysML_States["SysML v2 State Machines"]
        SysML_Ports["SysML v2 Parts & Ports"]
    end

    subgraph "MATLAB / Simulink Integration Adapter (deap-codegen)"
        Req_Exporter["Simulink Requirements Exporter - slreq.ReqSet API"]
        SFX_Exporter["Stateflow .sfx Chart Generator"]
        Bus_Exporter["Simulink.Bus Definition Generator"]
    end

    subgraph "Tier-1 Commercial Deliverables"
        SlReq_File["ReqSet File - .slreqx and .json"]
        Stateflow_Chart["Stateflow Chart - .sfx and .slx"]
        Simulink_Bus["Simulink Bus Objects - m-script and MAT"]
    end

    subgraph "Secondary Safety Exporters"
        Medini["Safety Analysis Worksheets"]
        Windchill["Quality Solutions Exporter"]
        EA["Enterprise Architect Safety Profiles"]
    end

    SysML_Reqs --> Req_Exporter --> SlReq_File
    SysML_States --> SFX_Exporter --> Stateflow_Chart
    SysML_Ports --> Bus_Exporter --> Simulink_Bus
    SlReq_File --> Medini
    SlReq_File --> Windchill
    Stateflow_Chart --> EA
```

### 4.1 Simulink Requirements (`slreq.ReqSet`) Mapping
SysML v2 `requirement` blocks are exported directly into MATLAB Simulink Requirements sets using the `slreq` programmatic API:
- Requirement ID -> `slreq.ID`
- Requirement text -> `slreq.Text`
- Rationale -> `slreq.Rationale`
- Safety Attributes / DO-178C DAL -> `slreq.setAttribute("SafetyDAL", "DAL_A")`

### 4.2 Stateflow `.sfx` Chart Generation
SysML v2 `state def` machines translate directly into standalone Stateflow chart definitions (`.sfx`):
- `state` -> Stateflow State Object
- `transition` -> Stateflow Transition with Guard Condition `[guard]` and Action `{action;}`
- Entry/Exit Actions -> `entry:` / `exit:` statements inside Stateflow states.

### 4.3 `Simulink.Bus` & Signal Objects
SysML v2 `part def` and `port` structures generate MATLAB scripts constructing `Simulink.Bus` objects:
```matlab
% Generated Simulink.Bus Definition Script
elems(1) = Simulink.BusElement;
elems(1).Name = 'signal_alpha';
elems(1).DataType = 'single';
elems(1).Min = 0.0;
elems(1).Max = 1000.0;

elems(2) = Simulink.BusElement;
elems(2).Name = 'subsystem_status';
elems(2).DataType = 'Enum: SubsystemStatusType';

Bus_Subsystem_01 = Simulink.Bus;
Bus_Subsystem_01.Elements = elems;
```

---

## 5. Evolved 3-Layer Pure Rust Code Synthesis Engine

The 3-Layer Code Synthesis Engine in `deap-codegen` translates the SysML v2 Semantic Graph DAG into target source code, guaranteeing strict separation across domain states, safety statecharts, and display/actuator interfaces.

```mermaid
flowchart TD
    subgraph "Layer 1: Domain State & Signal Model"
        L1_Desc["Parameter Buffers, Discrete Input Events, Raw Data Models"]
        L1_Code["SPARK Ada Package Spec / C Struct / Rust Struct"]
    end

    subgraph "Layer 2: Logic & Safety State Management"
        L2_Desc["FSM Statecharts, Failsafe Monitors, ViewModel State"]
        L2_Code["SPARK Ada FSM Body / MISRA-C / Rust State Machine"]
    end

    subgraph "Layer 3: Display & Actuator Interface Binding"
        L3_Desc["Display Kernel Renderers, Actuator Outputs, Drivers, BDD Verification"]
        L3_Code["Display Render / Hardware Publisher / Acceptance Harness"]
    end

    L1_Desc --> L1_Code
    L2_Desc --> L2_Code
    L3_Desc --> L3_Code
    L1_Code --> L2_Code --> L3_Code
```

### 5.1 Layer Definitions & Target Languages

#### Layer 1: Domain State & Signal Model
- **SPARK Ada 2014:** Strong scalar types with explicit range constraints and record definitions.
  ```ada
  type Signal_Alpha_Type is new Float range 0.0 .. 1000.0;
  type Subsystem_Status_Type is (Valid, Invalid, Degraded, No_Signal);
  
  type Subsystem_Buffer_Type is record
     Current_Signal : Signal_Alpha_Type;
     Status         : Subsystem_Status_Type;
  end record;
  ```
- **MISRA-C:2012:** Fixed-width integer types (`int32_t`, `uint32_t`) and defensive typedef structs.
- **High-Assurance Rust:** Strongly typed newtypes with validated range invariants:
  ```rust
  #[derive(Debug, Clone, Copy, PartialEq)]
  pub struct SignalAlpha(f32);
  
  impl SignalAlpha {
      pub fn new(val: f32) -> Result<Self, SignalError> {
          if (0.0..=1000.0).contains(&val) {
              Ok(Self(val))
          } else {
              Err(SignalError::ValueOutOfRange)
          }
      }
  }
  ```

#### Layer 2: Logic & Safety State Management
- **SPARK Ada 2014:** Formal verification contracts ensuring zero runtime errors (`Pre`, `Post`, `Global`, `Depends`).
  ```ada
  procedure Transition_Safety_State (
     Buffer : in     Subsystem_Buffer_Type;
     State  : in out FSM_State_Type)
  with
     Pre  => Buffer.Current_Signal >= 0.0,
     Post => (if Buffer.Status /= Valid then State = Failsafe_Fallback),
     Global => null;
  ```
- **MISRA-C:2012:** Defensive state machine with explicit switch/case logic, Rule 10.x compliance, and static variable scoping.
- **High-Assurance Rust:** Type-state pattern with compile-time state transitions:
  ```rust
  pub struct StateMachine<S> {
      state: S,
  }
  
  impl StateMachine<Operational> {
      pub fn handle_fault(self, fault: FaultEvent) -> StateMachine<FailsafeFallback> {
          StateMachine { state: FailsafeFallback::from(fault) }
      }
  }
  ```

#### Layer 3: Display & Actuator Interface Binding
- **SPARK Ada / C:** Display widget buffer driver rendering and actuator DAC bus updates.
- **High-Assurance Rust:** Zero-cost hardware abstraction layer (HAL) bindings and deterministic actuator updates.
- **Semantic Acceptance Tests:** Automated verification suites verifying that components render and actuate correctly in response to Layer 1/2 state changes.

---

## 6. Continuous Compliance Verification & Parity Gate (`deap-harness`)

Continuous compliance and structural parity are mechanically verified by `deap-harness` and `./target/release/verify-baseline`.

```mermaid
sequenceDiagram
    autonumber
    participant SysML as SysML v2 Source - .sysml
    participant Parser as AST Parser - deap-ast
    participant Backlog as Backlog Engine - deap-codegen
    participant Synthesizer as Code Synthesizer - deap-codegen
    participant Gate as AST Parity Auditor - deap-harness

    SysML->>Parser: Parse textual model files
    Parser->>Backlog: Generate Epics, Features, User Stories
    Parser->>Synthesizer: Generate SPARK Ada / MISRA-C / Rust code
    Backlog->>Gate: Extract Issue Keys & Realises tags
    Synthesizer->>Gate: Extract Code Symbol References
    Gate->>Gate: Perform 100% Bi-Directional Parity Audit
    alt Traceability 100% & Tri-Layer Complete
        Gate-->>SysML: PASS (Exit Code 0)
    else Missing Tag or Orphaned Requirement
        Gate-->>SysML: FAIL (Halt Pipeline & Report Defect)
    end
```

### 6.1 Parity Auditor Enforcement Rules
1. **100% Model Coverage:** Every requirement in the `.sysml` source file MUST map to at least one Feature issue.
2. **Zero Orphaned Requirements:** Every code file carrying a `/// Realises: [Feat-NNN/...]` tag must resolve to an active Feature issue key.
3. **Tri-Layer Definition of Done Gate:** Enforced by `deap-harness`. Every Feature issue must have all three layers (Layer 1 State/Signal, Layer 2 Statechart/ViewModel, Layer 3 Display/Actuator) explicitly implemented or mapped to a named task.

---

## 7. Implementation Roadmap & Phased Execution Plan

The SysML v2 Ingestion Engine solution blueprint is executed across four distinct phases:

```mermaid
gantt
    title SysML v2 Ingestion Engine Phased Implementation Roadmap
    dateFormat  YYYY-MM-DD
    section Phase 1: Rust AST & Slicing
    SysML v2 Parser & Lexer (deap-ast)               :p1a, 2026-08-15, 10d
    AST Slicing Engine & Incremental Cache (deap-ast):p1b, after p1a, 5d
    section Phase 2: MATLAB Adapter
    Simulink slreq.ReqSet Exporter (deap-codegen)    :p2a, after p1b, 7d
    Stateflow .sfx & Bus Generator (deap-codegen)    :p2b, after p2a, 7d
    section Phase 3: Backlog Engine
    Agile Backlog Projector (deap-codegen)           :p3a, after p2b, 7d
    Backlog Reconciliation Engine                    :p3b, after p3a, 5d
    section Phase 4: Parity Gate
    Semantic Parity Auditor (deap-harness)           :p4a, after p3b, 7d
    Baseline Acceptance Gate (verify_downstream)     :p4b, after p4a, 5d
```

### Phase Details & Exit Criteria

| Phase | Core Deliverables | Key Artifacts | Exit Criteria & Verification Gate |
| :--- | :--- | :--- | :--- |
| **Phase 1: AST Parser & Slicing Engine** | SysML v2 grammar parser, typed AST slices (<= 4,096 tokens) | `deap-ast`, `deap-core` | Parsed `.sysml` files output valid `SemanticGraphDAG` and context-isolated AST slices without data loss. |
| **Phase 2: MATLAB Tier-1 Adapter** | MBD exporter for Simulink & Stateflow | `deap-codegen` Simulink exporter | MATLAB CLI successfully loads exported `.slreqx` and executes `.sfx` Stateflow simulation without errors. |
| **Phase 3: Backlog Projection Engine** | Projection rules for Epics, Features, User Stories, Use Cases | `deap-codegen`, backlog reconciliation | 100% of SysML requirements and packages projected into specifications with valid YAML frontmatter tags. |
| **Phase 4: Parity Gate & Compliance** | Semantic acceptance verification harness | `deap-harness`, `./target/release/verify-baseline` | `./target/release/verify-baseline . --no-domain` passes 100% green; bi-directional audit verifies zero orphan code symbols or requirements. |

---
