# Handoff Report: WP-01 Forensic Adversarial Audit of README.md & Installer Scaffolding

- **Target Workspace**: `/Users/perkunas/jail/DEAP01-spec-core`
- **Work Package**: WP-01 (Forensic Adversarial Audit)
- **Assigned Agent**: `teamwork_preview_auditor` (`.agents/auditor_wp01/`)
- **Repository Classification**: `UPSTREAM_SPEC_CORE_COMPILER`
- **Primary Commercial Toolchain Integration Context**: `MATLAB / Simulink / Stateflow / Embedded Coder`
- **Date**: 2026-09-27

---

## 1. Observation

Direct inspection of `README.md`, `scripts/install_pipeline.sh`, and `tests/test_readme_scaffolding.py` revealed four distinct categories of structural and semantic defects.

### 1.1 Contradictory Architecture Tier Numbering
- **In `README.md`**:
  * **Line 17**: `...from which Tier 1 domain-specific distribution templates across 6 canonical cyber-physical sectors ... are maintained, and from which Tier 2 downstream customer application workspaces (uav-*) are onboarded via domain distribution templates.`
    - Both Domain Distribution Templates and the Upstream Compiler are designated "Tier 1", while Customer Workspaces are designated "Tier 2".
  * **Line 19**: Heading `### 1.2 Two-Tier Architecture Boundary: Upstream Compiler vs. Domain Templates vs. Customer Workspaces` contradicts the three distinct repository entities enumerated in the title.
  * **Line 21**: Text asserts `The DEAP framework strictly enforces a clean two-tier architecture separating upstream compiler tooling propagation from downstream customer onboarding:`.
  * **Lines 23–50 (ASCII Topology Diagram)**:
    - Line 25: `| Tier 1: Upstream Specification Core Compiler |`
    - Line 30: `| Tier 1 Compiler Propagation (Maintainers)`
    - Line 34: `| Tier 1: Domain Distribution Templates (Canonical Cyber-Physical Sectors) |` (Contradictory duplicate "Tier 1")
    - Line 41: `| Tier 2 Customer Onboarding (End-Users)`
    - Line 45: `| Tier 2: Customer Application Workspaces |` (Conflates onboarding step with repository tier)
  * **Lines 52–68 (Subsection Headings & Body)**:
    - Line 52: `#### Tier 1: Upstream Compiler to Domain Distribution Templates`
    - Line 55: `- **Tier 1 Domain Distribution Templates (DEAP-*):**` (Contradictory "Tier 1")
    - Line 62: `In Tier 1 domain templates, specification landing zones...` (Contradictory "Tier 1")
    - Line 64: `#### Tier 2: Domain Distribution Templates to Customer Application Workspaces`
  * **Lines 152–177 (Section 4 Repository Trees & Taxonomies)**:
    - Line 152: `#### Tier 1: Domain Distribution Template Repository (e.g. DEAP-uas-infrastructure-safety):` (Contradictory "Tier 1")
    - Line 175: `Tier 2 Customer Application Workspaces (uav-*) inherit this layout...` (Contradictory "Tier 2")
    - Line 177: `### 4.1 Supported Tier 1 Domain Distribution Templates (Canonical Taxonomies)` (Contradictory "Tier 1")
  * **Lines 260–289 (Section 5.4 Architectural Boundary)**:
    - Line 262: `The DEAP two-tier architecture strictly decouples compiler tooling propagation from customer project onboarding:`
    - Line 263: `- **Tier 1 (Upstream Compiler -> Domain Templates):**...`
    - Line 264: `- **Tier 2 (Domain Templates -> Customer Workspaces):**...`
    - Line 270: `| Tier 1: Upstream Specification Core Compiler |`
    - Line 277: `| Tier 1: Domain Distribution Templates |` (Contradictory duplicate "Tier 1")
    - Line 285: `| Tier 2: Customer Application Workspaces |` (Contradictory "Tier 2")
  * **Line 291 (Anchor Cross-Reference)**:
    - `(see [Section 4.1](#41-supported-tier-1-domain-distribution-templates-canonical-taxonomies))`
    - Points to the heading slug containing `tier-1`. When heading 4.1 is normalized, this anchor breaks unless updated in tandem.

- **In `scripts/install_pipeline.sh`**:
  * **Line 882**: In the Domain Distribution Template README generation template:
    `As a **Tier 1 Domain Distribution Template**, this repository maintains pristine, clean landing zones in \`schema/\`...`
  * **Line 953**: In the Customer Application Workspace README generation template:
    `As a **Tier 2 Customer Application Workspace**, this repository is authorized for concrete engineering delivery...`

