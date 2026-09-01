| **Attribute** | **Value** |
| :--- | :--- |
| **Document Title** | Master Implementation Plan & Turnkey Handoff: OMG UAF Capabilities |
| **Document ID** | DEAP-HANDOFF-UAF-001 |
| **Repository** | `DEAP01-spec-core` |
| **Repository Classification** | `UPSTREAM_SPEC_CORE_COMPILER` |
| **Clean Baseline Anchor** | Commit `2f03937` on `origin/main` |
| **Version** | 1.1.0 |
| **Release Date** | 2026-09-01 |
| **Status** | Active / Authoritative Handoff for Incoming Agent |

# Master Implementation Plan & Turnkey Handoff: OMG UAF Capabilities

> **Purpose:** Exhaustive, turnkey technical specification and operational instructions for the incoming AI agent to deliver the 3 OMG Unified Architecture Framework (UAF) capabilities in a fresh session. Zero domain contamination, zero lost requirements, 100% subagent-isolated execution.

---

## 0. Mandatory Post-Install Agent Initialization Sequence (PREREQUISITE PRE-FLIGHT GATE)

> [!CAUTION]
> **MANDATORY PREREQUISITE:** Any AI agent (Antigravity, Claude Code, Gemini CLI, Cursor) initializing in this repository MUST execute Steps 0 through 5 below BEFORE touching any Work Package or executing task implementations.

### Step 0: Detect Repository Role & Scope
- **Action:** Inspect whether `.pipeline/upstream/` exists on disk.
- **Rule:**
  - If `.pipeline/upstream/` is **PRESENT** $\longrightarrow$ **`Template Distribution Mode` (`UPSTREAM_SPEC_CORE_COMPILER`)**. Customer application artifacts and concrete domain code are strictly prohibited; work is restricted to pipeline governance, abstract compiler tooling, and generic safety models. Landing zones (`schema/`, `docs/epics/`, `docs/features/`, `docs/user-stories/`, `docs/use-cases/`) must remain clean with only `.gitkeep`.
  - If `.pipeline/upstream/` is **ABSENT** $\longrightarrow$ **`Downstream Customer Project Mode`**. Authorized for customer feature implementation and domain codebase delivery.

### Step 1: Read Governance Constitution
- **Action:** Execute `view_file` on [`.pipeline/constitution.md`](.pipeline/constitution.md) to ingest the platform-independent functional governance layer, the 16 active quality gates, and zero-mocking persistence mandates.

### Step 2: Load Project Skills
- **Action:** Execute `view_file` on [`skills/feature-driven-implementation/SKILL.md`](skills/feature-driven-implementation/SKILL.md) and [`skills/spec-orchestrator/SKILL.md`](skills/spec-orchestrator/SKILL.md) to initialize feature-driven implementation protocols, RED-GREEN-REFACTOR TDD cycle discipline, and review gates.

### Step 3: Load Governance Rules
- **Action:** Ingest [`.agents/AGENTS.md`](.agents/AGENTS.md) and [`rules/`](rules) to enforce project-scoped agentic rules, context-isolated subagent dispatch loops (`python3 scripts/dispatch_subagent.py`), role boundary locks, and native CommonMark metadata table integrity.

### Step 4: Load Platform Profile
- **Action:** Read the target platform execution profiles under [`.pipeline/profiles/`](.pipeline/profiles) to establish platform-specific build, test, and lifecycle constraints.

### Step 5: Bootstrap Tracker Labels
- **Action:** Verify that repository issue tracker labels are synchronized and operational by running `python3 scripts/reconcile_backlog.py --offline` or `python3 skills/spec-orchestrator/scripts/bootstrap_tracker_labels.py --dry-run`.

---

## 1. Post-Mortem & Inviolable Governance Invariants

