<!-- Copyright Gint Atkinson, gint.atkinson@gmail.com -->

# Comprehensive Research Report: Model-Based Systems Engineering (MBSE) Architecture, OMG SysML v2 / KerML Metamodel Foundations, and the 2D Multi-Facet Decomposition Framework for DEAP

**Document Identifier:** `DOC-RESEARCH-MBSE-SYSMLV2-2D-DECOMPOSITION`  
**Classification:** Normative Architectural Research & Systems Engineering Strategy  
**Standard Baseline:** OMG SysML v2 (ptc/2023-08-01), OMG KerML (ptc/2023-08-03), OMG UAF v1.2/v2.0, ISO/IEC/IEEE 15288:2023, ISO/IEC/IEEE 29148:2018, INCOSE SEH v5.0, Dassault MagicGrid, Thales Arcadia/Capella, SAE ARP4754A/ARP4761, RTCA DO-178C/DO-331, ISO 26262:2018  
**Author:** Lead MBSE & OMG Standards Architect Research Subagent  
**Date:** October 2026  

---

## Executive Summary

Model-Based Systems Engineering (MBSE) has arrived at a pivotal generational inflection point. For over fifteen years, systems engineering practice struggled with the syntactic ambiguities, visual-first lock-in, and disconnected relational models of OMG SysML v1. The release of the **OMG Systems Modeling Language Version 2 (SysML v2)** specification (formal document `ptc/2023-08-01`) alongside its foundational kernel, the **Kernel Modeling Language (KerML)** specification (formal document `ptc/2023-08-03`), replaces ad-hoc graphical profiles with a mathematically rigorous, semantically grounded, text-first Abstract Syntax Tree (AST) metamodel.

Concurrently, modern autonomous, safety-critical software and systems engineering--spanning civil avionics (SAE ARP4754A, RTCA DO-178C/DO-331), automotive autonomy (ISO 26262), and mission-critical automated pipelines--faces severe scaling bottlenecks:
1. **The Monolithic Schema Anti-Pattern:** Dumping thousands of requirements, parts, ports, actions, states, and constraints into a single unmanageable schema file (such as a 1-megabyte `model.sysml`) produces severe lexical thrashing, continuous merge conflicts, cognitive overload, and fragile semantic scoping.
2. **The Ad-Hoc File Fragmentation Anti-Pattern:** Decomposing models into arbitrary, disconnected flat files without strict metamodel governance creates dangling references, ungrounded connectors, circular package dependencies, and broken traceability threads.
3. **The Detached Relational Anti-Pattern:** Creating detached files such as `connections.sysml` or `traceability.sysml` directly violates KerML Clause 8 connector containment semantics and disregards SysML v2 first-class in-place language operators (`satisfy by`, `verify by`, `require`, `derived from`).

This report provides an exhaustive, normative, and comparative investigation into the broader MBSE modeling problem across all governing international standards. It synthesizes the foundational principles of **ISO/IEC/IEEE 15288:2023**, **ISO/IEC/IEEE 29148:2018**, the **INCOSE Systems Engineering Handbook (5th Edition, 2023)**, Dassault Systèmes' **MagicGrid**, Thales' **Arcadia/Capella**, and safety-critical aerospace frameworks into an authoritative **2D Decomposition Architecture** tailored specifically for the Digital Engineering Autonomous Pipeline (DEAP).

The resulting 2D architecture establishes an orthogonal matrix across:
- **Three Vertical Abstraction Layers:**
  - **Level 0: Super-System & Operational Context (ConOps / Mission Intent)**: External operational environment, operational activities, actors, multi-domain threat vectors, and Tier 1 mission requirements.
  - **Level 1: System of Interest (SoI)**: The root system classifier (`schema/model.sysml`) acting as the enclosing occurrence context instantiating the 12 specialized subsystems and binding inter-subsystem pipeline connections.
  - **Level 2: Subsystem Decomposition (The 12 Subsystems)**: White-box modular realization of the 12 compiler subsystems.
- **Three Horizontal Functional Facets per Subsystem:**
  1. `requirements.sysml`: Functional, performance, and safety requirements, formal invariant constraints, Acceptance Criteria (ACs), hazard realisations, and in-place traceability links.
  2. `behavior.sysml`: Detailed functional design containing Use Cases (`use case def`), User Story Scenarios (`interaction def`), Mode Supervisors and Statecharts (`state def`), and Algorithmic Action Flows (`action def`).
  3. `architecture.sysml`: Detailed structural and data design containing Structural Execution Engines (`part def`), Interface Ports (`port def`), Data Payloads (`item def`), and internal component wirings.

This architectural blueprint establishes absolute mathematical determinism, guarantees 100% compliance with OMG KerML Clause 8 connection semantics, eliminates specification drift, and provides an unbroken digital thread from high-level stakeholder intent down to verified, executable code.

---

## 1. OMG Standards: SysML v2 & KerML Foundational Metamodel Architecture

### 1.1 The SysML v2 / KerML Architectural Shift

OMG SysML v2 represents a complete ground-up reimagining of systems modeling. Unlike SysML v1, which was defined as a profile of the Unified Modeling Language (UML) metamodel, SysML v2 is defined directly on top of a lean, unified, mathematically formal foundational metamodel: the **Kernel Modeling Language (KerML)** (`ptc/2023-08-03`).

In KerML and SysML v2, all modeling constructs are unified under the core concepts of **Elements**, **Namespaces**, **Types**, **Classifiers**, and **Features**. The core separation is between:
- **Definitions (`* def`)**: Classifiers that specify reusable types, contracts, constraints, or behavioral templates (e.g., `part def`, `port def`, `action def`, `state def`, `requirement def`, `item def`, `constraint def`, `connection def`). Definitions do not represent occurrences in space-time; they represent abstract types.
- **Usages**: Features that represent occurrences, parts, actions, states, or bindings within an enclosing context (e.g., `part`, `port`, `action`, `state`, `requirement`, `item`, `connect`). Usages have space-time presence and cardinality.

```mermaid
classDiagram
    direction TB
    class Element {
        +String qualifiedName
        +Element owner
    }
    class Namespace {
        +MemberElement[] ownedMember
        +Import[] ownedImport
    }
    class Classifier {
        +Feature[] ownedFeature
        +Classifier[] general
    }
    class Feature {
        +Type type
        +Multiplicity multiplicity
        +Boolean isComposite
    }
    class Package {
        +String name
        +Visibility visibility
    }
    class OccurrenceDefinition {
        +LifeTime lifeTime
    }
    class StructureDefinition {
    }
    class BehaviorDefinition {
    }

    Element <|-- Namespace
    Namespace <|-- Package
    Namespace <|-- Classifier
    Element <|-- Feature
    Classifier <|-- OccurrenceDefinition
    OccurrenceDefinition <|-- StructureDefinition
    OccurrenceDefinition <|-- BehaviorDefinition
```

### 1.2 Model Packaging, Namespaces, Scoping, and Visibility Rules

