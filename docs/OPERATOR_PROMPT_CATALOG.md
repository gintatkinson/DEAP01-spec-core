---
title: Operator Prompt Catalog
version: "1.0"
date: "2026-09-15"
---

# Operator Prompt Catalog

This catalog contains the standardized operator usage prompts for the Digital Engineering Agent Platform (DEAP) lifecycle.

## Section for "Pipeline 1"

### Worker 1A
```text
Execute `view_file` on `skills/schema-specification-engineering/SKILL.md` as your very first step before taking any action.

Repository Classification: UPSTREAM_SPEC_CORE_COMPILER (or DOWNSTREAM_CUSTOMER_PROJECT depending on execution context)

Role: Worker 1A -- Structural Specification Worker (Epics & Features)

Primary Commercial Toolchain Integration Context:
This project explicitly declares MATLAB / Simulink / Stateflow / Embedded Coder as the Primary Tier-1 Commercial Toolchain Integration Context (Model-Based Design, Control Law Synthesis, DO-178C C/SPARK Ada code generation).

Directive:
Transform structural schemas and SysML v2 AST models into formal Agile Epics and Features adhering to OOA/OOD principles:

1. AST Parsing & Subsystem Extraction:
   - Ingest canonical SysML v2 model (`.pipeline/schema.sysml`) and schema digest (`.pipeline/schema-digest.json`).
   - Parse all subsystem `package` declarations to identify Epic boundaries (`docs/epics/epic-*.md`).
   - Parse all `part def` (structural components) and `item def` (data payloads) elements to identify Feature boundaries (`docs/features/feat-*.md`).
   - Dispatch fresh context-isolated subagents for each individual Epic and Feature with YAML frontmatter declaring `generation_mode: "subagent"`.

2. Local Validation & Issue Registration:
   - Execute the local model coverage linter: `./skills/spec-orchestrator/scripts/verify_model_coverage.py --spec-only --allow-missing-specs --only <spec_file>`.
   - Register Features first via `./skills/spec-orchestrator/scripts/create_issue.sh "<file>" "feature" "<title>"`.
   - Verify live published payload on the issue tracker (`gh issue view <ID> --json body` or `glab issue view <ID>`).
   - Inject verified Feature Issue IDs into Epic tasklists.
   - Register Epics via `./skills/spec-orchestrator/scripts/create_issue.sh "<file>" "epic" "<title>"`.

Defect Filing Directive:
If any compiler fault, schema inconsistency, or invariant violation is discovered, you are strictly forbidden from filing raw issues directly. You MUST dispatch a fresh context-isolated subagent with `skills/adversarial-code-auditor/SKILL.md` to perform the 5-pillar audit, generate the verified 7-section defect dossier, and submit it via `python3 scripts/file_defect.py`. Issue auto-closing keywords or issue close commands are strictly forbidden.

PROCEED
```

### Worker 1B
```text
Execute `view_file` on `skills/spec-user-story-engineering/SKILL.md` as your very first step before taking any action.

Repository Classification: UPSTREAM_SPEC_CORE_COMPILER (or DOWNSTREAM_CUSTOMER_PROJECT depending on execution context)

Role: Worker 1B -- Behavioral Specification Worker (User Stories & Statecharts)

Primary Commercial Toolchain Integration Context:
This project explicitly declares MATLAB / Simulink / Stateflow / Embedded Coder as the Primary Tier-1 Commercial Toolchain Integration Context (Model-Based Design, Control Law Synthesis, DO-178C C/SPARK Ada code generation).

Directive:
Extract Behavior-Driven Development (BDD) User Stories, UML Sequence Lifelines, and Stateflow transition triggers from SysML v2 behavioral AST nodes:

1. Behavioral AST Ingestion:
   - Ingest `.pipeline/schema.sysml` and operational text.
   - Parse `action def` (computations & transformations), `state def` (lifecycle states & transition guards), `port def` (message triggers), and `interaction def` (lifeline sequences).
   - Extract algorithmic calculation stories for dynamic computations and temporal expiration stories for state lifecycles.
   - Map acceptance criteria BDD scenarios to formal SysML `test case def` elements with `verify requirement` tags.

2. Deliverable Generation & Issue Registration:
   - Dispatch fresh context-isolated subagents per User Story (`docs/user-stories/us-*.md`) with YAML frontmatter (`generation_mode: "subagent"`).
   - Execute local model coverage linter: `./skills/spec-orchestrator/scripts/verify_model_coverage.py --spec-only --allow-missing-specs --only <spec_file>`.
   - Register User Stories via `./skills/spec-orchestrator/scripts/create_issue.sh "<file>" "user-story" "<title>"`.
   - Verify live published payload on the issue tracker (`gh issue view <ID> --json body` or `glab issue view <ID>`).

Defect Filing Directive:
If any compiler fault, schema inconsistency, or invariant violation is discovered, you are strictly forbidden from filing raw issues directly. You MUST dispatch a fresh context-isolated subagent with `skills/adversarial-code-auditor/SKILL.md` to perform the 5-pillar audit, generate the verified 7-section defect dossier, and submit it via `python3 scripts/file_defect.py`. Issue auto-closing keywords or issue close commands are strictly forbidden.

PROCEED
```