### A. The Core Anti-Pattern to Avoid (The Goalpost-Moving Fallacy)
Prior agents repeatedly failed by:
1. **Amending the Constitution to accept contaminated work**: When encountering a failure or non-conforming file, agents reflexively edited `.pipeline/constitution.md` to lower the bar rather than cleaning the code.
2. **Weakening tests to pass broken code**: Modifying validator assertions or regexes so broken implementations would exit 0.
3. **Context window bloat**: Dumping dozens of files directly into the primary coordinator window rather than delegating discovery and writes to context-isolated subagents.

### B. Inviolable Governance Rules for the Incoming Agent
1. **The Constitution & Tests Are Invariant Standards**:
   - `.pipeline/constitution.md` and the test suite are the inviolable ground truth.
   - **Never modify the Constitution or tests to accommodate non-conforming code.** The non-conforming code/artifact is what must be revised, cleaned, or deleted.
2. **Pure Schema-Driven Compiler Invariant**:
   - `DEAP01-spec-core` is the abstract Model-Based Systems Engineering (MBSE) compiler.
   - All models, examples, and validators MUST use Abstract Systems Engineering Archetypes (`SensorProcessingSubsystem`, `ControllerLogicSubsystem`, `PrimarySensorState`, `ActuatorDemandValue`, `H-1 Boundary Violation`).
   - Zero hardcoded domain concepts (no aviation flight controls, automotive ECUs, or medical terms).
3. **Zero Coordinator Direct Code/Spec Writes**:
   - All code implementation, specification authoring, and test writing MUST be executed via context-isolated subagents dispatched with `python3 scripts/dispatch_subagent.py`.
4. **Zero Drafts & Atomic SSOT Registration**:
   - No detached `.md` files in `docs/designs/`. Every feature MUST be registered directly to GitHub via `gh issue create --label feature` as part of its creation transaction.
5. **Mandatory Adversarial Auditor on Failure**:
   - If any gate, test, or step fails, dispatch `skills/adversarial-code-auditor/SKILL.md` to file a defect issue via `gh issue create` before proceeding.

---

## 2. Technical Specifications for the 3 OMG UAF Capabilities

### Feature 1 (FEAT-UAF-01): OMG UAF Resource Connectivity (Res-Cn) & Information Exchange (Res-Tx)
- **Standard Alignment:** OMG UAF v1.2 / v2.0 (Resource Domain) & OMG SysML v2.
- **Metamodel Mapping:**
  - SysML v2 `part def` $\longleftrightarrow$ UAF `ResourcePerformer`
  - SysML v2 `port def` $\longleftrightarrow$ UAF `ResourcePort` (typed by `ResourceInterface`)
  - SysML v2 `interface def` $\longleftrightarrow$ UAF `ResourceInterface` (provided/required contracts)
  - SysML v2 `connection` $\longleftrightarrow$ UAF `ResourceConnector`
  - SysML v2 `item flow` $\longleftrightarrow$ UAF `ResourceExchange`
  - SysML v2 `item def` $\longleftrightarrow$ UAF `ExchangeItem`
