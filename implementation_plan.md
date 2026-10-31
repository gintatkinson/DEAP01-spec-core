# Implementation Plan -- Update Multi-Tier Repository Taxonomy & Add Missing DEAP Repositories in `README.md`

## 1. Objective & Scope

Update the outdated repository tier and taxonomy sections in `README.md` (Section 1 / 1.2, Section 4 / 4.1, and Sections 5.3, 5.4, and 5.6) so that:
1. **4-Tier Poly-Repo Hierarchy (`Tier 0 -> Tier 1 -> Tier 2 -> Tier 3`)** aligns with `docs/architecture/blueprints/DEAP_POLY_REPO_BINARY_DISTRIBUTION_ARCHITECTURE.md` (`DEAP-BLUEPRINT-POLYREPO-013`) and the Binary-Only Toolchain Distribution Firewall (`target/release/` + `.pipeline/toolchain-manifest.json`, zero Tier 0 `crates/*` source duplication downstream).
2. **100% Additive Repository Inventory (Zero Removals of Existing Canonical Domain Templates)**:
   - **Tier 0 (`UPSTREAM_SPEC_CORE_COMPILER`)**:
     - `DEAP01-spec-core`
   - **Tier 1 (`UPSTREAM_PLATFORM_TEMPLATE` -- Platform Profile & Implementation Driver Templates, extensible `DEAP-profile-*`)**:
     - `DEAP-profile-flutter-app` (`profile.yaml`: `flutter-app` -- Shared Flutter Application Shell `package:app_flutter`, universal 5-table SQLite engine, `DeapComponentScope` open-closed widget registry, Cesium Native C++ 3D rendering FFI)
     - `DEAP-profile-react-web` (`profile.yaml`: `react-web` -- React/TypeScript high-density operator web console template `web_react/`)
     - `DEAP-profile-backend-api` (`profile.yaml`: `backend-api` -- Backend API, gRPC/REST service, and telemetry gateway profile template)
     - `DEAP-profile-vhdl-hardware` (`profile.yaml`: `vhdl-hardware` -- High-assurance VHDL / FPGA RTL hardware synthesis profile template)
     - `DEAP-implementation-driver` (General downstream implementation driver & binary-only DEAP governance bootstrap template)
   - **Tier 2 (`DOMAIN_DISTRIBUTION_TEMPLATE` & `SPECIALIZED_DOMAIN_DEAP_PRODUCT_LINE` -- Domain Distribution Templates & Specialized Domain Product-Line Engines, extensible `DEAP-*` / `DEAP01-*-engine`)**:
     - **Tier 2A -- Clean-Landing-Zone Domain Distribution Templates (`DOMAIN_DISTRIBUTION_TEMPLATE`, `DEAP-*`)** *(preserving all 6 existing canonical sector templates + adding missing active templates + open-ended extensibility for upcoming sector templates)*:
       1. **Avionics & Flight Safety:** `DEAP-avionic-flight-safety` (RTCA DO-178C / DO-254, SAE ARP4754A / ARP4761, ASTM F3269-17 RTA, RTCA DO-365B)
       2. **Aerospace & UAS Infrastructure Safety:** `DEAP-uas-infrastructure-safety` (JARUS SORA v2.5 SAIL I-VI, ASTM F3269-17 RTA, ASTM F3411-22a Remote ID, RTCA DO-365B DAA)
       3. **Network & Telecommunications:** `DEAP-netcom` (YANG Network Topology, OpenAPI v3, Protobuf v3, IETF / 3GPP Control-Plane & Telemetry Assurance)
       4. **Medical & Healthcare:** `DEAP-surgical-robotics-console` (IEC 62304 Class C, ISO 14971, FDA Class III)
       5. **Space & Satellites:** `DEAP-space-cubesat-constellation` (ECSS-E-ST-40C, NASA-STD-8739.8)
       6. **Industrial Robotics:** `DEAP-industrial-warehouse-agv` (ISO 3691-4, IEC 61508, VDA 5050)
       7. **Subsea & Maritime:** `DEAP-subsea-oceanographic-auv` (DNV-GL-ST-E403, IMO MASS)
       8. **Rail & Transportation:** `DEAP-rail-autonomous-locomotive` (EN 50126, EN 50128 SIL 4)
     - **Tier 2B -- Active-Schema Specialized Domain Product-Line Engines (`SPECIALIZED_DOMAIN_DEAP_PRODUCT_LINE`, `DEAP01-*-engine`)** *(adding `DEAP01-cuas-coa-engine` + open-ended extensibility for upcoming domain engines)*:
       1. **Counter-UAS Tactical Planning & C2:** `DEAP01-cuas-coa-engine` (active `schema/cuas_coa.sysml`, curated Epics/Features/User Stories/Use Cases, Native Rust multi-objective Pareto solver `crates/cuas-coa-engine/` via Zero-Trust C-ABI, and Flutter ARINC 661 Tactical Cockpit `app_flutter/` consuming `DEAP-profile-flutter-app`; MIL-STD-882E, ARINC 661, STANAG 4586, MIL-STD-2525D, DO-178C / DO-330)
   - **Tier 3 (`DOWNSTREAM_CUSTOMER_PROJECT` / `DOWNSTREAM_APPLICATION_WORKSPACE` -- Customer Application Workspaces)**:
     - Generic `customer-project-*` downstream customer workspaces (keeping private customer/GitLab repository names unexposed in upstream docs), onboarded strictly from Tier 1 Platform Profile / Driver Templates, Tier 2A Domain Distribution Templates, or Tier 2B Specialized Domain Product-Line Engines -- never directly from `DEAP01-spec-core`.

---

## 2. Exact Target File & Approved Surgical Edits

### Target File: `README.md`

#### Edit 1: Section 1 & Section 1.2 (`README.md` lines 17-74)

**Target:**
```markdown
Operating purely on Abstract Syntax Tree (AST) tokens without hardcoding domain concepts, `DEAP01-spec-core` serves as the upstream parent compiler (`UPSTREAM_SPEC_CORE_COMPILER`) from which Tier 2 domain-specific distribution templates across 6 canonical cyber-physical sectors (Aerospace & Defense, Medical & Healthcare, Space & Satellites, Industrial Robotics, Subsea & Maritime, Rail & Transportation) are maintained, and from which Tier 3 downstream customer application workspaces are onboarded via domain distribution templates.

### 1.1 Primary Commercial Toolchain Integration

This platform explicitly declares **MATLAB / Simulink / Stateflow / Embedded Coder** as the Primary Tier-1 Commercial Toolchain Integration (Model-Based Design, Control Law Synthesis, DO-178C C/SPARK Ada Code Generation).

### 1.2 Three-Tier Architecture & Repository Boundaries: Upstream Compiler vs. Domain Templates vs. Customer Workspaces

The DEAP framework strictly enforces a clean three-tier architecture separating upstream compiler tooling propagation from downstream customer onboarding:

```text
+-----------------------------------------------------------------------------------+
| Tier 1: Upstream Specification Core Compiler                                      |
| Repository: DEAP01-spec-core (Role: UPSTREAM_SPEC_CORE_COMPILER)                  |
| Sentinel: .pipeline/upstream/ present | Pure Schema-Driven Abstract Compiler       |
+-----------------------------------------------------------------------------------+
                                         |
                                         |  Compiler Tooling Propagation (Maintainers)
                                         |  Template Distribution & verify-baseline setup-hooks
                                         v
