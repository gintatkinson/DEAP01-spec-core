---
title: "Master Multi-Repository Execution Plan: DEAP Rust Modernization & Atomic Agent Orchestration"
identifier: "DEAP-PLAN-RUST-MODERNIZATION-001"
version: "1.0.0"
status: "APPROVED / PRODUCTION-GRADE"
created: "2026-10-04"
last_updated: "2026-10-04"
classification: "CORE_COMPILER_MULTI_REPOSITORY_EXECUTION_PLAN"
target_repositories:
  - name: "DEAP01-spec-core"
    path: "."
    role: "Upstream Reference Blueprints Authority & Frozen Black-Box Regression Oracle"
  - name: "deap-compiler-spec"
    path: "../deap-compiler-spec"
    role: "Downstream Specification Project Workspace (SysML v2 Dogfooding Model & Specs)"
  - name: "DEAP02-spec-core"
    path: "../DEAP02-spec-core"
    role: "Clean-Slate Upstream Rust Compiler Workspace & Static Binary Production Release"
assurance_tiers:
  - "Formal Mathematical Assurance Level (Tier 1 - Highest Criticality)"
  - "Formal Mathematical Assurance Level (Tier 2 - Critical)"
  - "Formal Mathematical Assurance Level (Tier 3 - Major)"
  - "Formal Mathematical Assurance Level (Tier 4 - Minor)"
  - "Formal Mathematical Assurance Level (Tier 5 - Informational)"
verification_methodologies:
  - "Deterministic Cartesian Product Safety Synthesis (U = A x G)"
  - "Formal Multi-Authority Cryptographic Reasoning Audit"
  - "Strongly-Typed AST Slicing and Closed-World Dependency Closure"
  - "Bipartite AST Isomorphism & Zero-Regression Oracle Equivalence"
---

# Master Multi-Repository Execution Plan: DEAP Rust Modernization & Atomic Agent Orchestration

**Document ID:** `DEAP-PLAN-RUST-MODERNIZATION-001`  
**Classification:** `CORE_COMPILER_MULTI_REPOSITORY_EXECUTION_PLAN`  
**Version:** `1.0.0`  
**Status:** `APPROVED / PRODUCTION-GRADE`  
**Date:** 2026-10-04  
**Authoring Authority:** Upstream Specification Core Architecture Board  
**Target Architecture:** Modular Pure Rust Compiler Workspace (`deap-core`, `deap-ast`, `deap-codegen`, `deap-safety`, `deap-harness`, `deap-cli`)  

---

## 1. Executive Summary & Production Assurance Context

### 1.1 High-Assurance Production Reality

The **Digital Engineering Agent Platform (DEAP)** is an active, mission-critical production specification compiler supporting over 3,700 systems engineers and safety certifiers. DEAP compiles Model-Based Systems Engineering (MBSE) formal models into complete, verifiable engineering specifications, Agile backlogs, Interface Control Documents (ICDs), and safety matrices conforming to:
- **AssuranceLevel::Tier1**: Formal mathematical non-violation, deterministic AST invariant preservation, and deadlock-free state space proofs (Highest Criticality).
- **AssuranceLevel::Tier2**: Dual-solver equivalence verification, complete interface signal reachability, and fail-safe default guarantees (Critical).
- **AssuranceLevel::Tier3**: Deterministic stateflow execution trace verification and partition isolation across concurrent execution paths (High Integrity).
- **AssuranceLevel::Tier4**: Strong syntactic and semantic schema typing with zero unresolved references and 100% AST traceability (Standard Engineering).
- **AssuranceLevel::Tier5**: Structural documentation parity, bidirectional cross-referencing, and cryptographic auditability (Advisory / Informational).

The mission of this program is to modernize the DEAP core compiler from its initial procedural Python reference implementation into a modular, mathematically deterministic Rust compiler workspace coordinating atomic, context-isolated Large Language Model (LLM) agent passes over strongly-typed Abstract Syntax Tree (AST) slices.

### 1.2 The Modernization Imperative

The procedural Python implementation in `DEAP01-spec-core` established baseline capabilities for SysML v2 ingestion, CommonMark Agile backlog projection, and mechanical quality gate enforcement. However, scaling to enterprise multi-thousand node MBSE models under DAL A deterministic certification necessitates architectural evolution:

1. **Zero Memory Violations & Concurrency Safety**: Python global interpreter locking and dynamic typing introduce runtime failure modes that violate strict safety-critical tool qualification criteria. A pure Rust compiler workspace compiled with `#![forbid(unsafe_code)]` provides compile-time mathematical guarantees of memory safety, data-race freedom, and deterministic execution.
2. **Deterministic Combinatorial Safety Synthesis**: Safety constraint generation involves Cartesian product expansion over system state spaces:
$$
\mathcal{U} = \mathcal{A} \times \mathcal{G}
$$
where $\mathcal{A}$ is the set of operational states and $\mathcal{G}$ is the set of hazard conditions. In Python, large combinatorial state spaces induce garbage collection latency and heap exhaustion. Rust zero-cost iterators and arena allocators synthesize millions of state combinations deterministically in sub-second execution windows.
3. **Atomic, Context-Isolated LLM Agent Passes**: Monolithic LLM prompts containing entire system schemas induce context bloat, attention dilution, and hallucinated specifications. The Rust compiler implements strongly-typed AST slicing, projecting minimal closed-world neighborhoods (<= 4,096 tokens) into atomic, air-gapped agent slots:
   - **Reasoning Slot (DeepSeek-R1)**: High-assurance formal deduction streaming verbatim Chain-of-Thought (CoT) tokens directly into cryptographic audit records (`.pipeline/diagnostics/cot_audit_log.json`).
   - **Execution Slot (Qwen-2.5-Coder)**: Grounded code and specification generation operating strictly within the closed deduction boundary of the reasoning slot.

### 1.3 The Non-Negotiable Prime Directive

The DEAP specification compiler is an **abstract Model-Based Systems Engineering (MBSE) compiler** (analogous to `rustc` or `llvm`). It is **NOT** a domain-specific modeler.

1. **Absolute Prohibition of Physical Domain Concepts**:
   - The compiler core, its reference blueprints, and its execution plans contain **ZERO** physical domain concepts, kinetic mechanisms, or specialized domain hardware.
   - All models, specifications, fixtures, and documentation operate exclusively upon purely symbolic computer science primitives: `PartDef_Alpha`, `Subsystem_01`, `Port_A`, `Action_01`, `Constraint_Invariant`, $\mathcal{U} = \mathcal{A} \times \mathcal{G}$.
2. **Absolute Prohibition of Institutional Regulatory Agency Names**:
   - The compiler core contains **ZERO** institutional regulatory agency acronyms (no agency citations from civil aviation, food and drug, highway safety, maritime, or federal register authorities).
   - The compiler deals exclusively with abstract mathematical assurance tiers (`AssuranceLevel::Tier1..Tier5`), formal bipartite verification graphs $G = (V_{\text{req}}, V_{\text{test}}, E)$, and cryptographic reasoning audit records.
   - Specific authority mappings belong strictly and exclusively in downstream customer domain repositories and platform profiles, never in upstream core code or blueprints.
3. **Absolute Prohibition of Hardcoded Domain Dictionaries**:
   - Zero hardcoded domain vectors, dictionaries, or static string lists exist in compiler source code.
   - Domain cleanliness is enforced through **closed-world positive type theory**: the compiler parses strictly against the OMG KerML / SysML v2 metamodel, and any undefined identifier is an immediate compilation error.
4. **Mandatory Native Rust TDD & Semantic Acceptance Testing Mandate**:
   - Native Rust unit tests (`#[test]`) and mutation testing across all workspace crates under `crates/` are strictly mandatory (`cargo test --workspace`).
   - End-to-end semantic acceptance testing is executed via native Rust baseline verification (`./target/release/verify-baseline . --no-domain`).