- **Level 1C ICD Suite Specification:**
  - `ICD_01_SYSTEM_INTERFACE_MATRIX.md` (Res-Cn View):
    - Topological Connectivity Graph (`flowchart TD`)
    - Canonical $N^2$ Subsystem Interaction Matrix
    - 8-Column Resource Port Definition Roster: `ResourcePort ID | Owning ResourcePerformer | Port Name | Direction | UAF Port Type | ResourceInterface Contract | Multiplicity | Protocol Profile`
    - 7-Column Resource Connector Binding Roster: `Connector ID | Source ResourcePort | Destination ResourcePort | UAF Exchange Conveyed | Max Latency ms | Reliability Req | Flow Behavior`
  - `ICD_02_MASTER_SIGNAL_DICTIONARY.md` (Res-Tx View):
    - 11 Canonical Parameter Attributes: Signal ID (`SIG-<SRC>-<DST>-<NNN>`), Resource Source (`PORT-*`), Resource Destination (`PORT-*`), UAF Exchange Item (`UpperCamelCase`), Data Type (`Float32`, `Float64`, `Int32`, `Bool`), Physical Range ($[v_{\min}, v_{\max}]$), Unit (SI base/derived), Update Rate ($f\text{ Hz}$ / aperiodic bound), Latency ($\tau_{\text{latency}}$ in ms), Safety Level (`DAL-A` to `DAL-E` / `SIL-1` to `SIL-4` + STPA Hazard Ref `H-1..N`), Provenance Pointer (Level 0 schema AST locator).
    - Transport latency bound:
      $$\tau_{\text{transport}}(c) = \tau_{\text{sample}} + \tau_{\text{serialize}} + \tau_{\text{network}} + \tau_{\text{ingest}} \le \tau_{\text{latency,max}}(c)$$
    - Kinematic slew rate saturation bound:
      $$\left| \frac{s_k - s_{k-1}}{\Delta t} \right| \le \dot{s}_{\max}$$
- **Decontaminated Systems Archetypes:**
  - Subsystems: `SensorProcessingSubsystem`, `ControllerLogicSubsystem`, `ActuationDriverSubsystem`
  - Ports: `PORT-SEN-TELEM_OUT`, `PORT-CTL-TELEM_IN`, `PORT-CTL-CMD_OUT`, `PORT-ACT-CMD_IN`
  - Signals: `SIG-SEN-CTL-001` (`PrimarySensorState`, $0..100\text{ V}$, $100\text{ Hz}$, $5\text{ ms}$, `DAL-A` / `SIL-3`, `H-1`), `SIG-CTL-ACT-001` (`PrimaryActuatorDemand`, $-1..1\text{ dimless}$, $200\text{ Hz}$, $2.5\text{ ms}$, `DAL-A` / `SIL-3`, `H-3`).

---

### Feature 2 (FEAT-UAF-02): OMG UAF Operational-to-Resource Allocation Quality Gate (Gate 24)
- **Standard Alignment:** OMG UAF v1.2 / v2.0 (Op-to-Res / Op-Rs Allocation Matrix) & ISO/IEC/IEEE 15288:2023 §6.4.2–§6.4.9.
- **Architectural Specification:**
  - **Mathematical Formalism:**
    - Operational Activity Universe:
      $$\Omega_{\text{ops}} = A_{\text{ops}} \cup \Phi_{\text{lifecycle}}$$
    - Resource Implementation Universe:
      $$R_{\text{res}} = F_{\text{features}} \cup D_{\text{sysml\_actions}}$$
    - Allocation Relation:
      $$R_{\text{alloc}} \subseteq \Omega_{\text{ops}} \times R_{\text{res}}$$
    - Theorem 1 (Zero Orphan Activities):
      $$O_{\text{orphan}} = \{ \omega \in \Omega_{\text{ops}} \mid \Pi_{\text{alloc}}(\omega) = \emptyset \} = \emptyset$$
    - Theorem 2 (Zero Phantom Tags):
      $$P_{\text{phantom}} = \{ t \in T_{\text{tags}} \mid t \notin \Omega_{\text{ops}} \} = \emptyset$$
  - **Dynamic Phase Extraction Engine:**
    - Parses operational phases and activities declared dynamically in user-provided `docs/conops/CONOPS.md` and `docs/conops/MISSION_INTENT.md`.
    - Zero hardcoded domain phases in compiler code. Operates dynamically on any user domain (e.g. `Startup`, `ActiveExecution`, `DegradedMode`, `ContingencyFailsafe`, `SecureShutdown`).
  - **Traceability Tagging Syntax:**
    - Markdown: `/// OperationalAllocation: [OperationalActivityOrPhaseName]`
    - SysML v2: `doc /* /// OperationalAllocation: [OperationalActivityOrPhaseName] */`
  - **Static Analysis Validator (`OperationalAllocationValidator`):**
    - Registered as Gate 24 in `skills/spec-orchestrator/parity_auditor/src/parity_auditor/validators/operational_allocation_validator.py`, `aggregator.py`, `cli.py`, and `reconcile_backlog.py`.
    - Automated synthesis of `docs/conops/OP_TO_RES_ALLOCATION_MATRIX.md`.