+-----------------------------------------------------------------------------------+
| Tier 2: Domain Distribution Templates (Canonical Cyber-Physical Sectors)         |
| Repositories: DEAP-* (e.g. DEAP-surgical-robotics-console, DEAP-industrial-agv)   |
| Role: DOMAIN_DISTRIBUTION_TEMPLATE | Sentinel: .pipeline/upstream/ removed        |
| Content: Domain SysML v2 schemas (schema/), sector blueprints, domain profiles    |
| Landing zones (docs/epics/, docs/features/, etc.) remain clean (.gitkeep)         |
+-----------------------------------------------------------------------------------+
                                         |
                                         |  Customer Project Onboarding (End-Users)
                                         |  Self-contained bootstrap via Domain URL
                                         v
+-----------------------------------------------------------------------------------+
| Tier 3: Customer Application Workspaces                                           |
| Repositories: customer-project-* (e.g. downstream domain workspaces)              |
| Role: DOWNSTREAM_APPLICATION_WORKSPACE / DOWNSTREAM_CUSTOMER_PROJECT             |
| Content: Concrete application code, ROS2 nodes, embedded modules, SIL/HIL tests    |
+-----------------------------------------------------------------------------------+
```

#### Tier 1: Upstream Specification Core Compiler (`DEAP01-spec-core`)
- **Upstream Spec Core Compiler (`DEAP01-spec-core`):** Abstract, domain-agnostic specification compiler and multi-agent verification platform. Operates with the sentinel `.pipeline/upstream/` directory present. All upstream landing zones (`docs/conops/`, `docs/safety/`, `docs/epics/`, `docs/features/`, `docs/user-stories/`, `docs/use-cases/`, `schema/`) remain pristine with zero concrete domain specifications or domain-specific `.sysml` files.
- **Pure Schema-Driven Compiler Invariant:** `DEAP01-spec-core` is an abstract Model-Based Systems Engineering (MBSE) compiler; all domain semantics, safety statecharts, and specifications derive deterministically from user-provided schemas in `schema/`.

#### Tier 2: Domain Distribution Templates (`DEAP-*`)
- **Tier 2 Domain Distribution Templates (`DEAP-*`):** Canonical sector repositories derived from `DEAP01-spec-core` by compiler maintainers. Tooling propagation removes `.pipeline/upstream/`, enabling domain-specific SysML v2 models, schemas in `schema/`, and sector architecture blueprints across 6 canonical domains:
  1. **Aerospace & Defense:** `DEAP-uas-infrastructure-safety` (SORA, ASTM F3269, DO-365B)
  2. **Medical & Healthcare:** `DEAP-surgical-robotics-console` (IEC 62304 Class C, ISO 14971, FDA Class III)
  3. **Space & Satellites:** `DEAP-space-cubesat-constellation` (ECSS-E-ST-40C, NASA-STD-8739.8)
  4. **Industrial Robotics:** `DEAP-industrial-warehouse-agv` (ISO 3691-4, IEC 61508, VDA 5050)
  5. **Subsea & Maritime:** `DEAP-subsea-oceanographic-auv` (DNV-GL-ST-E403, IMO MASS)
  6. **Rail & Transportation:** `DEAP-rail-autonomous-locomotive` (EN 50126, EN 50128 SIL 4)
  In Tier 2 domain templates, specification landing zones (`docs/epics/`, `docs/features/`, `docs/user-stories/`, `docs/use-cases/`) remain clean landing zones with `.gitkeep` files until projected for a concrete implementation.

#### Tier 3: Customer Application Workspaces
- **Customer Application Workspaces (`DOWNSTREAM_APPLICATION_WORKSPACE` / `DOWNSTREAM_CUSTOMER_PROJECT`):** Concrete engineering repositories where end-user engineering teams build proprietary software, ROS2 nodes, embedded modules, and hardware-in-the-loop tests.
- **Clean Customer Boundary Mandate:** Customer projects onboard strictly from their relevant **Domain Distribution Template** (`DEAP-*`), NOT from the upstream compiler (`DEAP01-spec-core`). Onboarding from the domain template guarantees that all domain-specific SysML v2 schemas, safety contracts, and profile rules are seeded directly into the customer workspace in a single self-contained command with zero external sibling path dependencies.
- **Illustrative Schema Payloads:** Any concrete flight controller, robotic surgical console, satellite bus, AGV guidance, subsea vehicle, rail locomotive controller, STPA hazard analysis, or domain safety examples presented throughout this README and documentation are strictly **illustrative schema payloads** demonstrating compiler ingestion, AST synthesis, projection, and verification capabilities.
```

