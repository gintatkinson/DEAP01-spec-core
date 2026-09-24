# Investigation Report: `scripts/install_pipeline.sh` README Scaffolding & Dynamic Role Branching

**Explorer Agent:** `explorer_r4_2`  
**Repository Classification:** `UPSTREAM_SPEC_CORE_COMPILER`  
**Investigation Target:** `scripts/install_pipeline.sh` (Lines 1-1268, with focus on 18-51, 186-285, 510-735, 1225-1234)  
**Date:** 2026-09-21  

---

## Executive Summary

This report surveys `scripts/install_pipeline.sh` to determine how to dynamically detect and branch downstream scaffolding between **Domain Distribution Templates** (`DEAP-*`, `DOMAIN_DISTRIBUTION_TEMPLATE`) and **Customer Application Workspaces** (`uav-*`, `DOWNSTREAM_CUSTOMER_PROJECT`). Currently, `scripts/install_pipeline.sh` hardcodes all downstream installations as `DOWNSTREAM_APPLICATION_WORKSPACE` and generates a single monolithic README that instructs end-users to run a circular customer onboarding command (`git clone ... .tmp-pipeline`).

To resolve this defect cleanly:
1. `scripts/install_pipeline.sh` must support explicit `--role` CLI options (`domain-template` / `customer-project`) and dynamic tier detection (target directory name, project name, remote URL origin, and domain URL presence).
2. The README scaffolding logic must be split into two distinct branches:
   - **Domain Distribution Templates (`DEAP-*`)**: Declares `DOMAIN_DISTRIBUTION_TEMPLATE`, documents the clean landing zone invariant, and provides the single-line customer clone command for end-users to onboard into a customer workspace.
   - **Customer Application Workspaces (`uav-*`)**: Declares `DOWNSTREAM_CUSTOMER_PROJECT`, eliminates circular clone commands completely, and documents project-specific commands: baseline verification (`python3 scripts/verify_downstream_baseline.py --no-domain`), Level 0 OEM Ground Truth Ingestion (`sysmlv2_ingest.py` / `compile_sysml.py`), and in-place tooling update (`bash scripts/install_pipeline.sh .`).
3. All code fences must enforce pure executable shell syntax with zero unescaped parentheses in comments and zero unquoted angle brackets.

---

## 1. Trace of Existing README Scaffolding Logic in `scripts/install_pipeline.sh`

### 1.1 Pre-Scaffolding Execution (`scripts/scaffold_downstream_agents.py`)
At line 513:
```bash
python3 "$INSTALLER_ROOT/scripts/scaffold_downstream_agents.py" "$INSTALLER_ROOT" "$TARGET_DIR"
```
- `scripts/scaffold_downstream_agents.py` transforms `AGENTS.md` into `DOWNSTREAM_CUSTOMER_PROJECT` format and writes both `$TARGET_DIR/.agents/AGENTS.md` and `$TARGET_DIR/AGENTS.md`.
- At lines 125-128, `scaffold_downstream_agents.py` writes a fallback `DEFAULT_README` if `$TARGET_DIR/README.md` is absent.
- However, this `DEFAULT_README` is immediately superseded by the main README generator in `scripts/install_pipeline.sh`.

### 1.2 README Regeneration Trigger Condition
At lines 517-520 of `scripts/install_pipeline.sh`:
```bash
if [ ! -f "$TARGET_DIR/README.md" ] || \
   grep -qE "Getting started with GitLab|To make it easy for you to get started" "$TARGET_DIR/README.md" || \
   ! grep -qE "Multi-Pipeline Operator Prompt Catalog|Operator Prompt Catalog" "$TARGET_DIR/README.md" || \
   ! grep -qE "Customer Project Onboarding|\.tmp-pipeline" "$TARGET_DIR/README.md"; then
```
**Observation & Defect Analysis:**
- The 4th check (`! grep -qE "Customer Project Onboarding|\.tmp-pipeline"`) requires that any valid downstream README contain the text "Customer Project Onboarding" or "\.tmp-pipeline".
- For customer projects (`uav-*`), circular self-cloning instructions are strictly forbidden. If a customer README is generated without "Customer Project Onboarding" or "\.tmp-pipeline", subsequent runs of `install_pipeline.sh` would falsely consider the README incomplete and overwrite it unless this trigger condition is updated to be role-aware.