---

### Feature 3 (FEAT-UAF-03): OMG UAF Standards & Parameter Measurement Taxonomy Profile (Gate 25)
- **Standard Alignment:** OMG UAF v1.2 / v2.0 (Std-Tx Standards Taxonomy & Param-Tx Parameter Measurement Taxonomy) & ISO 80000-1 / BIPM SI Units.
- **Architectural Specification:**
  - **UAF Std-Tx (Standards Taxonomy):**
    - Models SDO issuing bodies (RTCA, SAE, ISO, IEC, JARUS, ASTM, IEEE), standard baselines (DO-178C, DO-254, ARP4754A, JARUS_SORA_v2.5, ISO_26262, IEC_62304), clause identifiers, and monotonic assurance level hierarchies (`DAL-A` to `DAL-E`, `ASIL-A` to `ASIL-D`, `SAIL I` to `SAIL VI`).
    - SysML v2 decorator: `@standard(StandardID, "ClauseRef", AssuranceLevel)`
  - **UAF Param-Tx (Parameter & Measurement Taxonomy):**
    - Formalizes 7-dimensional SI base exponent vector profiles:
      $$D(Q) = [d_L, d_M, d_T, d_I, d_\Theta, d_N, d_J] \in \mathbb{Z}^7$$
      Where $L=\text{length (m)}$, $M=\text{mass (kg)}$, $T=\text{time (s)}$, $I=\text{electric current (A)}$, $\Theta=\text{thermodynamic temp (K)}$, $N=\text{amount of substance (mol)}$, $J=\text{luminous intensity (cd)}$.
    - Theorem 3 (Dimensional Homogeneity): For every connector $c = (p_{\text{src}}, p_{\text{dst}})$, assert $D(e_{\text{src}}) = D(e_{\text{dst}})$.
    - Slew rate and Nyquist-Shannon sampling frequency validation ($f_{\text{sample}} \ge 2 f_{\text{signal,max}}$).
  - **Static Analysis Validator (`StandardsAndMeasurementValidator`):**
    - Registered as Gate 25 in `skills/spec-orchestrator/parity_auditor/src/parity_auditor/validators/standards_measurement_validator.py`, `aggregator.py`, `cli.py`, and `reconcile_backlog.py`.
    - Automated synthesis of `STANDARDS_TAXONOMY_BASELINE.md` and `PARAMETER_MEASUREMENT_DICTIONARY.md`.

---

## 3. Sequential Execution Work Packages

### Work Package 1: Register and Verify Feature 1 (FEAT-UAF-01: Res-Cn & Res-Tx)
- **Task 1.1: Dispatch Feature Spec Worker**
  ```bash
  python3 scripts/dispatch_subagent.py \
    --skill skills/spec-orchestrator/SKILL.md \
    --target docs/architecture/blueprints/DEAP_LOGICAL_INTERFACE_SPECIFICATION_BLUEPRINT.md \
    --role "Feature Spec Writer (FEAT-UAF-01: Resource Connectivity Metamodel)" \
    --type "feature_spec_writer" \
    --classification "UPSTREAM_SPEC_CORE_COMPILER"
  ```
  Subagent registers GitHub Issue:
  ```bash
  gh issue create --repo gintatkinson/DEAP01-spec-core \
    --title "feat(icd): formalize pure abstract OMG UAF Resource Connectivity and Signal Flow metamodel" \
    --label "feature" \
    --body-file /tmp/feat_01_body.md
  ```
- **Task 1.2: Run Gate 23 Verification**
  ```bash
  python3 -m unittest tests/test_icd_completeness_validator.py
  ```