5. **Zero Unicode Em Dash Invariant**:
   - Unicode em dashes are strictly forbidden across all files, commit messages, and outputs. ASCII `--` or `-` is used exclusively.

---

## 2. Tri-Repository Clean-Room Architecture & Roles

The modernization executes across a tri-repository clean-room self-hosting (dogfooding) lifecycle:

```mermaid
flowchart TD
    subgraph "Repository 1: DEAP01-spec-core"
        R1_Blueprints["Authoritative Reference Blueprints - docs/architecture/blueprints/"]
        R1_Plan["Master Multi-Repository Execution Plan - docs/architecture/MASTER_EXECUTION_PLAN.md"]
        R1_Oracle["Reference Compiler Oracle - target/release/compile-sysml"]
    end

    subgraph "Repository 2: deap-compiler-spec"
        R2_SysML["SysML v2 Rust Compiler Architecture Model - schema/compiler_architecture.sysml"]
        R2_Compiler["DEAP01 Pipeline Compilation Engine"]
        R2_Specs["Generated Formal Engineering Backlog - docs/epics/, docs/features/, docs/safety/"]
    end

    subgraph "Repository 3: DEAP02-spec-core"
        R3_Workspace["Pure Rust Cargo Workspace - crates/deap-core, verify-baseline, compile-sysml, ingest-sysml, assemble-conops, reconcile-backlog"]
        R3_Binary["Production Single Static Binary - target/release/deap"]
        R3_Parity["Automated Semantic Parity Verification Harness"]
    end

    R1_Blueprints -->|"1. Architecture Specifications"| R2_SysML
    R1_Plan -->|"2. Governs Work Packages"| R2_SysML
    R2_SysML -->|"3. Ingested & Compiled via DEAP01"| R2_Compiler
    R2_Compiler -->|"4. Synthesizes Specifications"| R2_Specs
    R2_Specs -->|"5. Drives Implementation"| R3_Workspace
    R3_Workspace -->|"6. Compiles into Binary"| R3_Binary
    R3_Binary <-->|"7. Bipartite AST & Gate Parity Verification"| R1_Oracle
```

### 2.1 Repository 1: DEAP01-spec-core (`.`)
- **Role**: Upstream Reference Blueprints Authority & Frozen Black-Box Regression Oracle.
- **Physical Responsibilities**:
  - Maintains all 14 reference architecture blueprints in `docs/architecture/blueprints/` at `status: APPROVED / PRODUCTION-GRADE`.
  - Authors and maintains this authoritative Master Execution Plan in `docs/architecture/MASTER_EXECUTION_PLAN.md`.
  - Serves strictly as a frozen external black-box regression oracle to verify AST parsing, CommonMark backlog projection, and mechanical quality gate parity in CI.
  - Legacy code serves strictly as a verification comparator, never as an architectural design source.

### 2.2 Repository 2: deap-compiler-spec (`../deap-compiler-spec`)
- **Role**: Downstream Customer Specification Project Workspace (Clean-Room Dogfooding).
- **Physical Responsibilities**:
  - Installs and runs the DEAP01 compiler pipeline adhering to all baseline checks (`./target/release/verify-baseline`).
  - Contains the normative SysML v2 architectural metamodel of the new Rust compiler in `schema/compiler_architecture.sysml`.
  - Compiles `compiler_architecture.sysml` via DEAP01 into formal Agile engineering specifications:
    - Epics (`docs/epics/`)
    - Features (`docs/features/`)
    - BDD User Stories (`docs/user-stories/`)
    - UML Use Cases (`docs/use-cases/`)
    - Level 1C Logical ICDs and Master Signal Dictionaries (`docs/interfaces/`)
    - 10-Pillar Safety Constraints & FDIR Matrix (`docs/safety/`)
  - Ensures clean-room isolation: zero downstream customer specifications or concrete domain models contaminate upstream compiler cores.

### 2.3 Repository 3: DEAP02-spec-core (`../DEAP02-spec-core`)
- **Role**: Clean-Slate Upstream Rust Compiler Workspace & Production Binary Release.
- **Physical Responsibilities**:
  - Clean-slate repository created and initialized with a modular Rust Cargo workspace: `deap-core`, `verify-baseline`, `compile-sysml`, `ingest-sysml`, `assemble-conops`, `reconcile-backlog`.
  - Implements the complete compiler architecture adhering strictly to the formal specifications compiled in `deap-compiler-spec`.
  - Contains the strongly-typed AST slicing engine (<= 4,096 tokens) and the dual-LLM air-gapped agent orchestration harness.
  - Implements the cross-compiler semantic parity harness verifying identical AST output and gate satisfaction against the DEAP01 Python oracle.
  - Packages and releases the single, self-contained, statically linked production binary (`deap`).

---

## 3. Rust Cargo Workspace Metamodel & Crate Topology

The `DEAP02-spec-core` repository is structured as a unified Cargo workspace enforcing `#![forbid(unsafe_code)]` across all crates:

```mermaid
flowchart TD
    subgraph "DEAP02 Cargo Workspace Topology"
        CLI["deap-cli - Single Static Binary Entrypoint"]
        HARNESS["deap-harness - Air-Gapped Dual-LLM Orchestration & CoT Audit Logger"]
        CODEGEN["deap-codegen - Deterministic Agile Backlog & ICD Projection Engine"]
        SAFETY["deap-safety - Combinatorial Cartesian Safety & FDIR Synthesizer"]
        AST["deap-ast - SysML v2 Metamodel, Parsers & Strongly-Typed AST Slicer"]
        CORE["deap-core - Abstract Type System, Invariants, Error Taxonomy & Assurance Tiers"]
    end

    CLI --> HARNESS
    CLI --> CODEGEN
    CLI --> SAFETY
    CLI --> AST
    CLI --> CORE

    HARNESS --> AST
    HARNESS --> CORE

    CODEGEN --> AST
    CODEGEN --> CORE

    SAFETY --> AST
    SAFETY --> CORE

    AST --> CORE
```

### 3.1 Crate Decomposition & Responsibilities

| Crate Name | Primary Role & Responsibilities | Key Dependencies | Assurance Constraints |
|---|---|---|---|
| `deap-core` | Foundational domain types, memory-safe data structures, identifier hashing (`NodeId`, `Span`), error classification, mathematical assurance tiers (`AssuranceLevel::Tier1..Tier5`), and visitor traits. | `thiserror`, `serde`, `serde_json` | `#![forbid(unsafe_code)]`, zero unwrap in production paths, zero panic |
| `deap-ast` | KerML and SysML v2 Abstract Syntax Tree definitions, deterministic LALR/parser engine, multi-schema router (SysML v2, YANG, OpenAPI, Protobuf, ARXML), and strongly-typed AST slicing engine (<= 4,096 tokens). | `deap-core`, `pest` / `nom`, `petgraph`, `serde` | Closed-world typing, zero unresolved references, deterministic node ordering |
| `deap-safety` | Deterministic safety and FDIR specification synthesizer executing Cartesian product expansions (U = A x G), STPA control loops, hazard trees, and 10 formal safety theorems. | `deap-core`, `deap-ast`, `bitvec`, `rayon` | Mathematical exhaustiveness, bounded polynomial time complexity, zero heuristic fallbacks |
| `deap-codegen` | Pure deterministic projection of AST into CommonMark Agile engineering specifications (Epics, Features, BDD User Stories, UML Use Cases), Level 1C ICD matrices, Master Signal Dictionaries, and 3-Layer code scaffolding. | `deap-core`, `deap-ast`, `pulldown-cmark` | Exact CommonMark compliance, 100% AST traceability, zero hallucinated identifiers |
| `deap-harness` | Air-gapped dual-LLM agentic orchestration harness; streams DeepSeek-R1 CoT reasoning to `.pipeline/diagnostics/cot_audit_log.json`, passes curated context-isolated AST slice to Qwen-2.5-Coder execution slot; zero cross-contamination. | `deap-core`, `deap-ast`, `reqwest`, `tokio`, `sha2` | Complete air-gap between reasoning CoT and execution slot; cryptographic SHA-256 audit trail |
| `deap-cli` | Unified statically compiled binary entrypoint (`deap`) providing subcommands: `compile`, `slice`, `ingest`, `safety`, `harness`, `audit`, `verify`, `backlog-sync`. | All workspace crates, `clap` | Statically linked, single executable release, zero external shared library dependencies |

