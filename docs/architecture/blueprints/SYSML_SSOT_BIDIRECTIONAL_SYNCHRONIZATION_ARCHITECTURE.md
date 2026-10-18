---
identifier: DEAP-BLUEPRINT-SSOT-002
title: "SysML v2 Single Source of Truth (SSOT) & Closed-Loop Bidirectional Synchronization Architecture"
version: "2.0.0"
status: "APPROVED / PRODUCTION-GRADE"
created: "2026-09-01"
authority: "SysML v2 Model-as-SSOT & Non-Drifting Elaboration Invariant (rules/sysml-ssot-completeness.md)"
primary_toolchain: "SysML v2 AST, MATLAB / Simulink / Stateflow (R2024b), Embedded Coder (DO-178C / DO-331), Discrete Digital Twin Engine"
---

# Architecture Blueprint: SysML v2 Single Source of Truth (SSOT) & Closed-Loop Bidirectional Synchronization Architecture

| Attribute | Specification Detail |
| :--- | :--- |
| **Version** | 2.0.0 |
| **Date** | 2026-09-01 |

- **Document Version**: 2.0.0
- **Classification**: Normative Architectural Blueprint
- **Authority**: SysML v2 Model-as-SSOT & Non-Drifting Elaboration Invariant (`rules/sysml-ssot-completeness.md`)
- **Primary Toolchain**: SysML v2 AST, MATLAB / Simulink / Stateflow (R2024b), Embedded Coder (DO-178C / DO-331), Discrete Digital Twin Engine

---

## 1. Executive Summary & System Vision

In the **Digital Engineering Autonomous Pipeline (DEAP)**, system models and textual engineering specifications do not exist as decoupled or loosely correlated artifacts. Instead, **SysML v2** (`.pipeline/schema.sysml`) serves as the 100% authoritative, mathematically grounded **Single Source of Truth (SSOT)** for all system structure, behaviors, interfaces, data definitions, and safety invariants.

All downstream engineering artifacts--including Agile Epics, Features, BDD User Stories, formal Use Cases, verification suites, and target code generators--are direct formal projections and non-drifting elaborations of the SysML v2 Abstract Syntax Tree (AST).

```mermaid
flowchart TD
    subgraph Ingestion["1. Schema Ingestion & AST Canonicalization (deap-ast & deap-core)"]
        HET_SCHEMA["Heterogeneous Schemas: schema/\nSysML v2, OpenAPI, ARXML, IDL, Proto"]
        INGEST_ENG["SysML v2 Ingestion Engine: deap-ast"]
        CACHE["Incremental AST Cache: .pipeline/cache/ast/"]
        SYSML_AST["Canonical SysML v2 AST Model: .pipeline/schema.sysml"]
        DIGEST["Cryptographic Digest & Symbol Tables: .pipeline/schema-digest.json"]
    end

    subgraph Downstream["2. Forward Specification Projection (deap-codegen)"]
        SLICES["AST Slices (<= 4,096 tokens)"]
        EPICS["docs/epics/ (Subsystems & Packages)"]
        FEATS["docs/features/ (Parts & Item Payloads)"]
        STORIES["docs/user-stories/ (Actions & States)"]
        USECASES["docs/use-cases/ (Formal Use Cases)"]
        SAFETY["docs/safety/ (STPA / FMECA Requirements)"]
    end

    subgraph ReverseSync["3. Closed-Loop Reverse Synchronization (deap-ast & deap-codegen)"]
        REVERSE_ENG["Automated Reverse Sync Engine: deap-codegen --reverse-sync"]
        DELTA_MERGE["Non-Destructive Semantic AST Delta Merge: deap-ast"]
    end

    subgraph DualTrack["4. High-Integrity Synthesis & Dual-Track Verification (deap-harness)"]
        TRACK_A["Track A: Native MATLAB/Simulink/Stateflow Synthesis\nDO-178C / DO-331 Embedded Coder & SLDV"]
        TRACK_B["Track B: Headless CI Digital Twin Engine\nLicense-Free 250 Hz Continuous Regression"]
    end

    HET_SCHEMA --> INGEST_ENG
    INGEST_ENG <--> CACHE
    INGEST_ENG --> SYSML_AST
    SYSML_AST --> DIGEST

    SYSML_AST --> SLICES
    SLICES --> EPICS
    SLICES --> FEATS
    SLICES --> STORIES
    SLICES --> USECASES
    SLICES --> SAFETY

    EPICS -.->|"Elaborated Deltas"| REVERSE_ENG
    FEATS -.->|"Elaborated Deltas"| REVERSE_ENG
    STORIES -.->|"Elaborated Deltas"| REVERSE_ENG
    USECASES -.->|"Elaborated Deltas"| REVERSE_ENG
    SAFETY -.->|"Elaborated Deltas"| REVERSE_ENG

    REVERSE_ENG --> DELTA_MERGE
    DELTA_MERGE --> SYSML_AST

    SYSML_AST ==> TRACK_A
    SYSML_AST ==> TRACK_B
```