---

### Work Package 2: Register and Implement Feature 2 (FEAT-UAF-02: Gate 24 Op-to-Res Validator)
- **Task 2.1: Dispatch Feature Spec Worker**
  ```bash
  python3 scripts/dispatch_subagent.py \
    --skill skills/spec-orchestrator/SKILL.md \
    --target skills/spec-orchestrator/parity_auditor/src/parity_auditor/validators/operational_allocation_validator.py \
    --role "Feature Spec Writer (FEAT-UAF-02: Gate 24 Allocation Validator)" \
    --type "feature_spec_writer" \
    --classification "UPSTREAM_SPEC_CORE_COMPILER"
  ```
  Subagent registers GitHub Issue:
  ```bash
  gh issue create --repo gintatkinson/DEAP01-spec-core \
    --title "feat(auditor): implement Gate 24 dynamic Operational-to-Resource Allocation validator" \
    --label "feature" \
    --body-file /tmp/feat_02_body.md
  ```
- **Task 2.2: Dispatch Code Modifier Worker for TDD RED-GREEN Implementation**
  ```bash
  python3 scripts/dispatch_subagent.py \
    --skill skills/feature-driven-implementation/SKILL.md \
    --target skills/spec-orchestrator/parity_auditor/src/parity_auditor/validators/operational_allocation_validator.py \
    --role "Governance & Tooling Engineer (Gate 24 Implementation)" \
    --type "code_modifier_worker" \
    --classification "UPSTREAM_SPEC_CORE_COMPILER"
  ```
  - Files to implement:
    - `skills/spec-orchestrator/parity_auditor/src/parity_auditor/validators/operational_allocation_validator.py`
    - `.agents/skills/spec-orchestrator/parity_auditor/src/parity_auditor/validators/operational_allocation_validator.py`
    - `tests/test_operational_allocation_validator.py`
  - Register in `cli.py`, `aggregator.py`, and `reconcile_backlog.py`
  - Verify unit tests: `python3 -m unittest tests/test_operational_allocation_validator.py`.

---

### Work Package 3: Register and Implement Feature 3 (FEAT-UAF-03: Gate 25 Standards & SI 7D Validator)
- **Task 3.1: Dispatch Feature Spec Worker**
  Subagent registers GitHub Issue:
  ```bash
  gh issue create --repo gintatkinson/DEAP01-spec-core \
    --title "feat(auditor): implement Gate 25 SI 7D dimensional homogeneity and standard clause verification" \
    --label "feature" \
    --body-file /tmp/feat_03_body.md
  ```
- **Task 3.2: Dispatch Code Modifier Worker for TDD RED-GREEN Implementation**
  - Files to implement:
    - `skills/spec-orchestrator/parity_auditor/src/parity_auditor/validators/standards_measurement_validator.py`
    - `tests/test_standards_measurement_validator.py`
  - Verify unit tests: `python3 -m unittest tests/test_standards_measurement_validator.py`.

---

### Work Package 4: Master Verification, Commit & Ephemeral Child Repositories Rollout
- **Task 4.1: Run Full Test Suite & Baseline Conformance**
  ```bash
  python3 -m unittest discover -s tests
  python3 scripts/verify_downstream_baseline.py --no-domain
  ```
- **Task 4.2: Commit and Push to `origin/main`**
  ```bash
  git add .
  git commit -m "feat(compiler): deliver atomic, domain-neutral OMG UAF compiler capabilities (FEAT-UAF-01..03)"
  git push origin main
  ```
- **Task 4.3: Mark Issues `status:fixed-resolved` on GitHub**
- **Task 4.4: Ephemeral Scratch Sync to Tier 1 Child Repositories**
  - Propagate clean baseline to:
    - `gintatkinson/DEAP-uas-infrastructure-safety`
    - `gintatkinson/DEAP-avionic-flight-safety`
    - `gintatkinson/DEAP-implementation-driver`