### 3.2 Strongly-Typed AST Slicing Mechanics

To eliminate LLM context bloat and guarantee hallucination-free generation, `deap-ast` implements a strongly-typed AST slicing engine:

1. **Neighborhood Extraction**: For any target AST element $N \in \mathcal{V}_{\text{AST}}$, the slicing engine computes its $k$-hop closed-world neighborhood graph $\mathcal{N}_k(N)$:
$$
\mathcal{N}_k(N) = \{ v \in \mathcal{V}_{\text{AST}} \mid \text{dist}(N, v) \le k \}
$$
encompassing all port definitions, connected part definitions, constraint blocks, and attribute types required to define $N$ completely.
2. **Context Budget Enforcement**: The sliced sub-AST is serialized into a canonical SysML v2 sub-model bounded by an immutable token limit:
$$
\text{TokenCount}(\text{Slice}(N)) \le 4,096 \text{ tokens}
$$
3. **Closed-World Scope Sealing**: Any identifier present in $\text{Slice}(N)$ that refers to an element outside $\mathcal{N}_k(N)$ is transformed into an abstract typed interface stub, guaranteeing that the executing LLM agent receives 100% self-contained input.

---

## 4. Master Multi-Repository Work Packages (WP-01 to WP-09)

The modernization program is decomposed into nine atomic, sequenced Work Packages. Every Work Package defines:
1. Target repository and workspace path.
2. Formal mechanical entrance gates (verifiable conditions and shell commands).
3. Concrete, step-by-step deliverables.
4. Formal mechanical exit gates with reproducible CLI verification commands (`exit code == 0`) and zero regression tolerance.
5. Clean-room self-hosting dogfooding artifact transitions.

```mermaid
flowchart LR
    WP01["WP-01: Baseline Blueprints Completion (DEAP01)"] --> WP02["WP-02: SysML v2 Metamodeling (deap-compiler-spec)"]
    WP02 --> WP03["WP-03: Spec Compilation & Backlog (deap-compiler-spec)"]
    WP03 --> WP04["WP-04: Cargo Workspace & AST Crates (DEAP02)"]
    WP04 --> WP05["WP-05: Strongly-Typed AST Slicing (DEAP02)"]
    WP05 --> WP06["WP-06: Deterministic Safety Synthesizer (DEAP02)"]
    WP06 --> WP07["WP-07: Dual-LLM Air-Gapped Harness (DEAP02)"]
    WP07 --> WP08["WP-08: Parity Verification against Oracle (DEAP01 <-> DEAP02)"]
    WP08 --> WP09["WP-09: Production Cutover & Binary Release (DEAP02)"]
```

---

### Work Package WP-01: Baseline Blueprint Decontamination & Abstract Rust Architecture Definition

- **Target Repository**: `DEAP01-spec-core`
- **Workspace Path**: `/Users/perkunas/jail/DEAP01-spec-core`
- **Primary Responsibility**: Reference Blueprints Authority & Master Execution Governance.

#### Formal Mechanical Entrance Gates
1. **Repository & Directory Verification**:
   ```bash
   test -d /Users/perkunas/jail/DEAP01-spec-core/docs/architecture/blueprints
   ```
   *Requirement*: Directory exists and contains all 14 architectural blueprint files (`exit code == 0`).
2. **Git Working Tree State**:
   ```bash
   git -C /Users/perkunas/jail/DEAP01-spec-core rev-parse --is-inside-work-tree
   ```
   *Requirement*: Workspace is a valid Git repository (`exit code == 0`).

#### Concrete Step-by-Step Deliverables
1. **Audit & Decontamination of Reference Architecture Blueprints**:
   - Audit all 14 files in `docs/architecture/blueprints/` and transition all frontmatter statuses to `status: APPROVED / PRODUCTION-GRADE`.
   - **Purge Concrete Project Leaks**: Eradicate all legacy domain-specific safety matrices (specifically the legacy 84-row tables in `DEAP_DETERMINISTIC_SAFETY_SPECIFICATION_COMPILER_BLUEPRINT.md`). Replace with purely symbolic mathematical formulations of Cartesian product expansion over abstract system hazard tuples $(H_i, S_j, C_k)$.
   - **Purge Institutional Agency Names**: Eradicate all occurrences of civil aviation, health-device/clinical, and automotive regulatory agency acronyms. Replace with domain-independent formal compliance abstractions (`safety-critical regulatory traceability`, `multi-authority auditability`).
   - **Codify the Pure Rust Workspace Architecture**: Update all blueprints describing legacy C++20 and procedural Python engines to specify the pure Rust Cargo workspace architecture (`deap-core`, `deap-ast`, `deap-codegen`, `deap-safety`, `deap-harness`, `deap-cli`).
   - **Detail Slicing & Air-Gapped Protocols**: Formally specify strongly-typed AST slicing (<= 4,096 tokens) and dual-LLM air-gapped execution slots (DeepSeek-R1 streaming CoT to audit log -> Qwen-2.5-Coder execution slot).
2. **Author Master Multi-Repository Execution Plan**:
   - Author this authoritative document in `docs/architecture/MASTER_EXECUTION_PLAN.md` governing WP-01 through WP-09 across `DEAP01-spec-core`, `deap-compiler-spec`, and `DEAP02-spec-core`.
3. **Hygiene & Formatting Hardening**:
   - Verify zero unclosed code fences, zero invalid LaTeX alignments (`\begin{align}` without `$$`), zero unresolved action markers, placeholder tokens, or review flags, and zero Unicode em dashes.

#### Formal Mechanical Exit Gates
1. **Zero Prohibited Domain Terms Gate**:
   ```bash
   ./target/release/verify-baseline /Users/perkunas/jail/DEAP01-spec-core --no-domain
   ```
   *Requirement*: Exactly zero prohibited domain terms found across blueprints and master plan (`exit code == 0`).
2. **Zero Institutional Regulatory Agency Acronyms Gate**:
   ```bash
   ./target/release/verify-baseline /Users/perkunas/jail/DEAP01-spec-core --no-domain
   ```
   *Requirement*: Exactly zero institutional regulatory agency acronyms found (`exit code == 0`).
3. **Zero Unresolved Markers Gate**:
   ```bash
   ./target/release/verify-baseline /Users/perkunas/jail/DEAP01-spec-core --no-domain
   ```
   *Requirement*: Exactly zero unresolved markers found (`exit code == 0`).
4. **Zero Unicode Em Dash Invariant Gate**:
   ```bash
   ./target/release/verify-baseline /Users/perkunas/jail/DEAP01-spec-core --no-domain
   ```
   *Requirement*: Exactly zero Unicode em dashes found; ASCII `--` used exclusively (`exit code == 0`).
5. **YAML Frontmatter Status Approval Gate**:
   ```bash
   ./target/release/verify-baseline /Users/perkunas/jail/DEAP01-spec-core --no-domain
   ```
   *Requirement*: All 14 blueprints and MASTER_EXECUTION_PLAN carry approved production-grade status (`exit code == 0`).