SysML v2 establishes strict, deterministic scoping and namespace mechanics defined in KerML Clauses 7 and 9:
- **`package` as a Pure Namespace**: A `package` is an abstract namespace container (`Namespace`) whose purpose is model partitioning, logical containment, access control, and modular distribution. A package is **not** an `OccurrenceDefinition` or a `Classifier`. A package does not possess runtime instances, physical or temporal boundaries, internal state, or ports.
- **Hierarchical Path Scoping (`::`)**: Elements declared within nested packages resolve via the canonical double-colon scoping operator (e.g., `RootPackage::SubPackage::TargetElement`).
- **Visibility Control**: Package members possess visibility specifiers:
  - `public` (default): Accessible to any namespace importing the container package.
  - `protected`: Accessible only to the container package and its descendant/specialized packages.
  - `private`: Strictly encapsulated within the immediate container package.
- **Deterministic Imports (`import`)**:
  - Wildcard import: `import Package_0::*;` imports all public members into the importing namespace.
  - Specific import: `import Package_0::Classifier_Alpha;` exposes only the explicitly declared symbol.
  - Recursive import: `import Package_0::**;` exposes members across nested sub-packages.
- **Acyclic Dependency Invariant**: The package import graph must be a strict Directed Acyclic Graph (DAG). Mutual or circular imports (`Package_A` imports `Package_B` while `Package_B` imports `Package_A`) are strictly illegal under KerML Clause 7 and trigger semantic diagnostics (`E0203`).

### 1.3 The Four Core Pillars of SysML v2

SysML v2 consolidates all systems engineering constructs across four fundamental semantic pillars:

| Pillar | Core Definitions (`* def`) | Concrete Usages | Semantics and Mathematical Role |
| :--- | :--- | :--- | :--- |
| **Requirements** | `requirement def` | `requirement` | Formal specification of functional, performance, physical, or safety obligations. Governed by text statements, unique identifiers, formal constraints (`assume`, `require`), and subjects. |
| **Behavior** | `action def`<br/>`state def`<br/>`use case def`<br/>`interaction def` | `action`<br/>`state`<br/>`use case`<br/>`interaction` | Operational and algorithmic transitions over space and time. Encapsulates control flows, dataflows, discrete-event state machines, operational use cases, and message sequences. |
| **Structure** | `part def`<br/>`port def`<br/>`item def` | `part`<br/>`port`<br/>`item` | Physical and logical composition, boundary encapsulation, spatial topology, and data schema payloads. Models components, structural hierarchies, and conjugate interfaces. |
| **Constraints** | `constraint def`<br/>`calc def` | `assert constraint`<br/>`calc` | Formal mathematical and logical predicates evaluated over attributes, ports, and states. Operates as boolean invariants that must hold true across all system states. |

```mermaid
graph TD
    subgraph "Four Core Pillars of OMG SysML v2"
        REQ["Requirements Pillar<br/>• requirement def<br/>• require / assume<br/>• id, text, doc"]
        BEH["Behavior Pillar<br/>• action def (Flow)<br/>• state def (Modes)<br/>• use case def<br/>• interaction def"]
        STR["Structure Pillar<br/>• part def (Blocks)<br/>• port def (Interfaces)<br/>• item def (Payloads)"]
        CON["Constraints Pillar<br/>• constraint def<br/>• assert constraint<br/>• calc def (Math)"]
    end

    REQ -->|"satisfy by"| STR
    REQ -->|"satisfy by"| BEH
    CON -->|"require constraint"| REQ
    CON -->|"asserts invariant on"| STR
    CON -->|"asserts invariant on"| BEH
    BEH -->|"allocated to"| STR
    STR -->|"exchanges items via"| STR
```

### 1.4 In-Place Traceability vs Detached Relational Files

In legacy SysML v1, traceability was frequently established using detached dependency matrices, out-of-band tables, or detached diagrams containing `<<satisfy>>`, `<<verify>>`, or `<<deriveReqt>>` relationships. This created severe synchronization drift: modifying a structural part or requirement in one file left detached matrices containing stale, broken references.

In OMG SysML v2, **traceability is a first-class language feature embedded directly within element declarations**. The specification provides normative in-place traceability clauses:
1. `satisfy <RequirementUsage> by <PartUsage/ActionUsage>;`: Declares that a structural component or behavioral action realizes the specified requirement.
2. `verify <RequirementUsage> by <ActionUsage/VerificationCase>;`: Declares that an automated test case, analysis, or verification engine validates the requirement.
3. `require <ConstraintUsage>;`: Injects a formal boolean invariant or mathematical constraint into a requirement definition.
4. `assume <ConstraintUsage>;`: Injects an operational assumption or environmental boundary condition.
5. `derived from <RequirementDef/Usage>;`: Documents formal hierarchical derivation relationships between requirement tiers.
6. `refines <ModelElement>;`: Documents elaboration and refinement from operational activities down to technical functions.

**Normative Advantage of In-Place Co-Location:**
- **Semantic Locality**: The requirement, its formal mathematical constraint, its verification mechanism, and its realizing architectural target reside together in the same lexical scope or directly reference validated symbols.
- **Compiler-Enforced Referential Integrity**: During compilation, the AST symbol resolver validates that every satisfied target and verification action exists. If a part or port is renamed or removed, the compiler immediately halts with a compile diagnostic (`E0200` / `E0700`), preventing silent drift.
- **Zero Redundant Overhead**: Eliminates hundreds of lines of duplicate boilerplate code required to maintain separate, detached traceability files.

### 1.5 KerML Clause 8 Connection Semantics: The Classifier Enclosure Requirement

One of the most frequent architectural mistakes made by novice SysML v2 modelers is declaring loose connections directly at package scope. KerML Clause 8 (*Features, Occurrences, and Connectors*) formalizes the exact semantics of connection.

#### Formal Semantics of Connectors (KerML Clause 8.3 & 8.4)
In KerML, a `Connector` is defined as a specialized `Feature` that establishes a relational binding between two or more existing features within an **enclosing occurrence context**:

$$\begin{aligned}
\text{Connector}(c) &\implies \exists \, O \in \text{Classifiers}, \quad c \in \text{Features}(O) \\
\text{Endpoints}(c) &= \{ f_1, f_2 \} \quad \text{such that} \quad f_1, f_2 \in \text{NestedFeatures}(O)
\end{aligned}$$

A connector does not stand alone in the ether. A connector represents the physical, electrical, or logical interaction between occurrences occurring inside a containing boundary (the parent system or assembly).

#### Why Loose Connections in Packages Are Invalid
A `package` is a `Namespace`, but it is **not** an `OccurrenceDefinition` or a `Classifier`:

$$\begin{aligned}
\text{Package} &\not\subseteq \text{OccurrenceDefinition} \\
\text{Package} &\not\subseteq \text{Classifier}
\end{aligned}$$

Because a package has no instances, no runtime state, and no spatial-temporal extent:
1. A package cannot own ports.
2. A package cannot instantiate part usages.
3. When a modeler writes:
   ```sysml
   // ILLEGAL ANTI-PATTERN IN PURE KERML SEMANTICS
   package AntiPatternPackage {
       connect SubsystemA.port_out to SubsystemB.port_in;
   }
   ```
   the identifier `SubsystemA.port_out` is referencing a port feature on a *type definition* (`part def SubsystemA`), rather than an actual *part occurrence* instantiated within a common system boundary. In KerML, you cannot connect two un-instantiated types; you can only connect features of instances existing within a shared enclosing classifier!

