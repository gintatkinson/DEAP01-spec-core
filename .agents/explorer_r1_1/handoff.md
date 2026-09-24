# Handoff Report: Two-Tier Architecture Alignment (Work Package 1 / R1)

**Agent:** Explorer 1 (`teamwork_preview_explorer`)  
**Working Directory:** `/Users/perkunas/jail/DEAP01-spec-core/.agents/explorer_r1_1`  
**Repository Classification:** `UPSTREAM_SPEC_CORE_COMPILER`  
**Primary Commercial Toolchain Integration Context:** `MATLAB / Simulink / Stateflow / Embedded Coder`  
**Target Milestone:** Work Package 1 (R1: Two-Tier Architecture Alignment)  
**Date:** 2026-09-21  

---

## 1. Observation

### 1.1 Conflation in `README.md` Section 1.2 & Section 4
Direct inspection of `/Users/perkunas/jail/DEAP01-spec-core/README.md` shows:
- **Lines 24–30:**
  ```markdown
  - **Downstream Application Workspaces (Multi-Domain Cyber-Physical Exemplars):** Concrete domain application repositories derived from `DEAP01-spec-core`. Turnkey installation or manual setup removes `.pipeline/upstream/`, transitioning the project into `DOWNSTREAM_CUSTOMER_PROJECT` mode to enable concrete domain statecharts, safety-critical controllers, middleware bindings, and domain test suites across 6 canonical domains:
    1. **Aerospace & Defense:** `DEAP-uas-infrastructure-safety` (SORA, ASTM F3269, DO-365B)
    2. **Medical & Healthcare:** `DEAP-surgical-robotics-console` (IEC 62304 Class C, ISO 14971, FDA Class III)
    3. **Space & Satellites:** `DEAP-space-cubesat-constellation` (ECSS-E-ST-40C, NASA-STD-8739.8)
    4. **Industrial Robotics:** `DEAP-industrial-warehouse-agv` (ISO 3691-4, IEC 61508, VDA 5050)
    5. **Subsea & Maritime:** `DEAP-subsea-oceanographic-auv` (DNV-GL-ST-E403, IMO MASS)
    6. **Rail & Transportation:** `DEAP-rail-autonomous-locomotive` (EN 50126, EN 50128 SIL 4)
  ```
  *Observation:* The 6 canonical domain distribution templates (`DEAP-*`) are classified as "Downstream Application Workspaces" and stated to transition to `DOWNSTREAM_CUSTOMER_PROJECT`. This collapses Tier 1 (Domain Distribution Templates) and Tier 2 (Customer Application Workspaces) into a single concept.
- **Lines 116–118:**
  ```markdown
  #### Downstream Customer Project Workspace (Multi-Domain Cyber-Physical Exemplar):
  downstream-workspace/ (e.g. DEAP-uas-infrastructure-safety, DEAP-surgical-robotics-console, ...)
  ```
  *Observation:* Exemplar domain repositories (`DEAP-uas-infrastructure-safety`) are again mislabeled as "Downstream Customer Project Workspaces".

### 1.2 Conflation in `README.md` Section 5
Direct inspection of `/Users/perkunas/jail/DEAP01-spec-core/README.md` Section 5.2 shows:
- **Lines 175–181:**
  ```markdown
  #### Primary Installation (from Sibling Checkout)

  When initializing a new downstream repository alongside a local `DEAP01-spec-core` checkout, run the installer from the root of your downstream repository:

  ```bash
  bash ../DEAP01-spec-core/scripts/install_pipeline.sh
  ```
  ```
  *Observation:* The documented "Primary Installation" instructs users to execute a command with a sibling checkout dependency (`../DEAP01-spec-core`). This is suitable only for a compiler maintainer propagating tooling to a sibling domain template repo, not for a customer engineer onboarding an isolated project.