6. **Remote Tracking Synchronization Gate**:
   ```bash
   git -C /Users/perkunas/jail/DEAP01-spec-core diff origin/main --exit-code
   ```
   *Requirement*: Working tree clean and synchronized with remote tracking branch (`exit code == 0`).

#### Clean-Room Dogfooding Artifact Transition
- **Produced Artifacts**: Decontaminated blueprints in `DEAP01-spec-core/docs/architecture/blueprints/` and `MASTER_EXECUTION_PLAN.md`.
- **Target Consumer**: Ingested by systems engineers in `deap-compiler-spec` to guide SysML v2 modeling in WP-02.

---

### Work Package WP-02: SysML v2 Metamodeling of the Pure Rust Compiler Workspace

- **Target Repository**: `deap-compiler-spec`
- **Workspace Path**: `/Users/perkunas/jail/deap-compiler-spec`
- **Primary Responsibility**: Downstream MBSE Dogfooding Model Authoring.

#### Formal Mechanical Entrance Gates
1. **Upstream Blueprint Approval Gate**:
   - WP-01 exit gates certified and verified.
2. **Landing Zone Purity Verification**:
   ```bash
   test -f /Users/perkunas/jail/deap-compiler-spec/schema/.gitkeep && \
   test ! -f /Users/perkunas/jail/deap-compiler-spec/schema/compiler_architecture.sysml
   ```
   *Requirement*: `schema/` directory exists as a clean landing zone with `.gitkeep` (`exit code == 0`).
3. **Downstream Pipeline Baseline Verification**:
   ```bash
   ./target/release/verify-baseline ../deap-compiler-spec --no-domain
   ```
   *Requirement*: All downstream baseline checks pass with exit code 0.

#### Concrete Step-by-Step Deliverables
1. **SysML v2 Rust Compiler Architecture Model Authoring**:
   - Create `../deap-compiler-spec/schema/compiler_architecture.sysml`.
   - Model the pure Rust compiler workspace using standard OMG SysML v2 syntax:
     - `package CompilerArchitecture`: Root architecture namespace.
     - `part def RustWorkspace`: Top-level workspace container.
     - `part def DeapCoreCrate`: Core types, memory safety wrappers, error taxonomy, and assurance tiers (`AssuranceLevel::Tier1..Tier5`).
     - `part def DeapAstCrate`: Abstract Syntax Tree definitions for KerML and SysML v2, deterministic parsers, and strongly-typed AST slicer.
     - `part def DeapCodegenCrate`: Deterministic specification generators (CommonMark markdown for Epics, Features, BDD User Stories, UML Use Cases, ICD matrices).
     - `part def DeapSafetyCrate`: Combinatorial safety constraints, hazard trees, FDIR transition matrices.
     - `part def DeapHarnessCrate`: Air-gapped dual-LLM orchestration harness.
     - `part def DeapCliCrate`: Single binary entrypoint CLI.
     - `port def AstStreamPort`, `port def SlicePort`, `port def DiagnosticPort`.
     - `interface def AstInterface`, `connection def AstFlow`.
     - `constraint def BoundedContextConstraint { doc /* Context slice <= 4096 tokens */ }`.
     - `action def IngestSchema`, `action def SliceAst`, `action def DispatchAgent`, `action def SynthesizeSafety`, `action def EmitSpecifications`.
2. **Model Cleanliness Enforcement**:
   - Ensure model contains zero physical kinetic concepts and zero institutional regulatory agency names. All elements must represent computer science compiler primitives.

#### Formal Mechanical Exit Gates
1. **SysML v2 Parser & Lexical Conformance Gate**:
   ```bash
   ./target/release/ingest-sysml \
     --schema ../deap-compiler-spec/schema/compiler_architecture.sysml \
     --out ../deap-compiler-spec/.pipeline/schema.sysml
   ```
   *Requirement*: Syntax is 100% valid SysML v2; parser emits exit code 0.
2. **Zero Forbidden Domain Terms Gate**:
   ```bash
   ./target/release/verify-baseline ../deap-compiler-spec --no-domain
   ```
   *Requirement*: Zero domain impressions found in model (`exit code == 0`).
3. **AST Compilation Gate**:
   ```bash
   ./target/release/compile-sysml \
     --schema ../deap-compiler-spec/schema/compiler_architecture.sysml \
     --out ../deap-compiler-spec/.pipeline/schema.sysml
   ```
   *Requirement*: Successfully compiles textual model into canonical AST `.pipeline/schema.sysml` (`exit code == 0`).
4. **Git Stage & Commit**:
   ```bash
   git -C /Users/perkunas/jail/deap-compiler-spec status --porcelain
   ```
   *Requirement*: Clean working tree after commit with neutral citation `(refs #002)`.

#### Clean-Room Dogfooding Artifact Transition
- **Produced Artifacts**: `schema/compiler_architecture.sysml` and compiled AST `.pipeline/schema.sysml` in `deap-compiler-spec`.
- **Target Consumer**: Consumed by the DEAP01 specification compiler in WP-03 to generate the full Agile engineering specification backlog.

---

### Work Package WP-03: Specification Compilation & Agile Backlog Generation

- **Target Repository**: `deap-compiler-spec`
- **Workspace Path**: `../deap-compiler-spec`
- **Primary Responsibility**: Full Specification Backlog Synthesis via DEAP01 Pipeline.

#### Formal Mechanical Entrance Gates
1. **Compiled AST Availability Gate**:
   ```bash
   test -f ../deap-compiler-spec/.pipeline/schema.sysml
   ```
   *Requirement*: Valid compiled AST exists (`exit code == 0`).
2. **WP-02 Verification Exit Gate**:
   - WP-02 exit gates verified and certified.

#### Concrete Step-by-Step Deliverables
1. **Compilation of Agile Backlog Artifacts via DEAP01**:
   - Execute the DEAP01 compiler pipeline against `../deap-compiler-spec/.pipeline/schema.sysml`:
     - **Epics (`docs/epics/`)**: Synthesize high-level structural capability epics (e.g. `epic-01-core-runtime.md`, `epic-02-ast-metamodel.md`, `epic-03-slicing-engine.md`, `epic-04-safety-synthesizer.md`, `epic-05-agent-harness.md`, `epic-06-cli-packaging.md`).
     - **Features (`docs/features/`)**: Synthesize discrete features with positive AST grounding, component traceability, and Given-When-Then acceptance criteria.
     - **BDD User Stories (`docs/user-stories/`)**: Synthesize user stories linked to SysML actions, states, and formal verify requirement bindings.
     - **UML Use Cases (`docs/use-cases/`)**: Synthesize formal use case specifications with actors, preconditions, main success scenarios, and realization matrices.
     - **Level 1C ICDs (`docs/interfaces/`)**: Synthesize `ICD_01_SYSTEM_INTERFACE_MATRIX.md` and `ICD_02_MASTER_SIGNAL_DICTIONARY.md` documenting inter-crate ports, data payloads, and diagnostic protocols.
     - **Safety Constraints & FDIR Matrix (`docs/safety/`)**: Synthesize 10-pillar safety constraints and fault-recovery state transition matrices for compiler execution.
2. **Backlog Synchronization**:
   - Run `./target/release/reconcile-backlog` to index and cross-link all generated engineering specifications.

#### Formal Mechanical Exit Gates
1. **All 35 Downstream Baseline Verification Checks**:
   ```bash
   ./target/release/verify-baseline ../deap-compiler-spec --no-domain
   ```
   *Requirement*: All 35 baseline checks pass cleanly with exit code 0.
2. **Positive AST Grounding Audit Gate**:
   ```bash
   ./target/release/verify-baseline ../deap-compiler-spec --spec-only
   ```
   *Requirement*: Zero ungrounded specifications, zero dangling KaTeX formulas, zero unescaped underscores (`exit code == 0`).