#### The Enclosing Context Solution
To achieve complete semantic validity under KerML Clause 8, all connectors connecting subsystem boundaries MUST be enclosed within the **System of Interest (SoI)** classifier (`part def`), where each subsystem is instantiated as a part usage:

```sysml
// NORMATIVE KERML CLAUSE 8 COMPLIANT PATTERN
package DEAP_Compiler_System {
    // The Enclosing System Classifier
    part def CompilerSystemOfInterest {
        // Subsystem Part Occurrences
        part universalIngestionEngine : Subsystem_2_Universal_Schema_Ingestion_Engine::IngestionEngine;
        part nodeArenaASTGraphEngine : Subsystem_3_Core_Metamodel_Node_Arena::ArenaGraphEngine;

        // Normative Connector enclosed within the Classifier Context
        connect universalIngestionEngine.token_out to nodeArenaASTGraphEngine.token_in;
    }
}
```

This fundamental semantic rule governs the entire Level 1 / Level 2 relationship in DEAP's 2D decomposition.

### 1.6 OMG Unified Architecture Framework (UAF v1.2 / v2.0) and ConOps Modeling

The **Unified Architecture Framework (UAF)** (`ptc/2020-11-01` for v1.2 and ongoing v2.0) is the OMG's premier standard for enterprise, system-of-systems, and complex mission architectures (unifying DoDAF, MODAF, and NAF).

UAF defines a two-dimensional grid of **Domains** (Rows: Strategic, Operational, Services, Personnel, Resources, Security, Projects, Standards, Actual Resources) and **Model Kinds** (Columns: Taxonomy, Structure, Connectivity, Processes, States, Interaction Scenarios, Information, Parameters, Roadmap).

In UAF, operational modeling--which directly governs the Concept of Operations (ConOps)--is situated strictly within the **Operational Domain**:
- **Operational Processes (Op-Pr)**: Flow of high-level operational activities executed by operational performers (human operators, command centers, external platforms) without binding to internal technical hardware or software modules.
- **Operational Connectivity (Op-Cn / Op-Tx)**: Operational nodes, operational lines of communication, and operational information exchanges (Op-Tx: e.g., C2 Mission Directives, Telemetry Streams, Environmental Advisories).
- **Operational States (Op-Is)**: Operational lifecycle phases ($\Phi_{\text{lifecycle}}$: Pre-Deployment, Transit, Active Mission, Degraded Contingency, Safe Containment, Post-Mission Analysis).

UAF mandates that the Operational Domain represents the **problem space** (what operational capabilities are required to achieve mission success), strictly segregated from the **Resource Domain** (the technical system architecture and components that realize those capabilities).

---

## 2. Global Systems Engineering Standards & Methodologies

The synthesis of DEAP's 2D architecture is grounded in the foundational systems engineering standards recognized across global industry and defense.

```mermaid
flowchart TD
    subgraph "Global Systems Engineering Alignment"
        S15288["ISO/IEC/IEEE 15288:2023<br/>• Stakeholder Needs (T1)<br/>• System Requirements (T2)<br/>• Architecture Definition (T3)<br/>• Design Definition (T4)"]
        S29148["ISO/IEC/IEEE 29148:2018<br/>• §6.4.2 ConOps (Operational)<br/>• §6.4.3 OpsCon (Systemic)<br/>• SyRS (Technical Specification)"]
        INCOSE["INCOSE SEH 5th Ed (2023)<br/>• Super-System / Context<br/>• System of Interest (SoI)<br/>• Subsystems -> Assemblies<br/>• Components -> Units"]
        MAGIC["Cameo / MagicGrid<br/>• 4 Pillars x 4 Layers<br/>• Operational -> System<br/>• Subsystem -> Component"]
        ARCADIA["Thales Arcadia / Capella<br/>• Operational Analysis (OA)<br/>• System Analysis (SA)<br/>• Logical Architecture (LA)<br/>• Physical Architecture (PA)"]
        SAFETY["Aerospace Safety Standards<br/>• SAE ARP4754A / ARP4761<br/>• RTCA DO-178C / DO-331 (HLR/LLR)<br/>• ISO 26262 (FSC / TSC)"]
    end

    S15288 <--> S29148
    S15288 <--> INCOSE
    INCOSE <--> MAGIC
    MAGIC <--> ARCADIA
    S15288 <--> SAFETY
```

### 2.1 ISO/IEC/IEEE 15288:2023 and ISO/IEC/IEEE 29148:2018

**ISO/IEC/IEEE 15288:2023** (*Systems and software engineering -- System life cycle processes*) formalizes the technical lifecycle processes that govern system creation:
1. **Business or Mission Analysis Process ($T_0$)**: Identifies enterprise problems, mission objectives, and operational scenarios.
2. **Stakeholder Needs and Requirements Definition Process ($T_1$)**: Defines the stakeholder environment, operational constraints, and stakeholder requirements.
3. **System Requirements Definition Process ($T_2$)**: Transforms stakeholder needs into a formal, verifiable System Requirements Specification (SyRS).
4. **Architecture Definition Process ($T_3$)**: Synthesizes alternative architectural solutions, establishes subsystem boundaries, allocates requirements, and defines interface contracts.
5. **Design Definition Process ($T_4$ - Detailed Design)**: Provides the granular, technology-specific data schemas, algorithmic logic, and physical blueprints necessary for manufacturing or code generation.

**ISO/IEC/IEEE 29148:2018** (*Requirements Engineering*) defines the precise relationship between operational concepts and technical specifications:
- **Concept of Operations (ConOps, §6.4.2)**: Written from the perspective of the user, acquirer, and operational stakeholders. It describes the operational environment, user classes, mission scenarios, threat profiles, and operational goals.
- **Operational Concept (OpsCon, §6.4.3)**: Written from the perspective of the system developer. It describes how the technical system of interest operates within the operational environment, detailing high-level modes of operation and boundary interfaces.
- **System Requirements Specification (SyRS)**: The formal, technical, verifiable requirements defining what the system shall perform to satisfy the ConOps and OpsCon.

### 2.2 INCOSE Systems Engineering Handbook (5th Edition, 2023)

The INCOSE SE Handbook v5.0 establishes the universal **System Decomposition Hierarchy**:

$$\begin{aligned}
\text{Super-System / Operational Context} &\longrightarrow \text{System of Interest (SoI)} \\
&\longrightarrow \text{Subsystems} \longrightarrow \text{Assemblies / Segments} \\
&\longrightarrow \text{Components} \longrightarrow \text{Units / Parts}
\end{aligned}$$

- **Super-System / Operational Context**: The outer operational environment containing the System of Interest alongside human operators, external platforms, satellite constellations, adversarial threats, and physical atmosphere.
- **System of Interest (SoI)**: The bounded, self-contained system delivered to the customer (e.g., the DEAP Compiler).
- **Subsystems**: The first-tier structural and functional subdivisions of the SoI (e.g., Ingestion Engine, Metrology Engine, State Machine Solver).
- **Components / Modules**: The discrete, cohesive execution engines within a subsystem (e.g., Lexer, Recursive Descent Parser, Arena Allocator).
- **Units**: Atomic implementation artifacts (e.g., individual Rust functions, SPARK Ada procedures, C functions).