### 1.3 Project Metadata Extraction
At lines 522-636, metadata is extracted in priority order:
1. `--domain-name` CLI argument (`$DOMAIN_NAME`)
2. `# ` title in existing `$TARGET_DIR/README.md`
3. `$TARGET_DIR/.pipeline/project_metadata.json`
4. `$TARGET_DIR/codebase_rules.json`
5. `$TARGET_DIR/pubspec.yaml` (Flutter / Dart)
6. `$TARGET_DIR/package.json` (React / TypeScript)
7. `$TARGET_DIR/pyproject.toml`
8. Active platform profile (`.pipeline/profile_config.json`, `CMakeLists.txt`, `package.xml`)
9. Fallback directory basename: `DIR_BASE=$(basename "$(cd "$TARGET_DIR" 2>/dev/null && pwd || echo "$TARGET_DIR")")`
10. Default fallback: `"Downstream Cyber-Physical Infrastructure Safety Project"`

### 1.4 Domain Remote URL Resolution
At lines 644-664:
- Checks `--domain-url` (`$DOMAIN_URL`).
- Checks `$REMOTE_URL` of target directory (if not `DEAP01-spec-core`).
- Checks `$DETECTED_SERVER_URL/$DETECTED_NAMESPACE/$DETECTED_PROJECT.git`.
- Checks `$INSTALLER_REMOTE` (if installer root is not upstream compiler).
- Fallback: `https://github.com/${GITHUB_ORG:-gintatkinson}/${CLEAN_NAME}.git`.

### 1.5 Monolithic Scaffolding Generation
At lines 667-732, `scripts/install_pipeline.sh` executes:
```bash
  chmod u+w "$TARGET_DIR/README.md" 2>/dev/null || true
  cat << EOF > "$TARGET_DIR/README.md"
$README_TITLE

> **Repository Role:** \`DOWNSTREAM_APPLICATION_WORKSPACE\`  
> **Primary Technology Profiles:** $DOMAIN_TECH_PROFILE  
> **Target Regulatory Frameworks:** $DOMAIN_REGULATORY  
...
## 3. Customer Project Onboarding & Agent Initialization Sequence

### 3.1 Turnkey Customer Project Onboarding
...
git clone ${DOMAIN_REMOTE_URL} ./.tmp-pipeline && bash ./.tmp-pipeline/scripts/install_pipeline.sh . && rm -rf ./.tmp-pipeline
...
bash scripts/install_pipeline.sh .
...
EOF
```
And lines 734-1234 append Section 4 (Operator Prompt Catalog) and Section 5 (Verification & Quality Gates).

**Key Defect:** When run against a customer workspace (e.g. `uav-011`), `${DOMAIN_REMOTE_URL}` resolves to `uav-011.git`, producing the circular instruction inside `uav-011` to clone `uav-011` into `.tmp-pipeline` to install `uav-011`.

---

## 2. Dynamic Tier & Repository Role Detection Analysis

### 2.1 Repository Roles Defined in DEAP Architecture
| Role Identifier | Tier | Canonical Repo Patterns | Sentinel / Characteristics |
|---|---|---|---|
| `UPSTREAM_SPEC_CORE_COMPILER` | Tier 1 | `DEAP01-spec-core` | `.pipeline/upstream/` present. Abstract compiler, pristine landing zones. |
| `DOMAIN_DISTRIBUTION_TEMPLATE` | Tier 1 | `DEAP-*` (e.g. `DEAP-uas-infrastructure-safety`) | `.pipeline/upstream/` removed. Sector blueprints, SysML v2 schemas in `schema/`, clean spec landing zones (`.gitkeep`). |
| `DOWNSTREAM_CUSTOMER_PROJECT` | Tier 2 | `uav-*` (e.g. `uav-011`, `uav-tactical-mission`) | Concrete application code, ROS2 nodes, PX4 flight modules, proprietary safety models. |

