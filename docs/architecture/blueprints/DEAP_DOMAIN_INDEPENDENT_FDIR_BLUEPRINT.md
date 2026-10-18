---
identifier: DEAP-BLUEPRINT-FDIR-001
title: "Architectural Blueprint: Pure Domain-Independent FDIR Framework for DEAP"
version: "2.0.0"
date: "2026-10-04"
status: "APPROVED / PRODUCTION-GRADE"
created: "2026-09-14"
target_regulatory_frameworks:
  - ISO/IEC/IEEE 15288:2023
  - ISO/IEC/IEEE 29148:2018
  - System-Theoretic Process Analysis (STPA)
  - Formal Failure Modes, Effects, and Criticality Analysis (FMECA)
  - High-Integrity Deterministic Software & Hardware Design Assurance (AssuranceLevel::Tier1..5)
  - Formal Bipartite Verification Graphs G = (V_req, V_test, E)
  - Multi-Authority Regulatory Traceability & Auditability
---

| Attribute | Specification Detail |
| :--- | :--- |
| **Document Identifier** | DEAP-BLUEPRINT-FDIR-001 |
| **Title** | Architectural Blueprint: Pure Domain-Independent FDIR Framework for DEAP |
| **Version** | 2.0.0 |
| **Date** | 2026-10-04 |
| **Status** | APPROVED / PRODUCTION-GRADE |

# Architectural Blueprint: Pure Domain-Independent FDIR Framework for DEAP

> **Document Identifier:** `DEAP-BLUEPRINT-FDIR-001`  
> **Status:** `APPROVED / PRODUCTION-GRADE`  
> **Classification:** `UPSTREAM_SPEC_CORE_COMPILER`  
> **Architecture Paradigm:** Model-Based Systems Engineering (MBSE) Pure Schema-Driven Compiler  
> **Invariants:** Zero Hardcoded Domain Concepts | Deterministic AST Derivation | Clean Landing Zones  
> **Target Regulatory Frameworks:** `ISO/IEC/IEEE 15288:2023` | `ISO/IEC/IEEE 29148:2018` | `STPA` | `Formal FMECA` | `AssuranceLevel::Tier1..5` | `Formal Bipartite Verification Graphs G = (V_req, V_test, E)`  

---

## 1. Executive Summary & Problem Statement

In mission-critical autonomous and physical systems, safety is not merely passive hazard documentation (FMECA/STPA); it requires **active, deterministic runtime control reconfiguration**. 

The **Digital Engineering Agent Platform (DEAP)** integrates **FDIR (Fault Detection, Isolation, and Recovery)** as a first-class, **purely domain-independent compiler capability**. Rather than hardcoding domain-specific concepts (such as physical plant structures, fluid dispensers, or friction brakes), DEAP formalizes FDIR through abstract mathematical primitives parameterized by:
1. **Universal Failure Dimensions ($\Gamma, \Phi, \Omega, \Psi$)**
2. **Detection Bounds ($\tau_{\text{detect}}, N_{\text{persist}}$)**
3. **Containment & Isolation Boundaries**
4. **Target Statechart Fallback Transitions & Containment Actions**

```mermaid
flowchart TD
    subgraph "1. Upstream AST Metamodel"
        direction TB
        F_DEF["fault def<br/>(Dimensions Γ, Φ, Ω, Ψ<br/>& Severity)"]
        D_DEF["detection def<br/>(Latency Bound τ,<br/>Persistence N, BIT)"]
        I_DEF["isolation def<br/>(Structural Boundary,<br/>Propagation Block)"]
        R_DEF["recovery def<br/>(Target Fallback State,<br/>Degraded Mode)"]
    end

    subgraph "2. Schema Compiler & Digest Engine"
        direction TB
        F_DEF & D_DEF & I_DEF & R_DEF --> PARSER["crates/ingest-sysml &<br/>crates/compile-sysml"]
        PARSER --> DIGEST[".pipeline/schema-digest.json<br/>('fdir_defs' Key)"]
    end

    subgraph "3. Downstream Projections"
        direction TB
        DIGEST --> MAT_PROJ["Pipeline: FDIR Master Matrix<br/>(docs/safety/<br/>FDIR_MASTER_MATRIX.md)"]
        DIGEST --> BDD_PROJ["Pipeline: BDD Fault Stories<br/>(docs/user-stories/us-fdir-*.md)"]
        DIGEST --> REQ_PROJ["Pipeline: Formal Test Cases<br/>(test case def TC_FDIR_*)"]
    end

    subgraph "4. Automated Quality Gates"
        direction TB
        DIGEST --> G1["Gate: 100% Failure Mode Closure"]
        DIGEST --> G2["Gate: Detection Latency<br/>Boundedness (τ > 0)"]
        DIGEST --> G3["Gate: Recovery State<br/>Reachability in Statecharts"]
        DIGEST --> G4["Gate: Zero Hardcoded<br/>Domain Concept Linter"]
    end
```