**Replacement:**
```markdown
Operating purely on Abstract Syntax Tree (AST) tokens without hardcoding domain concepts, `DEAP01-spec-core` serves as the **Tier 0** upstream parent compiler (`UPSTREAM_SPEC_CORE_COMPILER`) from which precompiled native release binaries (`target/release/`) and governance bundles propagate into **Tier 1** Platform Profile & Implementation Driver Templates (`DEAP-profile-*`, `DEAP-implementation-driver`), **Tier 2** Domain Distribution Templates (`DEAP-*`) and Specialized Domain Product-Line Engines (`DEAP01-*-engine`), and **Tier 3** downstream Customer Application Workspaces (`customer-project-*`).

### 1.1 Primary Commercial Toolchain Integration

This platform explicitly declares **MATLAB / Simulink / Stateflow / Embedded Coder** as the Primary Tier-1 Commercial Toolchain Integration (Model-Based Design, Control Law Synthesis, DO-178C C/SPARK Ada Code Generation).

### 1.2 Four-Tier Poly-Repo Architecture & Repository Boundaries

Per [DEAP_POLY_REPO_BINARY_DISTRIBUTION_ARCHITECTURE.md](docs/architecture/blueprints/DEAP_POLY_REPO_BINARY_DISTRIBUTION_ARCHITECTURE.md) (`DEAP-BLUEPRINT-POLYREPO-013`), the DEAP ecosystem enforces a four-tier poly-repo architecture decoupling upstream compiler crate sources from binary-only platform templates, domain templates/product lines, and customer workspaces:

```text
+-----------------------------------------------------------------------------------+
| Tier 0: Upstream Specification Core Compiler                                      |
| Repository: DEAP01-spec-core (Role: UPSTREAM_SPEC_CORE_COMPILER)                  |
| Sentinel: .pipeline/upstream/ present | 6 Pure-Rust Crates + Master Blueprints    |
+-----------------------------------------------------------------------------------+
                                         |
                                         |  Binary-Only Toolchain & Governance Propagation
                                         |  (5 target/release/* binaries + toolchain-manifest.json)
                                         v
+-----------------------------------------------------------------------------------+
| Tier 1: Platform Profile & Implementation Driver Templates                        |
| Repositories: DEAP-profile-flutter-app, DEAP-profile-react-web,                   |
|               DEAP-profile-backend-api, DEAP-profile-vhdl-hardware,               |
|               DEAP-implementation-driver (+ future DEAP-profile-* runtimes)       |
| Role: UPSTREAM_PLATFORM_TEMPLATE | Sentinel: .pipeline/upstream/ removed          |
| Content: Shared platform shells/SDKs, profile.yaml, clean landing zones (.gitkeep)|
+-----------------------------------------------------------------------------------+
                                         |
                                         |  Platform SDK Consumption & Sector Governance
                                         v
+-----------------------------------------------------------------------------------+
| Tier 2: Domain Distribution Templates & Specialized Domain Product-Line Engines   |
| Tier 2A Clean Templates (DEAP-*): DEAP-avionic-flight-safety,                     |
|   DEAP-uas-infrastructure-safety, DEAP-netcom, DEAP-surgical-robotics-console,    |
|   DEAP-space-cubesat-constellation, DEAP-industrial-warehouse-agv,                |
|   DEAP-subsea-oceanographic-auv, DEAP-rail-autonomous-locomotive (+ future DEAP-*)|
| Tier 2B Product-Line Engines (DEAP01-*-engine): DEAP01-cuas-coa-engine (+ future) |
| Roles: DOMAIN_DISTRIBUTION_TEMPLATE / SPECIALIZED_DOMAIN_DEAP_PRODUCT_LINE        |
+-----------------------------------------------------------------------------------+
                                         |
                                         |  Customer Project Onboarding (End-Users)
                                         |  Self-contained bootstrap from Tier 1 or Tier 2
                                         v
+-----------------------------------------------------------------------------------+
| Tier 3: Customer Application Workspaces                                           |
| Repositories: customer-project-* (GitHub, GitLab SaaS/Self-Hosted, SCIF Air-Gap)  |
| Role: DOWNSTREAM_APPLICATION_WORKSPACE / DOWNSTREAM_CUSTOMER_PROJECT              |
| Content: Proprietary mission schemas, site data, app/embedded code, SIL/HIL tests |
+-----------------------------------------------------------------------------------+
```

#### Tier 0: Upstream Specification Core Compiler (`DEAP01-spec-core`)
- **Upstream Spec Core Compiler (`DEAP01-spec-core`):** Abstract, domain-agnostic specification compiler and multi-agent verification platform operating with the sentinel `.pipeline/upstream/` directory present. Houses the 6 canonical workspace crates (`deap-core`, `verify-baseline`, `compile-sysml`, `ingest-sysml`, `assemble-conops`, `reconcile-backlog`) and all canonical master architecture blueprints (`docs/architecture/blueprints/`). All upstream landing zones (`docs/conops/`, `docs/safety/`, `docs/epics/`, `docs/features/`, `docs/user-stories/`, `docs/use-cases/`, `schema/`) remain pristine with only `.gitkeep` files.
- **Pure Schema-Driven Compiler & Binary Firewall Invariant:** `DEAP01-spec-core` contains zero hardcoded customer domain concepts; Tier 0 crate source directories (`crates/*`) never propagate downstream. Downstream tiers receive only the 5 precompiled native release binaries (`target/release/{verify-baseline,compile-sysml,ingest-sysml,assemble-conops,reconcile-backlog}`) tracked by cryptographic SHA-256 provenance in `.pipeline/toolchain-manifest.json`.

#### Tier 1: Platform Profile & Implementation Driver Templates (`DEAP-profile-*`, `DEAP-implementation-driver`)
- **Tier 1 Platform Profile & Driver Templates (`UPSTREAM_PLATFORM_TEMPLATE`):** Reusable runtime shell, SDK, and driver templates maintained with `.pipeline/upstream/` removed, precompiled Tier 0 binaries in `target/release/`, and clean specification landing zones (`.gitkeep`). Active repositories (with additional `DEAP-profile-*` runtime targets expanding continuously):
  1. **Cross-Platform Flutter Desktop / Cockpit Shell:** `DEAP-profile-flutter-app` (`profile.yaml`: `flutter-app` -- shared `package:app_flutter` GUI SDK, universal 5-table SQLite persistence engine, `DeapComponentScope` open-closed widget extension registry, and Cesium Native C++ 3D rendering FFI)
  2. **Operator Console / Web Application:** `DEAP-profile-react-web` (`profile.yaml`: `react-web` -- high-density React/TypeScript operator web console template under `web_react/` with REST/WebSocket transport bindings)
  3. **Backend API & Telemetry Services:** `DEAP-profile-backend-api` (`profile.yaml`: `backend-api` -- distributed backend API, gRPC/REST service, and operational telemetry gateway profile template)
  4. **VHDL / FPGA Hardware Synthesis:** `DEAP-profile-vhdl-hardware` (`profile.yaml`: `vhdl-hardware` -- high-assurance VHDL / FPGA RTL hardware synthesis and register-interface execution profile template)
  5. **General Downstream Implementation Driver:** `DEAP-implementation-driver` (turnkey binary-only DEAP governance, git-hook, and multi-profile bootstrap template)

#### Tier 2: Domain Distribution Templates (`DEAP-*`) & Specialized Domain Product-Line Engines (`DEAP01-*-engine`)
- **Tier 2A -- Clean-Landing-Zone Domain Distribution Templates (`DOMAIN_DISTRIBUTION_TEMPLATE`, `DEAP-*`):** Canonical cyber-physical sector repositories derived from Tier 0 precompiled tooling (`.pipeline/upstream/` removed). Specification landing zones (`docs/epics/`, `docs/features/`, `docs/user-stories/`, `docs/use-cases/`) remain clean landing zones with `.gitkeep` files until projected for a concrete implementation. Active and canonical sector templates (extensible across additional incoming `DEAP-*` domains):
  1. **Avionics & Flight Safety:** `DEAP-avionic-flight-safety` (RTCA DO-178C / DO-254, SAE ARP4754A / ARP4761, ASTM F3269-17 RTA, RTCA DO-365B)
  2. **Aerospace & UAS Infrastructure Safety:** `DEAP-uas-infrastructure-safety` (JARUS SORA v2.5 SAIL I-VI, ASTM F3269-17 RTA, ASTM F3411-22a Remote ID, RTCA DO-365B DAA)
  3. **Network & Telecommunications:** `DEAP-netcom` (YANG Network Topology, OpenAPI v3, Protobuf v3, IETF / 3GPP Control-Plane & Telemetry Assurance)
  4. **Medical & Healthcare:** `DEAP-surgical-robotics-console` (IEC 62304 Class C, ISO 14971, FDA Class III)
  5. **Space & Satellites:** `DEAP-space-cubesat-constellation` (ECSS-E-ST-40C, NASA-STD-8739.8)
  6. **Industrial Robotics:** `DEAP-industrial-warehouse-agv` (ISO 3691-4, IEC 61508, VDA 5050)
  7. **Subsea & Maritime:** `DEAP-subsea-oceanographic-auv` (DNV-GL-ST-E403, IMO MASS)
  8. **Rail & Transportation:** `DEAP-rail-autonomous-locomotive` (EN 50126, EN 50128 SIL 4)
- **Tier 2B -- Active-Schema Specialized Domain Product-Line Engines (`SPECIALIZED_DOMAIN_DEAP_PRODUCT_LINE`, `DEAP01-*-engine`):** Reusable, production-grade domain engines operating in active-schema mode with formal SysML v2 domain metamodels (`schema/*.sysml`), curated Agile specifications (`docs/{epics,features,user-stories,use-cases}/`), deterministic Native Rust domain solver crates (`crates/<domain>-engine/`), and Tier 1 platform shell integration (while keeping customer-specific site data and coordinates strictly isolated to Tier 3). Active product-line engines (with additional `DEAP01-*-engine` product lines planned):
  1. **Counter-UAS Tactical Planning & C2:** `DEAP01-cuas-coa-engine` (`schema/cuas_coa.sysml`, Native Rust multi-objective Pareto solver `crates/cuas-coa-engine/` via Zero-Trust C-ABI, and Flutter ARINC 661 Tactical Cockpit `app_flutter/` consuming `DEAP-profile-flutter-app`; MIL-STD-882E, ARINC 661, STANAG 4586, MIL-STD-2525D, DO-178C / DO-330)

#### Tier 3: Customer Application Workspaces (`customer-project-*`)
- **Customer Application Workspaces (`DOWNSTREAM_APPLICATION_WORKSPACE` / `DOWNSTREAM_CUSTOMER_PROJECT`):** Concrete mission and customer engineering repositories (hosted on GitHub, GitLab SaaS, Self-Hosted GitLab, or Air-Gapped / SCIF defense enclaves) where end-user engineering teams build proprietary application software, site-specific operational schemas, ROS2/PX4 nodes, Flutter/React operator consoles, VHDL modules, and hardware-in-the-loop (SIL/HIL) tests.
- **Clean Customer Boundary Mandate:** Customer projects onboard strictly from their designated **Tier 1 Platform Profile / Driver Template** (`DEAP-profile-*`, `DEAP-implementation-driver`), **Tier 2A Domain Distribution Template** (`DEAP-*`), or **Tier 2B Specialized Domain Product-Line Engine** (`DEAP01-*-engine`) -- NOT directly from the Tier 0 upstream compiler (`DEAP01-spec-core`). Onboarding from Tier 1 or Tier 2 seeds all precompiled verification binaries (`target/release/`), safety contracts, and profile rules directly into the customer workspace with zero Tier 0 compiler source leakage and zero external sibling path dependencies.
- **Illustrative Schema Payloads:** Any concrete flight controller, UAS infrastructure safety net, network control plane, C-UAS tactical planner, robotic surgical console, satellite bus, AGV guidance, subsea vehicle, rail locomotive controller, STPA hazard analysis, or domain safety examples presented throughout this README and documentation are strictly **illustrative schema payloads** demonstrating compiler ingestion, AST synthesis, projection, and verification capabilities.
```

---

#### Edit 2: Section 4 Canonical Blueprints, Repository Trees & Section 4.1 (`README.md` lines 118-190)

**Target:**
```markdown
- **Local Air-Gapped DeepSeek Workstation Blueprint**: [DEAP_LOCAL_AIRGAPPED_DEEPSEEK_WORKSTATION_BLUEPRINT.md](docs/architecture/blueprints/DEAP_LOCAL_AIRGAPPED_DEEPSEEK_WORKSTATION_BLUEPRINT.md)
- **DeepSeek Harness Integration Blueprint**: [DEAP_DEEPSEEK_HARNESS_INTEGRATION_BLUEPRINT.md](docs/architecture/blueprints/DEAP_DEEPSEEK_HARNESS_INTEGRATION_BLUEPRINT.md)

### Repository Trees:

#### Upstream Spec Core Compiler (`DEAP01-spec-core`):
```text
DEAP01-spec-core/
├── .agents/
│   ├── AGENTS.md                  # Project-scoped agentic governance rules & delegation gates
│   └── skills -> ../skills        # Project skills symlink
├── .pipeline/
│   ├── upstream/                  # Sentinel directory marking UPSTREAM_SPEC_CORE_COMPILER role
│   ├── constitution.md            # Platform-independent functional safety governance tier
│   └── profiles/                  # Multi-platform execution profiles (ros2_cpp, px4_module, etc.)
├── docs/
│   ├── architecture/
│   │   └── blueprints/            # Canonical architecture specifications & multi-provider blueprints
│   ├── conops/                    # Abstract landing zone (.gitkeep)
│   │   └── units/                 # Modular ConOps unit landing zones (.gitkeep)
│   │       ├── conops/
│   │       └── mission_intent/
│   ├── interfaces/                # Level 1C ICD interface matrix & signal dictionary landing zone (.gitkeep)
│   ├── safety/                    # Abstract landing zone (.gitkeep)
│   ├── epics/                     # Abstract landing zone (.gitkeep)
│   ├── features/                  # Abstract landing zone (.gitkeep)
│   ├── user-stories/              # Abstract landing zone (.gitkeep)
│   ├── use-cases/                 # Abstract landing zone (.gitkeep)
│   └── management/                # Abstract landing zone (.gitkeep)
├── rules/                         # Abstract verification & governance rules
├── schema/                        # Abstract landing zone (.gitkeep)
├── crates/                        # Pure-Rust workspace crates (verify-baseline, ingest-sysml, compile-sysml, reconcile-backlog, assemble-conops)
├── skills/                        # Multi-agent orchestrator & implementation skills
├── Cargo.toml                     # Workspace manifest & crate dependency configuration
└── README.md                      # Upstream compiler master specification & usage guide
```

#### Tier 2: Domain Distribution Template Repository (e.g. DEAP-surgical-robotics-console, DEAP-industrial-warehouse-agv):
```text
domain-distribution-template/ (e.g. DEAP-surgical-robotics-console, DEAP-industrial-warehouse-agv, ...)
├── .agents/
│   ├── AGENTS.md                  # Project-scoped agentic governance rules & delegation gates
│   └── skills -> ../skills        # Project skills symlink
├── .pipeline/
│   ├── constitution.md            # Platform-independent functional safety governance tier
│   └── profiles/
│       ├── ros2_cpp.md            # ROS2 C++ Real-Time Nodes platform execution profile
│       └── px4_module.md          # PX4 Autopilot / Embedded platform execution profile
├── docs/
│   ├── conops/                    # Domain mission intent & Concept of Operations landing zone
│   │   └── units/                 # Modular ConOps unit landing zones (.gitkeep)
│   │       ├── conops/
│   │       └── mission_intent/
│   ├── interfaces/                # Level 1C ICD interface matrix & signal dictionary landing zone (.gitkeep)
│   ├── safety/                    # STPA hazard analysis, FMECA & domain safety landing zone
│   ├── management/                # Level 4 WBS & Enterprise Realization deliverables (.md, .csv, .json)
│   └── architecture/
│       └── blueprints/            # Canonical architecture specifications & multi-provider blueprints
├── schema/                        # Domain-specific structural schemas & SysML v2 models
├── Cargo.toml                     # Workspace manifest & packaging configuration
└── README.md                      # Domain template master specification & usage guide
```

Tier 3 Customer Application Workspaces inherit this layout from the domain distribution template, populating `docs/epics/`, `docs/features/`, and application source directories (`app_flutter/`, `web_react/`, `ros2/`, `px4/`).

### 4.1 Supported Tier 2 Domain Distribution Templates (Canonical Taxonomies)
1. **Aerospace & Defense:** `DEAP-uas-infrastructure-safety` (SORA, ASTM F3269, DO-365B)
2. **Medical & Healthcare:** `DEAP-surgical-robotics-console` (IEC 62304 Class C, ISO 14971, FDA Class III)
3. **Space & Satellites:** `DEAP-space-cubesat-constellation` (ECSS-E-ST-40C, NASA-STD-8739.8)
4. **Industrial Robotics:** `DEAP-industrial-warehouse-agv` (ISO 3691-4, IEC 61508, VDA 5050)
5. **Subsea & Maritime:** `DEAP-subsea-oceanographic-auv` (DNV-GL-ST-E403, IMO MASS)
6. **Rail & Transportation:** `DEAP-rail-autonomous-locomotive` (EN 50126, EN 50128 SIL 4)
```

**Replacement:**
```markdown
- **Local Air-Gapped DeepSeek Workstation Blueprint**: [DEAP_LOCAL_AIRGAPPED_DEEPSEEK_WORKSTATION_BLUEPRINT.md](docs/architecture/blueprints/DEAP_LOCAL_AIRGAPPED_DEEPSEEK_WORKSTATION_BLUEPRINT.md)
- **DeepSeek Harness Integration Blueprint**: [DEAP_DEEPSEEK_HARNESS_INTEGRATION_BLUEPRINT.md](docs/architecture/blueprints/DEAP_DEEPSEEK_HARNESS_INTEGRATION_BLUEPRINT.md)
- **Poly-Repo Binary-Only Distribution & Tier Isolation Blueprint**: [DEAP_POLY_REPO_BINARY_DISTRIBUTION_ARCHITECTURE.md](docs/architecture/blueprints/DEAP_POLY_REPO_BINARY_DISTRIBUTION_ARCHITECTURE.md) (`DEAP-BLUEPRINT-POLYREPO-013`)
- **Domain-Agnostic Mission Lifecycle Architecture Blueprint**: [DEAP_DOMAIN_AGNOSTIC_MISSION_LIFECYCLE_ARCHITECTURE.md](docs/architecture/blueprints/DEAP_DOMAIN_AGNOSTIC_MISSION_LIFECYCLE_ARCHITECTURE.md)
- **Domain-Independent FDIR Architecture Blueprint**: [DEAP_DOMAIN_INDEPENDENT_FDIR_BLUEPRINT.md](docs/architecture/blueprints/DEAP_DOMAIN_INDEPENDENT_FDIR_BLUEPRINT.md)

### Repository Trees:

#### Tier 0: Upstream Spec Core Compiler (`DEAP01-spec-core`):
```text
DEAP01-spec-core/
├── .agents/
│   ├── AGENTS.md                  # Project-scoped agentic governance rules & delegation gates
│   └── skills -> ../skills        # Project skills symlink
├── .pipeline/
│   ├── upstream/                  # Sentinel directory marking UPSTREAM_SPEC_CORE_COMPILER role
│   ├── constitution.md            # Platform-independent functional safety governance tier
│   ├── ACTIVE_RULES_BUNDLE.md     # Consolidated 20-rule governance manifest
│   └── profiles/                  # Multi-platform execution profiles (flutter, react, ros2_cpp, px4_module)
├── docs/
│   ├── architecture/
│   │   └── blueprints/            # Canonical architecture specifications (SSOT, Tier 0 exclusive)
│   ├── conops/                    # Abstract landing zone (.gitkeep)
│   │   └── units/                 # Modular ConOps unit landing zones (.gitkeep)
│   │       ├── conops/
│   │       └── mission_intent/
│   ├── interfaces/                # Level 1C ICD interface matrix & signal dictionary landing zone (.gitkeep)
│   ├── safety/                    # Abstract landing zone (.gitkeep)
│   ├── epics/                     # Abstract landing zone (.gitkeep)
│   ├── features/                  # Abstract landing zone (.gitkeep)
│   ├── user-stories/              # Abstract landing zone (.gitkeep)
│   ├── use-cases/                 # Abstract landing zone (.gitkeep)
│   └── management/                # Abstract landing zone (.gitkeep)
├── rules/                         # Abstract verification & governance rules
├── schema/                        # Abstract landing zone (.gitkeep)
├── crates/                        # 6 Tier 0 compiler crates (deap-core, verify-baseline, ingest-sysml, compile-sysml, reconcile-backlog, assemble-conops)
├── skills/                        # Multi-agent orchestrator & implementation skills
├── Cargo.toml                     # Workspace manifest & crate dependency configuration
└── README.md                      # Upstream compiler master specification & usage guide
```

#### Tier 1 & Tier 2A: Clean-Landing-Zone Platform & Domain Distribution Templates (`DEAP-profile-*`, `DEAP-implementation-driver`, `DEAP-*`):
```text
distribution-template/ (e.g. DEAP-profile-flutter-app, DEAP-avionic-flight-safety, DEAP-uas-infrastructure-safety, DEAP-netcom, ...)
├── .agents/
│   ├── AGENTS.md                  # Project-scoped agentic governance rules & delegation gates
│   └── skills -> ../skills        # Project skills symlink
├── .pipeline/
│   ├── constitution.md            # Platform-independent functional safety governance tier
│   ├── ACTIVE_RULES_BUNDLE.md     # Consolidated 20-rule governance manifest
│   ├── toolchain-manifest.json    # Cryptographic SHA-256 provenance for precompiled Tier 0 binaries
│   └── profiles/                  # Target platform execution profiles (flutter, react, ros2_cpp, px4_module)
├── docs/
│   ├── conops/                    # Clean landing zone (.gitkeep)
│   │   └── units/                 # Modular ConOps & Mission Intent unit landing zones (.gitkeep)
│   ├── interfaces/                # Level 1C ICD interface matrix & signal dictionary landing zone (.gitkeep)
│   ├── safety/                    # STPA hazard analysis & FMECA landing zone (.gitkeep)
│   ├── epics/                     # Clean specification landing zone (.gitkeep)
│   ├── features/                  # Clean specification landing zone (.gitkeep)
│   ├── user-stories/              # Clean specification landing zone (.gitkeep)
│   ├── use-cases/                 # Clean specification landing zone (.gitkeep)
│   └── management/                # Level 4 WBS & Enterprise Realization landing zone (.gitkeep)
├── rules/                         # Synchronized verification & governance rules
├── schema/                        # Clean schema landing zone (.gitkeep)
├── skills/                        # Synchronized multi-agent workflow skills
├── target/release/                # 5 precompiled Tier 0 binaries (verify-baseline, compile-sysml, ingest-sysml, assemble-conops, reconcile-backlog)
├── profile.yaml                   # Optional platform profile descriptor (flutter-app, react-web, backend-api, vhdl-hardware)
├── app_flutter/ or web_react/     # Optional shared Tier 1 platform shell SDK (e.g. package:app_flutter)
└── README.md                      # Template specification & turnkey onboarding guide
```

**Tier 2B Specialized Domain Product-Line Engines (`DEAP01-*-engine`, e.g. `DEAP01-cuas-coa-engine`)** and **Tier 3 Customer Application Workspaces (`customer-project-*`)** inherit this binary-only toolchain layout (with zero duplicated Tier 0 `crates/*` or `docs/architecture/blueprints/` per Check 12), populating active domain schemas (`schema/*.sysml`), derived Agile specifications (`docs/{epics,features,user-stories,use-cases}/`), domain solver crates (`crates/<domain>-engine/`), and target platform applications (`app_flutter/`, `web_react/`, `ros2/`, `px4/`).

### 4.1 Supported Multi-Tier DEAP Repository Catalog (Canonical Taxonomies)

The following catalog enumerates the canonical Tier 0, Tier 1, Tier 2, and Tier 3 repositories across the DEAP ecosystem (designed as an open-ended architecture with additional `DEAP-profile-*`, `DEAP-*`, and `DEAP01-*-engine` repositories continuously onboarded):

| Tier | Repository / Pattern | Role Classification | Schema & Landing Zone Mode | Target Profile / Regulatory & Standards Scope |
| :--- | :--- | :--- | :--- | :--- |
| **Tier 0** | `DEAP01-spec-core` | `UPSTREAM_SPEC_CORE_COMPILER` | Clean landing zones (`.gitkeep`); `.pipeline/upstream/` present | Pure-Rust MBSE compiler (6 crates, 5 binaries), SysML v2 / IDL / ARXML / YANG / OpenAPI / Protobuf, MATLAB/Simulink hooks |
| **Tier 1** | `DEAP-profile-flutter-app` | `UPSTREAM_PLATFORM_TEMPLATE` | Clean landing zones (`.gitkeep`) + shared `app_flutter/` SDK | `flutter-app` (`package:app_flutter`), universal 5-table SQLite engine, `DeapComponentScope`, Cesium Native C++ 3D FFI |
| **Tier 1** | `DEAP-profile-react-web` | `UPSTREAM_PLATFORM_TEMPLATE` | Clean landing zones (`.gitkeep`) + shared `web_react/` shell | `react-web`, TypeScript high-density operator web console, CSS Container Queries, REST/WebSocket bindings |
| **Tier 1** | `DEAP-profile-backend-api` | `UPSTREAM_PLATFORM_TEMPLATE` | Clean landing zones (`.gitkeep`) + `profile.yaml` | `backend-api`, distributed gRPC / REST microservices, schema-driven API & telemetry gateways |
| **Tier 1** | `DEAP-profile-vhdl-hardware` | `UPSTREAM_PLATFORM_TEMPLATE` | Clean landing zones (`.gitkeep`) + `profile.yaml` | `vhdl-hardware`, high-assurance VHDL / FPGA RTL synthesis, deterministic hardware bus & register maps |
| **Tier 1** | `DEAP-implementation-driver` | `UPSTREAM_PLATFORM_TEMPLATE` | Clean landing zones (`.gitkeep`) + binary-only toolchain | Turnkey downstream implementation driver, precompiled `target/release/` gates, multi-profile bootstrap template |
| **Tier 2A** | `DEAP-avionic-flight-safety` | `DOMAIN_DISTRIBUTION_TEMPLATE` | Clean landing zones (`.gitkeep`) + sector safety profiles | Avionics & Flight Safety (RTCA DO-178C / DO-254, SAE ARP4754A / ARP4761, ASTM F3269-17 RTA, RTCA DO-365B) |
| **Tier 2A** | `DEAP-uas-infrastructure-safety` | `DOMAIN_DISTRIBUTION_TEMPLATE` | Clean landing zones (`.gitkeep`) + sector safety profiles | Aerospace & Low-Altitude UAS Infrastructure Safety (JARUS SORA v2.5 SAIL I-VI, ASTM F3269-17, ASTM F3411-22a, DO-365B) |
| **Tier 2A** | `DEAP-netcom` | `DOMAIN_DISTRIBUTION_TEMPLATE` | Clean landing zones (`.gitkeep`) + network profile rules | Network & Telecommunications (YANG Network Topology, OpenAPI v3, Protobuf v3, IETF / 3GPP Control-Plane Assurance) |
| **Tier 2A** | `DEAP-surgical-robotics-console` | `DOMAIN_DISTRIBUTION_TEMPLATE` | Clean landing zones (`.gitkeep`) + medical safety profiles | Medical & Healthcare (IEC 62304 Class C, ISO 14971 Risk Management, FDA Class III) |
| **Tier 2A** | `DEAP-space-cubesat-constellation` | `DOMAIN_DISTRIBUTION_TEMPLATE` | Clean landing zones (`.gitkeep`) + space flight profiles | Space & Satellites (ECSS-E-ST-40C, ECSS-Q-ST-80C, NASA-STD-8739.8) |
| **Tier 2A** | `DEAP-industrial-warehouse-agv` | `DOMAIN_DISTRIBUTION_TEMPLATE` | Clean landing zones (`.gitkeep`) + robotics safety profiles | Industrial Robotics & Autonomous Vehicles (ISO 3691-4, IEC 61508, VDA 5050) |
| **Tier 2A** | `DEAP-subsea-oceanographic-auv` | `DOMAIN_DISTRIBUTION_TEMPLATE` | Clean landing zones (`.gitkeep`) + maritime safety profiles | Subsea & Maritime Autonomous Systems (DNV-GL-ST-E403, IMO MASS) |
| **Tier 2A** | `DEAP-rail-autonomous-locomotive` | `DOMAIN_DISTRIBUTION_TEMPLATE` | Clean landing zones (`.gitkeep`) + rail safety profiles | Rail & Transportation Signaling / Traction (EN 50126, EN 50128 SIL 4, EN 50129) |
| **Tier 2B** | `DEAP01-cuas-coa-engine` | `SPECIALIZED_DOMAIN_DEAP_PRODUCT_LINE` | Active domain schema (`schema/cuas_coa.sysml`) + curated specs | Counter-UAS Tactical Planning & C2 (Native Rust Pareto Solver C-ABI + Flutter ARINC 661 Cockpit; MIL-STD-882E, STANAG 4586, MIL-STD-2525D) |
| **Tier 3** | `customer-project-*` | `DOWNSTREAM_CUSTOMER_PROJECT` | Active customer schemas, site data & proprietary source code | Concrete customer/mission deployments on GitHub, GitLab SaaS/Self-Hosted, or SCIF air-gapped enclaves |
```

---

#### Edit 3: Sections 5.3, 5.4, and 5.6 (`README.md` lines 230-308)

**Target:**
```markdown
### 5.3 Propagating Compiler Tooling to Domain Distribution Templates

This workflow is for **DEAP platform maintainers** propagating compiler tooling, governance rules, and verification scripts from `DEAP01-spec-core` to the canonical domain distribution template repositories (`DEAP-*`).

#### Local Propagation
When maintaining a local domain template checkout alongside `DEAP01-spec-core`, build the pure-Rust compiler binaries and install git hooks:
```bash
cargo build --release --workspace
./target/release/verify-baseline setup-hooks
```

#### Remote Bootstrap Propagation
To bootstrap or synchronize a domain distribution template directly from the upstream compiler repository:
```bash
git clone https://github.com/gintatkinson/DEAP01-spec-core.git
cargo build --release --workspace
./target/release/verify-baseline setup-hooks
```

#### In-Place Governance & Tooling Updates
Inside an already-installed domain template repository, update git hooks and run baseline verification:
```bash
./target/release/verify-baseline setup-hooks
./target/release/verify-baseline .
```

---

### 5.4 Architectural Boundary: Customer Project Onboarding

The DEAP three-tier architecture strictly decouples compiler tooling propagation from customer project onboarding:
- **Compiler Propagation (Tier 1 Compiler -> Tier 2 Domain Templates):** Maintainers propagate abstract compiler tooling and governance from `DEAP01-spec-core` to canonical sector distribution templates (`DEAP-*`).
- **Customer Onboarding (Tier 2 Domain Templates -> Tier 3 Customer Workspaces):** Application engineers and customer teams onboard concrete project workspaces exclusively from the appropriate **Domain Distribution Template** (`DEAP-*`), never from `DEAP01-spec-core`.

> **Critical Architecture Rule:** End-user customer projects clone from their designated domain distribution template (e.g. `DEAP-surgical-robotics-console` or `DEAP-industrial-warehouse-agv`), NOT from `DEAP01-spec-core`. Bootstrapping from the domain template seeds the customer workspace with domain-specific SysML v2 schemas, safety statecharts, and platform execution profiles.

```text
+-----------------------------------------------------------------------------------+
| Tier 1: Upstream Specification Core Compiler                                      |
| DEAP01-spec-core (UPSTREAM_SPEC_CORE_COMPILER)                                     |
+-----------------------------------------------------------------------------------+
                                         │
                                         │  cargo build --release && verify-baseline setup-hooks
                                         ▼
+-----------------------------------------------------------------------------------+
| Tier 2: Domain Distribution Templates                                             |
| DEAP-* (DOMAIN_DISTRIBUTION_TEMPLATE)                                              |
| Contains: Domain SysML v2 schemas (schema/), sector blueprints, domain profiles    |
+-----------------------------------------------------------------------------------+
                                         │
                                         │  git clone "<domain-url>" ./.tmp-pipeline && ...
                                         ▼
+-----------------------------------------------------------------------------------+
| Tier 3: Customer Application Workspaces                                           |
| customer-workspace (DOWNSTREAM_CUSTOMER_PROJECT)                                  |
| Contains: Concrete proprietary application code, ROS2 nodes, embedded modules, SIL/HIL tests |
+-----------------------------------------------------------------------------------+
```

For customer onboarding instructions and domain-specific schemas, refer to the documentation in the respective canonical domain distribution template repository (see [Section 4.1](#41-supported-tier-2-domain-distribution-templates-canonical-taxonomies)).

---

### 5.5 Setup for Google Antigravity

Google Antigravity automatically discovers workspace customizations, governance rules, and skills by traversing up to the repository root. Ensure native configuration parity with Claude Code (`CLAUDE.md`) and Cursor (`.cursor/rules/`) by maintaining persistent workspace rules without requiring manual chat steering:

1. **Native Configuration Discovery & Governance Rules**:
   Google Antigravity natively loads project-scoped governance rules from `AGENTS.md` directly into `<user_rules>` upon workspace initialization. Custom pipeline skills located in `.agents/skills/` are automatically indexed and discovered via progressive disclosure on demand. Ad-hoc interactive prompting or manual steering directives to ingest rules are unnecessary and strictly avoided to prevent turn-1 coordinator context bloat and subagent self-rejection gate trips. To read the consolidated active governance rules bundle, agents execute a direct-path read via `view_file` on `.pipeline/ACTIVE_RULES_BUNDLE.md`.
2. **Execute Universal Initialization**:
   Follow the mandatory post-install initialization sequence in [Section 5.6 (Universal Agent Governance & Initialization Sequence)](#56-universal-agent-governance--initialization-sequence-agentsmd).

### 5.6 Universal Agent Governance & Initialization Sequence (AGENTS.md)

When configuring repositories, the governance configuration defines `AGENTS.md` with the appropriate role classifications and governance rules for the target repository tier:
- **Upstream Compiler (`DEAP01-spec-core`):** `UPSTREAM_SPEC_CORE_COMPILER` (abstract MBSE compiler, zero hardcoded domain concepts).
- **Domain Distribution Templates (`DEAP-*`):** `DOMAIN_DISTRIBUTION_TEMPLATE` (domain SysML schemas, sector blueprints).
- **Customer Application Workspaces:** `DOWNSTREAM_CUSTOMER_PROJECT` (proprietary application code, ROS2 nodes, embedded software).
```

**Replacement:**
```markdown
### 5.3 Propagating Precompiled Binary Tooling to Tier 1 & Tier 2 Repositories

This workflow is for **DEAP platform maintainers** propagating precompiled Tier 0 release binaries (`target/release/{verify-baseline,compile-sysml,ingest-sysml,assemble-conops,reconcile-backlog}`), `.pipeline/toolchain-manifest.json`, and synchronized governance bundles (`AGENTS.md`, `.pipeline/constitution.md`, `.pipeline/ACTIVE_RULES_BUNDLE.md`, `rules/*.md`, `skills/**`) from `DEAP01-spec-core` to Tier 1 Platform Profile / Driver Templates (`DEAP-profile-*`, `DEAP-implementation-driver`), Tier 2A Domain Distribution Templates (`DEAP-*`), and Tier 2B Specialized Domain Product-Line Engines (`DEAP01-*-engine`). Tier 0 compiler crate sources (`crates/*`) are never copied downstream (Check 12).

#### Step 1: Build Precompiled Release Binaries in Tier 0 (`DEAP01-spec-core`)
```bash
cargo build --release --workspace
```

#### Step 2: Stage Precompiled Binaries & Verify Downstream Baseline
After staging the 5 `target/release/` binaries, `.pipeline/toolchain-manifest.json`, and synchronized governance files into a Tier 1 or Tier 2 repository checkout, install git hooks and run baseline verification:
```bash
chmod +x ./target/release/verify-baseline ./target/release/compile-sysml ./target/release/ingest-sysml ./target/release/assemble-conops ./target/release/reconcile-backlog
./target/release/verify-baseline setup-hooks .
./target/release/verify-baseline . --no-domain
```

---

### 5.4 Architectural Boundary: Customer Project Onboarding

The DEAP four-tier poly-repo architecture strictly decouples Tier 0 compiler binary propagation from Tier 3 customer project onboarding:
- **Binary Toolchain & Governance Propagation (Tier 0 -> Tier 1 & Tier 2):** Maintainers propagate precompiled native Rust binaries and governance bundles from `DEAP01-spec-core` into Tier 1 Platform Profile / Driver Templates (`DEAP-profile-*`, `DEAP-implementation-driver`), Tier 2A Domain Distribution Templates (`DEAP-*`), and Tier 2B Specialized Domain Product-Line Engines (`DEAP01-*-engine`).
- **Customer Onboarding (Tier 1 / Tier 2 -> Tier 3 Customer Workspaces):** Application engineers and customer teams onboard concrete project workspaces (`customer-project-*`) exclusively from their designated **Tier 1 Platform Template**, **Tier 2A Domain Distribution Template**, or **Tier 2B Specialized Domain Product-Line Engine** -- never directly from `DEAP01-spec-core`.

> **Critical Architecture Rule:** End-user customer projects bootstrap from their designated Tier 1 / Tier 2 template or product-line repository (e.g. `DEAP-implementation-driver`, `DEAP-profile-flutter-app`, `DEAP-avionic-flight-safety`, `DEAP-uas-infrastructure-safety`, `DEAP-netcom`, `DEAP-surgical-robotics-console`, `DEAP01-cuas-coa-engine`), NOT from `DEAP01-spec-core`. Bootstrapping from Tier 1 or Tier 2 seeds the customer workspace with precompiled verification binaries, platform SDK bindings, safety contracts, and execution profiles with zero Tier 0 compiler source leakage.

```text
+-----------------------------------------------------------------------------------+
| Tier 0: Upstream Specification Core Compiler                                      |
| DEAP01-spec-core (UPSTREAM_SPEC_CORE_COMPILER)                                    |
+-----------------------------------------------------------------------------------+
                                         |
                                         |  cargo build --release --workspace (Tier 0)
                                         |  Distribute 5 target/release/* binaries + governance
                                         v
+-----------------------------------------------------------------------------------+
| Tier 1: Platform Profile & Driver Templates                                       |
| DEAP-profile-flutter-app, DEAP-profile-react-web, DEAP-profile-backend-api,       |
| DEAP-profile-vhdl-hardware, DEAP-implementation-driver (+ future DEAP-profile-*)  |
+-----------------------------------------------------------------------------------+
                                         |
                                         |  Platform SDKs, Execution Profiles & Tooling
                                         v
+-----------------------------------------------------------------------------------+
| Tier 2: Domain Templates (DEAP-*) & Product-Line Engines (DEAP01-*-engine)        |
| Tier 2A: DEAP-avionic-flight-safety, DEAP-uas-infrastructure-safety, DEAP-netcom, |
|          DEAP-surgical-robotics-console, DEAP-space-cubesat-constellation,        |
|          DEAP-industrial-warehouse-agv, DEAP-subsea-oceanographic-auv,            |
|          DEAP-rail-autonomous-locomotive (+ future DEAP-* sector templates)       |
| Tier 2B: DEAP01-cuas-coa-engine (+ future DEAP01-*-engine product lines)          |
+-----------------------------------------------------------------------------------+
                                         |
                                         |  gh repo create / git clone template onboarding
                                         v
+-----------------------------------------------------------------------------------+
| Tier 3: Customer Application Workspaces                                           |
| customer-project-* (DOWNSTREAM_CUSTOMER_PROJECT)                                  |
| Contains: Proprietary mission schemas, site data, app/embedded code, SIL/HIL tests|
+-----------------------------------------------------------------------------------+
```

For customer onboarding instructions, platform shells, and domain schemas, refer to the documentation in the respective Tier 1 or Tier 2 repository (see [Section 4.1](#41-supported-multi-tier-deap-repository-catalog-canonical-taxonomies)).

---

### 5.5 Setup for Google Antigravity

Google Antigravity automatically discovers workspace customizations, governance rules, and skills by traversing up to the repository root. Ensure native configuration parity with Claude Code (`CLAUDE.md`) and Cursor (`.cursor/rules/`) by maintaining persistent workspace rules without requiring manual chat steering:

1. **Native Configuration Discovery & Governance Rules**:
   Google Antigravity natively loads project-scoped governance rules from `AGENTS.md` directly into `<user_rules>` upon workspace initialization. Custom pipeline skills located in `.agents/skills/` are automatically indexed and discovered via progressive disclosure on demand. Ad-hoc interactive prompting or manual steering directives to ingest rules are unnecessary and strictly avoided to prevent turn-1 coordinator context bloat and subagent self-rejection gate trips. To read the consolidated active governance rules bundle, agents execute a direct-path read via `view_file` on `.pipeline/ACTIVE_RULES_BUNDLE.md`.
2. **Execute Universal Initialization**:
   Follow the mandatory post-install initialization sequence in [Section 5.6 (Universal Agent Governance & Initialization Sequence)](#56-universal-agent-governance--initialization-sequence-agentsmd).

### 5.6 Universal Agent Governance & Initialization Sequence (AGENTS.md)

When configuring repositories, the governance configuration defines `AGENTS.md` with the appropriate role classifications and governance rules for the target repository tier:
- **Tier 0 Upstream Compiler (`DEAP01-spec-core`):** `UPSTREAM_SPEC_CORE_COMPILER` (abstract MBSE compiler, 6 workspace crates, zero hardcoded domain concepts).
- **Tier 1 Platform Profile & Driver Templates (`DEAP-profile-*`, `DEAP-implementation-driver`):** `UPSTREAM_PLATFORM_TEMPLATE` (shared platform shells/SDKs, binary-only toolchain, clean landing zones).
- **Tier 2A Domain Distribution Templates (`DEAP-*`):** `DOMAIN_DISTRIBUTION_TEMPLATE` (sector safety frameworks, execution profiles, binary-only toolchain, clean landing zones).
- **Tier 2B Specialized Domain Product-Line Engines (`DEAP01-*-engine`):** `SPECIALIZED_DOMAIN_DEAP_PRODUCT_LINE` (reusable active-schema domain engines, Rust C-ABI solvers, and platform UI cockpits, zero customer site data).
- **Tier 3 Customer Application Workspaces (`customer-project-*`):** `DOWNSTREAM_CUSTOMER_PROJECT` (proprietary customer schemas, site-specific operational data, ROS2/PX4/Flutter/React/VHDL implementations, SIL/HIL tests).
```

---

## 3. Execution & Verification Sequence (Post-`PROCEED`)

1. **Single-Item Surgical Edit Subagent Dispatch**:
   - Dispatch a fresh context-isolated subagent targeting **only** `README.md` via the Surgical Edit Fast-Path to apply Edits 1, 2, and 3 verbatim, then terminate/reclaim the subagent immediately.
2. **Tier 1 Path-Scoped Baseline Verification Gate**:
   - Because only `README.md` (Tier 1 documentation Markdown) is modified, run **zero** `cargo build` or `cargo test` commands per the 3-Tier Path-Scoped Incremental Build & Test Gate, and execute `./target/release/verify-baseline . --no-domain` to validate all baseline checks (hygiene, Check 12 blueprint isolation, Check 13 KaTeX/Mermaid syntax, Check 14 README completeness, Check 32/33 zero shell/Python references, Check 37 blueprint grounding).
3. **Commit & Remote Synchronization**:
   - Stage `README.md` (and `implementation_plan.md` if tracked cleanly with the atomic plan), commit using Conventional Commits (`docs(readme): update 4-tier repository catalog with all platform, domain, and product-line repos`), push to `origin/main`, and verify `git diff origin/main` is empty.