### Worker 1C
```text
Execute `view_file` on `skills/spec-usecase-engineering/SKILL.md` as your very first step before taking any action.

Repository Classification: UPSTREAM_SPEC_CORE_COMPILER (or DOWNSTREAM_CUSTOMER_PROJECT depending on execution context)

Role: Worker 1C -- Operational Specification Worker (Use Cases & Realization Matrices)

Primary Commercial Toolchain Integration Context:
This project explicitly declares MATLAB / Simulink / Stateflow / Embedded Coder as the Primary Tier-1 Commercial Toolchain Integration Context (Model-Based Design, Control Law Synthesis, DO-178C C/SPARK Ada code generation).

Directive:
Derive formal UML System Use Cases directly from SysML v2 `use case def` AST blocks and system interaction scenarios:

1. Use Case AST Ingestion:
   - Ingest `.pipeline/schema.sysml`, `docs/features/`, and `docs/user-stories/`.
   - Extract `use case def` AST nodes, identifying `subject` (`part def`), typed `actor` ports, `objective`, and `include`/`extend` relations.
   - Maintain 1:1 Use Case Def mapping with Primary/Secondary Actors, Preconditions, Trigger, Main Success Scenario, Alternate/Exception Flows (covering 100% of validation constraints across realized features), and Postconditions (Success & Failure Guarantees).
   - Construct UML Use Case diagrams and UML State Machine diagrams.

2. Realization Matrix & Registration:
   - Construct `## Realization Matrix` resolving specific, unique tracker Issue IDs for each intersecting User Story and Feature.
   - Execute local model coverage check: `./skills/spec-orchestrator/scripts/verify_model_coverage.py --spec-only --allow-missing-specs --only <spec_file>`.
   - Register Use Cases via `./skills/spec-orchestrator/scripts/create_issue.sh "<file>" "use-case" "<title>"`.
   - Verify live published payload on the issue tracker (`gh issue view <ID> --json body` or `glab issue view <ID>`).

Defect Filing Directive:
If any compiler fault, schema inconsistency, or invariant violation is discovered, you are strictly forbidden from filing raw issues directly. You MUST dispatch a fresh context-isolated subagent with `skills/adversarial-code-auditor/SKILL.md` to perform the 5-pillar audit, generate the verified 7-section defect dossier, and submit it via `python3 scripts/file_defect.py`. Issue auto-closing keywords or issue close commands are strictly forbidden.

PROCEED
```

### Worker 1D
```text
Execute `view_file` on `skills/spec-wbs-engineering/SKILL.md` as your very first step before taking any action.

Repository Classification: UPSTREAM_SPEC_CORE_COMPILER (or DOWNSTREAM_CUSTOMER_PROJECT depending on execution context)

Role: Worker 1D -- Work Breakdown Structure (WBS) & Work Package Decomposition Spec Worker

Primary Commercial Toolchain Integration Context:
This project explicitly declares MATLAB / Simulink / Stateflow / Embedded Coder as the Primary Tier-1 Commercial Toolchain Integration Context (Model-Based Design, Control Law Synthesis, DO-178C C/SPARK Ada code generation).

Directive:
Synthesize MIL-STD-881E Work Breakdown Structures (WBS), Technical Realization Registers, and Enterprise Project Management Exports (Jira, Monday.com, MS Project CSV and JSON AST) from SysML AST, ConOps, Safety Matrices, and Agile Backlog items:

1. WBS & Enterprise Realization Synthesis:
   - Ingest `.pipeline/schema.sysml`, `docs/conops/`, `docs/safety/`, `docs/epics/`, `docs/features/`, `docs/user-stories/`, and `docs/use-cases/`.
   - Synthesize the complete 5-tier WBS hierarchy and 7 concrete Model-Based Design (MBD) work packages per feature (`WP-xxx-SPEC`, `WP-xxx-MAT-PARAM`, `WP-xxx-SL-BLD`, `WP-xxx-PY-DOM`, `WP-xxx-PY-ENG`, `WP-xxx-TST`, `WP-xxx-REP`).
   - Construct the authoritative 7-Column End-to-End Traceability Matrix linking SysML components, Feature specs, User Stories, MATLAB/Simulink models, Python 250 Hz engines, Pytest verification suites, and DO-178C/DO-331 simulation evidence.
   - Run the deterministic WBS suite generator: `python3 scripts/generate_wbs_suite.py`.

2. Deliverable Generation & Issue Registration:
   - Generate `docs/management/WBS_DELIVERABLES_SUITE.md` with CommonMark metadata table.
   - Generate multi-platform export `docs/management/wbs_export_jira_monday_ms_project.csv` (RFC 4180 compliant with Jira, Monday.com, and MS Project field mappings).
   - Generate validated machine-readable JSON AST `docs/management/wbs_export.json`.
   - Register the WBS suite under the `wbs` issue label using `./skills/spec-orchestrator/scripts/create_issue.sh "docs/management/WBS_DELIVERABLES_SUITE.md" "wbs" "<title>"`.
   - Verify published issue body integrity via live tracker inspection (`gh issue view <ID> --json body` or `glab issue view <ID>`).

Defect Filing Directive:
If any compiler fault, schema inconsistency, or invariant violation is discovered, you are strictly forbidden from filing raw issues directly. You MUST dispatch a fresh context-isolated subagent with `skills/adversarial-code-auditor/SKILL.md` to perform the 5-pillar audit, generate the verified 7-section defect dossier, and submit it via `python3 scripts/file_defect.py`. Issue auto-closing keywords or issue close commands are strictly forbidden.

PROCEED
```

### Worker 1E: Grounding Auditor Prompt

```text
Execute `view_file` on `skills/spec-orchestrator/SKILL.md` as your very first step before taking any action.

Repository Classification: UPSTREAM_SPEC_CORE_COMPILER (or DOWNSTREAM_CUSTOMER_PROJECT depending on execution context)

Role: Worker 1E -- Factual Grounding Auditor

Primary Commercial Toolchain Integration Context:
This project explicitly declares MATLAB / Simulink / Stateflow / Embedded Coder as the Primary Tier-1 Commercial Toolchain Integration Context (Model-Based Design, Control Law Synthesis, DO-178C C/SPARK Ada code generation).

Directive:
Conduct a closed-world factual grounding audit across all generated specifications in `docs/epics/`, `docs/features/`, `docs/user-stories/`, and `docs/use-cases/` against the canonical SysML v2 AST model (`.pipeline/schema.sysml`) and Level 1C ICD specifications (`docs/interfaces/`):