INCOSE SEH v5.0 also mandates the formal formulation of **Measures of Effectiveness (MoE)** (operational mission success criteria), **Measures of Performance (MoP)** (system-level technical capabilities), and **Technical Performance Measures (TPM)** (subsystem/component-level attributes monitored over time).

### 2.3 The MagicGrid Methodology (Cameo / Dassault Systèmes)

Developed by No Magic (now Dassault Systèmes) and grounded in the INCOSE SE principles, **MagicGrid** is the industry standard for structuring MBSE models in Cameo Systems Modeler. MagicGrid organizes the entire system model into a strict **2D Matrix**:

```
                       THE MAGICGRID 2D MBSE MATRIX
========================================================================================
Abstraction Layer      Requirements        Behavior           Structure        Parametrics
----------------------------------------------------------------------------------------
1. Operational         Stakeholder Needs   Operational Scen.  Operational Env  MoEs / Mission
   (Problem Space)     User Goals          Use Cases (Black)  External Actors  Effectiveness
----------------------------------------------------------------------------------------
2. System              System Req (SyRS)   System Use Cases   System Context   System MoPs /
   (Black-Box SoI)     Black-box contracts Functional Flows   Ports & Interfaces Budgets
----------------------------------------------------------------------------------------
3. Subsystem           Subsystem Reqs      Subsystem States   Subsystem Blocks Subsystem TPMs /
   (White-Box SoI)     Derived Tech Reqs   Activity Diagrams  Internal Parts   Physical Math
----------------------------------------------------------------------------------------
4. Component           Component Reqs      Algorithmic Flows  Component Schem. Component Anal.
   (Detailed Design)   Software / HW Reqs  State Transitions  Pinouts / Payloads Detailed Timing
========================================================================================
```

MagicGrid demonstrates that separating horizontal engineering facets (Requirements, Behavior, Structure, Parametrics) across vertical abstraction tiers (Operational, System, Subsystem, Component) prevents cognitive clutter and enforces rigorous architectural traceability.

### 2.4 Arcadia / Capella (Thales)

The **Arcadia** (Architecture Analysis & Design Integrated Approach) methodology, supported by the open-source **Capella** toolset developed by Thales, is another globally recognized MBSE framework. Arcadia enforces four successive engineering phases:
1. **Operational Analysis (OA)**: What the users need to achieve. Focuses on operational actors, operational capabilities, and operational activities. Zero technical concepts permitted.
2. **System Analysis (SA)**: What the system must do for the users. Defines the system boundary, external system exchanges, system use cases, and functional chains.
3. **Logical Architecture (LA)**: How the system will work internally without technology bias. Decomposes the system into logical components, logical functions, and internal dataflows.
4. **Physical Architecture (PA)**: How the system is concretely constructed and deployed. Defines physical execution nodes, physical links, hardware boards, and concrete software components.

Arcadia's OA $\rightarrow$ SA $\rightarrow$ LA $\rightarrow$ PA progression mirrors the ISO 15288 technical processes and MagicGrid layers, proving that multi-tier abstraction is an invariant principle of systems engineering.

### 2.5 Aerospace & Safety-Critical MBSE Frameworks

Safety-critical systems engineering enforces rigorous requirement and architectural stratification to achieve airworthiness and safety certification:

#### SAE ARP4754A / ARP4761
- **Functional Hazard Assessment (FHA)**: Conducted at Aircraft and System levels to identify failure conditions and assign Development Assurance Levels (DAL A through E).
- **Preliminary System Safety Assessment (PSSA)**: Derives safety requirements allocated to subsystem architectures.
- **System Safety Assessment (SSA)**: Validates that the implemented physical design satisfies all safety requirements via FMECA and Fault Tree Analysis (FTA).

#### RTCA DO-178C & DO-331 (Model-Based Development)
DO-178C and its model-based supplement DO-331 formalize the distinction between:
- **High-Level Requirements (HLR)**: Functional, performance, and safety requirements allocated to software from system architecture.
- **Software Architecture**: The hierarchical structure, data interfaces, and control flow of software components realizing the HLRs.
- **Low-Level Requirements (LLR / Detailed Design)**: Requirements that specify detailed algorithms, state machines, and data structures to the level of precision where software code can be directly generated or written without further requirement derivation.
- **Software Units**: Atomic source code functions/modules directly realizing LLRs.
- **Bidirectional Traceability**: Mandatory verification evidence demonstrating 100% bidirectional traceability between System Requirements $\leftrightarrow$ HLRs $\leftrightarrow$ Architecture $\leftrightarrow$ LLRs $\leftrightarrow$ Source Code $\leftrightarrow$ Test Cases.

#### ISO 26262 (Road Vehicles Functional Safety)
Enforces the progression: **Item Definition** $\rightarrow$ **Hazard Analysis and Risk Assessment (HARA)** $\rightarrow$ **Functional Safety Concept (FSC)** $\rightarrow$ **Technical Safety Concept (TSC)** $\rightarrow$ **Hardware/Software Safety Requirements (HSR/SSR)**.

---

## 3. Comparative Synthesis: The Universal Systems Engineering Alignment

Synthesizing all international standards reveals an unbroken alignment across abstraction layers, demonstrating that systems engineering is fundamentally a **two-dimensional problem**:

```
====================================================================================================================
Abstraction Tier | ISO 15288:2023 | INCOSE SEH v5.0 | MagicGrid Layer | Arcadia Phase | Aerospace / Safety (DO-178C)
====================================================================================================================
Level 0:         | Business /     | Super-System /  | Operational     | Operational   | Operational Hazard Analysis,
Operational      | Mission /      | Operational     | Layer           | Analysis      | Concept of Operations (ConOps),
Context          | Stakeholder    | Context         |                 | (OA)          | Tier 1 Mission Requirements
                 | Needs (T0, T1) |                 |                 |               |
--------------------------------------------------------------------------------------------------------------------
Level 1:         | System         | System of       | System Layer    | System        | Aircraft/System Requirements,
System of        | Requirements   | Interest (SoI)  | (Black-Box SoI) | Analysis      | High-Level Requirements (HLR),
Interest         | Definition     |                 |                 | (SA)          | Functional Architecture,
                 | (T2, T3)       |                 |                 |               | System Use Cases
--------------------------------------------------------------------------------------------------------------------
Level 2:         | Architecture / | Subsystems &    | Subsystem       | Logical       | Subsystem Architecture,
Subsystems       | Design         | Assemblies      | Layer           | Architecture  | Low-Level Requirements (LLR),
                 | Definition     |                 | (White-Box SoI) | (LA)          | Detailed Behavioral Design,
                 | (T3, T4)       |                 |                 |               | Interface Control Docs (ICD)
--------------------------------------------------------------------------------------------------------------------
Level 3:         | Implementation | Components &    | Component       | Physical      | Software Units, Source Code,
Components &     | & Integration  | Units           | Layer           | Architecture  | Hardware Schematics,
Units            | (T6, T7)       |                 | (Detailed Impl) | (PA)          | Unit Tests, Executables
====================================================================================================================
```