---

## 2. Pure Rust Cargo Workspace Architecture & Synchronization Engine

The Single Source of Truth (SSOT) and bidirectional synchronization architecture is implemented across the modular, pure Rust Cargo workspace:

```
deap-compiler/
├── Cargo.toml                  # Workspace root manifest
├── crates/
│   ├── deap-core/              # Core primitives, cryptographic SHA-256 digests, symbol tables
│   ├── deap-ast/               # SysML v2 AST, grammar parser, bidirectional AST merge, slicing engine
│   ├── deap-codegen/           # Forward specification projection, reverse synchronization re-emitter
│   ├── deap-harness/           # Dual-track verification harness, discrete equivalence gate, 22 parity gates
│   └── deap-cli/               # Unified CLI (`deap sync --reverse`, `deap verify --parity`)
```

- **`deap-core`**: Computes SHA-256 cryptographic digests, line counts, and symbol manifests stored in `.pipeline/schema-digest.json`.
- **`deap-ast`**: Houses the canonical in-memory `SemanticGraphDAG`, executes non-destructive semantic AST delta merging, maintains incremental parser caches, and enforces strongly-typed AST slicing ($\le 4,096$ tokens).
- **`deap-codegen`**: Projects AST elements into Agile Epics, Features, BDD User Stories, and Use Cases, and re-emits canonical SysML v2 text during reverse synchronization.
- **`deap-harness`**: Executes the Dual-Track Verification Protocol, enforcing numerical precision bounds ($\le 10^{-6}$), discrete loop rate equivalence ($\Delta t = 0.004\,\text{s}$ / 250 Hz), and all 22 parity gates.
- **`deap-cli`**: Directs CLI operations for developers, CI/CD runners, and autonomous subagents.

### 2.1 Strongly-Typed AST Slicing Mechanics (<= 4,096 Tokens)
During forward projection and reverse synchronization, model complexity can exceed the context window limits of autonomous LLM subagents. The `deap-ast` slicing engine partitions the global model into strongly-typed, context-isolated subgraphs:
1. **Target Subsystem Slicing:** Extracts a discrete subsystem package and its immediate boundaries.
2. **Context Budget Enforcement:** Strictly bounds each slice to $\le 4,096$ tokens, preventing prompt truncation and output degradation.
3. **Contract Stubbing:** Replaces external dependencies with typed port interfaces, preserving closed-world validity within the slice.

### 2.2 Incremental AST Parser Caching
To ensure sub-second turnaround during multi-agent synchronization loops:
1. **Content-Hash Keyed Cache:** `.pipeline/cache/ast/<sha256>.bin` stores pre-parsed bincode representations of the AST.
2. **Selective Delta Parsing:** Only files whose SHA-256 fingerprint has changed are re-lexed and re-parsed. Unchanged modules are loaded instantly from disk cache.