- **Lines 191–197:**
  ```markdown
  #### Remote Bootstrap (Zero Local Checkout)

  For machines or CI environments without a pre-existing local `DEAP01-spec-core` checkout, run the single-command bootstrap from your downstream project root:

  ```bash
  git clone https://github.com/gintatkinson/DEAP01-spec-core.git /tmp/deap_installer && bash /tmp/deap_installer/scripts/install_pipeline.sh . && rm -rf /tmp/deap_installer
  ```
  ```
  *Observation:* The remote bootstrap command clones `DEAP01-spec-core`. In `DEAP01-spec-core`, landing zones (`schema/`, `docs/conops/`, etc.) contain only `.gitkeep`. If an end-user customer runs this command, their workspace receives an empty abstract compiler without any domain-specific SysML v2 schemas, safety statecharts, or domain contracts (such as UAS SORA models).

### 1.3 Audit of Shell Syntax Violations in `README.md`
Execution of an automated AST/regex audit across all markdown code blocks (`python3 .agents/explorer_r1_1/audit_codeblocks.py`) identified 3 existing shell code block violations in `README.md`:
1. **Line 285** (inside the Section 5.3 `bash` block starting at line 215):
   ```bash
   # Configure for GitLab (if applicable)
   ```
   *Violation:* Unescaped parentheses `(if applicable)` inside a bash code block comment.
2. **Line 398** (inside the Section 6.2 `bash` block starting at line 397):
   ```bash
   # Reconcile against GitHub Issues (default)
   ```
   *Violation:* Unescaped parentheses `(default)` inside a bash code block comment.
3. **Line 407** (inside the Section 6.2 `bash` block starting at line 397):
   ```bash
   # Perform Offline Reconciliation (No remote mutation)
   ```
   *Violation:* Unescaped parentheses `(No remote mutation)` inside a bash code block comment.

In addition, prospective customer onboarding commands using unquoted `<domain-repo-remote-url>` would trigger shell syntax errors (`<` and `>` parsed as redirection operators), violating the acceptance criteria.

---

## 2. Logic Chain

1. **Premise 1 (Upstream Abstract Compiler Invariant):** `DEAP01-spec-core` is an abstract systems engineering compiler (`UPSTREAM_SPEC_CORE_COMPILER`). Per `AGENTS.md` and `.pipeline/constitution.md`, `DEAP01-spec-core` contains zero domain-specific models or concrete schemas; its landing zones contain only `.gitkeep`.
2. **Premise 2 (Domain Distribution Template Role):** The 6 canonical sector repositories (`DEAP-*`, e.g. `DEAP-uas-infrastructure-safety`) are **Domain Distribution Templates** (`DOMAIN_DISTRIBUTION_TEMPLATE`). They receive compiler tooling from `DEAP01-spec-core` and hold canonical domain schemas (`schema/*.sysml`), sector blueprints, and domain profiles.
3. **Premise 3 (Customer Project Role):** End-user customer workspaces (`uav-*`) are concrete application repositories (`DOWNSTREAM_APPLICATION_WORKSPACE` / `DOWNSTREAM_CUSTOMER_PROJECT`). They implement proprietary flight code, ROS2 nodes, and live models.
4. **Inference 1 (Two-Tier Structure):** There are two distinct propagation transitions:
   - **Tier 1 (Compiler Maintainer Propagation):** `DEAP01-spec-core` $\rightarrow$ `DEAP-*` (Domain Distribution Templates). Maintained by compiler developers.
   - **Tier 2 (Customer Project Onboarding):** `DEAP-*` $\rightarrow$ `uav-*` (Customer Application Workspaces). Executed by end-user engineering teams.
5. **Inference 2 (Failure Mode of Current Documentation):** When `README.md` directs end users to clone `DEAP01-spec-core` or run `bash ../DEAP01-spec-core/scripts/install_pipeline.sh`:
   - Customer projects fail to receive domain schemas (`schema/` in `DEAP01-spec-core` is empty).
   - Customers face broken sibling path errors (`../DEAP01-spec-core` does not exist in standalone customer environments).
   - Maintainers and customers conflate compiler propagation with customer project initialization.
6. **Inference 3 (Shell Syntax Gate Compliance):** To guarantee that markdown documentation passes all automated linters and copy-paste shell executions:
   - All shell comments must avoid unescaped parentheses (e.g. replace `(default)` with `- default`).
   - All command placeholders must avoid unquoted angle brackets (e.g. use `"$DOMAIN_REMOTE_URL"` or `"https://github.com/..."`).

---

## 3. Caveats