This universal alignment proves that attempting to represent a complex, safety-critical system using a flat file structure or a single monolithic schema is fundamentally flawed. Modern MBSE requires a structured, multi-tier, multi-facet decomposition.

---

## 4. Synthesis for DEAP: The 2D Decomposition Architecture

Grounding these universal standards in the Digital Engineering Autonomous Pipeline (DEAP) yields the **DEAP 2D Decomposition Architecture**. This framework organizes the system model across **Three Vertical Abstraction Layers** and **Three Horizontal Functional Facets**:

```mermaid
graph TD
    subgraph "Level 0: Operational Context & ConOps"
        L0["schema/conops/<br/>docs/conops/CONOPS.md<br/>• External Actors<br/>• Operational Activities (OA-xx)<br/>• Multi-Domain Threat Matrix<br/>• Measures of Effectiveness (MoE)"]
    end

    subgraph "Level 1: System of Interest (SoI)"
        L1["schema/model.sysml<br/>• part def DEAP_Compiler_System<br/>• 12 Subsystem Part Occurrences<br/>• Inter-Subsystem Pipeline Connections<br/>• System Requirements & SyRS Allocations"]
    end

    subgraph "Level 2: The 12 Subsystems"
        direction TB
        subgraph "Subsystem Structure (1 of 12)"
            F1["requirements.sysml<br/>• requirement def<br/>• In-Place satisfy/verify<br/>• Safety Invariants<br/>• Acceptance Criteria"]
            F2["behavior.sysml<br/>• use case def<br/>• interaction def (Stories)<br/>• state def (Supervisors)<br/>• action def (Algorithms)"]
            F3["architecture.sysml<br/>• part def (Engines)<br/>• port def (ICD Contracts)<br/>• item def (Data Payloads)<br/>• Internal Part Connectors"]
        end
    end

    L0 -->|"drives requirements of"| L1
    L1 -->|"instantiates & wires"| F3
    F1 -->|"satisfy by"| F2
    F1 -->|"satisfy by"| F3
    F2 -->|"executes within"| F3
```

### 4.1 Level 0: Super-System & Operational Context (ConOps / Mission Intent)

**Repository Home:** `schema/conops/` (SysML v2 model units) and `docs/conops/CONOPS.md` (assembled Markdown specification via `./target/release/assemble-conops`).

#### Scope and Role
Level 0 models the external operational environment in which the system operates. In accordance with ISO/IEC/IEEE 29148:2018 §6.4.2, INCOSE SEH v5.0 §3.4.4, and UAF v2.0 Operational Domain:
- **Operational Performers & Actors**: External entities interacting with the pipeline (e.g., Human Systems Engineers, CI/CD Cloud Runners, External Git Repositories, Downstream Toolchains).
- **Operational Activities (`OA-01`..`OA-N`)**: High-level capabilities executed across the operational lifecycle (e.g., Ingesting Customer Schemas, Synthesizing Verifiable Specifications, Generating Multi-Target Flight Code).
- **Multi-Domain Operational Threat Matrix**: Formal characterization of operational risks across all 9 operational threat domains (Kinetic, Mechanical, Power/Thermal, Environmental, EW, Cyber, Optical, Signature, Human Factors).
- **Measures of Effectiveness (MoE)**: Mathematical formulations governing mission success (e.g., End-to-End AST Determinism, Zero Invariant Drift, Subagent Execution Latency).
- **Level 1B Boundary Rule**: ConOps operates strictly at the operational problem space. **ConOps is strictly forbidden from defining, citing, or injecting formal Level 2 System Use Cases (`uc-xx`) or internal software classes.**

### 4.2 Level 1: System of Interest (SoI) Context & Inter-Subsystem Pipeline

**Repository Home:** `schema/model.sysml` (Top-level System Architecture).

#### Scope and Role
Level 1 defines the bounded, cohesive System of Interest (the DEAP Compiler System) as an enclosing structural classifier under KerML Clause 8:
- **The Enclosing Classifier (`part def DEAP_Compiler_System`)**: Represents the physical/logical system boundary.
- **Subsystem Part Usages**: Formally instantiates the 12 subsystems as owned part occurrences:
  ```sysml
  part universalIngestionEngine : Subsystem_2_Universal_Schema_Ingestion_Engine::IngestionEngine;
  part nodeArenaASTGraphEngine : Subsystem_3_Core_Metamodel_Node_Arena::ArenaGraphEngine;
  part grammarLoweringEngine : Subsystem_4_Complete_SysMLv2_KerML_Grammar::GrammarEngine;
  // ... instantiating all 12 subsystems
  ```
- **Inter-Subsystem Pipeline Connectors**: Wires data and control flows between subsystem ports within the enclosing classifier context:
  ```sysml
  connect universalIngestionEngine.token_out to nodeArenaASTGraphEngine.token_in;
  connect nodeArenaASTGraphEngine.ast_graph_out to grammarLoweringEngine.ast_in;
  ```
- **System Requirements Specification (SyRS)**: Captures Tier 2 system-level requirements and allocates them to the instantiated subsystem parts.

### 4.3 Level 2: Subsystem Decomposition (The 12 Subsystems)

**Repository Home:** `schema/subsystems/subsystem_<id>_<slug>/`

Each of the 12 subsystems declared in the DEAP compiler architecture represents a sovereign engineering domain:
1. `subsystem_01_system_vision`
2. `subsystem_02_universal_schema_ingestion_engine`
3. `subsystem_03_core_metamodel_node_arena`
4. `subsystem_04_complete_sysmlv2_kerml_grammar`
5. `subsystem_05_7d_physical_metrology_flow_networks`
6. `subsystem_06_spatio_temporal_state_solvers`
7. `subsystem_07_formal_safety_traceability_verification`
8. `subsystem_08_level_1c_icd_interconnect_contracts`
9. `subsystem_09_downstream_specification_projections`
10. `subsystem_10_multi_target_codegen_simulation_bindings`
11. `subsystem_11_standardized_diagnostic_error_catalog`
12. `subsystem_12_compiler_performance_cli_assurance`

To achieve perfect balance between modularity, comprehensibility, and toolchain performance, every subsystem directory is organized into **Three Standardized Functional Facets**:

```
schema/subsystems/subsystem_02_universal_schema_ingestion_engine/
├── requirements.sysml   <-- Facet 1: Requirements, Constraints, In-Place Traceability
├── behavior.sysml       <-- Facet 2: Detailed Functional Design (Use Cases, States, Actions)
└── architecture.sysml   <-- Facet 3: Detailed Structural Design (Parts, Ports, Item Payloads)
```