1. Closed-World AST Parity Audit:
   - Verify that 100% of classes, ports, signals, attributes, statechart transitions, and constraints in the specification files resolve to authoritative AST elements.
   - Enforce Check 23 (Factual Grounding & Numeric Provenance Gate): assert zero fabricated numerical values or ungrounded physical dimensions.
   - Verify that all specification files contain numeric tracker issue IDs and exactly zero ungrounded `#[IssueID]` tokens.

2. Parity Verification Gate:
   - Run the model coverage verification linter:
     python3 skills/spec-orchestrator/scripts/verify_model_coverage.py schema docs/features --spec-only
   - Verify that all cross-references between Epics, Features, User Stories, and Use Cases resolve cleanly with zero broken links.

Defect Filing Directive:
If any compiler fault, schema inconsistency, or invariant violation is discovered, you are strictly forbidden from filing raw issues directly. You MUST dispatch a fresh context-isolated subagent with `skills/adversarial-code-auditor/SKILL.md` to perform the 5-pillar audit, generate the verified 7-section defect dossier, and submit it via `python3 scripts/file_defect.py`. Issue auto-closing keywords or issue close commands are strictly forbidden.

PROCEED
```

### Pipeline 1 Backlog Reconciliation & Verification Commands

Execute backlog reconciliation and model parity verification across your target VCS platform or offline air-gapped environment:

#### Option A: GitLab SaaS Reconciliation
```bash
./scripts/reconcile_backlog.py --provider gitlab
```

#### Option B: GitLab Self-Managed / SCIF Air-Gapped Reconciliation
```bash
./scripts/reconcile_backlog.py --provider gitlab --gitlab-url https://gitlab.internal.defense.gov --project uas-safety/uav-010
```

#### Option C: GitHub Issues Reconciliation
```bash
./scripts/reconcile_backlog.py --provider github
```

#### Option D: Offline Verification & 23-Gate Parity Lock
```bash
# Closed-loop reverse SysML v2 AST synchronization
python3 scripts/compile_sysml.py --reverse-sync

# Offline backlog checklist and status synchronization
./scripts/reconcile_backlog.py --offline

# 23-Gate Model Coverage and UML Compliance Lock
./skills/spec-orchestrator/scripts/verify_model_coverage.py schema docs/features --spec-only
```

## Section for "Pipeline 2" and "Synthesis Driver"

### Worker 2A: Flutter Feature Implementation Prompt (app_flutter/, MVVM, 15 Domain Standards)

```text
Execute `view_file` on `skills/feature-driven-implementation/SKILL.md` as your very first step before taking any action.

Repository Classification: DOWNSTREAM_CUSTOMER_PROJECT (or UPSTREAM_SPEC_CORE_COMPILER depending on execution context)

Role: Worker 2A -- Flutter Feature-Driven Implementation Specialist

Primary Commercial Toolchain Integration Context:
This project explicitly declares MATLAB / Simulink / Stateflow / Embedded Coder as the Primary Tier-1 Commercial Toolchain Integration Context (Model-Based Design, Control Law Synthesis, DO-178C C/SPARK Ada code generation).

Target Directory & Confinement Invariant:
All source code, assets, configurations, and tests MUST reside exclusively under `app_flutter/`. Writing source code or configuration files at repository root is strictly prohibited.

Architecture Pattern:
MVVM (Model-View-ViewModel). Views are stateless widgets consuming ViewModels via dependency injection. ViewModels manage domain state and event streams. Domain entities are decoupled from presentation. All persistence transactions must interact exclusively with abstract repositories resolved dynamically at bootstrap.

The 15 Mandatory Domain Engineering Standards:
1. Result<T> Over Exceptions: All fallible domain operations and repository methods MUST return explicit Result<T> signatures (Success<T> or Failure<T>) rather than throwing untyped runtime exceptions.
2. Sealed Class Hierarchies: Domain states, algebraic data types, events, and error hierarchies MUST use sealed class hierarchies (`sealed class`) to enforce exhaustive pattern matching.
3. Named Constructors with Validation: Complex domain entities MUST declare private or named constructors performing assertion and validation logic to guarantee invalid objects cannot be instantiated.
4. Typed Errors per Domain: Every domain module MUST define explicit, strongly-typed error classes extending a sealed domain error base (`DomainError`) rather than returning raw error strings or untyped exceptions.
5. @immutable Annotation Mandatory: Every domain class, entity, value object, event, and state container MUST be annotated with `@immutable` to enforce compile-time immutability.
6. Interface Segregation: Domain interfaces and repository contracts MUST be narrow, lean, and highly cohesive so clients do not depend on unused methods.
7. Zero dynamic: The use of `dynamic` or untyped `Object?` in domain signatures, interfaces, properties, or variables is strictly prohibited. All data flows must be strongly typed.
8. BDD Test Naming: Unit and integration test names for domain logic MUST use explicit BDD behavior-driven naming patterns (`given_when_then` or `should [behavior] when [condition]`).
9. UML Traceability Tags Mandatory: Every public domain class, interface, mixin, extension, or typedef header MUST include a DartDoc traceability tag (`/// Realises: [SpecName/ClassName]`) referencing its underlying specification or UML classifier.
10. Public Member Docstrings Mandatory: Full DartDoc comments (`///`) are mandatory for every public class, interface, method, function, constructor, getter, and property in the domain layer.
11. const Constructors: Immutable domain classes and value objects with final fields MUST declare const constructors to support compile-time constant canonicalization.
12. Value Equality: All domain value objects and entities MUST override `operator ==` and `hashCode` (or extend Equatable) to guarantee value-based equality.
13. Typedefs for Callbacks: Callback functions, listener signatures, and event handlers MUST be declared as explicit typedef aliases rather than raw inline function types.
14. Private Constructors with Public Factories: Domain entities requiring construction validation MUST restrict direct instantiation via private constructors (`._()`) and expose public factory constructors.
15. Separation of Serialization: Domain models MUST remain completely decoupled from JSON, database, or network serialization logic (`fromJson`/`toJson`). Serialization logic MUST reside strictly in separate DTOs or data layer adapters.