### 1.2 Inverted Heading Sequence in Section 1
- **In `README.md`**:
  * Line 13: `## 1. System Overview` (H2)
  * Line 19: `### 1.2 Two-Tier Architecture Boundary: Upstream Compiler vs. Domain Templates vs. Customer Workspaces` (H3, lines 19–70)
  * Line 71: `## 1.1 Primary Commercial Toolchain Integration` (H2, lines 71–75)
  * Section 1.1 appears after Section 1.2 rather than preceding it.
  * Section 1.1 is formatted as an H2 (`## 1.1`), which violates heading hierarchy since it is logically a subsection of `## 1. System Overview` (H2), while Section 1.2 is formatted as an H3 (`### 1.2`).

### 1.3 Upstream Compiler vs. Downstream Prompt Boundary Hardening
- **In `README.md` (Section 9.4)**:
  * Line 1083: `### 9.4 Pipeline 2 Prompts (Autonomous Feature Implementation & Two-Path Simulation Driver)` lacks an explicit Execution Boundary Invariant admonition warning that Pipeline 2 is prohibited in `UPSTREAM_SPEC_CORE_COMPILER`.
  * Line 1094 (Worker 2A.1 Flutter prompt preamble):
    `Repository Classification: DOWNSTREAM_CUSTOMER_PROJECT (or UPSTREAM_SPEC_CORE_COMPILER depending on execution context)`
  * Line 1149 (Worker 2A.2 ROS2/PX4 prompt preamble):
    `Repository Classification: DOWNSTREAM_CUSTOMER_PROJECT (or UPSTREAM_SPEC_CORE_COMPILER depending on execution context)`
  * Line 1180 (Worker 2B simulation driver prompt preamble):
    `Repository Classification: UPSTREAM_SPEC_CORE_COMPILER (or DOWNSTREAM_CUSTOMER_PROJECT depending on execution context)`
  * In `docs/OPERATOR_PROMPT_CATALOG.md`: lines 212, 267, 298 contain identical ambiguous preambles.
  * In `scripts/install_pipeline.sh`: lines 1430, 1484, 1516 contain identical ambiguous preambles embedded in the generated customer README prompt catalog.

### 1.4 Test Suite Gaps in `tests/test_readme_scaffolding.py`
- Line 381 docstring: `"""Verifies that install_pipeline.sh properly scaffolds Tier 1 Domain Distribution Templates."""` (Legacy Tier 1 reference).
- Line 482 docstring: `"""Verifies that install_pipeline.sh properly scaffolds Tier 2 Customer Application Workspaces."""` (Legacy Tier 2 reference).
- Missing test assertions:
  1. No assertion verifying that Section 1.1 precedes Section 1.2 in `README.md`.
  2. No assertion verifying that Section 1.1 is an H3 heading under `## 1. System Overview`.
  3. No assertion verifying that `README.md` has 0 occurrences of contradictory `Tier 1 Domain Distribution Template` or `Tier 2 Customer Application Workspace`.
  4. No assertion verifying that `scripts/install_pipeline.sh` scaffolds `Tier 2 Domain Distribution Template` and `Tier 3 Customer Application Workspace`.
  5. No assertion verifying that Section 9.4 prompt preambles strictly enforce `Repository Classification: DOWNSTREAM_CUSTOMER_PROJECT` with 0 occurrences of `UPSTREAM_SPEC_CORE_COMPILER`.

### 1.5 Baseline Test Execution Results
- `python3 -m unittest tests/test_readme_scaffolding.py`: Ran 31 tests in 35.0s, exit code 0 (`OK`).
- `python3 scripts/verify_downstream_baseline.py --no-domain`: All 31 checks passed cleanly with exit code 0.
- `python3 -m pytest tests/`: Ran 294 tests across 14 test modules, 294 passed in 166.9s, exit code 0.

---

## 2. Logic Chain