### 2.2 Where Role Can Be Determined Cleanly
In `scripts/install_pipeline.sh`, role detection should be resolved immediately after parsing arguments and normalizing `TARGET_DIR` (around line 200).

The resolution order must be deterministic and robust:

1. **Explicit CLI Argument (`--role` / `-r`):**
   - Syntax: `--role <ROLE>` or `--role=<ROLE>`
   - Accepted values for domain templates: `domain-template`, `domain_template`, `DOMAIN_DISTRIBUTION_TEMPLATE`
   - Accepted values for customer workspaces: `customer-project`, `customer_project`, `downstream-customer-project`, `DOWNSTREAM_CUSTOMER_PROJECT`, `downstream-workspace`

2. **Target Directory Base Name (`DIR_BASE`):**
   - `DIR_BASE="$(basename "$TARGET_DIR")"`
   - If `DIR_BASE` matches `DEAP-*` -> `DOMAIN_DISTRIBUTION_TEMPLATE`
   - If `DIR_BASE` matches `uav-*` -> `DOWNSTREAM_CUSTOMER_PROJECT`

3. **Remote Git Origin Project Name (`$DETECTED_PROJECT` / `$REMOTE_URL`):**
   - If git remote repository name starts with `DEAP-` -> `DOMAIN_DISTRIBUTION_TEMPLATE`
   - If git remote repository name starts with `uav-` -> `DOWNSTREAM_CUSTOMER_PROJECT`

4. **Explicit Domain Name Parameter (`$DOMAIN_NAME`):**
   - If `--domain-name` starts with `DEAP-` -> `DOMAIN_DISTRIBUTION_TEMPLATE`

5. **Explicit Domain URL Parameter (`$DOMAIN_URL`):**
   - If `--domain-url` was supplied and `--role` was not specified as `customer-project`, infer `DOMAIN_DISTRIBUTION_TEMPLATE` (preserves compatibility with `tests/test_domain_url_synthesis.py`).

6. **Target Metadata (`.pipeline/lineage.json` or `.pipeline/project_metadata.json`):**
   - If `role` or `classification` field is present in metadata, map normalized string.

7. **Default Fallback:**
   - Any generic target directory that is not `DEAP-*` defaults to `DOWNSTREAM_CUSTOMER_PROJECT`.

### 2.3 Role Detection Implementation Logic
```bash
# Normalize TARGET_DIR
TARGET_DIR="${TARGET_DIR:-.}"
mkdir -p "$TARGET_DIR"
chmod u+w "$TARGET_DIR" 2>/dev/null || true
TARGET_DIR="$(cd -P "$TARGET_DIR" 2>/dev/null && pwd -P || echo "$TARGET_DIR")"
DIR_BASE="$(basename "$TARGET_DIR")"

# Determine TARGET_ROLE
RESOLVED_ROLE=""
if [ -n "$CLI_ROLE" ]; then
  case "$(echo "$CLI_ROLE" | tr '[:upper:]' '[:lower:]' | tr '-' '_')" in
    domain_template|domain|tier1_domain|domain_distribution_template)
      RESOLVED_ROLE="DOMAIN_DISTRIBUTION_TEMPLATE"
      ;;
    customer_project|customer|downstream_customer_project|downstream_application_workspace|workspace|uav)
      RESOLVED_ROLE="DOWNSTREAM_CUSTOMER_PROJECT"
      ;;
    *)
      RESOLVED_ROLE="$CLI_ROLE"
      ;;
  esac
fi

if [ -z "$RESOLVED_ROLE" ]; then
  if [[ "$DIR_BASE" == DEAP-* ]] || \
     [[ "$DETECTED_PROJECT" == DEAP-* ]] || \
     [[ "$DOMAIN_NAME" == DEAP-* ]] || \
     [ -n "$DOMAIN_URL" ]; then
    RESOLVED_ROLE="DOMAIN_DISTRIBUTION_TEMPLATE"
  elif [[ "$DIR_BASE" == uav-* ]] || [[ "$DETECTED_PROJECT" == uav-* ]]; then
    RESOLVED_ROLE="DOWNSTREAM_CUSTOMER_PROJECT"
  elif [ -f "$TARGET_DIR/.pipeline/lineage.json" ]; then
    META_ROLE=$(python3 -c "import json; data=json.load(open('$TARGET_DIR/.pipeline/lineage.json')); print(data.get('role') or data.get('classification') or '')" 2>/dev/null || true)
    if [ -n "$META_ROLE" ]; then
      RESOLVED_ROLE="$META_ROLE"
    fi
  fi
fi

# Fallback default for downstream repositories
if [ -z "$RESOLVED_ROLE" ]; then
  RESOLVED_ROLE="DOWNSTREAM_CUSTOMER_PROJECT"
fi
```