### 2.3 Dual-LLM Air-Gapped Agent Dispatch Protocol
To guarantee mathematical determinism, total intellectual property containment, and complete immunity to cloud rate-limiting or content censorship, DEAP utilizes a dual-LLM air-gapped agent dispatch architecture orchestrated by `deap-harness` and `deap-cli`:

1. **Reasoning Slot (DeepSeek-R1)**:
   - Dedicated Chain-of-Thought (CoT) reasoning model executing on sovereign air-gapped workstation hardware.
   - Ingests strongly-typed AST slices ($\le 4,096$ tokens) from `deap-ast` to derive bidirectional AST delta transformations, merge resolutions, and synchronization invariants.
   - Streams formal mathematical proofs, reachability derivations, and safety invariant checks directly into `.pipeline/diagnostics/cot_audit_log.json`.
   - Never generates raw code directly; outputs verified formal derivations and boundary assertions.

2. **Execution Slot (Qwen-2.5-Coder)**:
   - High-throughput deterministic code and specification synthesis model.
   - Ingests verified reasoning derivations from `.pipeline/diagnostics/cot_audit_log.json` alongside the strongly-typed AST slice.
   - Generates verified SysML v2 AST updates, bidirectional specification projections, and reverse sync patches.
   - All generated model and specification artifacts are validated through `deap-harness` mechanical gates before committing to the repository.

---

## 3. Core Architectural Principles & Model-as-SSOT Foundations

The architecture adheres to four fundamental principles:

1. **Model-as-SSOT Priority**: Natural language prose is never the origin of system truth. Every structural component, interface boundary, state transition, and safety requirement must be formally declared in the SysML v2 AST before or in tandem with downstream specification generation.
2. **Prohibition of Heuristic Prose Parsing**: Downstream code synthesis, validation linters, and verification test harnesses operate strictly on structured AST nodes and verified YAML/Mermaid metadata--never on free-form natural language heuristics.
3. **Pure Schema-Driven Compilation**: SysML v2 parsing, AST extraction, and verification gates operate purely on generic AST tokens (`package`, `part def`, `item def`, `action def`, `state def`, `port def`, `requirement def`, `use case def`) without hardcoded domain bias.
4. **Non-Drifting Bidirectional Synchronization**: Specifications and models are linked through a closed loop. Any refinement, edge case, or transition guard added in downstream specifications is automatically extracted and merged back into `.pipeline/schema.sysml`.

---

## 4. Schema Ingestion Pipeline & SysML v2 AST Metamodel

Input specifications and interface definitions placed in `schema/` are ingested via `deap-ast` / `./target/release/ingest-sysml`. The ingestion engine normalizes heterogeneous domain descriptions into standardized SysML v2 packages:

```
schema/  -->  deap-ast (./target/release/ingest-sysml)  -->  .pipeline/schema.sysml  +  .pipeline/schema-digest.json
```

### AST Element Metamodel Mapping

| Source Schema Element | SysML v2 AST Classifier | Downstream Target Specification |
| :--- | :--- | :--- |
| Package / Module / Domain Container | `package` | Epic (`docs/epics/EPIC-*.md`) |
| Physical / Logical Component / Subsystem | `part def` | Feature (`docs/features/feat-*.md`) |
| Message Payload / Data Structure / DTO | `item def` | Feature Item Definitions & Interfaces |
| Computational Operation / Method | `action def` | User Story BDD Actions (`docs/user-stories/`) |
| Mode / Lifecycle State Machine | `state def` | User Story & Feature Statecharts |
| Directional Interface Boundary | `port def` | Feature Interface Connection Matrices |
| System Interaction & Capability Realization | `use case def` | Use Case Specification (`docs/use-cases/uc-*.md`) |
| STPA UCA / Loss Scenario / FMECA Rule | `requirement def` | Safety Matrix (`docs/safety/`) |

---

## 5. Downstream Specification Projection & Traceability Threads

Forward specification generation decomposes the canonical SysML v2 AST model into an unbroken digital thread:

1. **Epics (`docs/epics/EPIC-*.md`)**: Realize architectural subsystem packages, defining high-level capabilities and tracking child features.
2. **Features (`docs/features/feat-*.md`)**: Realize `part def` components and `item def` payloads. Every Feature mandates YAML frontmatter declaring `sysml_source`, `part_def`, `ports`, `state_defs`, and `requirements`.
3. **User Stories (`docs/user-stories/us-*.md`)**: Realize fine-grained behavioral actions (`action def`) and state transitions (`state def`) mapped to Given-When-Then BDD scenarios.
4. **Use Cases (`docs/use-cases/uc-*.md`)**: Realize formal `use case def` nodes, declaring initiating/participating actors, subjects (`part def`), preconditions, numbered steps, alternate flows, and realization matrices.
5. **Safety Invariants (`docs/safety/`)**: Realize `requirement def`, `constraint def`, and `assert constraint` elements mapping STPA Unsafe Control Actions and FMECA failure modes.

---

## 6. Automated Closed-Loop Reverse Synchronization Standard

When agile specification workers elaborate edge cases, exception flows, or refined parameters during development phases, those deltas MUST be synchronized back into `.pipeline/schema.sysml` via the reverse synchronization compiler:

```bash
deap sync --reverse # or ./target/release/compile-sysml --reverse-sync
```

### Reverse Synchronization Engine Workflow

1. **AST Delta Extraction**: Scans all backlog Markdown files, BDD action signatures, Mermaid state diagrams, and STPA/FMECA tables.
2. **Non-Destructive Semantic AST Merge**: Merges newly discovered components, states, actions, ports, and requirements into `.pipeline/schema.sysml` without overwriting existing formal constraints.
3. **Canonical Serialization**: Re-emits `.pipeline/schema.sysml` using strict canonical indentation and symbol ordering.
4. **Cryptographic Parity Digest Regeneration**: Computes updated SHA-256 hash, line counts, and symbol manifests in `.pipeline/schema-digest.json`.
5. **Pre-Commit Parity Lock**: Automatically validates all 22 parity gates via `deap-harness` and `./target/release/verify-baseline`.

---

## 7. Primary Commercial Toolchain Synthesis & High-Integrity Verification Architecture

The primary commercial toolchain integration context for DEAP is **MATLAB / Simulink / Stateflow / Embedded Coder** (R2024b), delivering DO-178C / DO-331 Level A/B safety-critical target software synthesis.

### 7.1 MATLAB / Simulink / Stateflow Target Architecture
- Structural blocks map to Simulink Subsystem hierarchies and Model References.
- SysML v2 statecharts compile to Stateflow charts with deterministic transition priorities.
- Port interfaces bind to Simulink Buses and Typed Inport/Outport blocks.

### 7.2 DO-178C / DO-331 Software Qualification Workflow
- Traceability matrices link SysML v2 requirements directly to Simulink blocks and generated source lines.
- Model Coverage objectives (100% Decision, Condition, and MC/DC coverage) are verified via Simulink Coverage.

### 7.3 Simulink Design Verifier (SLDV) & Formal Property Proving
- SysML v2 `assert constraint` nodes are synthesized into SLDV Proof Objectives to mathematically verify absence of deadlocks, integer overflows, and out-of-bounds array accesses.

### 7.4 Polyspace Static Analysis & Target Code Generation
- Embedded Coder generates MISRA C:2012 and SPARK Ada safety-critical source code.
- Polyspace Bug Finder and Code Prover formally prove absence of run-time errors in generated C/Ada code.

---

### 7.5 Dual-Track Verification Protocol (Native MATLAB/Simulink Synthesis + Headless CI Digital Twin)

To bridge rigorous DO-178C / DO-331 Model-Based Design with modern high-velocity continuous integration (CI) and containerized regression pipelines, DEAP mandates the **Dual-Track Verification Protocol**.