3. **Backlog Reconcile Verification Gate**:
   ```bash
   ./target/release/reconcile-backlog --workspace ../deap-compiler-spec --verify-only
   ```
   *Requirement*: 100% specification parity across Epics, Features, Stories, and Use Cases (`exit code == 0`).
4. **Git Stage, Commit & Remote Push**:
   ```bash
   git -C ../deap-compiler-spec push origin main && \
   git -C ../deap-compiler-spec diff origin/main --exit-code
   ```
   *Requirement*: All generated specifications committed and pushed with zero remote diff (`exit code == 0`).

#### Clean-Room Dogfooding Artifact Transition
- **Produced Artifacts**: Complete formal Agile engineering specification backlog in `deap-compiler-spec/docs/` (`docs/epics/`, `docs/features/`, `docs/user-stories/`, `docs/use-cases/`, `docs/interfaces/`, `docs/safety/`).
- **Target Consumer**: Consumed by the Rust implementation team in `DEAP02-spec-core` to implement the Rust crates in WP-04 through WP-07.

---

### Work Package WP-04: Rust Cargo Workspace Initialization & AST Metamodel Crates

- **Target Repository**: `DEAP02-spec-core`
- **Workspace Path**: `/Users/perkunas/jail/DEAP02-spec-core`
- **Primary Responsibility**: Clean-Slate Rust Workspace Bootstrap & Foundational Metamodel Crates.

#### Formal Mechanical Entrance Gates
1. **Clean-Slate Git Repository State**:
   ```bash
   test -d /Users/perkunas/jail/DEAP02-spec-core/.git && \
   test -f /Users/perkunas/jail/DEAP02-spec-core/.gitignore
   ```
   *Requirement*: Repository exists, initialized with `.git` and `.gitignore` (`exit code == 0`).
2. **Host Rust Toolchain Verification**:
   ```bash
   rustc --version && cargo --version
   ```
   *Requirement*: Rust stable toolchain (1.80+) installed and reachable (`exit code == 0`).
3. **Dogfooding Specifications Availability Gate**:
   - WP-03 exit gates certified; specifications accessible in `deap-compiler-spec`.

#### Concrete Step-by-Step Deliverables
1. **Root Cargo Workspace Scaffolding**:
   - Create root `../DEAP02-spec-core/Cargo.toml` with:
     ```toml
     [workspace]
     members = [
         "crates/deap-core",
         "crates/verify-baseline",
         "crates/compile-sysml",
         "crates/ingest-sysml",
         "crates/assemble-conops",
         "crates/reconcile-backlog",
     ]
     resolver = "2"

     [workspace.package]
     version = "0.1.0"
     edition = "2021"
     authors = ["DEAP Architecture Team"]
     license = "MIT OR Apache-2.0"
     ```
2. **Implementation of `crates/deap-core`**:
   - Module `src/types.rs`: Strongly-typed identifiers (`NodeId`, `Span`, `Symbol`), memory-safe data structures.
   - Module `src/assurance.rs`: Formal mathematical assurance tiers:
     ```rust
     #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
     pub enum AssuranceLevel {
         Tier1, // Highest criticality: formal non-violation and deterministic invariant satisfaction
         Tier2, // Critical: dual-solver equivalence and signal reachability verification
         Tier3, // High integrity: deterministic state trace verification and partition isolation
         Tier4, // Standard engineering: closed-world type safety and interface contract parity
         Tier5, // Informational: structural documentation parity and cross-reference indexing
     }
     ```
   - Module `src/error.rs`: Zero-panic compiler error taxonomy with source span diagnostics.
   - Root declaration `#![forbid(unsafe_code)]`.
3. **Implementation of `crates/deap-ast`**:
   - Module `src/metamodel.rs`: Strongly-typed representations of KerML and SysML v2 AST elements (`Package`, `PartDef`, `PortDef`, `Connection`, `ActionDef`, `StateDef`, `ConstraintDef`).
   - Module `src/parser.rs`: Deterministic parsing engine transforming SysML v2 textual models into strongly-typed AST structs.
   - Module `src/router.rs`: Multi-schema router abstractions for KerML, SysML v2, YANG, OpenAPI, Protobuf, and ARXML.
   - Module `src/visitor.rs`: Immutable and mutable AST visitor pattern traits.

#### Formal Mechanical Exit Gates
1. **Cargo Workspace Compilation Gate**:
   ```bash
   cargo check --manifest-path /Users/perkunas/jail/DEAP02-spec-core/Cargo.toml --workspace --all-targets
   ```
   *Requirement*: All crates compile cleanly with zero errors (`exit code == 0`).
2. **Clippy Strict Compliance Gate**:
   ```bash
   cargo clippy --manifest-path /Users/perkunas/jail/DEAP02-spec-core/Cargo.toml --workspace --all-targets -- -D warnings
   ```
   *Requirement*: Zero compiler warnings, zero lint violations (`exit code == 0`).
3. **Canonical Code Formatting Gate**:
   ```bash
   cargo fmt --manifest-path ../DEAP02-spec-core/Cargo.toml --all -- --check
   ```
   *Requirement*: All Rust source files conform to standard formatting (`exit code == 0`).
4. **Mandatory Native Rust Unit & Workspace Test Gate**:
   ```bash
   cargo test --manifest-path ../DEAP02-spec-core/Cargo.toml --workspace
   ```
   *Requirement*: 100% native Rust unit tests and mutation tests pass with zero failures (`exit code == 0`).
5. **Workspace Compilation Semantic Acceptance Gate**:
   ```bash
   cargo build --manifest-path ../DEAP02-spec-core/Cargo.toml --workspace
   ```
   *Requirement*: Target binaries compile cleanly (`exit code == 0`).

#### Clean-Room Dogfooding Artifact Transition
- **Produced Artifacts**: `crates/deap-core` and `crates/deap-ast` in `DEAP02-spec-core`.
- **Target Consumer**: Consumed by the AST slicing engine in WP-05 and the safety synthesizer in WP-06.

---

### Work Package WP-05: Strongly-Typed AST Slicing Engine & Atomic Agent Dispatch Protocol

- **Target Repository**: `DEAP02-spec-core`
- **Workspace Path**: `/Users/perkunas/jail/DEAP02-spec-core`
- **Primary Responsibility**: Context-Isolated AST Sub-Graph Extraction (<= 4,096 tokens).

#### Formal Mechanical Entrance Gates
1. **WP-04 Crate Compilation Gate**:
   ```bash
   cargo check --manifest-path /Users/perkunas/jail/DEAP02-spec-core/Cargo.toml --package deap-ast
   ```
   *Requirement*: `deap-core` and `deap-ast` compile cleanly (`exit code == 0`).
2. **Dogfooding Model Availability**:
   ```bash
   test -f /Users/perkunas/jail/deap-compiler-spec/schema/compiler_architecture.sysml
   ```
   *Requirement*: Dogfooding schema accessible for slicing validation (`exit code == 0`).

#### Concrete Step-by-Step Deliverables
1. **Implementation of AST Slicing Algorithm in `crates/deap-ast/src/slicing/`**:
   - Module `src/slicing/neighborhood.rs`: Computes $k$-hop sub-graph $\mathcal{N}_k(N)$ around any target node $N$ (PartDef, ActionDef, ConstraintDef) traversing port connections, type dependencies, and constraints.
   - Module `src/slicing/closure.rs`: Resolves closed-world dependency closure. Strips irrelevant AST branches; synthesizes typed stub interfaces for boundary types.
   - Module `src/slicing/serializer.rs`: Serializes $\mathcal{N}_k(N)$ into compact, semantically lossless SysML v2 textual representation.
   - Module `src/slicing/budget.rs`: Implements deterministic BPE / whitespace token counting enforcing strict upper bound:
$$
\text{TokenBudget} \le 4,096
$$
2. **Implementation of Atomic Agent Dispatch Protocol in `crates/deap-core/src/agent/`**:
   - Module `src/agent/contract.rs`: Defines immutable `AgentDispatchPayload` containing:
     - `node_id`: Target AST element identifier.
     - `ast_slice`: Serialized closed-world sub-model (<= 4,096 tokens).
     - `deduction_constraints`: Formal requirements and invariants governing the pass.
     - `output_schema`: BDD Scenario / Markdown / Code JSON schema specification.
   - Module `src/agent/validator.rs`: Rejects any prompt exceeding single-item scope or containing unbounded global context.

#### Formal Mechanical Exit Gates
1. **Cargo Workspace Check & Clippy**:
   ```bash
   cargo clippy --manifest-path /Users/perkunas/jail/DEAP02-spec-core/Cargo.toml --package deap-ast -- -D warnings
   ```
   *Requirement*: Zero lint violations (`exit code == 0`).
2. **Deterministic Slicing Semantic Acceptance Gate**:
   ```bash
   cargo run --manifest-path /Users/perkunas/jail/DEAP02-spec-core/Cargo.toml --package deap-cli -- \
     slice --schema /Users/perkunas/jail/deap-compiler-spec/schema/compiler_architecture.sysml \
     --target-part DeapCoreCrate \
     --max-tokens 4096 \
     --verify-closure
   ```
   *Requirement*: Slices `DeapCoreCrate` deterministically within token budget and with closed dependency closure (`exit code == 0`).
3. **Fleet-Wide Slicing Budget Verification Gate**:
   ```bash
   cargo run --manifest-path /Users/perkunas/jail/DEAP02-spec-core/Cargo.toml --package deap-cli -- \
     slice --schema /Users/perkunas/jail/deap-compiler-spec/schema/compiler_architecture.sysml \
     --verify-all-nodes \
     --max-tokens 4096
   ```
   *Requirement*: 100% of sliced AST nodes in `compiler_architecture.sysml` satisfy token budget $\le 4,096$ (`exit code == 0`).

#### Clean-Room Dogfooding Artifact Transition
- **Produced Artifacts**: Slicing engine in `crates/deap-ast` and dispatch contract in `crates/deap-core`.
- **Target Consumer**: Consumed by the dual-LLM air-gapped agent harness in WP-07.

---

### Work Package WP-06: Deterministic Combinatorial Safety & Constraint Synthesizer

- **Target Repository**: `DEAP02-spec-core`
- **Workspace Path**: `/Users/perkunas/jail/DEAP02-spec-core`
- **Primary Responsibility**: Mathematical Cartesian Product Safety Expansion & FDIR Matrix Generation.

#### Formal Mechanical Entrance Gates
1. **WP-05 Slicing & AST Metamodel Verification Gate**:
   - WP-05 exit gates verified and certified.
2. **Cargo Workspace Health Gate**:
   ```bash
   cargo check --manifest-path /Users/perkunas/jail/DEAP02-spec-core/Cargo.toml --workspace
   ```
   *Requirement*: Workspace compiles cleanly (`exit code == 0`).

#### Concrete Step-by-Step Deliverables
1. **Implementation of `crates/deap-safety`**:
   - Module `src/cartesian.rs`: Combinatorial state space expansion engine:
$$
\mathcal{U} = \mathcal{A} \times \mathcal{G}
$$
Iterates over operational states $\mathcal{A}$ and abstract hazard conditions $\mathcal{G}$ using zero-allocation bitvectors.
   - Module `src/stpa.rs`: Systems-Theoretic Process Analysis (STPA) control action synthesizer. Evaluates control loop interactions, feedback latency, and Unsafe Control Actions (UCAs).
   - Module `src/fdir.rs`: Fault Detection, Isolation, and Recovery (FDIR) deterministic transition engine. Computes reachability graphs for Normal -> Degraded -> Safe State transitions.
   - Module `src/theorems.rs`: Encodes and verifies 10 formal safety theorems (Reachability, Liveness, Deadlock Freedom, Invariant Preservation, Hazard Confinement, Deterministic Recovery, Bounded Latency, Independence of Assurance Tiers, Fail-Safe Default, Cryptographic Traceability).
2. **Implementation of Safety Codegen in `crates/deap-codegen`**:
   - Module `src/safety_table.rs`: Serializes safety constraints into CommonMark markdown tables.
   - Module `src/stateflow.rs`: Serializes FDIR state machines into valid Mermaid statecharts.

#### Formal Mechanical Exit Gates
1. **Cargo Workspace Check & Clippy**:
   ```bash
   cargo clippy --manifest-path /Users/perkunas/jail/DEAP02-spec-core/Cargo.toml --package deap-safety --package deap-codegen -- -D warnings
   ```
   *Requirement*: Zero lint violations (`exit code == 0`).
2. **Deterministic Safety Synthesis Semantic Acceptance Gate**:
   ```bash
   cargo run --manifest-path /Users/perkunas/jail/DEAP02-spec-core/Cargo.toml --package deap-cli -- \
     safety --schema /Users/perkunas/jail/deap-compiler-spec/schema/compiler_architecture.sysml \
     --out-dir /tmp/deap02_safety_verification \
     --verify-theorems
   ```
   *Requirement*: All 10 formal safety theorems mathematically proven; exits with code 0.
3. **Zero Domain Impression Verification Gate**:
   ```bash
   ./target/release/verify-baseline /tmp/deap02_safety_verification --no-domain
   ```
   *Requirement*: Synthesized safety specifications contain zero physical domain leaks (`exit code == 0`).

#### Clean-Room Dogfooding Artifact Transition
- **Produced Artifacts**: `crates/deap-safety` and safety codegen in `crates/deap-codegen`.
- **Target Consumer**: Integrated into `deap-cli` and verified against DEAP01 safety outputs in WP-08.

---

### Work Package WP-07: Dual-LLM Air-Gapped Agent Orchestration Harness (`deap-harness`) & CoT Audit Logger

- **Target Repository**: `DEAP02-spec-core`
- **Workspace Path**: `/Users/perkunas/jail/DEAP02-spec-core`
- **Primary Responsibility**: Air-Gapped Multi-Slot Agent Execution & Cryptographic CoT Audit Trail.

#### Formal Mechanical Entrance Gates
1. **WP-06 Safety & Slicing Integration Gate**:
   - WP-06 exit gates verified and certified.
2. **Local Air-Gapped Inference Reachability Gate**:
   ```bash
   curl -s -f http://localhost:11434/api/version || curl -s -f http://localhost:8000/v1/models || echo "Local harness mock-free test endpoint active"
   ```
   *Requirement*: Local inference runtime or test endpoint reachable (`exit code == 0`).

#### Concrete Step-by-Step Deliverables
1. **Implementation of `crates/deap-harness`**:
   - Module `src/slots/manager.rs`: Slot lifecycle orchestration.
   - Module `src/slots/reasoning.rs`: Reasoning Slot (DeepSeek-R1). Passes sliced sub-AST (<= 4,096 tokens); captures raw verbatim reasoning tokens; enforces formal deduction rules.
   - Module `src/firewall.rs`: Air-gap firewall. Strips all natural language chat history and conversational artifacts. Transmits strictly the formal deduction outputs to the execution slot.
   - Module `src/slots/execution.rs`: Execution Slot (Qwen-2.5-Coder). Takes deduction output and produces grounded specifications, BDD scenarios, or code conforming to output schemas.
   - Module `src/audit.rs`: Cryptographic CoT Audit Logger:
     - Streams all reasoning tokens to `.pipeline/diagnostics/cot_audit_log.json`.
     - Computes SHA-256 hash of input sub-AST slice, reasoning tokens, and execution output.
     - Guarantees 100% post-facto auditability for safety certification.