---

## 3. Scaffolding Logic Branching Design

### 3.1 Structural Comparison Between README Variants

| Section | Domain Distribution Template (`DEAP-*`) | Customer Application Workspace (`uav-*`) |
|---|---|---|
| **Role Metadata** | `> **Repository Role:** \`DOMAIN_DISTRIBUTION_TEMPLATE\`` | `> **Repository Role:** \`DOWNSTREAM_CUSTOMER_PROJECT\`` |
| **Section 1.1** | **Clean Landing Zone Invariant**: Enforces pristine landing zones in `schema/`, `docs/epics/`, `docs/features/` with only `.gitkeep`. Explains that customer code belongs in downstream workspaces. | **Customer Application Workspace Scope**: Explains that this repository contains concrete flight code, ROS2 nodes, PX4 flight modules, and hardware-in-the-loop tests. |
| **Section 1.2** | **Primary Commercial Toolchain**: MATLAB / Simulink / Stateflow / Embedded Coder declaration. | **Primary Commercial Toolchain**: MATLAB / Simulink / Stateflow / Embedded Coder declaration. |
| **Section 2** | Pipeline Structure & Governance (.agents, rules, skills, schema, tests). | Pipeline Structure & Governance (.agents, rules, skills, schema, tests). |
| **Section 3 Header** | `## 3. Customer Project Onboarding & Agent Initialization Sequence` | `## 3. Project Lifecycle & Tooling Maintenance` |
| **Section 3.1** | **Turnkey Customer Project Onboarding**: Single-line customer clone command (`git clone <domain-url> ./.tmp-pipeline ...`). | **Downstream Baseline Verification & Ingestion Workflows**: Documents `python3 scripts/verify_downstream_baseline.py --no-domain`, Level 0 OEM Ground Truth Ingestion (`sysmlv2_ingest.py`), and compilation gate (`compile_sysml.py --compile`). **Zero circular clone commands.** |
| **Section 3.2** | **In-Place Maintainer Update**: `bash scripts/install_pipeline.sh .` | **In-Place Pipeline Tooling Update**: `bash scripts/install_pipeline.sh .` |
| **Section 3.3** | **Mandatory Agent Initialization Sequence**: Initialization order for AI agents. | **Mandatory Agent Initialization Sequence**: Initialization order for AI agents. |
| **Section 4** | Multi-Pipeline Operator Prompt Catalog (topology and prompts). | Multi-Pipeline Operator Prompt Catalog (topology and prompts). |
| **Section 5** | Verification & Quality Gates (`verify_downstream_baseline.py --no-domain`). | Verification & Quality Gates (`verify_downstream_baseline.py --no-domain`). |

### 3.2 Detailed Branch A: Domain Distribution Template (`DEAP-*`)