1. **Work Package Scope Boundary:** This investigation strictly focuses on Work Package 1 (`README.md` documentation alignment and code block syntax). Script modifications to `scripts/install_pipeline.sh` (WP2 parameterization and WP3 schema copy) are analyzed only to ensure complete interface compatibility.
2. **Downstream Baseline Gate Invariant:** `scripts/verify_downstream_baseline.py` Check 14 requires that `README.md` contains either `# Downstream Cyber-Physical Infrastructure Safety Project` or `Operator Prompt Catalog`. Any edits to `README.md` must preserve this string. (The proposed replacement preserves Section 8 `Multi-Pipeline Operator Prompt Catalog`).
3. **No Code Modification Undertaken:** As an explorer agent, no repository source or documentation files have been modified.

---

## 4. Conclusion & Actionable Recommendations

To satisfy Requirement R1 and the acceptance criteria, `README.md` must be updated with the concrete line-level changes detailed below.

### 4.1 Detailed Line-Level Recommendations for `README.md`

#### Recommendation 1: Section 1 System Overview (Line 17)
**Current:**
```markdown
Operating purely on Abstract Syntax Tree (AST) tokens without hardcoding domain concepts, `DEAP01-spec-core` serves as the upstream parent compiler (`UPSTREAM_SPEC_CORE_COMPILER`) from which domain-specific distribution templates across 6 canonical cyber-physical sectors (Aerospace & Defense, Medical & Healthcare, Space & Satellites, Industrial Robotics, Subsea & Maritime, Rail & Transportation) and downstream customer projects are derived via `scripts/install_pipeline.sh` or Direct Copy.
```
**Replacement:**
```markdown
Operating purely on Abstract Syntax Tree (AST) tokens without hardcoding domain concepts, `DEAP01-spec-core` serves as the upstream parent compiler (`UPSTREAM_SPEC_CORE_COMPILER`) from which Tier 1 domain-specific distribution templates across 6 canonical cyber-physical sectors (Aerospace & Defense, Medical & Healthcare, Space & Satellites, Industrial Robotics, Subsea & Maritime, Rail & Transportation) are maintained, and from which Tier 2 downstream customer application workspaces (`uav-*`) are ultimately onboarded via `scripts/install_pipeline.sh` or Direct Copy.
```

---

#### Recommendation 2: Section 1.2 Boundary Definition (Lines 19–33)
**Current:**
```markdown
### 1.2 Upstream Compiler vs. Downstream Application Workspace Boundary

The DEAP framework strictly delineates the boundary between the upstream specification compiler and downstream application workspaces:
- **Upstream Spec Core Compiler (`DEAP01-spec-core`):** Abstract, domain-agnostic specification compiler and multi-agent verification platform. Operates with the sentinel `.pipeline/upstream/` directory present. All upstream landing zones (`docs/conops/`, `docs/safety/`, `docs/epics/`, `docs/features/`, `docs/user-stories/`, `docs/use-cases/`, `schema/`) remain pristine with zero concrete domain specifications or domain-specific `.sysml` files.
- **Pure Schema-Driven Compiler Invariant:** `DEAP01-spec-core` is an abstract Model-Based Systems Engineering (MBSE) compiler; all domain semantics, safety statecharts, and specifications derive deterministically from user-provided schemas in `schema/`.
- **Downstream Application Workspaces (Multi-Domain Cyber-Physical Exemplars):** Concrete domain application repositories derived from `DEAP01-spec-core`. Turnkey installation or manual setup removes `.pipeline/upstream/`, transitioning the project into `DOWNSTREAM_CUSTOMER_PROJECT` mode to enable concrete domain statecharts, safety-critical controllers, middleware bindings, and domain test suites across 6 canonical domains:
  1. **Aerospace & Defense:** `DEAP-uas-infrastructure-safety` (SORA, ASTM F3269, DO-365B)
  2. **Medical & Healthcare:** `DEAP-surgical-robotics-console` (IEC 62304 Class C, ISO 14971, FDA Class III)
  3. **Space & Satellites:** `DEAP-space-cubesat-constellation` (ECSS-E-ST-40C, NASA-STD-8739.8)
  4. **Industrial Robotics:** `DEAP-industrial-warehouse-agv` (ISO 3691-4, IEC 61508, VDA 5050)
  5. **Subsea & Maritime:** `DEAP-subsea-oceanographic-auv` (DNV-GL-ST-E403, IMO MASS)
  6. **Rail & Transportation:** `DEAP-rail-autonomous-locomotive` (EN 50126, EN 50128 SIL 4)
- **Illustrative Schema Payloads:** Any concrete flight controller, robotic surgical console, satellite bus, AGV guidance, subsea vehicle, rail locomotive controller, STPA hazard analysis, or domain safety examples presented throughout this README and documentation are strictly **illustrative schema payloads** demonstrating compiler ingestion, AST synthesis, projection, and verification capabilities.
```