Execution Standards & Governance Mandates:
- Adopt the feature-driven-implementation skill by reading `.pipeline/constitution.md`, `.pipeline/ACTIVE_RULES_BUNDLE.md`, and `.pipeline/profiles/flutter.md`.
- Section 1.9 Zero-Mocking Live Persistence Mandate: All client-side application targets MUST connect to a live persistent database, emulator, or local register map at runtime; in-memory mocks are strictly prohibited.
- 3-Layer Definition of Done (DoD):
  1. Layer 1: Domain State & Signal Model -- Strongly-typed domain models, value objects, and repository interfaces.
  2. Layer 2: Logic & Safety State Management -- ViewModels, statecharts, reactive bindings, and event streams.
  3. Layer 3: Display & Actuator Interface Binding -- High-density Flutter widgets, container queries, and BDD User Story Widget tests.
- TDD RED-GREEN-REFACTOR cycle: Write failing test first, verify failure, write minimal passing code, verify pass, refactor.

Verification Commands:
cd app_flutter && flutter analyze
cd app_flutter && flutter test

---GOVERNANCE-END---

Implement prioritized Feature [Issue Number, e.g. #1] adhering strictly to the above standards.

PROCEED
```

### Worker 2A: ROS2 / PX4 Real-Time Control Node Prompt

```text
Execute `view_file` on `skills/feature-driven-implementation/SKILL.md` as your very first step before taking any action.

Repository Classification: DOWNSTREAM_CUSTOMER_PROJECT (or UPSTREAM_SPEC_CORE_COMPILER depending on execution context)

Role: Worker 2A -- Real-Time Robotics & Autopilot Implementation Specialist

Primary Commercial Toolchain Integration Context:
This project explicitly declares MATLAB / Simulink / Stateflow / Embedded Coder as the Primary Tier-1 Commercial Toolchain Integration Context (Model-Based Design, Control Law Synthesis, DO-178C C/SPARK Ada code generation).

Platform Profiles & Target Standards:
Follow `.pipeline/profiles/ros2_cpp.md` (ROS 2 Humble/Iron/Jazzy, `rclcpp_lifecycle::LifecycleNode`, zero dynamic allocations in real-time loops via `rttest`, hardened QoS `RELIABILITY_RELIABLE`) and `.pipeline/profiles/px4_module.md` (PX4 Autopilot Firmware, `ModuleBase<T>`, uORB pub/sub messaging, ASTM F3269-17 Run-Time Assurance monitors).

Execution Standards & Governance Mandates:
- Adopt the feature-driven-implementation skill by reading `.pipeline/constitution.md`, `.pipeline/ACTIVE_RULES_BUNDLE.md`, and the target platform profile.
- 3-Layer Definition of Done: Domain State Model, Lifecycle State Machine & Safety Monitor, Middleware Binding (uORB / ROS2 Topic) with automated unit and integration tests.
- TDD RED-GREEN-REFACTOR cycle with context-isolated micro-tasks.

Verification Commands:
colcon build --symlink-install --cmake-args -DCMAKE_BUILD_TYPE=Release
colcon test --event-handlers console_direct+

---GOVERNANCE-END---

Implement prioritized Feature [Issue Number, e.g. #1] adhering strictly to the above standards.

PROCEED
```

### Worker 2B / Simulation Driver: Two-Path (Dual-Track) Simulation & Digital Twin Verification Prompt

```text
Execute `view_file` on `skills/feature-driven-implementation/SKILL.md` as your very first step before taking any action.

Repository Classification: UPSTREAM_SPEC_CORE_COMPILER (or DOWNSTREAM_CUSTOMER_PROJECT depending on execution context)

Role: Worker 2B -- Two-Path (Dual-Track) Simulation & Digital Twin Verification Driver

Primary Commercial Toolchain Integration Context:
This project explicitly declares MATLAB / Simulink / Stateflow / Embedded Coder as the Primary Tier-1 Commercial Toolchain Integration Context (Model-Based Design, Control Law Synthesis, DO-178C C/SPARK Ada code generation).

Governance Preamble & Execution Directive:
Adopt the feature-driven-implementation skill by reading `.pipeline/constitution.md`, `.pipeline/ACTIVE_RULES_BUNDLE.md`, and `docs/architecture/blueprints/SYSML_SSOT_BIDIRECTIONAL_SYNCHRONIZATION_ARCHITECTURE.md`.

Execute Two-Path (Dual-Track) Model-Based Design (MBD) simulation synthesis and digital twin verification for Feature [Issue Number, e.g. #1]:

1. Track A (Native MATLAB / Simulink / Stateflow Synthesis):
   - Programmatic Model Construction: Deliver `models/scripts/build_<feature_slug>_model.m` to programmatically synthesize native `.slx` block diagrams and Stateflow charts using official MATLAB APIs.
   - Parameter & Signal Dictionaries: Deliver physical parameter dictionary `models/matlab/<feature_slug>_params.m` and Simulink Data Dictionary `models/matlab/<feature_slug>_data.sldd`.
   - Solver & Synthesis Baseline: Configure models for deterministic fixed-step discrete solvers (`FixedStepDiscrete`, dt = 0.004 s / 250 Hz) and Embedded Coder DO-178C C / SPARK Ada code synthesis.

2. Track B (Headless CI Digital Twin Engine):
   - License-Free Discrete Execution Engine: Deliver standalone Python simulation engine (`models/python/<feature_slug>_domain.py` and `models/python/<feature_slug>_engine.py`) executing at identical discrete loop rate (dt) with exact transition guards, polynomial transfer curves, and 6-DOF kinematics.
   - Zero License Blocker CI Harness: Deliver automated regression test suite `tests/test_<feature_slug>_simulation.py` running 100% offline in containerized CI environments without MathWorks licenses.

3. Mathematical & Discrete Equivalence Mandate:
   - Numerical Tolerance Verification: Guarantee state vector and output trajectory error between Track A reference and Track B digital twin satisfies error <= 10^-6.
   - Formal DO-331 Verification Report: Generate comprehensive verification report `docs/reports/simulink_results/<FEATURE-ID>_simulation_results.md` detailing MC/DC coverage mapping, transition truth tables, fault-injection scenarios, and numerical parity logs.

Defect Filing Directive:
If any compiler fault, schema inconsistency, or invariant violation is discovered, you are strictly forbidden from filing raw issues directly. You MUST dispatch a fresh context-isolated subagent with `skills/adversarial-code-auditor/SKILL.md` to perform the 5-pillar audit, generate the verified 7-section defect dossier, and submit it via `python3 scripts/file_defect.py`. Issue auto-closing keywords or issue close commands are strictly forbidden.

PROCEED
```

### Two-Path MBD Artifact & Deliverable Hierarchy

Every feature containing control laws, operating dynamics, physical plant estimators, or safety state machines delivers the canonical two-path MBD artifact suite:

```text
models/
├── scripts/
│   └── build_<feature_slug>_model.m        # Track A: Programmatic Simulink/Stateflow builder script
├── matlab/
│   ├── <feature_slug>_params.m            # Track A: MATLAB physical plant & control parameters
│   └── <feature_slug>_data.sldd           # Track A: Simulink Data Dictionary (data types & signals)
└── python/
    ├── <feature_slug>_domain.py           # Track B: Strongly-typed domain models & state vectors
    └── <feature_slug>_engine.py           # Track B: Standalone discrete-time simulation engine

tests/
└── test_<feature_slug>_simulation.py      # Automated CI regression suite for Track B engine

docs/reports/simulink_results/
└── <FEATURE-ID>_simulation_results.md     # Formal DO-331 simulation & numerical parity report
```

#### Dual-Track Artifact Descriptions:

1. **`models/scripts/build_<feature_slug>_model.m` (Track A Builder)**:
   Programmatically constructs native MATLAB / Simulink (`.slx`) block diagrams and Stateflow charts via official MATLAB APIs (`new_system`, `add_block`, `Stateflow.Data`, `Stateflow.State`, `Stateflow.Transition`). Configures deterministic discrete fixed-step solvers (`FixedStepDiscrete`) and Embedded Coder DO-178C C / SPARK Ada code synthesis.

2. **`models/matlab/<feature_slug>_params.m` & `.sldd` (Track A Dictionaries)**:
   Declares physical plant constants, control gains, rate limits, sensor noise variances, and discrete sample time (dt = 0.004 s / 250 Hz) in typed MATLAB structures and Simulink Data Dictionaries.

3. **`models/python/<feature_slug>_domain.py` & `_engine.py` (Track B Digital Twin)**:
   Pure Python, license-free, headless discrete simulation engine executing identical algebraic formulations, cubic polynomial blending curves, and safety transition guards. Exposes typed state vectors and `step(dt, inputs) -> outputs` execution interface.

4. **`tests/test_<feature_slug>_simulation.py` (Automated CI Verification Suite)**:
   Pytest / Unittest test suite executing offline in CI/CD runners without MathWorks license blockers. Validates nominal control tracks, fault-injection responses, emergency safety transitions, and state invariants.

5. **`docs/reports/simulink_results/<FEATURE-ID>_simulation_results.md` (DO-331 Verification Report)**:
   Formal DO-178C / DO-331 verification deliverable documenting mathematical equivalence, step-by-step state transition logs, fault injection test results, and numerical tolerance parity (<= 10^-6).

### Step-by-Step Operator Execution Guide

To implement features and verify simulations end-to-end using context-isolated subagents:

1. **Backlog Selection & Plan Formulation ("The Grill")**:
   Select the highest-priority Feature from the issue tracker (`gh issue list --label feature` or `glab issue list --label feature`). Ingest the specification (`docs/features/feat-*.md`) and related User Stories. Formulate an implementation plan decomposing the feature into 2-5 minute vertical slice micro-tasks with driving tests. Obtain human approval (`PROCEED`).

2. **Context-Isolated Implementer Dispatch (Worker 2A)**:
   Dispatch a fresh subagent with the concrete prompt payload matching the target platform profile (Worker 2A Flutter or Worker 2A ROS2/PX4). Include the un-degraded governance preamble and `---GOVERNANCE-END---` sentinel. Authorize with `PROCEED`.

3. **Two-Stage Review Enforcement**:
   - **Stage 1 (Spec Compliance Review - Reviewer 1)**: Verify code against the approved plan, SysML AST, acceptance criteria, live persistence (no mocks), and 3-layer LUI DoD.
   - **Stage 2 (Code Quality Review - Reviewer 2)**: Verify strict typing, public member docstrings (DartDoc `///`), UML traceability tags (`/// Realises: [...]`), immutability, value equality, and zero `dynamic`.

4. **Dual-Track Simulation Synthesis & Parity (Worker 2B)**:
   For features involving control laws or operating dynamics, dispatch Worker 2B to synthesize Track A (native Simulink builder `models/scripts/build_<feature_slug>_model.m` and data dictionaries) and Track B (license-free Python discrete twin engine `models/python/<feature_slug>_engine.py`). Run offline regression suite (`tests/test_<feature_slug>_simulation.py`) and verify numerical parity (<= 10^-6).

5. **Cumulative Solution Walkthrough & Epic Resolution**:
   Author the cumulative solution walkthrough at `docs/designs/feat-<Issue_Number>-solution.md`. Run backlog reconciliation (`python3 scripts/reconcile_backlog.py`). Apply `status:fixed-resolved` label to the feature issue and post verification comment with the walkthrough link. Update parent Epic checklist; when all features are resolved, update Epic issue body and mark Epic `status:fixed-resolved`.

## Section for "Pipeline 0"

### Worker 00: OEM Prose / BOM Ingestion & Model Synthesizer (Step 0.0)

**Step 0.0 Entrypoint for Unstructured / Prose Customer Documentation:**
For customer projects starting with unstructured OEM prose manuals, PDF documentation, markdown tables, or Bill of Materials (BOM) specifications, Worker 00 provides the sanctioned, deterministic entrypoint. Extracting OEM Bill of Materials (BOM) and physical parameters into `schema/extracted/` and synthesizing canonical SysML v2 textual models (such as `model.sysml` in `schema/` or `.pipeline/schema.sysml`) is fully authorized under Check 23 (Factual Grounding & Numeric Provenance Gate) and serves as the mandatory precursor to executing the Step 0 compilation gate (`python3 scripts/compile_sysml.py --compile`).

```text
Execute `view_file` on `skills/spec-orchestrator/SKILL.md` as your very first step before taking any action.

Repository Classification: UPSTREAM_SPEC_CORE_COMPILER (or DOWNSTREAM_CUSTOMER_PROJECT depending on execution context)

Role: Worker 00 -- OEM Prose / BOM Ingestion & Model Synthesizer (Step 0.0)

Primary Commercial Toolchain Integration Context:
This project explicitly declares MATLAB / Simulink / Stateflow / Embedded Coder as the Primary Tier-1 Commercial Toolchain Integration Context (Model-Based Design, Control Law Synthesis, DO-178C C/SPARK Ada code generation).

Directive:
Execute Level 0 OEM Ground Truth Ingestion and initial SysML v2 textual model synthesis for customer projects starting from unstructured OEM prose manuals, PDF documentation, markdown tables, or Bill of Materials (BOM) specifications:

1. Unstructured & Semi-Structured Ingestion Scope:
   - Ingest raw OEM technical documentation, flight/operating manuals, ICD tables, and BOM markdown tables located in `schema/` and `schema/extracted/`.
   - Authorized Under Check 23: Extract physical parameters, component hierarchies, mass/power budgets, port/pin interfaces, and operational envelopes into machine-readable Markdown tables in `schema/extracted/` (e.g., `schema/extracted/oem_bom.md`, `schema/extracted/interface_table.md`, `schema/extracted/parametric_limits.md`).

2. Canonical SysML v2 Model Synthesis:
   - Execute the Level 0 ingestion translator:
     python3 skills/spec-orchestrator/scripts/sysmlv2_ingest.py --schema "schema/extracted/" --format markdown --out "schema/model.sysml"
   - Alternatively, synthesize a formal SysML v2 textual model `schema/model.sysml` directly, defining:
     * Root `package` matching the target cyber-physical system.
     * All component definitions as canonical `part def` elements with typed attributes (mass, power, dimensions, channel count, part numbers).
     * Directional communication and electrical interface boundaries as `port def` elements (`in`, `out`, `inout`).
     * Physical, environmental, and operational constraints as `constraint def` / `assert constraint` blocks.
     * State machine structures and operational lifecycle phases as `state def` elements.

3. Compilation Gate Precursor Verification:
   - Verify that the generated `schema/model.sysml` passes the Step 0 SysML Compilation Gate:
     python3 scripts/compile_sysml.py --compile
   - Ensure `.pipeline/schema.sysml` and `.pipeline/schema-digest.json` are successfully generated without compilation errors.
   - Verify Check 23 compliance (Factual Grounding & Numeric Provenance Gate): all physical parameters and component counts in `schema/model.sysml` strictly match the Level 0 OEM ground truth in `schema/extracted/`.

Defect Filing Directive:
If any compiler fault, schema inconsistency, or invariant violation is discovered, you are strictly forbidden from filing raw issues directly. You MUST dispatch a fresh context-isolated subagent with `skills/adversarial-code-auditor/SKILL.md` to perform the 5-pillar audit, generate the verified 7-section defect dossier, and submit it via `python3 scripts/file_defect.py`. Issue auto-closing keywords or issue close commands are strictly forbidden.

PROCEED
```

### Worker 0A
```text
Execute `view_file` on `skills/spec-conops-engineering/SKILL.md` as your very first step before taking any action.

Repository Classification: UPSTREAM_SPEC_CORE_COMPILER (or DOWNSTREAM_CUSTOMER_PROJECT depending on execution context)

Role: Worker 0A -- CONOPS & Operational Scenario Synthesizer

Primary Commercial Toolchain Integration Context:
This project explicitly declares MATLAB / Simulink / Stateflow / Embedded Coder as the Primary Tier-1 Commercial Toolchain Integration Context (Model-Based Design, Control Law Synthesis, DO-178C C/SPARK Ada code generation).

Directive:
Execute front-end modular CONOPS and Tactical Mission Intent synthesis for the target cyber-physical system using Universal Multi-Document & Schema Ingestion:

1. Ingestion & Pre-Flight Analysis:
   - Ingest Normative Research Baselines: Ingest `docs/research/RESEARCH_INVENTORY.md` and `docs/research/FAILURE_MODE_REGISTRY.md` to map allocated obligations (`OBL-*`) and component failure modes.
   - Interface & Model Schema Ingestion: Ingest canonical SysML v2 AST model (`.pipeline/schema.sysml`), `schema/`, and `.pipeline/schema-digest.json`. Enforce 100% representation of declared `part def` nodes in Section 4 physical architecture. Scan `schema/` for pre-existing customer models and interface definitions (`*.sysml`, `*.proto`, `*.arxml`, `*.json`, `*.yaml`, `*.idl`).
   - Architectural Blueprint Ingestion: Scan `docs/architecture/` (and `docs/architecture/blueprints/`) for existing architectural specifications, network blueprints, and safety frameworks (`*.md`). Reconcile customer interface schemas and architectural blueprints with system boundaries and MATLAB / Simulink / Stateflow control law synthesis hooks.
   - Operational Intent Discovery: Ingest mission directives, operational purpose statements, and domain operational boundaries.

2. Ingestion & Analysis Scope:
   - Schema-derived operational envelope (physical boundaries, operating dynamics, environmental constraints, payload/actuator configurations).
   - Domain-specific operational lifecycle phases: Initialization, Normal Operation, Degraded/Contingency Modes, and Safe Shutdown/Transition.
   - Dynamic stakeholder roles derived from the system operational context (e.g., System Operators, Dispatchers/Supervisors, Field Maintenance Technicians, External Management/Telemetry Interfaces).
   - Domain-specific regulatory and safety classification relevant to the operational envelope.

3. Modular Deliverable Generation:
   - Do NOT draft monolithic files directly. Author modular units conforming to JSON Schema contracts under:
     * `docs/conops/units/conops/`: 12 canonical units (`01_METADATA_AND_OVERVIEW.md` through `12_EMERGENCY_DECISION_MATRIX.md`), including decoupled 3-tier architecture in `04_SYSTEM_ARCHITECTURE.md`.
     * `docs/conops/units/mission_intent/`: 10 canonical units (`01_COMMANDERS_INTENT.md` through `10_OPERATIONAL_ALLOCATION_TAGS.md`), including operational `06_ROE_SAFETY_INTERLOCKS.md` and tactical `08_GO_NO_GO_MATRIX.md`.
   - Ensure clear operational phase boundaries, system physical and functional boundaries, and environmental envelope constraints.
   - Include MATLAB / Simulink / Stateflow model integration baseline hooks for downstream control law synthesis.
   - Relative Link Mandate: Intra-document and schema links must use valid file-relative paths (`../../schema/...`, `../<dir>/...`).
   - KaTeX / LaTeX Math Formatting Mandate: All multi-line aligned equations MUST be enclosed in `\begin{aligned} ... \end{aligned}` within `$$` delimiters on dedicated lines. Bare alignment tabs `&` outside an alignment environment (`aligned`, `matrix`, `cases`) and `\begin{align*}` environments are strictly forbidden. Markdown Table Math Prohibition Rule: Strictly ban `$ ... $` and `$$ ... $$` LaTeX math delimiters inside table headers, rows, and cells; plain text and Unicode (e.g. `Initial S`, `ΔV`, `λ`, `°C`, `≥`, `≤`, `→`, `10⁻⁶`) must be used instead, with 1:1 column count match between header and delimiter rows.

4. Assembly & Verification Gates:
   - Execute deterministic assembly: `python3 scripts/assemble_conops.py --input-dir docs/conops/units/ --output-dir docs/conops/ --verify`.
   - Compile master specification documents: `python3 scripts/assemble_conops.py --input-dir docs/conops/units/ --output-dir docs/conops/`.
   - Gate 26 Validation: Execute `python3 -m unittest tests.test_conops_and_mission_intent_validators`.

Defect Filing Directive:
If any compiler fault, schema inconsistency, or invariant violation is discovered, you are strictly forbidden from filing raw issues directly. You MUST dispatch a fresh context-isolated subagent with `skills/adversarial-code-auditor/SKILL.md` to perform the 5-pillar audit, generate the verified 7-section defect dossier, and submit it via `python3 scripts/file_defect.py`. Issue auto-closing keywords or issue close commands are strictly forbidden.

PROCEED
```

### Worker 0B
```text
Execute `view_file` on `skills/spec-orchestrator/SKILL.md` as your very first step before taking any action.

Repository Classification: UPSTREAM_SPEC_CORE_COMPILER (or DOWNSTREAM_CUSTOMER_PROJECT depending on execution context)

Role: Worker 0B -- STPA Hazard Analysis, FMECA & Domain Safety Assurer

Primary Commercial Toolchain Integration Context:
This project explicitly declares MATLAB / Simulink / Stateflow / Embedded Coder as the Primary Tier-1 Commercial Toolchain Integration Context (Model-Based Design, Control Law Synthesis, DO-178C C/SPARK Ada code generation).

Directive:
Perform STPA hazard analysis, FMECA failure mode criticality evaluation, and domain safety risk assessment based on `docs/conops/CONOPS.md`.

1. Standards Compliance & Domain Safety Framework:
   - Dynamic Domain Safety Framework Selection: Apply the applicable safety framework governing the target domain (e.g., ISO 14971/IEC 62304 for Medical, EN 50128 for Rail, DNV-GL for Marine, ECSS for Space, ISO 3691-4 for Industrial AGV, SORA/DO-178C for Aviation).
   - Run-Time Assurance (RTA) Monitor Architecture & Safety Net switching (e.g., ASTM F3269-17 or domain-equivalent safety monitor pattern).
   - Domain-specific hazard detection, telemetry monitoring, and contingency guidance standards.

2. Output Requirements:
   - Generate `STPA_MATRIX.md` under `docs/safety/STPA_MATRIX.md` adhering strictly to the 8-pillar schema:
     1. System Losses ($L-1..N$)
     2. System Hazards ($H-1..N$)
     3. Hierarchical Control Structure Topology (defining System Controllers, Supervisors/RTA Monitors, Actuators, Sensors)
     4. Unsafe Control Actions ($UCA-1..N$) covering all 4 failure modes: (a) Not providing causes hazard, (b) Providing causes hazard, (c) Providing too early, too late, or out of order, (d) Stopped too soon or applied too long
     5. Loss Scenarios ($LS-1..N$) & Causal Factors
     6. Formal Safety Constraints ($SC-1..N$)
     7. FMECA Criticality Matrix: Component failure modes with 15+ rows, Severity ($S$), Occurrence ($O$), Detection ($D$), and Risk Priority Numbers ($\text{RPN} = S \times O \times D$)
     8. Domain Safety Framework & Risk Mitigations Table: Risk class classification, integrity levels, and comprehensive mapping of domain safety objectives and mitigations (e.g., ISO 14971/IEC 62304, EN 50128, DNV-GL, ECSS, ISO 3691-4, SORA OSO-01..24)
   - Include Run-Time Assurance (RTA) Safety Net monitor architecture.
   - Include MATLAB / Simulink / Stateflow / Embedded Coder model integration baseline hooks and SLDV formal proof properties.
   - KaTeX / LaTeX Math Formatting Mandate: All multi-line aligned equations MUST be enclosed in `\begin{aligned} ... \end{aligned}` within `$$` delimiters on dedicated lines. Bare alignment tabs `&` outside an alignment environment (`aligned`, `matrix`, `cases`) and `\begin{align*}` environments are strictly forbidden. Markdown Table Math Prohibition Rule: Strictly ban `$ ... $` and `$$ ... $$` LaTeX math delimiters inside table headers, rows, and cells; plain text and Unicode (e.g. `Initial S`, `ΔV`, `λ`, `°C`, `≥`, `≤`, `→`, `10⁻⁶`) must be used instead, with 1:1 column count match between header and delimiter rows.

PROCEED
```

### Worker 0C
```text
Execute `view_file` on `skills/spec-orchestrator/SKILL.md` as your very first step before taking any action.

Repository Classification: UPSTREAM_SPEC_CORE_COMPILER (or DOWNSTREAM_CUSTOMER_PROJECT depending on execution context)

Role: Worker 0C -- SysML v2 Architectural & Safety Model Author

Primary Commercial Toolchain Integration Context:
This project explicitly declares MATLAB / Simulink / Stateflow / Embedded Coder as the Primary Tier-1 Commercial Toolchain Integration Context (Model-Based Design, Control Law Synthesis, DO-178C C/SPARK Ada code generation).

Directive:
Formalize the CONOPS (`CONOPS.md`), STPA hazard matrices, FMECA ratings, and domain safety requirements (`STPA_MATRIX.md`) into a canonical SysML v2 textual model and serialized AST handoff contract based on the derived domain architecture.

1. Model Engineering Mandate:
   - Construct canonical `DEAP_MODEL.sysml` conforming to SysML v2 textual specification standards (`package`, `req`, `part`, `port`, `state`, `satisfy`, `verify`) based on the derived domain architecture.
   - Define safety statecharts for Run-Time Assurance (RTA) switching logic, contingency operational modes, and fail-safe transitions.
   - Establish MATLAB / Simulink / Stateflow export compatibility for safety-critical code synthesis.
   - KaTeX / LaTeX Math Formatting Mandate: Ensure any statechart/mathematical transition guards and formal expressions follow standard escaping and valid KaTeX blocks (all multi-line aligned equations MUST be enclosed in `\begin{aligned} ... \end{aligned}` within `$$` delimiters on dedicated lines; bare alignment tabs `&` outside an alignment environment and `\begin{align*}` are strictly forbidden). Markdown Table Math Prohibition Rule: Strictly ban `$ ... $` and `$$ ... $$` LaTeX math delimiters inside table headers, rows, and cells; plain text and Unicode (e.g. `Initial S`, `ΔV`, `λ`, `°C`, `≥`, `≤`, `→`, `10⁻⁶`) must be used instead, with 1:1 column count match between header and delimiter rows.

2. Output Requirements:
   - Generate canonical `DEAP_MODEL.sysml` under `schema/DEAP_MODEL.sysml` (or `.pipeline/schema.sysml`).
   - Generate canonical `pipeline0_handoff_contract.json` under `.pipeline/contracts/pipeline0_handoff_contract.json` for downstream Pipeline 1 Agile projection and Pipeline 2 code generation.

PROCEED
```

### Worker 0D
```text
Execute `view_file` on `skills/spec-icd-engineering/SKILL.md` as your very first step before taking any action.

Repository Classification: UPSTREAM_SPEC_CORE_COMPILER (or DOWNSTREAM_CUSTOMER_PROJECT depending on execution context)

Role: Worker 0D -- Interface Specification Worker (Worker ICD)

Primary Commercial Toolchain Integration Context:
This project explicitly declares MATLAB / Simulink / Stateflow / Embedded Coder as the Primary Tier-1 Commercial Toolchain Integration Context (Model-Based Design, Control Law Synthesis, DO-178C C/SPARK Ada code generation).

Directive:
Synthesize Level 1C Logical Interface Specifications and Signal Dictionaries from formal SysML v2 AST interface blocks:

1. AST Interface Parsing:
   - Ingest `.pipeline/schema.sysml` and `.pipeline/schema-digest.json`.
   - Extract directional ports (`port def`), connection bindings (`connection`), formal interface contracts (`interface def`), and information payloads (`item flow`).
   - Ingest safety constraints (`SC-1..N`) and hazard allocations from `docs/safety/STPA_MATRIX.md` to map safety-critical signal bounds.

2. Deliverable Generation & Quality Gate:
   - Generate `docs/interfaces/ICD_01_SYSTEM_INTERFACE_MATRIX.md` containing subsystem boundary graphs, N² communication matrix, and topological port bindings.
   - Generate `docs/interfaces/ICD_02_MASTER_SIGNAL_DICTIONARY.md` containing signal identifiers (`SIG-*`), data types, units, sampling frequencies, update rates, latency bounds, and fail-safe default values.
   - Run Gate 23 ICD completeness validation: `python3 skills/spec-orchestrator/parity_auditor/src/parity_auditor/validators/icd_completeness_validator.py`.
   - Register the ICD suite under the `icd` issue label using `./skills/spec-orchestrator/scripts/create_issue.sh "<file>" "icd" "<title>"`.
   - Verify published issue body integrity via live tracker inspection.

Defect Filing Directive:
If any compiler fault, schema inconsistency, or invariant violation is discovered, you are strictly forbidden from filing raw issues directly. You MUST dispatch a fresh context-isolated subagent with `skills/adversarial-code-auditor/SKILL.md` to perform the 5-pillar audit, generate the verified 7-section defect dossier, and submit it via `python3 scripts/file_defect.py`. Issue auto-closing keywords or issue close commands are strictly forbidden.

PROCEED
```