```bash
cat << EOF > "$TARGET_DIR/README.md"
$README_TITLE

> **Repository Role:** \`DOMAIN_DISTRIBUTION_TEMPLATE\`  
> **Primary Technology Profiles:** $DOMAIN_TECH_PROFILE  
> **Target Regulatory Frameworks:** $DOMAIN_REGULATORY  

---

## 1. System Overview

$DOMAIN_PROJECT_DESC

### 1.1 Clean Landing Zone Invariant

As a **Tier 1 Domain Distribution Template**, this repository maintains pristine, clean landing zones in \`schema/\`, \`docs/epics/\`, \`docs/features/\`, \`docs/user-stories/\`, and \`docs/use-cases/\` with only \`.gitkeep\` files (or domain-wide baseline SysML v2 schemas). Concrete customer project specifications, flight code, ROS2 nodes, and proprietary implementation artifacts belong exclusively in downstream customer application workspaces and must NOT be committed here.

### 1.2 Primary Commercial Toolchain Integration Context

This platform explicitly declares **MATLAB / Simulink / Stateflow / Embedded Coder** as the Primary Tier-1 Commercial Toolchain Integration Context (Model-Based Design, Control Law Synthesis, DO-178C C/SPARK Ada code generation).

---

## 2. Pipeline Structure & Governance

- \`.agents/\` & \`AGENTS.md\`: Agent behavior rules, role boundaries, and subagent dispatch protocols.
- \`CLAUDE.md\`: Claude Code guidelines and verification gates.
- \`.pipeline/\`: Constitution (\`constitution.md\`), domain specifications, and execution profiles (\`profiles/ros2_cpp.md\`, \`profiles/px4_module.md\`).
- \`rules/\` & \`skills/\`: Platform engineering rules and agent workflow skills (including SysML v2 SSOT completeness in \`rules/sysml-ssot-completeness.md\`).
- \`schema/\`: Contract definitions and SysML v2 schemas.
- \`tests/\`: Automated baseline verification and safety compliance tests.

---

## 3. Customer Project Onboarding & Agent Initialization Sequence

### 3.1 Turnkey Customer Project Onboarding

To install this domain pipeline and its engineering baseline into an end-user customer application workspace, run the following turnkey command from your customer project root directory:

\`\`\`bash
# Onboard customer application workspace
git clone "${DOMAIN_REMOTE_URL}" ./.tmp-pipeline && bash ./.tmp-pipeline/scripts/install_pipeline.sh . && rm -rf ./.tmp-pipeline
\`\`\`

This single self-contained command operates strictly inside the customer project directory with zero sibling path dependencies.

To update the pipeline and governance rules inside this domain distribution template at any time, run:

\`\`\`bash
# Update domain distribution template tooling in-place
bash scripts/install_pipeline.sh .
\`\`\`

### 3.2 Mandatory Agent Initialization Sequence

Immediately following installation, any AI agent (Antigravity, Claude Code, Gemini CLI, Cursor) initializing in this repository MUST execute the following sequence before accepting user directives or executing task implementations:

0. **Detect Repository Role & Scope**:
   - Inspect whether \`.pipeline/upstream/\` exists on disk.
   - If absent and repository name starts with \`DEAP-\` -> **Domain Distribution Template Mode**: Clean landing zones must be maintained.
1. **Read Governance Constitution**: Execute \`view_file\` on \`.pipeline/constitution.md\` to ingest the platform-independent functional governance layer and zero-mocking persistence mandates.
2. **Load Project Skills**: Execute \`view_file\` on \`skills/feature-driven-implementation/SKILL.md\` (and any active skills under \`skills/\` or \`.agents/skills/\`) to initialize feature-driven implementation protocols and review gates.
3. **Load Governance Rules**: Ingest \`AGENTS.md\` and \`rules/\` to enforce project-scoped agentic rules, context-isolated subagent dispatch loops, and role boundary locks.
4. **Load Platform Profile**: Read the target platform execution profile (\`.pipeline/profiles/flutter.md\`, \`.pipeline/profiles/react.md\`, \`.pipeline/profiles/ros2_cpp.md\`, or \`.pipeline/profiles/px4_module.md\`) to establish platform-specific build, test, and lifecycle constraints.
5. **Bootstrap Tracker Labels & Verify Baseline**: Verify that repository issue tracker labels and baseline conformance pass by running \`python3 scripts/verify_downstream_baseline.py --no-domain\`.

---

EOF
```