**Replacement:**
```markdown
### 1.2 Two-Tier Architecture Boundary: Upstream Compiler vs. Domain Templates vs. Customer Workspaces

The DEAP framework strictly enforces a clean two-tier architecture separating upstream compiler tooling propagation from downstream customer onboarding:

```
+-----------------------------------------------------------------------------------+
| Tier 1: Upstream Specification Core Compiler                                      |
| Repository: DEAP01-spec-core (Role: UPSTREAM_SPEC_CORE_COMPILER)                  |
| Sentinel: .pipeline/upstream/ present | Pure Schema-Driven Abstract Compiler       |
+-----------------------------------------------------------------------------------+
                                         |
                                         |  Tier 1 Compiler Propagation (Maintainers)
                                         |  scripts/install_pipeline.sh
                                         v
+-----------------------------------------------------------------------------------+
| Tier 1: Domain Distribution Templates (Canonical Cyber-Physical Sectors)         |
| Repositories: DEAP-* (e.g. DEAP-uas-infrastructure-safety)                        |
| Role: DOMAIN_DISTRIBUTION_TEMPLATE | Sentinel: .pipeline/upstream/ removed        |
| Content: Domain SysML v2 schemas (schema/), sector blueprints, domain profiles    |
| Landing zones (docs/epics/, docs/features/, etc.) remain clean (.gitkeep)         |
+-----------------------------------------------------------------------------------+
                                         |
                                         |  Tier 2 Customer Onboarding (End-Users)
                                         |  Self-contained bootstrap via Domain URL
                                         v
+-----------------------------------------------------------------------------------+
| Tier 2: Customer Application Workspaces                                           |
| Repositories: uav-* (e.g. uav-tactical-mission, customer flight projects)         |
| Role: DOWNSTREAM_APPLICATION_WORKSPACE / DOWNSTREAM_CUSTOMER_PROJECT             |
| Content: Concrete proprietary flight code, ROS2 nodes, PX4 modules, SIL/HIL tests |
+-----------------------------------------------------------------------------------+
```

#### Tier 1: Upstream Compiler to Domain Distribution Templates
- **Upstream Spec Core Compiler (`DEAP01-spec-core`):** Abstract, domain-agnostic specification compiler and multi-agent verification platform. Operates with the sentinel `.pipeline/upstream/` directory present. All upstream landing zones (`docs/conops/`, `docs/safety/`, `docs/epics/`, `docs/features/`, `docs/user-stories/`, `docs/use-cases/`, `schema/`) remain pristine with zero concrete domain specifications or domain-specific `.sysml` files.
- **Pure Schema-Driven Compiler Invariant:** `DEAP01-spec-core` is an abstract Model-Based Systems Engineering (MBSE) compiler; all domain semantics, safety statecharts, and specifications derive deterministically from user-provided schemas in `schema/`.
- **Tier 1 Domain Distribution Templates (`DEAP-*`):** Canonical sector repositories derived from `DEAP01-spec-core` by compiler maintainers. Tooling propagation removes `.pipeline/upstream/`, enabling domain-specific SysML v2 models, schemas in `schema/`, and sector architecture blueprints across 6 canonical domains:
  1. **Aerospace & Defense:** `DEAP-uas-infrastructure-safety` (SORA, ASTM F3269, DO-365B)
  2. **Medical & Healthcare:** `DEAP-surgical-robotics-console` (IEC 62304 Class C, ISO 14971, FDA Class III)
  3. **Space & Satellites:** `DEAP-space-cubesat-constellation` (ECSS-E-ST-40C, NASA-STD-8739.8)
  4. **Industrial Robotics:** `DEAP-industrial-warehouse-agv` (ISO 3691-4, IEC 61508, VDA 5050)
  5. **Subsea & Maritime:** `DEAP-subsea-oceanographic-auv` (DNV-GL-ST-E403, IMO MASS)
  6. **Rail & Transportation:** `DEAP-rail-autonomous-locomotive` (EN 50126, EN 50128 SIL 4)
  In Tier 1 domain templates, specification landing zones (`docs/epics/`, `docs/features/`, `docs/user-stories/`, `docs/use-cases/`) remain clean landing zones with `.gitkeep` files until projected for a concrete implementation.

#### Tier 2: Domain Distribution Templates to Customer Application Workspaces
- **Customer Application Workspaces (`uav-*`, `DOWNSTREAM_APPLICATION_WORKSPACE` / `DOWNSTREAM_CUSTOMER_PROJECT`):** Concrete engineering repositories where end-user engineering teams build proprietary software, ROS2 nodes, PX4 flight modules, and hardware-in-the-loop tests.
- **Clean Customer Boundary Mandate:** Customer projects onboard strictly from their relevant **Domain Distribution Template** (`DEAP-*`), NOT from the upstream compiler (`DEAP01-spec-core`). Onboarding from the domain template guarantees that all domain-specific SysML v2 schemas, safety contracts, and profile rules are seeded directly into the customer workspace in a single self-contained command with zero external sibling path dependencies.
- **Illustrative Schema Payloads:** Any concrete flight controller, robotic surgical console, satellite bus, AGV guidance, subsea vehicle, rail locomotive controller, STPA hazard analysis, or domain safety examples presented throughout this README and documentation are strictly **illustrative schema payloads** demonstrating compiler ingestion, AST synthesis, projection, and verification capabilities.
```