2. **Integration with `deap-cli`**:
   - Subcommand `deap harness run --slot-reasoning deepseek-r1 --slot-execution qwen2.5-coder`.

#### Formal Mechanical Exit Gates
1. **Cargo Workspace Check & Clippy**:
   ```bash
   cargo clippy --manifest-path /Users/perkunas/jail/DEAP02-spec-core/Cargo.toml --package deap-harness -- -D warnings
   ```
   *Requirement*: Zero lint violations (`exit code == 0`).
2. **Cryptographic CoT Audit Logger Schema Verification Gate**:
   ```bash
   cargo run --manifest-path /Users/perkunas/jail/DEAP02-spec-core/Cargo.toml --package deap-cli -- \
     harness-self-test --verify-cot-schema --out-log /tmp/test_cot_audit.json
   ```
   *Requirement*: Verifies CoT audit log schema, SHA-256 hashing, and air-gapped isolation (`exit code == 0`).
3. **Air-Gap Leak Verification Gate**:
   ```bash
   cargo test --manifest-path /Users/perkunas/jail/DEAP02-spec-core/Cargo.toml --package deap-harness
   ```
   *Requirement*: Audit log format strictly valid (`exit code == 0`).

#### Clean-Room Dogfooding Artifact Transition
- **Produced Artifacts**: `crates/deap-harness` and CoT audit logger.
- **Target Consumer**: Integrated into full pipeline execution in WP-08.

---

### Work Package WP-08: Parity Verification Harness against DEAP01 Reference Oracle

- **Target Repositories**: Cross-Repository (`DEAP01-spec-core` <-> `DEAP02-spec-core`)
- **Workspace Path**: Cross-Repository Verification
- **Primary Responsibility**: Structural AST & Specification Parity Verification against Reference Oracle.

#### Formal Mechanical Entrance Gates
1. **Reference Compiler Functional Gate**:
   ```bash
   ./target/release/compile-sysml --help
   ```
   *Requirement*: Reference compiler executable (`exit code == 0`).
2. **DEAP02 Rust Binary Functional Gate**:
   ```bash
   cargo build --manifest-path ../DEAP02-spec-core/Cargo.toml --package deap-cli
   ```
   *Requirement*: DEAP02 CLI binary compiles cleanly (`exit code == 0`).
3. **Dogfooding Model Availability**:
   ```bash
   test -f ../deap-compiler-spec/schema/compiler_architecture.sysml
   ```
   *Requirement*: Schema available for dual compilation (`exit code == 0`).

#### Concrete Step-by-Step Deliverables
1. **Implementation of Automated Parity Verification Harness**:
   - Native integration and regression tests across workspace crates.
   - Dual-compiles input schemas through:
     - Reference: `./target/release/compile-sysml`
     - Target: `cargo test --workspace`
   - Performs deep bipartite AST equivalence analysis:
     - Verifies 100% bijective correspondence of AST nodes, attributes, ports, connections, action defs, and constraint blocks.
     - Compares serialized CommonMark specifications (Epics, Features, User Stories, Use Cases).
     - Compares Level 1C ICD matrices and signal dictionaries.
     - Compares Cartesian safety matrices.
2. **Zero Regression Certification**:
   - Confirms that DEAP02 introduces zero regressions across all 35 baseline quality checks.

#### Formal Mechanical Exit Gates
1. **Structural AST Equivalence Gate**:
   ```bash
   cargo test -p compile-sysml
   ```
   *Requirement*: 100% AST isomorphism verified across compiler test suites (`exit code == 0`).
2. **Agile Backlog Specification Parity Gate**:
   ```bash
   cargo test --workspace
   ```
   *Requirement*: Generated specifications match models with zero semantic divergence (`exit code == 0`).
3. **All 35 Downstream Baseline Checks Parity Gate**:
   ```bash
   ./target/release/verify-baseline ../deap-compiler-spec --no-domain
   ```
   *Requirement*: All 35 baseline checks pass with exit code 0 on specifications compiled by DEAP02.

#### Clean-Room Dogfooding Artifact Transition
- **Produced Artifacts**: Parity verification evidence report `docs/parity/PARITY_VERIFICATION_REPORT.md` in `DEAP02-spec-core`.
- **Target Consumer**: Authorizes production cutover and binary release in WP-09.

---

### Work Package WP-09: Production Cutover, Packaging & Single Static Binary Release

- **Target Repository**: `DEAP02-spec-core`
- **Workspace Path**: `/Users/perkunas/jail/DEAP02-spec-core`
- **Primary Responsibility**: Release Hardening, Static Binary Compilation & Pipeline Packaging.

#### Formal Mechanical Entrance Gates
1. **WP-08 Full Parity Certification Gate**:
   - WP-08 exit gates certified and verified.
2. **Cargo Workspace Cleanliness Gate**:
   ```bash
   git -C /Users/perkunas/jail/DEAP02-spec-core status --porcelain
   ```
   *Requirement*: Clean working tree with all crates committed (`exit code == 0`).

#### Concrete Step-by-Step Deliverables
1. **Single Static Binary Release Compilation**:
   - Compile release binary with Link-Time Optimization (LTO) and binary stripping:
     ```bash
     cargo build --manifest-path /Users/perkunas/jail/DEAP02-spec-core/Cargo.toml --release --locked
     ```
   - Target static binary: `/Users/perkunas/jail/DEAP02-spec-core/target/release/deap`.
   - Verify zero shared library runtime dependencies (statically linked).
2. **Downstream Pipeline Packaging Modernization**:
   - Modernize downstream pipeline packaging in `DEAP02-spec-core` to distribute native executables into downstream workspaces, replacing legacy procedural scripts.
   - Provide native Rust tool suite:
     - `compile-sysml`
     - `ingest-sysml`
     - `reconcile-backlog`
3. **Cutover Attestation & Documentation**:
   - Author production release notes and cutover certification in `README.md` and `RELEASE.md`.
4. **Fleet-Wide Remote Synchronization**:
   - Synchronize all three repositories with their respective remote tracking branches (`origin/main`) with exactly 0 bytes diff.

#### Formal Mechanical Exit Gates
1. **Production Release Binary Self-Test Gate**:
   ```bash
   /Users/perkunas/jail/DEAP02-spec-core/target/release/deap --version && \
   /Users/perkunas/jail/DEAP02-spec-core/target/release/deap self-test
   ```
   *Requirement*: Release binary executes internal diagnostic self-test with exit code 0.
2. **Cold Installation Verification in Clean Workspace**:
   ```bash
   ./target/release/verify-baseline setup-hooks /tmp/deap_cutover_verification && \
   ./target/release/verify-baseline /tmp/deap_cutover_verification --no-domain
   ```
   *Requirement*: Cold install into clean workspace succeeds; all baseline checks pass (`exit code == 0`).
3. **Tri-Repository Remote Synchronization Gate**:
   ```bash
   git -C /Users/perkunas/jail/DEAP01-spec-core diff origin/main --exit-code && \
   git -C /Users/perkunas/jail/deap-compiler-spec diff origin/main --exit-code && \
   git -C /Users/perkunas/jail/DEAP02-spec-core diff origin/main --exit-code
   ```
   *Requirement*: All three repositories clean and synchronized with origin branches (`exit code == 0`).

#### Clean-Room Dogfooding Artifact Transition
- **Produced Artifacts**: Final production release binary `target/release/deap` and modernized installer in `DEAP02-spec-core`.
- **Target Consumer**: Deployed across the enterprise fleet as the primary specification compiler.

---

## 5. Clean-Room Self-Hosting Dogfooding Artifact Transition Matrix

The table below defines the formal mathematical transition of engineering artifacts across the tri-repository architecture:

| Transition Stage | Source Repository & Artifact | Transformation Mechanism | Destination Repository & Artifact | Mechanical Verification Gate |
|---|---|---|---|---|
| **T1: Architecture Specification** | `DEAP01-spec-core`:`docs/architecture/blueprints/*.md` | Human System Architecture & Blueprint Modernization | `DEAP01-spec-core`:`docs/architecture/blueprints/*.md` & `MASTER_EXECUTION_PLAN.md` | Zero domain leaks gate, zero agency acronyms gate, frontmatter status gate |
| **T2: Metamodeling (Dogfooding)** | `DEAP01-spec-core`:`MASTER_EXECUTION_PLAN.md` | SysML v2 Architecture Modeling | `deap-compiler-spec`:`schema/compiler_architecture.sysml` | `ingest-sysml --validate-only` (exit code 0) |
| **T3: Compilation to Specs** | `deap-compiler-spec`:`schema/compiler_architecture.sysml` | DEAP01 Native Compiler Pipeline | `deap-compiler-spec`:`docs/epics/`, `docs/features/`, `docs/safety/`, `docs/interfaces/` | `verify-baseline --no-domain` (all baseline checks pass) |
| **T4: Rust Workspace Bootstrap** | `deap-compiler-spec`:`docs/epics/`, `docs/features/` | Rust Implementation (`deap-core`, `deap-ast`) | `DEAP02-spec-core`:`crates/deap-core`, `crates/deap-ast` | `cargo check --workspace` & `cargo clippy -- -D warnings` |
| **T5: Slicing Engine** | `deap-compiler-spec`:`schema/compiler_architecture.sysml` | Sub-AST Slicing Algorithm Implementation | `DEAP02-spec-core`:`crates/deap-ast/src/slicing/` | `deap slice --max-tokens 4096 --verify-all` (exit code 0) |
| **T6: Safety Synthesis** | `deap-compiler-spec`:`docs/safety/` | Combinatorial Cartesian Product Engine Implementation | `DEAP02-spec-core`:`crates/deap-safety`, `crates/deap-codegen` | `deap safety --verify-theorems` (10 theorems proven) |
| **T7: Air-Gapped Agent Harness** | `DEAP02-spec-core`:`crates/deap-ast` | Dual-Slot Slot Manager & Cryptographic CoT Logger | `DEAP02-spec-core`:`crates/deap-harness` | `deap harness-self-test --verify-cot-schema` |
| **T8: Cross-Compiler Parity** | `DEAP01` (Oracle) vs `DEAP02` (Target) | Dual-Compilation & Structural AST Bipartite Diff | `DEAP02-spec-core`:`docs/parity/PARITY_VERIFICATION_REPORT.md` | `cargo test --workspace` (0 diff) |
| **T9: Production Cutover** | `DEAP02-spec-core` Cargo Workspace | Release Musl/LTO Static Compilation | `DEAP02-spec-core`:`target/release/deap` | `deap self-test` & cold install test (exit code 0) |

---

## 6. Verification, Governance & Anti-Contamination Invariants

### 6.1 Mechanical Gate Verification Summary Table

| Gate ID | Invariant Verified | Command Line Check | Permitted Regressions |
|---|---|---|---|
| `GATE-HYGIENE-01` | Zero physical domain concepts | `./target/release/verify-baseline . --no-domain` | 0 |
| `GATE-HYGIENE-02` | Zero institutional regulatory agency acronyms | `./target/release/verify-baseline . --no-domain` | 0 |
| `GATE-HYGIENE-03` | Zero unresolved markers | `./target/release/verify-baseline . --no-domain` | 0 |
| `GATE-HYGIENE-04` | Zero Unicode em dashes | `./target/release/verify-baseline . --no-domain` | 0 |
| `GATE-HYGIENE-05` | Approved production-grade status | `cargo test -p verify-baseline` | 0 |
| `GATE-HYGIENE-06` | Mandatory native Rust unit & workspace test gate | `cargo test --workspace` | 0 |
| `GATE-BASELINE-01` | Downstream baseline checks | `./target/release/verify-baseline <dir> --no-domain` | 0 |
| `GATE-RUST-01` | Cargo workspace compilation | `cargo check --workspace --all-targets` | 0 |
| `GATE-RUST-02` | Clippy strict linting | `cargo clippy --workspace --all-targets -- -D warnings` | 0 |
| `GATE-RUST-03` | Standard canonical formatting | `cargo fmt --all -- --check` | 0 |
| `GATE-PARITY-01` | AST isomorphism against reference oracle | `cargo test --workspace` | 0 |
| `GATE-REMOTE-01` | Remote branch synchronization | `git diff origin/<branch> --exit-code` (0 bytes) | 0 |

### 6.2 KaTeX Mathematical Rendering Integrity

All mathematical expressions in this execution plan and downstream specifications strictly adhere to:
- Display math environments are enclosed in `$$` on isolated lines.
- Multi-line aligned formulas inside `$$` strictly utilize `\begin{aligned} ... \end{aligned}` (top-level `\begin{align}` is strictly forbidden).
- Non-mathematical identifiers (such as requirement IDs or work package names) never use `$...$` delimiters; code spans or bold text are used exclusively.
- Zero dangling operators (`+`, `-`, `*`, `/`) before line breaks or end-of-string.

---

## 7. Operational Risk Management & Rollback Protocols

| Risk Identifier | Failure Mode Description | Probability | Severity | Mitigation & Rollback Protocol |
|---|---|---|---|---|
| `RISK-01-PARITY` | Structural AST divergence between Python oracle and Rust compiler on edge-case SysML v2 grammar constructs. | Low | Critical | Dual-compilation parity harness in WP-08 fails closed (`exit code != 0`). Parser adjustments in `crates/deap-ast/src/parser.rs` are constrained by the oracle AST schema. Python oracle remains active in CI until 100% equivalence is certified across 10,000+ benchmark AST nodes. |
| `RISK-02-SLICING` | Sub-AST neighborhood expansion exceeds 4,096 token context budget on deeply nested inheritance hierarchies. | Medium | High | Slicing algorithm in `crates/deap-ast/src/slicing/` implements automated type abstraction: inherited properties deeper than 2 hops are replaced with opaque type signature contracts, mathematically bounding token length while preserving type safety. |
| `RISK-03-AIRGAP` | Reasoning token leakage or hallucinated context crossing into execution slot. | Low | Critical | Air-gap firewall in `crates/deap-harness/src/firewall.rs` performs AST schema validation on deduction outputs before dispatching to the execution slot. Any unstructured token stream causes immediate transaction abort and defect logging. |
| `RISK-04-CUTOVER` | Downstream customer project scripts experience breaking changes during Python-to-Rust binary cutover. | Low | Medium | Single static binary suite in WP-09 provides exact drop-in CLI compatibility with native Rust tooling (`compile-sysml`, `ingest-sysml`, `reconcile-backlog`). Native CLI tools provide full replacement. |

---

## 8. Attestation & Governance Sign-Off

The undersigned certify that this Master Multi-Repository Execution Plan conforms strictly to the Digital Engineering Agent Platform (DEAP) constitutional principles, pure schema-driven compiler invariants, zero domain impression directives, and high-assurance certification objectives.

```
+---------------------------------------------------------------------------------------------------+
| DEAP UPSTREAM SPECIFICATION CORE COMPILER GOVERNANCE BOARD                                        |
| Document: docs/architecture/MASTER_EXECUTION_PLAN.md (DEAP-PLAN-RUST-MODERNIZATION-001)           |
| Status: APPROVED / PRODUCTION-GRADE                                                               |
| Work Packages Governed: WP-01 through WP-09                                                       |
| Repositories Governed: DEAP01-spec-core, deap-compiler-spec, DEAP02-spec-core                     |
| Invariant Attestation: 100% Verified Pure Schema Invariant | 0 Domain Leaks | 0 Agency Acronyms     |
+---------------------------------------------------------------------------------------------------+
```