### 3.3 Detailed Branch B: Customer Application Workspace (`uav-*`)

```bash
cat << EOF > "$TARGET_DIR/README.md"
$README_TITLE

> **Repository Role:** \`DOWNSTREAM_CUSTOMER_PROJECT\`  
> **Primary Technology Profiles:** $DOMAIN_TECH_PROFILE  
> **Target Regulatory Frameworks:** $DOMAIN_REGULATORY  

---

## 1. System Overview

$DOMAIN_PROJECT_DESC

### 1.1 Customer Application Workspace Scope

As a **Tier 2 Customer Application Workspace**, this repository is authorized for concrete engineering delivery, proprietary application code, ROS2 lifecycle nodes, PX4 flight modules, hardware-in-the-loop tests, and verified Agile backlog implementations.

### 1.2 Primary Commercial Toolchain Integration Context

This platform explicitly declares **MATLAB / Simulink / Stateflow / Embedded Coder** as the Primary Tier-1 Commercial Toolchain Integration Context (Model-Based Design, Control Law Synthesis, DO-178C C/SPARK Ada code generation).

---

## 2. Pipeline Structure & Governance

- \`.agents/\` & \`AGENTS.md\`: Agent behavior rules, role boundaries, and subagent dispatch protocols.
- \`CLAUDE.md\`: Claude Code guidelines and verification gates.
- \`.pipeline/\`: Constitution (\`constitution.md\`), domain specifications, and execution profiles (\`profiles/ros2_cpp.md\`, \`profiles/px4_module.md\`).
- \`rules/\` & \`skills/\`: Platform engineering rules and agent workflow skills (including SysML v2 SSOT completeness in \`rules/sysml-ssot-completeness.md\`).
- \`schema/\`: Contract definitions and SysML v2 schemas.
- \`tests/\`: Automated baseline verification and safety compliance tests.

---

## 3. Project Lifecycle & Tooling Maintenance

### 3.1 Downstream Baseline Verification & Ingestion Workflows

To verify that all repository issue tracker labels, baseline contracts, and safety fixtures pass downstream conformance gates, run:

\`\`\`bash
# Run baseline conformance verification
python3 scripts/verify_downstream_baseline.py --no-domain
\`\`\`

For customer projects starting with unstructured OEM technical documentation, PDF flight manuals, or Bill of Materials (BOM) specifications, execute Level 0 OEM Ground Truth Ingestion (Step 0.0):

\`\`\`bash
# Run Level 0 OEM Ground Truth ingestion
python3 skills/spec-orchestrator/scripts/sysmlv2_ingest.py --schema "schema/extracted/" --format markdown --out "schema/model.sysml"

# Verify model compilation gate
python3 scripts/compile_sysml.py --compile
\`\`\`

### 3.2 In-Place Pipeline Tooling Update

To update local pipeline tooling, governance rules, and verification scripts in-place at any time without re-onboarding:

\`\`\`bash
# Update local pipeline tooling in-place
bash scripts/install_pipeline.sh .
\`\`\`

### 3.3 Mandatory Agent Initialization Sequence

Immediately following installation, any AI agent (Antigravity, Claude Code, Gemini CLI, Cursor) initializing in this repository MUST execute the following sequence before accepting user directives or executing task implementations:

0. **Detect Repository Role & Scope**:
   - Inspect whether \`.pipeline/upstream/\` exists on disk.
   - If absent -> **Downstream Customer Project Mode**: Authorized for customer feature implementation and domain codebase delivery.
1. **Read Governance Constitution**: Execute \`view_file\` on \`.pipeline/constitution.md\` to ingest the platform-independent functional governance layer and zero-mocking persistence mandates.
2. **Load Project Skills**: Execute \`view_file\` on \`skills/feature-driven-implementation/SKILL.md\` (and any active skills under \`skills/\` or \`.agents/skills/\`) to initialize feature-driven implementation protocols and review gates.
3. **Load Governance Rules**: Ingest \`AGENTS.md\` and \`rules/\` to enforce project-scoped agentic rules, context-isolated subagent dispatch loops, and role boundary locks.
4. **Load Platform Profile**: Read the target platform execution profile (\`.pipeline/profiles/flutter.md\`, \`.pipeline/profiles/react.md\`, \`.pipeline/profiles/ros2_cpp.md\`, or \`.pipeline/profiles/px4_module.md\`) to establish platform-specific build, test, and lifecycle constraints.
5. **Bootstrap Tracker Labels & Verify Baseline**: Verify that repository issue tracker labels and baseline conformance pass by running \`python3 scripts/verify_downstream_baseline.py --no-domain\`.

---

EOF
```