1. **Step 1 (Taxonomy Conflation)**: In early iterations of the repository propagation scripts, maintainers formulated propagation as a two-phase process: Step 1 (Compiler to Domain Template) and Step 2 (Domain Template to Customer Workspace). These two process steps were colloquially labeled "Tier 1" and "Tier 2".
2. **Step 2 (Structural Inconsistency)**: Because the upstream compiler is itself Tier 1 of the platform hierarchy, labeling the first propagation target (Domain Distribution Templates) as "Tier 1" resulted in two distinct tiers sharing the exact same label. Customer workspaces were labeled "Tier 2", leaving no designation for Tier 3.
3. **Step 3 (Heading Sequence Defect)**: When `1.1 Primary Commercial Toolchain Integration` was added to fulfill the commercial toolchain governance invariant (MATLAB / Simulink / Stateflow / Embedded Coder), it was appended at line 71 as an H2 (`## 1.1`) after `### 1.2 Two-Tier Architecture Boundary...`, creating an inverted heading order (`1.2` before `1.1`) and broken hierarchical nesting.
4. **Step 4 (Boundary Leakage in Pipeline 2)**: Pipeline 2 implements concrete features (Flutter UI widgets under `app_flutter/`, ROS2 nodes, PX4 modules, Python simulation engines). The prompt preambles in Section 9.4 were authored with generic fallback text `(or UPSTREAM_SPEC_CORE_COMPILER depending on execution context)`. If an autonomous agent executes Pipeline 2 in `DEAP01-spec-core`, it directly violates the Upstream Clean Landing Zone Invariant (`schema/`, `docs/epics/`, `docs/features/` with only `.gitkeep`) and the Pure Schema-Driven Compiler Invariant (zero hardcoded domain concepts).
5. **Step 5 (Test Suite Immunity)**: `tests/test_readme_scaffolding.py` previously tested shell code fence hygiene (`bash -n`), circular clone command absence, and role string presence (`DOMAIN_DISTRIBUTION_TEMPLATE`), but lacked structural assertions for Section 1.1/1.2 sequence, three-tier normalization, and prompt preamble boundary locks. As a result, the defects persisted without failing tests.

---

## 3. Caveats

- **Scope Boundary**: This audit investigated `README.md`, `scripts/install_pipeline.sh`, `tests/test_readme_scaffolding.py`, and referenced prompt catalogs. It did not alter or execute code changes, as audit agents are strictly audit-only.
- **Platform Independence vs. Toolchain Tier**: The phrase `Primary Tier-1 Commercial Toolchain Integration Context` (referencing MATLAB / Simulink / Stateflow / Embedded Coder) denotes the *commercial vendor partnership tier*, NOT the architectural repository tier. It must remain unchanged across prompt bodies.
- **Constitutional Three-Tier Layering**: `.pipeline/constitution.md` lines 15–47 defines a complementary three-tier platform isolation model for specifications (Tier 1: Functional Layer, Tier 2: Dynamic Context, Tier 3: Platform Profiles). The repository propagation model (Tier 1: Upstream Compiler, Tier 2: Domain Templates, Tier 3: Customer Workspaces) is fully orthogonal and harmonious with this constitutional layering.

---

## 4. Conclusion

The audit identifies four specific, solvable structural defects across `README.md`, `scripts/install_pipeline.sh`, and `tests/test_readme_scaffolding.py`. 

The required normalization establishes:
- **Tier 1**: Upstream Specification Core Compiler (`DEAP01-spec-core`, `UPSTREAM_SPEC_CORE_COMPILER`)
- **Tier 2**: Domain Distribution Templates (`DEAP-*`, `DOMAIN_DISTRIBUTION_TEMPLATE`)
- **Tier 3**: Customer Application Workspaces (`uav-*`, `DOWNSTREAM_CUSTOMER_PROJECT`)
- **Propagation Boundary 1**: Compiler Tooling Propagation (Tier 1 -> Tier 2, maintained via `scripts/install_pipeline.sh`)
- **Propagation Boundary 2**: Customer Project Onboarding (Tier 2 -> Tier 3, bootstrapped via Domain Remote URL)

---

## 5. Exact Remediation Specifications

### 5.1 Remediation Specification for WP-02 (README.md & scripts/install_pipeline.sh)

#### File: `README.md`