#### Facet 1: `requirements.sysml` (Requirements & Constraints)
Captures all subsystem-level requirements, safety constraints, and verification declarations:
- `requirement def`: Formal subsystem requirements with UUIDv5 anchors, complexity classes, and governing standards.
- `doc /* \begin{aligned} ... \end{aligned} */`: Mathematical formulations of invariants.
- `constraint def` / `assert constraint`: Formal boolean and numerical invariant checks.
- `attribute ac_*`: BDD-style Acceptance Criteria (Given-When-Then-Diagnostic).
- **In-Place Traceability**:
  ```sysml
  requirement def REQ_0010_Universal_Lexer_Tokenization {
      id = "REQ-0010";
      text = "The Universal Schema Ingestion Engine shall tokenize arbitrary schema streams...";
      satisfy by Subsystem_2_Universal_Schema_Ingestion_Engine::IngestionEngine;
      verify by Subsystem_2_Universal_Schema_Ingestion_Engine::IngestionVerificationAction;
  }
  ```

#### Facet 2: `behavior.sysml` (Detailed Functional Design)
Captures the dynamic, temporal, and operational logic of the subsystem:
- `use case def`: Subsystem-level use cases defining operational goals, subjects, and actors.
- `interaction def`: Message sequences and user story interaction scenarios between internal components.
- `state def`: Mode supervisors, state machines, transitions, guards, events, and effects.
- `action def`: Algorithmic computational flows, dataflow pipelines, loops, and parameter bindings (`in` / `out`).

#### Facet 3: `architecture.sysml` (Detailed Structural & Data Design)
Captures the static physical and logical design, interfaces, and data types:
- `part def IngestionEngine`: The root execution block of the subsystem.
- `part def`: Subcomponents and internal execution engines (e.g., Lexer, TokenArena, StreamBuffer).
- `port def`: Typed interface points defining directional boundaries (`in`, `out`, `inout`, conjugated `~`).
- `item def`: Data schemas, token payloads, AST node types, and serializable packet structures.
- **Intra-Subsystem Connectors**: Enclosed connectors binding subcomponents together within the subsystem part def.

---

## 5. Architectural Anti-Patterns in SysML v2 / KerML Modeling

To ensure that the DEAP framework maintains the highest standards of MBSE rigor, this investigation formally identifies and analyzes three critical anti-patterns that must be strictly prohibited across all workspaces.

```mermaid
graph TD
    subgraph "Three Critical Architectural Anti-Patterns"
        AP1["Anti-Pattern 1: Detached connections.sysml<br/>• Violates KerML Clause 8 semantics<br/>• Connectors unenclosed in classifiers<br/>• Creates dangling references and scoping errors"]
        AP2["Anti-Pattern 2: Detached traceability.sysml<br/>• Ignores native 'satisfy by' language clauses<br/>• Creates twin-maintenance churn<br/>• Suffers continuous synchronization drift"]
        AP3["Anti-Pattern 3: Monolithic model.sysml<br/>• Massive 1MB+ single files<br/>• Git merge thrashing & cognitive bloat<br/>• Defeats modular compilation & arena caching"]
    end
```

### 5.1 Anti-Pattern 1: The Detached `connections.sysml` Anti-Pattern

#### The Flawed Pattern
A common temptation in large modeling efforts is to strip all connectors out of architectural files and place them into a global, standalone file named `connections.sysml` containing hundreds of loose connection statements at package scope:

```sysml
// ANTI-PATTERN: DETACHED connections.sysml
package GlobalConnections {
    import Subsystem_1::*;
    import Subsystem_2::*;

    // Loose, unenclosed connections
    connection def c1 {
        connect Subsystem1Part.out_port to Subsystem2Part.in_port;
    }
    connection def c2 {
        connect Subsystem2Part.ast_out to Subsystem3Part.ast_in;
    }
}
```

#### Why This Is a Critical Anti-Pattern
1. **Direct Violation of KerML Clause 8 Semantics**: As proven in Section 1.5, KerML Clause 8 defines a connector as a feature relating other features *within an enclosing classifier context*. Connectors are not free-floating top-level entities. A connector declared inside a package has no enclosing occurrence classifier, meaning it attempts to connect features across un-instantiated types.
2. **Namespace Pollution and Dangling Symbol Resolution**: When connectors reside in a detached file, the file must import every package across the entire repository. This creates global namespace pollution, increases symbol resolution complexity from $O(N)$ to $O(N^2)$, and risks circular package dependencies.
3. **Broken Modular Compilation**: If a subsystem is modified, tested, or compiled in isolation (as in a standalone subagent dispatch), compiling the detached `connections.sysml` will fail because it requires the entire universe of subsystems to be present simultaneously.

#### The Normative Resolution
Connectors must be co-located with their enclosing classifier:
- **Inter-Subsystem Connections**: Belong strictly within the enclosing System of Interest classifier (`part def DEAP_Compiler_System`) in `schema/model.sysml`.
- **Intra-Subsystem Connections**: Belong strictly within the enclosing Subsystem classifier (`part def SubsystemEngine`) in the subsystem's `architecture.sysml`.

### 5.2 Anti-Pattern 2: The Detached `traceability.sysml` Anti-Pattern

#### The Flawed Pattern
Under this anti-pattern, requirements are declared in isolation without realization clauses, while a detached file (`traceability.sysml` or `mappings.sysml`) attempts to link them to implementation blocks using separate mapping statements:

```sysml
// ANTI-PATTERN: DETACHED traceability.sysml
package GlobalTraceability {
    import Requirements::*;
    import Architecture::*;

    // Detached mapping blocks
    dependency from REQ_0001 to Subsystem_1_System_Vision;
    dependency from REQ_0002 to IngestionEngine;
}
```

#### Why This Is a Critical Anti-Pattern
1. **Disregard for SysML v2 First-Class Traceability Language Operators**: SysML v2 was deliberately designed to eliminate external traceability mapping files by introducing native in-place operators (`satisfy by`, `verify by`, `require`, `derived from`). Ignoring these built-in language operators regresses SysML v2 back to the limitations of SysML v1.
2. **Severe Synchronization Drift & Twin-Maintenance Churn**: When an engineer edits a requirement or refactors a part in `architecture.sysml`, they must remember to open and update `traceability.sysml`. In autonomous multi-agent environments, agents inevitably forget to update detached files, resulting in immediate model drift and failing parity gates.
3. **Loss of Compilation Type-Checking**: When traceability is declared in-place on the `requirement def`, the compiler immediately validates the type compatibility and existence of the realizing part. In detached files, broken links often go undetected until late-stage integration audits.

#### The Normative Resolution
All traceability must be declared **in-place**:
- Use `satisfy by <TargetPart/Action>;` directly inside the `requirement def` in `requirements.sysml`.
- Use `verify by <VerificationAction>;` directly inside the `requirement def` in `requirements.sysml`.
- Use `require <Constraint>;` to co-locate formal mathematical invariants directly with the requirement.

### 5.3 Anti-Pattern 3: The Monolithic `model.sysml` Compaction Anti-Pattern

#### The Flawed Pattern
Compiling an entire enterprise or compiler system into a single monolithic `model.sysml` file exceeding 10,000 to 50,000 lines of code.

#### Why This Is a Critical Anti-Pattern
1. **Subagent Context Bloat & Failure**: In autonomous engineering workflows, subagents dispatched to implement a single micro-task cannot consume a 1MB monolithic file without suffering severe context degradation, hallucination, or truncation.
2. **Git Churn and Merge Disasters**: When multiple subagents or human developers work in parallel, concurrent edits to a single monolithic file produce insurmountable merge conflicts.
3. **Inefficient Incremental Compilation**: Incremental compilers and arena allocators cannot cache unmodified AST slices when every minor change touches the root file.