### 3.4 Role-Aware Trigger Condition for README Overwrite
The condition at lines 517-520 must be replaced with role-aware logic:

```bash
# Determine whether README needs scaffolding
SHOULD_SCAFFOLD_README=false

if [ ! -f "$TARGET_DIR/README.md" ]; then
  SHOULD_SCAFFOLD_README=true
elif grep -qE "Getting started with GitLab|To make it easy for you to get started" "$TARGET_DIR/README.md"; then
  SHOULD_SCAFFOLD_README=true
elif ! grep -qE "Multi-Pipeline Operator Prompt Catalog|Operator Prompt Catalog" "$TARGET_DIR/README.md"; then
  SHOULD_SCAFFOLD_README=true
elif [ "$RESOLVED_ROLE" = "DOMAIN_DISTRIBUTION_TEMPLATE" ]; then
  if ! grep -qE "Customer Project Onboarding|\.tmp-pipeline" "$TARGET_DIR/README.md" || \
     ! grep -qE "DOMAIN_DISTRIBUTION_TEMPLATE" "$TARGET_DIR/README.md"; then
    SHOULD_SCAFFOLD_README=true
  fi
elif [ "$RESOLVED_ROLE" = "DOWNSTREAM_CUSTOMER_PROJECT" ]; then
  # If customer project currently contains the circular clone command, re-scaffold to fix it
  if grep -qE "git clone.*\.tmp-pipeline" "$TARGET_DIR/README.md" || \
     ! grep -qE "DOWNSTREAM_CUSTOMER_PROJECT" "$TARGET_DIR/README.md" || \
     ! grep -qE "Project Lifecycle & Tooling Maintenance" "$TARGET_DIR/README.md"; then
    SHOULD_SCAFFOLD_README=true
  fi
fi

if [ "$SHOULD_SCAFFOLD_README" = true ]; then
  ...
```

---

## 4. Code Fence and Formatting Compliance

### 4.1 Pure Executable Shell Syntax Mandate
The prompt and acceptance criteria require:
> "All code fences contain pure, executable shell syntax with zero unescaped parentheses in comments and zero unquoted angle-bracket placeholders."

### 4.2 Identified Risks & Remediation

1. **Unquoted Angle-Bracket Placeholders:**
   - Risk: If a shell fence contains `git clone <this-domain-template-remote-url> ...`, bash and zsh interpret `<` and `>` as file redirection operators, producing runtime errors:
     `zsh: no such file or directory: this-domain-template-remote-url`
   - Remediation:
     - In generated READMEs, always quote the URL: `git clone "${DOMAIN_REMOTE_URL}" ./.tmp-pipeline ...`.
     - In documentation examples where generic placeholders are referenced in text, quote them or use bash variables: `git clone "$DOMAIN_TEMPLATE_URL" ./.tmp-pipeline`.