1. **Lines 17–76 (Section 1 Normalization & Repositioning)**:
   Replace lines 17–76 with:
   ```markdown
   Operating purely on Abstract Syntax Tree (AST) tokens without hardcoding domain concepts, `DEAP01-spec-core` serves as the upstream parent compiler (`UPSTREAM_SPEC_CORE_COMPILER`) from which Tier 2 domain-specific distribution templates across 6 canonical cyber-physical sectors (Aerospace & Defense, Medical & Healthcare, Space & Satellites, Industrial Robotics, Subsea & Maritime, Rail & Transportation) are maintained, and from which Tier 3 downstream customer application workspaces (`uav-*`) are onboarded via domain distribution templates.

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
                                            |  scripts/install_pipeline.sh
                                            v
   +-----------------------------------------------------------------------------------+
   | Tier 2: Domain Distribution Templates (Canonical Cyber-Physical Sectors)         |
   | Repositories: DEAP-* (e.g. DEAP-uas-infrastructure-safety)                        |
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
   | Repositories: uav-* (e.g. uav-tactical-mission, customer flight projects)         |
   | Role: DOWNSTREAM_APPLICATION_WORKSPACE / DOWNSTREAM_CUSTOMER_PROJECT             |
   | Content: Concrete proprietary flight code, ROS2 nodes, PX4 modules, SIL/HIL tests |
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

   #### Tier 3: Customer Application Workspaces (`uav-*`)
   - **Customer Application Workspaces (`uav-*`, `DOWNSTREAM_APPLICATION_WORKSPACE` / `DOWNSTREAM_CUSTOMER_PROJECT`):** Concrete engineering repositories where end-user engineering teams build proprietary software, ROS2 nodes, PX4 flight modules, and hardware-in-the-loop tests.
   - **Clean Customer Boundary Mandate:** Customer projects onboard strictly from their relevant **Domain Distribution Template** (`DEAP-*`), NOT from the upstream compiler (`DEAP01-spec-core`). Onboarding from the domain template guarantees that all domain-specific SysML v2 schemas, safety contracts, and profile rules are seeded directly into the customer workspace in a single self-contained command with zero external sibling path dependencies.
   - **Illustrative Schema Payloads:** Any concrete flight controller, robotic surgical console, satellite bus, AGV guidance, subsea vehicle, rail locomotive controller, STPA hazard analysis, or domain safety examples presented throughout this README and documentation are strictly **illustrative schema payloads** demonstrating compiler ingestion, AST synthesis, projection, and verification capabilities.
   ```

2. **Line 152**:
   - Change `#### Tier 1: Domain Distribution Template Repository (e.g. DEAP-uas-infrastructure-safety):` to:
     `#### Tier 2: Domain Distribution Template Repository (e.g. DEAP-uas-infrastructure-safety):`

3. **Line 175**:
   - Change `Tier 2 Customer Application Workspaces (`uav-*`) inherit this layout...` to:
     `Tier 3 Customer Application Workspaces (`uav-*`) inherit this layout...`

4. **Line 177**:
   - Change `### 4.1 Supported Tier 1 Domain Distribution Templates (Canonical Taxonomies)` to:
     `### 4.1 Supported Tier 2 Domain Distribution Templates (Canonical Taxonomies)`

5. **Lines 260–291 (Section 5.4 Normalization)**:
   Replace with:
   ```markdown
   ### 5.4 Architectural Boundary: Customer Project Onboarding

   The DEAP three-tier architecture strictly decouples compiler tooling propagation from customer project onboarding:
   - **Compiler Propagation (Tier 1 Compiler -> Tier 2 Domain Templates):** Maintainers propagate abstract compiler tooling and governance from `DEAP01-spec-core` to canonical sector distribution templates (`DEAP-*`).
   - **Customer Onboarding (Tier 2 Domain Templates -> Tier 3 Customer Workspaces):** Application engineers and customer teams onboard concrete project workspaces (`uav-*`) exclusively from the appropriate **Domain Distribution Template** (`DEAP-*`), never from `DEAP01-spec-core`.

   > **Critical Architecture Rule:** End-user customer projects (`uav-*`) clone from their domain distribution template (e.g. `DEAP-uas-infrastructure-safety`), NOT from `DEAP01-spec-core`. Bootstrapping from the domain template seeds the customer workspace with domain-specific SysML v2 schemas, safety statecharts, and platform execution profiles.

   ```text
   +-----------------------------------------------------------------------------------+
   | Tier 1: Upstream Specification Core Compiler                                      |
   | DEAP01-spec-core (UPSTREAM_SPEC_CORE_COMPILER)                                     |
   +-----------------------------------------------------------------------------------+
                                            │
                                            │  bash scripts/install_pipeline.sh "<path-to-domain-template>"
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
   | uav-* (DOWNSTREAM_CUSTOMER_PROJECT)                                               |
   | Contains: Concrete proprietary flight code, ROS2 nodes, PX4 modules, SIL/HIL tests |
   +-----------------------------------------------------------------------------------+
   ```

   For customer onboarding instructions and domain-specific schemas, refer to the documentation in the respective canonical domain distribution template repository (see [Section 4.1](#41-supported-tier-2-domain-distribution-templates-canonical-taxonomies)).
   ```