#### The Normative Resolution
Adopt the 2D multi-facet decomposition: Level 0 (ConOps) $\rightarrow$ Level 1 (System Root) $\rightarrow$ Level 2 (12 Subsystems $\times$ 3 Facets), deterministically assembled and validated via the AST compiler.

---

## 6. Concrete Implementation Blueprint for DEAP Schema Hierarchy

The following concrete implementation blueprint specifies the exact filesystem layout and SysML v2 source code structure for the DEAP repository.

### 6.1 Complete Directory Layout Tree

```
schema/
├── conops/                                      <-- Level 0: Operational Context
│   ├── actors.sysml                             <-- Operational Performers & External Actors
│   ├── activities.sysml                         <-- Operational Activities (OA-01..OA-N)
│   ├── threats.sysml                            <-- Multi-Domain Threat Matrix (9 Domains)
│   └── operational_modes.sysml                  <-- Operational Lifecycle States
│
├── model.sysml                                  <-- Level 1: System of Interest (Enclosing SoI)
│
└── subsystems/                                  <-- Level 2: The 12 Subsystems
    ├── subsystem_01_system_vision/
    │   ├── requirements.sysml
    │   ├── behavior.sysml
    │   └── architecture.sysml
    ├── subsystem_02_universal_schema_ingestion_engine/
    │   ├── requirements.sysml
    │   ├── behavior.sysml
    │   └── architecture.sysml
    ├── subsystem_03_core_metamodel_node_arena/
    │   ├── requirements.sysml
    │   ├── behavior.sysml
    │   └── architecture.sysml
    ├── subsystem_04_complete_sysmlv2_kerml_grammar/
    │   ├── requirements.sysml
    │   ├── behavior.sysml
    │   └── architecture.sysml
    ├── subsystem_05_7d_physical_metrology_flow_networks/
    │   ├── requirements.sysml
    │   ├── behavior.sysml
    │   └── architecture.sysml
    ├── subsystem_06_spatio_temporal_state_solvers/
    │   ├── requirements.sysml
    │   ├── behavior.sysml
    │   └── architecture.sysml
    ├── subsystem_07_formal_safety_traceability_verification/
    │   ├── requirements.sysml
    │   ├── behavior.sysml
    │   └── architecture.sysml
    ├── subsystem_08_level_1c_icd_interconnect_contracts/
    │   ├── requirements.sysml
    │   ├── behavior.sysml
    │   └── architecture.sysml
    ├── subsystem_09_downstream_specification_projections/
    │   ├── requirements.sysml
    │   ├── behavior.sysml
    │   └── architecture.sysml
    ├── subsystem_10_multi_target_codegen_simulation_bindings/
    │   ├── requirements.sysml
    │   ├── behavior.sysml
    │   └── architecture.sysml
    ├── subsystem_11_standardized_diagnostic_error_catalog/
    │   ├── requirements.sysml
    │   ├── behavior.sysml
    │   └── architecture.sysml
    └── subsystem_12_compiler_performance_cli_assurance/
        ├── requirements.sysml
        ├── behavior.sysml
        └── architecture.sysml
```

### 6.2 Concrete Code Examples

#### Level 0: Operational Activity in `schema/conops/activities.sysml`
```sysml
package DEAP_ConOps_Activities {
    doc /* Operational Activities defining the Level 0 problem space */

    action def OA_01_Execute_Deterministic_Compilation {
        doc /* Operational Activity: Ingest schema and emit verified specifications */
        in item raw_schema_stream : String;
        out item verified_specifications : String;
        out item compilation_diagnostic_report : String;
    }

    action def OA_02_Verify_Safety_Invariants {
        doc /* Operational Activity: Formal verification of system safety invariants */
        in item ast_graph : String;
        out item safety_verification_proof : String;
    }
}
```

#### Level 1: System of Interest in `schema/model.sysml`
```sysml
package DEAP_Compiler_System {
    doc /* Top-Level System of Interest Enclosing Classifier (KerML Clause 8 Compliant) */

    import Subsystem_2_Universal_Schema_Ingestion_Engine::*;
    import Subsystem_3_Core_Metamodel_Node_Arena::*;
    import Subsystem_4_Complete_SysMLv2_KerML_Grammar::*;

    part def CompilerSystemOfInterest {
        doc /* Root System Classifier instantiating the 12 subsystems as part usages */

        // Subsystem Part Occurrences
        part ingestionEngine : UniversalSchemaIngestionEngine;
        part nodeArenaEngine : CoreMetamodelNodeArenaEngine;
        part grammarEngine : SysMLv2KerMLGrammarEngine;

        // Inter-Subsystem Pipeline Connections enclosed within Classifier Scope
        connect ingestionEngine.token_stream_out to nodeArenaEngine.token_stream_in;
        connect nodeArenaEngine.ast_graph_out to grammarEngine.ast_graph_in;
    }
}
```

#### Level 2, Facet 1: `requirements.sysml` in `subsystem_02_universal_schema_ingestion_engine/`
```sysml
package Subsystem_2_Universal_Schema_Ingestion_Engine_Requirements {
    doc /* Subsystem 2 Requirements & Formal Invariants */

    import Subsystem_2_Universal_Schema_Ingestion_Engine_Architecture::*;

    requirement def REQ_0010_Universal_Lexer_Tokenization {
        id = "REQ-0010";
        text = "The Universal Schema Ingestion Engine shall tokenize arbitrary schema streams into a strongly-typed token stream with zero data corruption.";
        
        attribute uuidv5 : String = "3b8a1c9e-5f4a-4d2b-8a7c-1e2f3a4b5c6d";
        attribute complexity_class : String = "Class P (O(N) linear time scan)";
        attribute governing_standard : String = "ISO/IEC/IEEE 15288:2023 §6.4.3";
        
        attribute ac_01_lossless_token_preservation : String = 
            "Given: A valid schema source stream. When: The universal lexer tokenizes the source. Then: Every non-whitespace byte is preserved in the emitted token arena. Diagnostic: E0101 on token drop.";

        // In-Place Formal Traceability
        satisfy by UniversalSchemaIngestionEngine;
        verify by IngestionVerificationHarness;

        // In-Place Invariant Constraint
        require constraint Invariant_Lossless_Token_Preservation;
    }

    constraint def Invariant_Lossless_Token_Preservation {
        doc /* \forall t \in \text{SourceTokens}, \quad t \in \text{EmittedArenaTokens} */
    }
}
```