---

## 2. Universal Domain-Independent FDIR Metamodel

```mermaid
classDiagram
    class DEAP_Abstract_FDIR {
        <<package>>
    }
    class FailureDimension {
        <<enum def>>
        +FailureDimension GAMMA
        +FailureDimension PHI
        +FailureDimension OMEGA
        +FailureDimension PSI
    }
    class FaultDef {
        <<abstract action def>>
        +FailureDimension dimension
        +Integer severityLevel
        +String affectedPart
    }
    class DetectionDef {
        <<abstract action def>>
        +Real maxDetectionLatency_ms
        +Integer persistenceThresholdCount
        +String monitoredSignal
    }
    class IsolationDef {
        <<abstract action def>>
        +String containmentBoundaryPart
        +Boolean blocksPropagation
    }
    class RecoveryDef {
        <<abstract action def>>
        +String targetFallbackState
        +String fallbackAction
        +Boolean isReversible
    }
    class FdirContract {
        <<abstract constraint def>>
        +Boolean assertDetectionBounded()
        +Boolean assertIsolationClosed()
        +Boolean assertRecoveryReachable()
    }

    DEAP_Abstract_FDIR *-- FailureDimension
    DEAP_Abstract_FDIR *-- FaultDef
    DEAP_Abstract_FDIR *-- DetectionDef
    DEAP_Abstract_FDIR *-- IsolationDef
    DEAP_Abstract_FDIR *-- RecoveryDef
    DEAP_Abstract_FDIR *-- FdirContract
    FaultDef --> DetectionDef : triggers
    DetectionDef --> IsolationDef : isolates
    IsolationDef --> RecoveryDef : transitions to
```

### 2.1 Formal KerML / SysML v2 Concrete Syntax

```sysml
// Abstract Domain-Independent FDIR Package in SysML v2 / KerML
package DEAP_Abstract_FDIR {
    private import ScalarValues::*;

    // 1. Universal Failure Dimensions
    enum def FailureDimension {
        enum Gamma; // Transient / Intermittent Signal or Hardware Degrade
        enum Phi;   // Physical / Structural / Mechanical Failure
        enum Omega; // Timing / Performance / Latency / Rate Exceedance
        enum Psi;   // Value / Logic / Protocol / Corruption Anomaly
    }

    // 2. Abstract Fault Definition
    abstract action def AbstractFault {
        attribute dimension : FailureDimension;
        attribute severityLevel : Integer; // 1 (Minor) to 4 (Catastrophic)
        attribute faultSourcePart : String;
    }

    // 3. Abstract Detection Mechanism
    abstract action def AbstractDetection {
        attribute maxDetectionLatency_ms : Real;
        attribute persistenceThresholdCount : Integer;
        attribute monitoredSignalName : String;
    }

    // 4. Abstract Isolation Boundary
    abstract action def AbstractIsolation {
        attribute containmentBoundaryPart : String;
        attribute isolatesPropagation : Boolean;
    }

    // 5. Abstract Recovery Action
    abstract action def AbstractRecovery {
        attribute targetFallbackState : String;
        attribute containmentAction : String;
        attribute isReversible : Boolean;
    }

    // 6. Universal FDIR Contract (Formal Mathematical Invariant)
    abstract constraint def AbstractFdirContract {
        doc /* Asserts deterministic closure across Fault -> Detection -> Isolation -> Recovery */
    }
}
```

---

## 3. Upstream Compiler Pipeline Architecture (`DEAP01-spec-core`)

The compiler enhancements reside entirely within the abstract toolchain:

### 3.1 AST Parser Extensions (`crates/ingest-sysml`)
- Recognizes `fault def`, `detection def`, `isolation def`, `recovery def`, and `fdir contract def` AST nodes.
- Extracts attributes into clean dictionary AST representations:
  ```json
  {
    "fdir_defs": [
      {
        "name": "Fault_TelemetryDropout",
        "dimension": "Omega",
        "severity": 3,
        "detection": {
          "name": "Detect_HeartbeatTimeout",
          "latency_ms": 200.0,
          "persistence": 3
        },
        "isolation": {
          "boundary": "CommsSubsystem",
          "blocks_propagation": true
        },
        "recovery": {
          "target_state": "State_DegradedMode",
          "action": "SwitchToFallbackChannel",
          "is_reversible": true
        }
      }
    ]
  }
  ```