6. **Section 9.4 Preamble (Lines 1083–1086)**:
   Add explicit Execution Boundary Invariant admonition:
   ```markdown
   ### 9.4 Pipeline 2 Prompts (Autonomous Feature Implementation & Two-Path Simulation Driver)

   > **Execution Boundary Invariant:** Pipeline 2 prompts are strictly confined to downstream customer application workspaces (`DOWNSTREAM_CUSTOMER_PROJECT`, e.g. `uav-*`). Autonomous feature implementation, UI widgets (`app_flutter/`), real-time robotic nodes (`ros2/`, `px4/`), and digital twin simulation engines must NEVER be executed directly within the upstream specification compiler (`UPSTREAM_SPEC_CORE_COMPILER`), preserving the Upstream Clean Landing Zone and Pure Schema-Driven Compiler Invariants.

   Execute the following prompts within downstream customer application workspaces to drive feature implementation and two-path (dual-track) simulation verification through context-isolated TDD micro-tasks:
   ```

7. **Lines 1094, 1149, 1180 (Prompt Preambles)**:
   - Line 1094: Change `Repository Classification: DOWNSTREAM_CUSTOMER_PROJECT (or UPSTREAM_SPEC_CORE_COMPILER depending on execution context)` to:
     `Repository Classification: DOWNSTREAM_CUSTOMER_PROJECT`
   - Line 1149: Change `Repository Classification: DOWNSTREAM_CUSTOMER_PROJECT (or UPSTREAM_SPEC_CORE_COMPILER depending on execution context)` to:
     `Repository Classification: DOWNSTREAM_CUSTOMER_PROJECT`
   - Line 1180: Change `Repository Classification: UPSTREAM_SPEC_CORE_COMPILER (or DOWNSTREAM_CUSTOMER_PROJECT depending on execution context)` to:
     `Repository Classification: DOWNSTREAM_CUSTOMER_PROJECT`

#### File: `scripts/install_pipeline.sh`

1. **Line 882**:
   Replace `As a **Tier 1 Domain Distribution Template**` with `As a **Tier 2 Domain Distribution Template**`.
2. **Line 953**:
   Replace `As a **Tier 2 Customer Application Workspace**` with `As a **Tier 3 Customer Application Workspace**`.
3. **Lines 1430, 1484, 1516**:
   Replace `Repository Classification: DOWNSTREAM_CUSTOMER_PROJECT (or UPSTREAM_SPEC_CORE_COMPILER depending on execution context)` with:
   `Repository Classification: DOWNSTREAM_CUSTOMER_PROJECT`.

---

### 5.2 Remediation Specification for WP-03 (tests/test_readme_scaffolding.py)

#### File: `tests/test_readme_scaffolding.py`

1. **Lines 381 & 482 Docstrings**:
   - Line 381: Update docstring to: `"""Verifies that install_pipeline.sh properly scaffolds Tier 2 Domain Distribution Templates."""`
   - Line 482: Update docstring to: `"""Verifies that install_pipeline.sh properly scaffolds Tier 3 Customer Application Workspaces."""`

2. **Add Heading Order Assertion to `TestUpstreamCompilerReadme`**:
   ```python
   def test_upstream_readme_section_1_heading_order_and_hierarchy(self):
       """Verifies that Section 1.1 precedes Section 1.2 and both are valid H3 subsections of Section 1."""
       ast = parse_markdown_ast(self.content)
       sec1_nodes = ast.find_sections("1. System Overview")
       self.assertTrue(len(sec1_nodes) > 0, "Section 1 not found in README.md")

       lines = self.content.splitlines()
       sec_1_1_idx = None
       sec_1_2_idx = None
       for idx, line in enumerate(lines):
           if re.match(r'^###\s+1\.1\s+Primary Commercial Toolchain Integration', line):
               sec_1_1_idx = idx
           elif re.match(r'^###\s+1\.2\s+Three-Tier Architecture', line):
               sec_1_2_idx = idx

       self.assertIsNotNone(sec_1_1_idx, "### 1.1 Primary Commercial Toolchain Integration not found as H3")
       self.assertIsNotNone(sec_1_2_idx, "### 1.2 Three-Tier Architecture not found as H3")
       self.assertLess(
           sec_1_1_idx,
           sec_1_2_idx,
           f"Section 1.1 (line {sec_1_1_idx+1}) must precede Section 1.2 (line {sec_1_2_idx+1})"
       )
   ```