---

#### Recommendation 3: Section 4 Repository Trees (Lines 116–147)
Update the downstream tree header to explicitly designate Tier 1 Domain Distribution Templates and Tier 2 Customer Application Workspaces:
```markdown
#### Tier 1: Domain Distribution Template Repository (e.g. DEAP-uas-infrastructure-safety):
```
and note:
```markdown
Tier 2 Customer Application Workspaces (`uav-*`) inherit this layout from the domain distribution template, populating `docs/epics/`, `docs/features/`, and application source directories (`app_flutter/`, `ros2/`, `px4/`).
```

---

#### Recommendation 4: Section 5 Installation & Quick-Start Guide (Lines 171–210)
**Current:**
```markdown
### 5.2 Onboarding Quick-Start Guide (Turnkey Multi-Provider Installer)

Run the turnkey automated installer script to install the safety-critical engineering pipeline into your downstream project. The installer automatically detects your git remote, issue tracker provider (GitHub or GitLab), and organization or group.

#### Primary Installation (from Sibling Checkout)

When initializing a new downstream repository alongside a local `DEAP01-spec-core` checkout, run the installer from the root of your downstream repository:

```bash
bash ../DEAP01-spec-core/scripts/install_pipeline.sh
```

#### Updating an Existing Project

Inside an already-installed downstream repository, update the pipeline and governance rules at any time by running:

```bash
bash scripts/install_pipeline.sh
```

#### Remote Bootstrap (Zero Local Checkout)

For machines or CI environments without a pre-existing local `DEAP01-spec-core` checkout, run the single-command bootstrap from your downstream project root:

```bash
git clone https://github.com/gintatkinson/DEAP01-spec-core.git /tmp/deap_installer && bash /tmp/deap_installer/scripts/install_pipeline.sh . && rm -rf /tmp/deap_installer
```
```