### 3.2 Schema Digest Integration (`crates/compile-sysml`)
- Ingests user-provided `.sysml` schema files in downstream repositories.
- Emits `.pipeline/schema-digest.json` including the `"fdir_defs"` collection.

### 3.3 Abstract Matrix & BDD Projection Engine
- Translates `schema-digest.json` into markdown specifications:
  - **`docs/safety/FDIR_MASTER_MATRIX.md`**: Authoritative matrix mapping every failure mode to detection, isolation, recovery, and test case.
  - **`docs/user-stories/us-fdir-*.md`**: Behavior-Driven Development (BDD) User Stories with executable Gherkin scenarios for hardware/software fault injection.

---

## 4. Downstream Distribution & Lifecycle Model

```mermaid
flowchart TD
    UPSTREAM["Upstream Core Compiler<br/>(DEAP01-spec-core)<br/>• Abstract Metamodel<br/>• Parser & AST Digest<br/>• Projection Generators<br/>• Universal Quality Gates"] -->|"Upstream Baseline & Template Sync"| TEMPLATES["Distribution Templates<br/>(Clean Landing Zones)"]
    TEMPLATES -->|"Downstream Installation"| WORKSPACE["Downstream Application Workspace<br/>(e.g., Subsystem_01,<br/>CyberPhysical_Plant_A)<br/>• Concrete Schema (schema/*.sysml)<br/>• Concrete FDIR Fault Instances<br/>• Generated Agile Specifications"]
```

---

## 5. Domain Independence Verification & Quality Gates

The compiler enforces strict gates ensuring zero domain coupling:

1. **Gate 1: AST Extraction Gate (`crates/ingest-sysml/src/lib.rs` and `crates/deap-core/src/sysml_ast.rs`)**:
   - Asserts that generic models (`Node1`, `Node2`, `SigA`, `FaultB`) parse with 100% fidelity.
2. **Gate 2: Zero Domain String Linter (`crates/verify-baseline/src/lib.rs`)**:
   - Scans all upstream parser crates for forbidden concrete domain words and vehicle primitives to guarantee pure abstract semantics.
3. **Gate 3: Statechart Reachability Gate (`crates/compile-sysml/src/lib.rs` and `crates/verify-baseline/src/lib.rs`)**:
   - Proves mathematically that all declared `targetFallbackState` strings correspond to actual `state def` nodes in the schema.
4. **Gate 4: Bounded Detection Gate (`crates/compile-sysml/src/lib.rs` and `crates/verify-baseline/src/lib.rs`)**:
   - Asserts that every critical fault has $\tau_{\text{detect}} > 0$ and a non-empty isolation boundary.

---

## 6. Modular Cargo Workspace Architecture & Air-Gapped Verification

The deterministic FDIR engine is implemented within the 6-crate Rust workspace architecture:

### 6.1 Modular Cargo Workspace Crate Decomposition