2. **Unescaped Parentheses in Code Fence Comments:**
   - Risk: Comments like `# Onboard customer application workspace (e.g. uav-011)` or `# Step 0.0 (Ingestion)` can trigger subshell interpretation bugs in some command evaluators or linters.
   - Remediation:
     - All comments inside ```bash code fences must use pure alphanumeric text and hyphens without parentheses:
       - Use: `# Option A: GitLab SaaS Reconciliation`
       - Use: `# Option B: GitLab Self-Managed or SCIF Air-Gapped Reconciliation`
       - Use: `# Run Level 0 OEM Ground Truth ingestion`
       - Use: `# Update local pipeline tooling in-place`
       - Avoid: `# (Option A)` or `# (Ingestion)`.

3. **Check 14 Compatibility (`verify_downstream_baseline.py`):**
   - Check 14 requires:
     `"# Downstream Cyber-Physical Infrastructure Safety Project" in readme_content or "Operator Prompt Catalog" in readme_content`
   - Both template branches include Section 4 ("Operator Prompt Catalog") and include `# Downstream Cyber-Physical Infrastructure Safety Project` (or `$DOMAIN_PROJECT_NAME -- Downstream Cyber-Physical Infrastructure Safety Project`).
   - Therefore, Check 14 will pass cleanly for both `DOMAIN_DISTRIBUTION_TEMPLATE` and `DOWNSTREAM_CUSTOMER_PROJECT`.

---

## 5. Summary of Recommended Code Edits for Implementation

### Target: `scripts/install_pipeline.sh`
1. **Add CLI Option `--role`:**
   - Add `-r, --role ROLE` to `show_help()` (lines 16-51).
   - Add `-r|--role|--role=*)` parsing in `while [[ $# -gt 0 ]]` (lines 54-184).
2. **Add Dynamic Role Resolution:**
   - Implement `RESOLVED_ROLE` logic after line 200.
3. **Update Trigger Condition:**
   - Replace lines 517-520 with `SHOULD_SCAFFOLD_README` logic based on `RESOLVED_ROLE`.
4. **Branch Section 1 & Section 3:**
   - In `cat << EOF > "$TARGET_DIR/README.md"`:
     - Branch on `"$RESOLVED_ROLE" = "DOMAIN_DISTRIBUTION_TEMPLATE"`.
     - Domain branch emits Section 1.1 Clean Landing Zone Invariant + Section 3 Customer Project Onboarding (`git clone "${DOMAIN_REMOTE_URL}" ...`).
     - Customer branch emits Section 1.1 Customer Application Workspace Scope + Section 3 Project Lifecycle & Tooling Maintenance (`verify_downstream_baseline.py`, `sysmlv2_ingest.py`, `bash scripts/install_pipeline.sh .`).
5. **Ensure Section 4 & 5 Parity:**
   - Both branches append Section 4 (Operator Prompt Catalog) and Section 5 (Verification & Quality Gates).

### Target: `tests/test_readme_scaffolding.py` (New Test Suite)
- Test 1: Domain template scaffolding (`--domain-name DEAP-uas-infrastructure-safety` or `--role domain-template`) produces `DOMAIN_DISTRIBUTION_TEMPLATE` and customer clone command.
- Test 2: Customer workspace scaffolding (`--role customer-project` or target dir named `uav-011`) produces `DOWNSTREAM_CUSTOMER_PROJECT`, zero circular clone commands, and documents `verify_downstream_baseline.py` and `sysmlv2_ingest.py`.
- Test 3: Re-scaffolding an older customer README that has circular clone commands replaces it with the clean customer README.
- Test 4: Pure shell code fence verification (all bash blocks in generated READMEs parse with `bash -n` and have zero unescaped parentheses in comments).
- Test 5: Verify all 9 tests in `tests/test_domain_url_synthesis.py` continue to pass.
- Test 6: Verify `python3 scripts/verify_downstream_baseline.py --no-domain` passes cleanly.