**Replacement:**
```markdown
### 5.2 Tier 1: Compiler Maintainer Propagation Guide (Upstream Compiler -> Domain Templates)

This workflow is for **DEAP platform maintainers** propagating compiler tooling, governance rules, and verification scripts from `DEAP01-spec-core` to the canonical domain distribution template repositories (`DEAP-*`).

#### Sibling Propagation (Local Checkout)

When maintaining a local domain template checkout alongside `DEAP01-spec-core`, run the installer from the root of the target domain template repository:

```bash
# Run from the root of the target domain distribution template
bash ../DEAP01-spec-core/scripts/install_pipeline.sh
```

#### Remote Propagation (CI or Standalone Maintainer Workstation)

To synchronize a domain distribution template directly from the upstream compiler repository:

```bash
# Run from the root of the target domain distribution template
git clone "https://github.com/gintatkinson/DEAP01-spec-core.git" ./.tmp-pipeline && bash ./.tmp-pipeline/scripts/install_pipeline.sh . && rm -rf ./.tmp-pipeline
```

#### In-Place Governance & Script Updates

Inside an already-installed domain template repository, update the pipeline and governance rules at any time by running:

```bash
bash scripts/install_pipeline.sh
```

---

### 5.3 Tier 2: Customer Project Onboarding Guide (Domain Template -> Customer Workspace)

This workflow is for **downstream application engineers and customer teams** creating or scaffolding a concrete project workspace (e.g. `uav-tactical-mission`).

> **Critical Architecture Rule:** Customer projects MUST onboard from the corresponding **Domain Distribution Template** (`DEAP-*`), NOT from `DEAP01-spec-core`. Onboarding from the domain template seeds your project with the required domain SysML v2 schemas, safety models, and platform profiles.

#### Single-Command Self-Contained Customer Onboarding

Execute this command directly inside your new customer project root directory. It runs completely self-contained with zero external sibling path dependencies:

```bash
# Set your domain template repository remote URL
DOMAIN_REMOTE_URL="https://github.com/gintatkinson/DEAP-uas-infrastructure-safety.git"

# Clone installer into temporary directory, run setup, and clean up
git clone "$DOMAIN_REMOTE_URL" ./.tmp-pipeline && bash ./.tmp-pipeline/scripts/install_pipeline.sh . && rm -rf ./.tmp-pipeline
```

Or as a single copy-pasteable command:

```bash
git clone "https://github.com/gintatkinson/DEAP-uas-infrastructure-safety.git" ./.tmp-pipeline && bash ./.tmp-pipeline/scripts/install_pipeline.sh . && rm -rf ./.tmp-pipeline
```

Upon completion, your customer repository is fully configured:
1. `.pipeline/upstream/` is removed, activating `DOWNSTREAM_CUSTOMER_PROJECT` mode.
2. Canonical domain SysML v2 schemas and models from `DEAP-*` are copied into `schema/`.
3. Agent governance files (`.agents/AGENTS.md`, `AGENTS.md`, `CLAUDE.md`) are established.
4. Git hooks and baseline verification suites are installed.
```

---

#### Recommendation 5: Fix Existing Code Block Violations in `README.md`
- **Line 285** (in Section 5.3):
  ```bash
  # Change:
  # Configure for GitLab (if applicable)
  # To:
  # Configure for GitLab if applicable
  ```
- **Line 398** (in Section 6.2):
  ```bash
  # Change:
  # Reconcile against GitHub Issues (default)
  # To:
  # Reconcile against GitHub Issues - default
  ```
- **Line 407** (in Section 6.2):
  ```bash
  # Change:
  # Perform Offline Reconciliation (No remote mutation)
  # To:
  # Perform Offline Reconciliation - No remote mutation
  ```

---

## 5. Verification Method

To independently verify these findings and confirm that implementing the recommendations satisfies all requirements:

1. **Baseline Conformance Gate:**
   Run the canonical repository verification command:
   ```bash
   python3 scripts/verify_downstream_baseline.py --no-domain
   ```
   *Expected Result:* Exits with code 0; all checks (Check 10–30) pass. Check 14 passes because "Operator Prompt Catalog" is preserved.

2. **Shell Syntax and Code Block Integrity Check:**
   Run the code block scanner created in this investigation:
   ```bash
   python3 .agents/explorer_r1_1/audit_codeblocks.py
   ```
   *Verification Criteria:*
   - Zero unescaped parentheses in comments within all `bash`/`sh` code blocks.
   - Zero unquoted angle-bracket placeholders within all `bash`/`sh` code blocks.

3. **Diff Review against Acceptance Criteria:**
   Confirm via `git diff README.md`:
   - Tier 1 and Tier 2 architecture explicitly delineated.
   - Domain distribution templates (`DEAP-*`) clearly separated from customer application workspaces (`uav-*`).
   - Customer onboarding instructions use self-contained command with zero sibling path dependencies.