```mermaid
flowchart TD
    subgraph SSOT["SysML v2 Authoritative Architecture & Behavior"]
        AST["SysML v2 AST Model: .pipeline/schema.sysml"]
        FSM_SPEC["State Transition Tables & Guard Predicates"]
        DYN_SPEC["Kinematic Equations & Transfer Curves"]
    end

    subgraph TrackA["Track A: Native MATLAB / Simulink Synthesis (Commercial Toolchain)"]
        BUILD_M["Programmatic Builder: models/scripts/build_*.m"]
        PARAMS_M["Parameter Script: models/matlab/*_params.m"]
        SLDD_DATA["Data Dictionary: models/matlab/*.sldd"]
        SLX_MODEL["Simulink / Stateflow Model (.slx)"]
        EMBEDDED_CODER["Embedded Coder: DO-178C C / SPARK Ada Synthesis"]
        SLDV_COV["Simulink Test / SLDV / Polyspace Analysis"]
    end

    subgraph TrackB["Track B: Headless CI Digital Twin Engine (License-Free Runner)"]
        RS_DOMAIN["Domain Models: models/rust/*_domain.rs"]
        RS_ENGINE["Discrete 250 Hz Engine: models/rust/*_engine.rs"]
        SIM_HARNESS["CI Simulation Runner: deap-harness"]
        RS_COV["Branch & Condition Coverage Engine"]
    end

    subgraph ParityGate["Dual-Track Parity & Verification Deliverables"]
        EQUIV["Discrete Equivalence Gate (dt = 0.004s, tol <= 1e-6)"]
        REPORT["Verification Report: docs/reports/simulink_results/*.md"]
    end

    AST --> BUILD_M
    AST --> RS_DOMAIN
    FSM_SPEC --> BUILD_M
    FSM_SPEC --> RS_ENGINE
    DYN_SPEC --> PARAMS_M
    DYN_SPEC --> RS_ENGINE

    BUILD_M --> SLX_MODEL
    PARAMS_M --> SLX_MODEL
    SLDD_DATA --> SLX_MODEL
    SLX_MODEL --> EMBEDDED_CODER
    SLX_MODEL --> SLDV_COV

    RS_DOMAIN --> RS_ENGINE
    RS_ENGINE --> SIM_HARNESS
    SIM_HARNESS --> RS_COV

    SLX_MODEL -.-> EQUIV
    RS_ENGINE -.-> EQUIV
    EQUIV --> REPORT
```

#### 1. Track A: Native MATLAB / Simulink Synthesis
- **Purpose**: DO-178C / DO-331 qualified model construction, commercial formal verification, and embedded target software generation.
- **Deliverables**:
  - Programmatic build script: `models/scripts/build_<feature_slug>_model.m` using official MATLAB/Simulink APIs (`new_system`, `add_block`, `Stateflow.Data`, `Stateflow.State`, `Stateflow.Transition`).
  - Parameter dictionaries: `models/matlab/<feature_slug>_params.m` declaring all physical parameters, mass properties, and transition thresholds.
  - Data dictionaries: `models/matlab/<feature_slug>_data.sldd` defining bus types and storage classes.
- **Solver Configuration**: Strict fixed-step discrete solver (`FixedStepDiscrete`, $dt = 0.004\text{ s}$ / 250 Hz), signal logging enabled (`logsout`), zero continuous states in control units.

#### 2. Track B: Headless CI Digital Twin Engine
- **Purpose**: Autonomous, 100% license-free regression testing, continuous integration, and subagent verification loops.
- **Deliverables**:
  - Domain state and telemetry models: `models/rust/<feature_slug>_domain.rs` with immutable data structures and explicit enum states.
  - Discrete-time simulation engine: `models/rust/<feature_slug>_engine.rs` executing synchronous time-step updates (`step(dt, inputs) -> outputs`).
  - Automated CI simulation suites: executed by `deap-harness` asserting fault injection, boundary trips, and timing bounds.