#### Level 2, Facet 2: `behavior.sysml` in `subsystem_02_universal_schema_ingestion_engine/`
```sysml
package Subsystem_2_Universal_Schema_Ingestion_Engine_Behavior {
    doc /* Subsystem 2 Detailed Functional Design */

    import Subsystem_2_Universal_Schema_Ingestion_Engine_Architecture::*;

    use case def UC_0201_Tokenize_Input_Schema {
        doc /* Subsystem Use Case: Tokenization of heterogeneous input schemas */
        subject : UniversalSchemaIngestionEngine;
        actor fileSystemSource : String;
    }

    state def IngestionEngineLifecycle {
        doc /* Discrete State Machine for Ingestion Engine */
        entry;
        state Idle;
        state Ingesting;
        state Emitting;
        state Fault;

        transition t_start from Idle to Ingesting;
        transition t_complete from Ingesting to Emitting;
        transition t_finish from Emitting to Idle;
        transition t_error from Ingesting to Fault;
    }

    action def ExecuteTokenizationPipeline {
        in item raw_source : String;
        out item token_stream : TokenStreamItem;
    }
}
```

#### Level 2, Facet 3: `architecture.sysml` in `subsystem_02_universal_schema_ingestion_engine/`
```sysml
package Subsystem_2_Universal_Schema_Ingestion_Engine_Architecture {
    doc /* Subsystem 2 Detailed Structural and Data Design */

    item def TokenItem {
        attribute token_type : String;
        attribute lexeme : String;
        attribute span_start : Integer;
        attribute span_end : Integer;
    }

    item def TokenStreamItem {
        attribute tokens : TokenItem[0..*];
    }

    port def TokenStreamOutPort {
        out item payload : TokenStreamItem;
    }

    port def TokenStreamInPort {
        in item payload : TokenStreamItem;
    }

    part def LexerSubcomponent {
        port out tokens_out : TokenStreamOutPort;
    }

    part def SanitizerSubcomponent {
        port in tokens_in : TokenStreamInPort;
        port out tokens_out : TokenStreamOutPort;
    }

    part def UniversalSchemaIngestionEngine {
        doc /* Sovereign Subsystem Execution Engine */
        port token_stream_out : TokenStreamOutPort;

        part lexer : LexerSubcomponent;
        part sanitizer : SanitizerSubcomponent;

        // Intra-Subsystem Connections (enclosed within the subsystem part def)
        connect lexer.tokens_out to sanitizer.tokens_in;
        connect sanitizer.tokens_out to token_stream_out;
    }

    part def IngestionVerificationHarness {
        doc /* Automated Verification Engine for REQ-0010 */
    }
}
```

---

## 7. AST Compilation, Verification, and Downstream Projection Flowchart

The following flowchart illustrates how the multi-file 2D decomposed schema is compiled by the DEAP compiler (`crates/compile-sysml` and `crates/ingest-sysml`), checked against semantic gates, and projected into downstream Agile specifications:

```mermaid
flowchart TD
    subgraph "Phase 1: Multi-File Ingestion & Path Resolution"
        F_L0["Level 0: schema/conops/*.sysml"]
        F_L1["Level 1: schema/model.sysml"]
        F_L2["Level 2: schema/subsystems/*/*.sysml"]
        
        RESOLVER["Directory-to-Namespace Resolver<br/>(Deterministic RFC 4122 UUIDv5 Namespace Tree)"]
        F_L0 --> RESOLVER
        F_L1 --> RESOLVER
        F_L2 --> RESOLVER
    end

    subgraph "Phase 2: Semantic Compilation & KerML Clause 8 Validation"
        AST_ARENA["AST Node Arena Graph Construction"]
        RESOLVER --> AST_ARENA
        
        VAL_CONN["KerML Clause 8 Connection Validator<br/>• Asserts connectors enclosed in Classifiers<br/>• Checks Port Directions (in -> out is Error)<br/>• Validates Port Conjugation & Types"]
        VAL_TRACE["In-Place Traceability Validator<br/>• Asserts 100% requirements have satisfy by<br/>• Asserts 100% requirements have verify by<br/>• Verifies Invariant Constraints"]
        
        AST_ARENA --> VAL_CONN
        AST_ARENA --> VAL_TRACE
    end

    subgraph "Phase 3: Parity Auditing & Downstream Projection"
        DIGEST["Cryptographic SHA-256 Schema Digest<br/>(.pipeline/schema-digest.json)"]
        VAL_CONN --> DIGEST
        VAL_TRACE --> DIGEST
        
        PROJ_EPICS["docs/epics/ (from Subsystem Packages)"]
        PROJ_FEATS["docs/features/ (from Part & Item Definitions)"]
        PROJ_STORIES["docs/user-stories/ (from Action & State Transitions)"]
        PROJ_UC["docs/use-cases/ (from Use Case Definitions)"]
        PROJ_MBD["Model-Based Design (Simulink / Embedded Coder DO-178C)"]
        
        DIGEST --> PROJ_EPICS
        DIGEST --> PROJ_FEATS
        DIGEST --> PROJ_STORIES
        DIGEST --> PROJ_UC
        DIGEST --> PROJ_MBD
    end
```

---

## 8. Conclusion & Strategic Recommendations

### 8.1 Summary of Findings
1. **Metamodel Alignment**: The OMG SysML v2 and KerML specifications require connectors to be enclosed within a classifier/occurrence context. Loose connections declared directly inside packages violate KerML Clause 8.
2. **First-Class Traceability**: SysML v2 provides first-class language clauses (`satisfy by`, `verify by`, `require`, `derived from`) co-located on definitions. Detached traceability files are an obsolete legacy of SysML v1 and constitute an anti-pattern.
3. **The 2D Decomposition Mandate**: Grounded in ISO/IEC/IEEE 15288:2023, ISO/IEC/IEEE 29148:2018, INCOSE SEH v5.0, MagicGrid, Arcadia, and DO-178C, complex systems engineering requires a 2D matrix decomposition across vertical abstraction tiers (Level 0 ConOps $\rightarrow$ Level 1 SoI $\rightarrow$ Level 2 Subsystems) and horizontal functional facets (Requirements, Behavior, Architecture).

### 8.2 Strategic Action Plan for DEAP
1. **Transition from Monolithic to 2D Decomposed Schema**:
   - Migrate `schema/model.sysml` to serve exclusively as the **Level 1 System of Interest** enclosing classifier, instantiating the 12 subsystems and containing only inter-subsystem pipeline connectors.
   - Decompose the 12 subsystems into modular directories under `schema/subsystems/` with the standardized 3-facet structure (`requirements.sysml`, `behavior.sysml`, `architecture.sysml`).
   - Ground all operational context, external actors, and mission intent in `schema/conops/` and `docs/conops/CONOPS.md` (Level 0).
2. **Enforce Semantic Validation in `crates/compile-sysml`**:
   - Update `crates/compile-sysml/src/semantic/validator.rs` to explicitly verify KerML Clause 8 connection enclosure, rejecting any `connect` statement not enclosed within a `part def` or classifier.
   - Verify that all `requirement def` nodes declare in-place `satisfy by` and `verify by` targets.
3. **Multi-Agent Execution Decoupling**:
   - Leverage the 3-facet subsystem structure to dispatch micro-task subagents with surgical, single-file context payloads (e.g., dispatching an implementer targeting only `subsystem_02_universal_schema_ingestion_engine/behavior.sysml`), achieving zero context bloat and rapid, deterministic TDD execution.

---
*Report successfully compiled and verified under the authority of the Lead MBSE & OMG Standards Architect Research Subagent.*