| Crate Identifier | Architecture & Responsibilities | Key Dependencies | Quality Gate Contract |
| :--- | :--- | :--- | :--- |
| **`deap-core`** (`crates/deap-core`) | Foundational shared library hosting SysML v2 / KerML AST structures (`SysmlModel`, `PartDef`, `ActionDef`, `StateDef`, `HazardDef`, `RiskDef`, `ConstraintDef`, `RequirementDef`, `TestCaseDef`, `AstSlice`, `CotAuditLog` in `crates/deap-core/src/sysml_ast.rs`), source AST extractors (`crates/deap-core/src/source_ast.rs`), diagnostics (`crates/deap-core/src/diagnostics.rs`), markdown/Mermaid/KaTeX syntax verifiers (`crates/deap-core/src/markdown.rs`), governance rules (`crates/deap-core/src/rules.rs`), and workspace checks (`crates/deap-core/src/workspace.rs`). | `serde`, `serde_json`, `walkdir`, `chrono`, `uuid` | Pure deterministic domain-independent core with zero panics and strict Clippy lints. |
| **`ingest-sysml`** (`crates/ingest-sysml`) | Ground 0.0 SysML v2 ingestion engine and structural/behavioral schema extractor (`crates/ingest-sysml/src/lib.rs`, `crates/ingest-sysml/src/main.rs`). | `deap-core`, `compile-sysml`, `clap`, `serde`, `serde_json`, `walkdir`, `chrono`, `rayon` | Deterministically ingests source schemas and reverse-engineered packages into canonical `.sysml` AST models. |
| **`compile-sysml`** (`crates/compile-sysml`) | SysML v2 / KerML compiler, STPA/FMECA/FDIR safety transpiler, schema digest generator (`.pipeline/schema-digest.json`), and specification synchronization engine (`crates/compile-sysml/src/lib.rs`, `crates/compile-sysml/src/main.rs`). | `deap-core`, `clap`, `serde`, `serde_json`, `walkdir`, `chrono` | Compiles validated AST models into schema digests, safety matrices, and synchronized specification artifacts with zero semantic drift. |
| **`verify-baseline`** (`crates/verify-baseline`) | Deterministic baseline verification harness enforcing all workspace quality gates, Mermaid/KaTeX compliance, domain-independence checks, FDIR/STPA traceability closure, and defect dossier validation (`crates/verify-baseline/src/lib.rs`, `crates/verify-baseline/src/main.rs`). | `deap-core`, `clap`, `serde`, `serde_json`, `walkdir`, `chrono`, `pulldown-cmark`, `rayon` | Single verification entrypoint returning deterministic exit codes (`0` on full compliance, non-zero on violation). |
| **`assemble-conops`** (`crates/assemble-conops`) | Deterministic Concept of Operations (`docs/conops/`) and Tactical Mission Intent specification assembly engine (`crates/assemble-conops/src/lib.rs`, `crates/assemble-conops/src/main.rs`). | `deap-core`, `clap`, `pulldown-cmark`, `serde`, `serde_json`, `walkdir`, `chrono` | Enforces pure schema contracts and zero truncation across assembled operational specifications. |
| **`reconcile-backlog`** (`crates/reconcile-backlog`) | Native Rust backlog reconciler synchronizing local Epics, Features, User Stories, Use Cases, and safety checklists with the remote issue tracker (`crates/reconcile-backlog/src/lib.rs`, `crates/reconcile-backlog/src/main.rs`). | `deap-core`, `clap`, `serde`, `serde_json`, `walkdir`, `chrono`, `pulldown-cmark` | Enforces bi-directional traceability and verified defect filing across repository and issue tracker. |

### 6.2 Strongly-Typed AST Slicing Mechanics (<= 4,096 Tokens)

To guarantee that language model inference and formal constraint solvers remain immune to attention decay, context loss, and truncation, `deap-core` (`crates/deap-core/src/sysml_ast.rs` via `AstSlice` and `AstPersistenceEngine`) and `compile-sysml` (`crates/compile-sysml/src/lib.rs`) implement deterministic AST slicing:
- **Structural Partitioning**: Every structural subsystem graph $\mathcal{G}_{\mathrm{sub}} = (V, E)$ is traversed and partitioned along subsystem containment boundaries into self-contained AST slices.
- **Strict Token Floor & Ceiling**: Every AST slice $\mathcal{S}_k$ is strictly bounded to $\le 4,096$ tokens:
$$
\begin{aligned}
\forall \mathcal{S}_k \in \mathcal{P}(\mathcal{G}), \quad \mathrm{TokenCount}(\mathcal{S}_k) \le 4096
\end{aligned}
$$
- **Closed Dependency Context**: Each slice embeds all referenced upstream type definitions, interface port contracts, and failure dimensions, eliminating unresolved external identifier references.

### 6.3 Dual-LLM Air-Gapped Agent Dispatch Protocol

To guarantee mathematical determinism, total intellectual property containment, and complete immunity to cloud rate-limiting or content censorship, DEAP utilizes a dual-LLM air-gapped agent dispatch architecture:
1. **Reasoning Slot (DeepSeek-R1)**:
   - Dedicated Chain-of-Thought (CoT) reasoning model executing on air-gapped workstation hardware.
   - Streams formal mathematical proofs, reachability derivations, and safety invariant checks directly into `.pipeline/diagnostics/cot_audit_log.json` (`CotAuditLog` and `CotAuditRecord` in `crates/deap-core/src/sysml_ast.rs`).
   - Never generates raw code directly; outputs verified formal derivations and boundary assertions.
2. **Execution Slot (Qwen-2.5-Coder)**:
   - High-throughput deterministic code and specification synthesis model.
   - Ingests verified reasoning derivations from the CoT audit log along with the strongly-typed AST slice (<= 4,096 tokens).
   - Generates verified SysML v2 FDIR contracts, BDD test scenarios, and Stateflow integration hooks.