- **Runtime Model**: Operates without external MathWorks licenses, running inside standard headless Linux CI runners in under 1 second.

#### 3. Zero License Blocker Invariant
- Containerized CI/CD regression runners (GitHub Actions, GitLab CI, local CLI runners) MUST execute 100% of safety, fault-injection, and control verification scenarios without requiring a proprietary MathWorks desktop license, dongle, or cloud license server.
- All verification gates must pass cleanly in offline environments.

#### 4. Mathematical & Discrete Equivalence Mandate
- Both Track A and Track B must execute at the identical discrete time step:
  $$
  \Delta t = 0.004, \quad f_{\text{loop}} = \frac{1}{\Delta t} = 250.0
  $$
- Both tracks must evaluate identical polynomial blending curves (e.g. ASTM F3269-17 2nd-order cubic weighting):
  $$
  \lambda(\tau) = 3\tau^2 - 2\tau^3, \quad \tau = \frac{t - t_{\text{trip}}}{t_{\text{switch}}}
  $$
- Numerical precision tolerance between Simulink and the Digital Twin across identical inputs must satisfy:
  $$
  \max_k \|\mathbf{x}_{\text{Simulink}}[k] - \mathbf{x}_{\text{DigitalTwin}}[k]\|_\infty \le 10^{-6}
  $$

#### 5. DO-178C / DO-331 Alignment & Traceability
- **Traceability Thread**: Every state, action, and guard condition maps directly from SysML v2 AST (`rules/sysml-ssot-completeness.md`) to both Track A Stateflow transitions and Track B simulation engine branches.
- **Simulation Results Reporting**: Every control feature must deliver a formal verification report under `docs/reports/simulink_results/<FEATURE-ID>_simulation_results.md` detailing fault-injection test scenarios, transition logs, execution traces, and standard compliance.

---

## 8. 22-Gate Parity Lock & Automated Verification Enforcement

The integrity of the bidirectional digital thread is mechanically enforced by the 22 parity validation gates:

| Parity Gate ID | Category | Enforcing Mechanism | Blocking Threshold |
| :--- | :--- | :--- | :--- |
| **GATE-01** | Structural Completeness | `deap-harness` / `crates/verify-baseline/src/checks/uml.rs` | 100% Schema-to-Feature coverage |
| **GATE-02** | Behavioral Alignment | `deap-harness` / `crates/verify-baseline/src/checks/behavioral.rs` | 100% State/Action trigger parity |
| **GATE-03** | Use Case Realization | `deap-harness` / `crates/verify-baseline/src/checks/uml.rs` | 100% Use-Case-to-Story mapping |
| **GATE-04** | Safety Traceability | `deap-harness` / `crates/verify-baseline/src/checks/schema_mapping.rs` | 100% STPA/FMECA constraint satisfaction |
| **GATE-05** | Reverse Sync Parity | `deap-ast` / `./target/release/compile-sysml` | Zero AST delta drift |
| **GATE-06** | Mathematical KaTeX Integrity | `deap-harness` / `crates/verify-baseline/src/checks/katex.rs` | 0 dangling operators, 0 math-in-table errors |
| **GATE-07** | Dual-Track MBD Compliance | `rules/dual-track-mbd-verification.md` | Track A + Track B deliverable completeness |
| **GATE-08** | Discrete Loop Rate Parity | `deap-harness` Simulation Runner | Synchronous loop rate (dt = 0.004 s / 250 Hz) |
| **GATE-09** | Numerical Precision Bound | `deap-harness` Equivalence Engine | State divergence <= 10^-6 |
| **GATE-10** | Headless CI License Freedom | CI Test Runners | 0 proprietary license dependencies |

---

## 9. Conclusion

By treating SysML v2 as the authoritative single source of truth and binding it through automated bidirectional reverse synchronization and the Dual-Track MBD Verification Protocol, DEAP achieves uncompromised DO-178C / DO-331 high-assurance safety rigor while empowering autonomous, license-free, continuous integration pipelines.