3. **Add Three-Tier Architecture Normalization Assertion to `TestUpstreamCompilerReadme`**:
   ```python
   def test_upstream_readme_three_tier_architecture_normalization(self):
       """Verifies that README.md cleanly defines three tiers and has 0 contradictory tier labels."""
       self.assertIn("### 1.2 Three-Tier Architecture & Repository Boundaries", self.content)
       self.assertIn("Tier 1: Upstream Specification Core Compiler", self.content)
       self.assertIn("Tier 2: Domain Distribution Templates", self.content)
       self.assertIn("Tier 3: Customer Application Workspaces", self.content)

       # Assert 0 contradictory duplicate Tier 1 labels for domain templates
       self.assertNotIn("Tier 1: Domain Distribution Templates", self.content)
       self.assertNotIn("Tier 1 Domain Distribution Template", self.content)

       # Assert 0 contradictory Tier 2 labels for customer workspaces
       self.assertNotIn("Tier 2: Customer Application Workspaces", self.content)
       self.assertNotIn("Tier 2 Customer Application Workspace", self.content)
   ```

4. **Add Pipeline 2 Execution Boundary Assertion to `TestUpstreamCompilerReadme`**:
   ```python
   def test_upstream_readme_pipeline_2_prompt_boundary_confinement(self):
       """Verifies that Section 9.4 strictly confines Pipeline 2 prompts to DOWNSTREAM_CUSTOMER_PROJECT."""
       ast = parse_markdown_ast(self.content)
       sec9_4_nodes = ast.find_sections("9.4 Pipeline 2 Prompts")
       self.assertTrue(len(sec9_4_nodes) > 0, "Section 9.4 not found in README.md")

       sec9_4_text = sec9_4_nodes[0].get_all_text()
       self.assertIn("Execution Boundary Invariant", sec9_4_text)
       self.assertNotIn("(or UPSTREAM_SPEC_CORE_COMPILER depending on execution context)", sec9_4_text)
       self.assertNotIn("Repository Classification: UPSTREAM_SPEC_CORE_COMPILER", sec9_4_text)
       self.assertIn("Repository Classification: DOWNSTREAM_CUSTOMER_PROJECT", sec9_4_text)
   ```

5. **Update Scaffolded Template Assertions in `TestDomainDistributionTemplateScaffolding` and `TestCustomerWorkspaceScaffolding`**:
   - In `TestDomainDistributionTemplateScaffolding.test_domain_template_scaffolding_explicit_role`:
     Assert `content` contains `As a **Tier 2 Domain Distribution Template**`.
     Assert `content` does NOT contain `Tier 1 Domain Distribution Template`.
   - In `TestCustomerWorkspaceScaffolding.test_customer_workspace_scaffolding_explicit_role`:
     Assert `content` contains `As a **Tier 3 Customer Application Workspace**`.
     Assert `content` does NOT contain `Tier 2 Customer Application Workspace`.

---

## 6. Verification Method

To independently reproduce the audit observations and verify the remediation:

1. **Verify Baseline Tests**:
   ```bash
   python3 -m unittest tests/test_readme_scaffolding.py
   python3 scripts/verify_downstream_baseline.py --no-domain
   python3 -m pytest tests/
   ```
2. **Inspect Pre-Remediation Heading Hierarchy**:
   ```bash
   grep -n -E '^(##|###) 1\.' README.md
   ```
   *Observation*: `### 1.2` appears at line 19; `## 1.1` appears at line 71.
3. **Inspect Pre-Remediation Contradictory Tier Labels**:
   ```bash
   grep -n -E 'Tier 1.*Domain Distribution|Tier 2.*Customer Application' README.md scripts/install_pipeline.sh
   ```
   *Observation*: Multiple matches across `README.md:17`, `34`, `55`, `152`, `175`, `177`, `277`, `285` and `scripts/install_pipeline.sh:882`, `953`.
4. **Inspect Pipeline 2 Preambles**:
   ```bash
   grep -n -C 2 "depending on execution context" README.md scripts/install_pipeline.sh
   ```
5. **Invalidation Conditions**:
   - If any change causes `tests/test_readme_scaffolding.py` or `scripts/verify_downstream_baseline.py` to exit non-zero.
   - If changing heading 4.1 in `README.md` breaks the anchor link on line 291.
   - If any commit message uses issue-closing keywords (`fixes #...`).

---

**Report Status**: COMPLETE & VERIFIED. Ready for WP-02 implementation.
