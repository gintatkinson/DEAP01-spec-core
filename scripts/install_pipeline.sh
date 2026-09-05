#!/usr/bin/env bash
set -e

INSTALLER_ROOT="$(cd -P "$(dirname "${BASH_SOURCE[0]}")/.." && pwd -P)"
TARGET_DIR=""
PROVIDER="auto"
GITLAB_URL="https://gitlab.com"
GITLAB_GROUP=""
JIRA_URL="https://your-domain.atlassian.net"
JIRA_PROJECT=""
JIRA_EMAIL=""

show_help() {
  cat << 'EOF'
Usage: install_pipeline.sh [OPTIONS] [TARGET_DIR]

Installs the DEAP safety-critical engineering pipeline and governance baseline into a downstream project repository.

Primary Commercial Toolchain Integration Context:
  MATLAB / Simulink / Stateflow / Embedded Coder (Model-Based Design, Control Law Synthesis, DO-178C C/SPARK Ada code generation).

Arguments:
  TARGET_DIR                 Target project directory (default: current directory '.')

Options:
  -p, --provider PROVIDER    Target issue tracker and CI/CD provider: 'github', 'gitlab', 'jira', or 'auto' (default: 'auto')
  -t, --tracker TRACKER      Alias for --provider: 'github', 'gitlab', 'jira', or 'auto'
      --gitlab-url URL       GitLab instance base URL (default: 'https://gitlab.com')
      --gitlab-group GROUP   GitLab namespace/group path (e.g. 'uas-safety')
      --jira-url URL         Jira instance base URL (default: 'https://your-domain.atlassian.net')
      --jira-project PROJECT Jira project key code (e.g. 'UAS')
      --jira-email EMAIL     Jira account email address (for Jira Cloud Basic Auth)
  -h, --help                 Display this help documentation and exit

Examples:
  ./scripts/install_pipeline.sh /path/to/downstream-project
  ./scripts/install_pipeline.sh --provider gitlab --gitlab-url https://gitlab.internal.defense.gov /path/to/project
  ./scripts/install_pipeline.sh --tracker jira --jira-url https://my-org.atlassian.net --jira-project UAS /path/to/project
  ./scripts/install_pipeline.sh --provider github .
EOF
}

# Parse CLI options and arguments
while [[ $# -gt 0 ]]; do
  case "$1" in
    -h|--help)
      show_help
      exit 0
      ;;
    -p|--provider|-t|--tracker)
      if [[ -z "$2" || "$2" == -* ]]; then
        echo "Error: $1 requires an argument ('github', 'gitlab', 'jira', or 'auto')." >&2
        exit 1
      fi
      PROVIDER="$2"
      shift 2
      ;;
    --provider=*|--tracker=*)
      PROVIDER="${1#*=}"
      shift
      ;;
    --gitlab-url)
      if [[ -z "$2" || "$2" == -* ]]; then
        echo "Error: --gitlab-url requires a URL argument." >&2
        exit 1
      fi
      GITLAB_URL="$2"
      shift 2
      ;;
    --gitlab-url=*)
      GITLAB_URL="${1#*=}"
      shift
      ;;
    --gitlab-group)
      if [[ -z "$2" || "$2" == -* ]]; then
        echo "Error: --gitlab-group requires a group/namespace argument." >&2
        exit 1
      fi
      GITLAB_GROUP="$2"
      shift 2
      ;;
    --gitlab-group=*)
      GITLAB_GROUP="${1#*=}"
      shift
      ;;
    --jira-url)
      if [[ -z "$2" || "$2" == -* ]]; then
        echo "Error: --jira-url requires a URL argument." >&2
        exit 1
      fi
      JIRA_URL="$2"
      shift 2
      ;;
    --jira-url=*)
      JIRA_URL="${1#*=}"
      shift
      ;;
    --jira-project)
      if [[ -z "$2" || "$2" == -* ]]; then
        echo "Error: --jira-project requires a project key argument." >&2
        exit 1
      fi
      JIRA_PROJECT="$2"
      shift 2
      ;;
    --jira-project=*)
      JIRA_PROJECT="${1#*=}"
      shift
      ;;
    --jira-email)
      if [[ -z "$2" || "$2" == -* ]]; then
        echo "Error: --jira-email requires an email argument." >&2
        exit 1
      fi
      JIRA_EMAIL="$2"
      shift 2
      ;;
    --jira-email=*)
      JIRA_EMAIL="${1#*=}"
      shift
      ;;
    -*)
      echo "Error: Unknown option: $1" >&2
      show_help >&2
      exit 1
      ;;
    *)
      if [[ -z "$TARGET_DIR" ]]; then
        TARGET_DIR="$1"
      else
        echo "Error: Unexpected positional argument: $1" >&2
        show_help >&2
        exit 1
      fi
      shift
      ;;
  esac
done

TARGET_DIR="${TARGET_DIR:-.}"

if [[ "$PROVIDER" != "auto" && "$PROVIDER" != "github" && "$PROVIDER" != "gitlab" && "$PROVIDER" != "jira" ]]; then
  echo "Error: Invalid provider '$PROVIDER'. Must be one of 'github', 'gitlab', 'jira', or 'auto'." >&2
  exit 1
fi

mkdir -p "$TARGET_DIR"
TARGET_DIR="$(cd -P "$TARGET_DIR" 2>/dev/null && pwd -P || echo "$TARGET_DIR")"

if [ "$TARGET_DIR" = "$INSTALLER_ROOT" ]; then
  if [ -e "$INSTALLER_ROOT/.pipeline/upstream" ]; then
    echo "REFUSING: target is the pipeline repository itself, not a downstream project." >&2
  else
    echo "REFUSING: target directory is identical to installer root ($INSTALLER_ROOT)." >&2
  fi
  exit 1
fi

rm -rf "$TARGET_DIR/skills" "$TARGET_DIR/rules" "$TARGET_DIR/.pipeline" "$TARGET_DIR/.agents" "$TARGET_DIR/scripts"
cp -RP "$INSTALLER_ROOT/skills" "$TARGET_DIR/"
cp -RP "$INSTALLER_ROOT/rules" "$TARGET_DIR/"
cp -RP "$INSTALLER_ROOT/.pipeline" "$TARGET_DIR/"
rm -rf "$TARGET_DIR/.pipeline/upstream"
rm -rf "$TARGET_DIR/.pipeline/diagnostics"
cp -RP "$INSTALLER_ROOT/.agents" "$TARGET_DIR/"
cp -RP "$INSTALLER_ROOT/scripts" "$TARGET_DIR/"
if [ ! -e "$TARGET_DIR/schema" ]; then
  if [ -d "$INSTALLER_ROOT/schema" ]; then
    cp -RP "$INSTALLER_ROOT/schema" "$TARGET_DIR/"
  else
    mkdir -p "$TARGET_DIR/schema"
  fi
fi
cp -P "$INSTALLER_ROOT/requirements.txt" "$TARGET_DIR/" 2>/dev/null || true
cp -P "$INSTALLER_ROOT/pyproject.toml" "$TARGET_DIR/" 2>/dev/null || true
if [ -f "$TARGET_DIR/.gitignore" ]; then
  cat "$INSTALLER_ROOT/.gitignore" >> "$TARGET_DIR/.gitignore"
  # Deduplicate lines in .gitignore
  sort -u "$TARGET_DIR/.gitignore" -o "$TARGET_DIR/.gitignore"
elif [ -f "$INSTALLER_ROOT/.gitignore" ]; then
  cp "$INSTALLER_ROOT/.gitignore" "$TARGET_DIR/"
fi

if [ ! -e "$TARGET_DIR/schema" ]; then
  mkdir -p "$TARGET_DIR/schema"
fi
mkdir -p "$TARGET_DIR/tests"
cp -RP "$INSTALLER_ROOT/tests/test_baseline.py" "$TARGET_DIR/tests/" 2>/dev/null || true
cp -RP "$INSTALLER_ROOT/tests/test_safety_integrity.py" "$TARGET_DIR/tests/" 2>/dev/null || true
cp -RP "$INSTALLER_ROOT/tests/test_gitlab_provider.py" "$TARGET_DIR/tests/" 2>/dev/null || true
cp -RP "$INSTALLER_ROOT/tests/test_jira_provider.py" "$TARGET_DIR/tests/" 2>/dev/null || true
cp -RP "$INSTALLER_ROOT/tests/test_ground_truth_tooling.py" "$TARGET_DIR/tests/" 2>/dev/null || true
cp -RP "$INSTALLER_ROOT/tests/fixtures" "$TARGET_DIR/tests/" 2>/dev/null || true
mkdir -p "$TARGET_DIR/docs" "$TARGET_DIR/docs/conops" "$TARGET_DIR/docs/safety" "$TARGET_DIR/docs/architecture/blueprints" "$TARGET_DIR/docs/epics" "$TARGET_DIR/docs/features" "$TARGET_DIR/docs/user-stories" "$TARGET_DIR/docs/use-cases"
if [ -f "$INSTALLER_ROOT/docs/conops/README.md" ]; then
  cp -P "$INSTALLER_ROOT/docs/conops/README.md" "$TARGET_DIR/docs/conops/"
fi
if [ -f "$INSTALLER_ROOT/docs/safety/README.md" ]; then
  cp -P "$INSTALLER_ROOT/docs/safety/README.md" "$TARGET_DIR/docs/safety/"
fi
if [ -f "$INSTALLER_ROOT/docs/OPERATOR_PROMPT_CATALOG.md" ]; then
  cp -P "$INSTALLER_ROOT/docs/OPERATOR_PROMPT_CATALOG.md" "$TARGET_DIR/docs/"
fi
if [ -f "$INSTALLER_ROOT/docs/JIRA_INTEGRATION_GUIDE.md" ]; then
  cp -P "$INSTALLER_ROOT/docs/JIRA_INTEGRATION_GUIDE.md" "$TARGET_DIR/docs/"
fi
mkdir -p "$TARGET_DIR/.pipeline/contracts" "$TARGET_DIR/.pipeline/domain_specs" "$TARGET_DIR/.pipeline/profiles"
chmod +x "$TARGET_DIR"/scripts/*.sh "$TARGET_DIR"/scripts/*.py 2>/dev/null || true

# Apply provider configurations if specified
if [ "$PROVIDER" = "gitlab" ] || [ -n "$GITLAB_GROUP" ] || [ "$GITLAB_URL" != "https://gitlab.com" ]; then
  if [ -f "$INSTALLER_ROOT/.pipeline/templates/.gitlab-ci.yml" ]; then
    cp -P "$INSTALLER_ROOT/.pipeline/templates/.gitlab-ci.yml" "$TARGET_DIR/.gitlab-ci.yml"
  elif [ -f "$INSTALLER_ROOT/.pipeline/.gitlab-ci.yml" ]; then
    cp -P "$INSTALLER_ROOT/.pipeline/.gitlab-ci.yml" "$TARGET_DIR/.gitlab-ci.yml"
  elif [ -f "$TARGET_DIR/.pipeline/templates/.gitlab-ci.yml" ]; then
    cp -P "$TARGET_DIR/.pipeline/templates/.gitlab-ci.yml" "$TARGET_DIR/.gitlab-ci.yml"
  elif [ -f "$TARGET_DIR/.pipeline/.gitlab-ci.yml" ]; then
    cp -P "$TARGET_DIR/.pipeline/.gitlab-ci.yml" "$TARGET_DIR/.gitlab-ci.yml"
  fi
  for rules_file in "$TARGET_DIR/.pipeline/logical-ui/codebase_rules.json" "$TARGET_DIR/codebase_rules.json"; do
    if [ -f "$rules_file" ]; then
      python3 -c "
import json, sys
path = '$rules_file'
with open(path, 'r', encoding='utf-8') as f:
    data = json.load(f)
if 'tracker_rules' not in data:
    data['tracker_rules'] = {}
if '$PROVIDER' != 'auto':
    data['tracker_rules']['provider'] = '$PROVIDER'
if '$PROVIDER' == 'gitlab':
    data['tracker_rules']['labels'] = {
        'epic': 'type::epic',
        'feature': 'type::feature',
        'user_story': 'type::user-story',
        'use_case': 'type::use-case',
        'ready_for_review': 'status::ready-for-review',
        'resolved': 'status::fixed-resolved'
    }
if '$GITLAB_URL':
    data['tracker_rules']['server_url'] = '$GITLAB_URL'
if '$GITLAB_GROUP':
    data['tracker_rules']['group'] = '$GITLAB_GROUP'
with open(path, 'w', encoding='utf-8') as f:
    json.dump(data, f, indent=2)
" 2>/dev/null || true
    fi
  done
elif [ "$PROVIDER" = "jira" ] || [ -n "$JIRA_PROJECT" ] || [ -n "$JIRA_EMAIL" ] || [ "$JIRA_URL" != "https://your-domain.atlassian.net" ]; then
  for rules_file in "$TARGET_DIR/.pipeline/logical-ui/codebase_rules.json" "$TARGET_DIR/codebase_rules.json"; do
    if [ -f "$rules_file" ]; then
      python3 -c "
import json, sys
path = '$rules_file'
with open(path, 'r', encoding='utf-8') as f:
    data = json.load(f)
if 'tracker_rules' not in data:
    data['tracker_rules'] = {}
if '$PROVIDER' != 'auto':
    data['tracker_rules']['provider'] = '$PROVIDER'
if '$PROVIDER' == 'jira':
    data['tracker_rules']['numeric_prefix'] = ''
    data['tracker_rules']['alphanumeric_prefix'] = ''
    data['tracker_rules']['keys'] = {
        'issue_id': 'key',
        'title': 'title',
        'labels': 'labels',
        'state': 'state',
        'closed_state_value': 'CLOSED',
        'open_state_value': 'OPEN'
    }
    data['tracker_rules']['labels'] = {
        'epic': 'type::epic',
        'feature': 'type::feature',
        'user_story': 'type::user-story',
        'use_case': 'type::use-case',
        'ready_for_review': 'status::ready-for-review',
        'resolved': 'status::fixed-resolved'
    }
if '$JIRA_URL':
    data['tracker_rules']['server_url'] = '$JIRA_URL'
if '$JIRA_PROJECT':
    data['tracker_rules']['project_key'] = '$JIRA_PROJECT'
if '$JIRA_EMAIL':
    data['tracker_rules']['email'] = '$JIRA_EMAIL'
with open(path, 'w', encoding='utf-8') as f:
    json.dump(data, f, indent=2)
" 2>/dev/null || true
    fi
  done
elif [ "$PROVIDER" = "github" ]; then
  for rules_file in "$TARGET_DIR/.pipeline/logical-ui/codebase_rules.json" "$TARGET_DIR/codebase_rules.json"; do
    if [ -f "$rules_file" ]; then
      python3 -c "
import json, sys
path = '$rules_file'
with open(path, 'r', encoding='utf-8') as f:
    data = json.load(f)
if 'tracker_rules' not in data:
    data['tracker_rules'] = {}
data['tracker_rules']['provider'] = 'github'
with open(path, 'w', encoding='utf-8') as f:
    json.dump(data, f, indent=2)
" 2>/dev/null || true
    fi
  done
fi

# Generate .env.template in target workspace
cat << 'EOF' > "$TARGET_DIR/.env.template"
# Digital Engineering Agent Platform (DEAP) Environment Variables Template
# Copy this file to .env or export variables in your shell / CI/CD environment.

# ==============================================================================
# GitHub Configuration (for --provider github)
# ==============================================================================
# GITHUB_TOKEN=ghp_your_github_personal_access_token
# GITHUB_REPOSITORY=owner/repository_name

# ==============================================================================
# GitLab Configuration (for --provider gitlab)
# ==============================================================================
# GITLAB_URL=https://gitlab.com
# GITLAB_PROJECT=group/project_name
# GITLAB_TOKEN=glpat-your_gitlab_personal_access_token
# CI_JOB_TOKEN=your_ci_job_token_if_in_gitlab_ci
# GITLAB_CA_CERT_PATH=/path/to/custom_ca_cert.crt

# ==============================================================================
# Jira Cloud / Data Center Configuration (for --provider jira)
# ==============================================================================
# Base URL for Jira Cloud or Jira Data Center
JIRA_SERVER_URL=https://your-domain.atlassian.net

# Jira Project Key (e.g. UAS, SAFE, DEAP)
JIRA_PROJECT_KEY=UAS

# Atlassian Account Email (required for Jira Cloud Basic Authentication)
JIRA_EMAIL=engineer@your-domain.com

# Jira API Token (for Jira Cloud) or Personal Access Token (for Jira Data Center)
JIRA_API_TOKEN=your_jira_api_token_or_pat_here

# Optional: Path to custom Root CA bundle for self-hosted Jira Data Center
# JIRA_CA_CERT_PATH=/etc/ssl/certs/internal-ca.pem
EOF

# Transform and scaffold downstream .agents/AGENTS.md and root AGENTS.md with full governance armor
mkdir -p "$TARGET_DIR/.agents"
python3 -c "
import os, sys

installer_root = sys.argv[1]
target_dir = sys.argv[2]
src_agents_path = os.path.join(installer_root, 'AGENTS.md')

with open(src_agents_path, 'r', encoding='utf-8') as f:
    content = f.read()

upstream_header = '''## Repository Role & Scope Classification
- **Repository Classification:** \`UPSTREAM_SPEC_CORE_COMPILER\` (Digital Engineering Agent Platform Core Specification Compiler)
- **Sentinel Indicator:** The presence of \`.pipeline/upstream/\` and \`skills/spec-orchestrator/\` denotes that this repository is the **Upstream Specification Core Compiler**, NOT a downstream customer application workspace or domain template.
- **Domain Template & Customer Data Boundary:** Domain-specific platforms (e.g. UAS safety, automotive, medical) and customer applications belong in downstream distribution repositories, and must NOT be committed to this upstream specification core compiler repository.'''

downstream_header = '''## Repository Role & Scope Classification
- **Repository Classification:** \`DOWNSTREAM_CUSTOMER_PROJECT\` (Domain-Specific Safety-Critical Engineering Project)
- **Sentinel Indicator:** The absence of \`.pipeline/upstream/\` denotes that this repository is an active **Downstream Customer Project Workspace**, authorized for concrete application code implementation and domain feature delivery.
- **Customer Application Scope:** Customer-specific application code, domain nodes/modules, domain tests, mission envelopes, and proprietary safety models are developed, tested, and maintained directly within this project workspace across any target domain (Aerospace, Medical, Space, Industrial AGV, Subsea, Rail).'''

if upstream_header in content:
    transformed = content.replace(upstream_header, downstream_header)
else:
    import re
    transformed = re.sub(
        r'## Repository Role & Scope Classification\n- \*\*Repository Classification:\*\* `UPSTREAM_SPEC_CORE_COMPILER`[^\n]*\n- \*\*Sentinel Indicator:\*\* [^\n]*\n- \*\*Domain Template & Customer Data Boundary:\*\* [^\n]*',
        downstream_header,
        content
    )

dot_agents_path = os.path.join(target_dir, '.agents', 'AGENTS.md')
root_agents_path = os.path.join(target_dir, 'AGENTS.md')

with open(dot_agents_path, 'w', encoding='utf-8') as f:
    f.write(transformed)

with open(root_agents_path, 'w', encoding='utf-8') as f:
    f.write(transformed)
" "$INSTALLER_ROOT" "$TARGET_DIR"

# Scaffold downstream root CLAUDE.md if missing
if [ ! -f "$TARGET_DIR/CLAUDE.md" ]; then
  cat << 'EOF' > "$TARGET_DIR/CLAUDE.md"
# Claude Code Project Guidelines

## Primary Commercial Toolchain Integration Context
This project explicitly declares MATLAB / Simulink / Stateflow / Embedded Coder as the Primary Tier-1 Commercial Toolchain Integration Context (Model-Based Design, Control Law Synthesis, DO-178C C/SPARK Ada code generation).

## Workflow & Quality Gates
- Follow all pipeline rules in `rules/` and skills in `skills/` and `.agents/skills/`.
- Strict Planning Gate: Do not execute unauthorized modifications without an approved implementation plan.
- Execute baseline verification: `pytest tests/test_baseline.py` and `python3 scripts/verify_downstream_baseline.py --no-domain`.
EOF
fi

# Scaffold downstream root README.md if missing
if [ ! -f "$TARGET_DIR/README.md" ]; then
  cat << 'EOF' > "$TARGET_DIR/README.md"
# Downstream Low-Altitude UAS Infrastructure Safety Project

> **Repository Role:** `DOWNSTREAM_APPLICATION_WORKSPACE`  
> **Primary Technology Profiles:** `ROS2 C++ Real-Time` | `PX4 Autopilot Flight Module`  
> **Target Regulatory Frameworks:** `JARUS SORA v2.5 (SAIL I–VI)` | `ASTM F3269-17 RTA` | `ASTM F3411-22a Remote ID` | `RTCA DO-365B DAA`  

---

## 1. System Overview

This repository is an installed downstream implementation workspace governed by the **Digital Engineering Agent Platform (DEAP)** for low-altitude UAS infrastructure safety, detect-and-avoid (DAA), run-time assurance (RTA), and autonomous flight operations.

### 1.1 Primary Commercial Toolchain Integration Context

This platform explicitly declares **MATLAB / Simulink / Stateflow / Embedded Coder** as the Primary Tier-1 Commercial Toolchain Integration Context (Model-Based Design, Control Law Synthesis, DO-178C C/SPARK Ada code generation).

---

## 2. Pipeline Structure & Governance

- `.agents/` & `AGENTS.md`: Agent behavior rules, role boundaries, and subagent dispatch protocols.
- `CLAUDE.md`: Claude Code guidelines and verification gates.
- `.pipeline/`: Constitution (`constitution.md`), domain specifications, and execution profiles (`profiles/ros2_cpp.md`, `profiles/px4_module.md`).
- `rules/` & `skills/`: Platform engineering rules and agent workflow skills (including SysML v2 SSOT completeness in `rules/sysml-ssot-completeness.md`).
- `schema/`: Contract definitions and SysML v2 schemas.
- `tests/`: Automated baseline verification and safety compliance tests.

---

## 3. Mandatory Agent Initialization Sequence

Immediately following installation, any AI agent (Antigravity, Claude Code, Gemini CLI, Cursor) initializing in this repository MUST execute the following sequence before accepting user directives or executing task implementations:

0. **Detect Repository Role & Scope**:
   - Inspect whether `.pipeline/upstream/` exists on disk.
   - If absent -> **Downstream Customer Project Mode**: Authorized for customer feature implementation and domain codebase delivery.
1. **Read Governance Constitution**: Execute `view_file` on `.pipeline/constitution.md` to ingest the platform-independent functional governance layer and zero-mocking persistence mandates.
2. **Load Project Skills**: Execute `view_file` on `skills/feature-driven-implementation/SKILL.md` (and any active skills under `skills/` or `.agents/skills/`) to initialize feature-driven implementation protocols and review gates.
3. **Load Governance Rules**: Ingest `AGENTS.md` and `rules/` to enforce project-scoped agentic rules, context-isolated subagent dispatch loops, and role boundary locks.
4. **Load Platform Profile**: Read the target platform execution profile (`.pipeline/profiles/ros2_cpp.md` for ROS2 C++ Real-Time Nodes or `.pipeline/profiles/px4_module.md` for PX4 Autopilot Flight Modules) to establish platform-specific build, test, and lifecycle constraints.
5. **Bootstrap Tracker Labels & Verify Baseline**: Verify that repository issue tracker labels and baseline tests pass by running `pytest tests/` and `python3 scripts/verify_downstream_baseline.py --no-domain`.

---

## 4. Multi-Pipeline Operator Prompt Catalog & Autonomous Execution Workflows

This catalog contains the complete, unabridged, copy-pasteable operator prompt suite for executing all stages of the Digital Engineering Agent Platform (DEAP) lifecycle across context-isolated subagents in Antigravity, Claude Code, Gemini CLI, Cursor, and Cascade.

### 4.1 Master-Worker Subagent Topology

```mermaid
flowchart LR
    subgraph Ingestion["Universal Multi-Document & Schema Ingestion"]
        Doc1["Operational Intent (docs/conops/*.md)"]
        Doc2["Interface & Model Schemas (schema/*)"]
        Doc3["Architectural Blueprints (docs/architecture/*.md)"]
        Doc4["Prompt Directives (Fallback: Auto-Persist docs/conops/MISSION_INTENT.md)"]
    end
    Doc1 --> Worker_0A["Worker 0A: CONOPS Synthesizer"]
    Doc2 --> Worker_0A
    Doc3 --> Worker_0A
    Doc4 --> Worker_0A
    Worker_0A -->|"docs/conops/CONOPS.md"| Worker_0B["Worker 0B: STPA / FMECA / SORA Assurer"]
    Worker_0B -->|"docs/safety/STPA_MATRIX.md & SORA SAIL"| Worker_0C["Worker 0C: SysML v2 Authoring Worker"]
    Worker_0C -->|"DEAP_MODEL.sysml & Handoff AST JSON"| Pipeline_1["Pipeline 1 Projection Engine"]
```

### 4.2 Pipeline 0 Execution Prompts

Execute the following prompts in sequence using context-isolated subagents to transform unstructured intent, operational scenarios, and interface schemas into formal CONOPS, STPA hazard matrices, and SysML v2 AST models:

#### 4.2.1 Worker 0A: CONOPS & Operational Scenario Synthesis Prompt

```text
Execute `view_file` on `skills/spec-conops-engineering/SKILL.md` as your very first step before taking any action.

Repository Classification: DOWNSTREAM_CUSTOMER_PROJECT (or UPSTREAM_SPEC_CORE_COMPILER depending on execution context)

Role: Worker 0A -- CONOPS & Operational Scenario Synthesizer

Primary Commercial Toolchain Integration Context:
This project explicitly declares MATLAB / Simulink / Stateflow / Embedded Coder as the Primary Tier-1 Commercial Toolchain Integration Context (Model-Based Design, Control Law Synthesis, DO-178C C/SPARK Ada code generation).

Directive:
Execute front-end CONOPS synthesis for the target cyber-physical system using Universal Multi-Document & Schema Ingestion:

1. Universal Multi-Document & Schema Discovery:
   - Operational Intent Discovery: Scan `docs/conops/` for all mission/operational intent markdown files (`*.md`, excluding `README.md`). If present, ingest all as authoritative operational specifications. If `docs/conops/` contains no intent files, ingest prompt directives and auto-persist `docs/conops/MISSION_INTENT.md`.
   - Interface & Model Schema Ingestion: Scan `schema/` for pre-existing customer models and interface definitions (`*.sysml`, `*.proto`, `*.arxml`, `*.json`, `*.yaml`, `*.idl`). Ingest all port types, message structures, and subsystem definitions into the operational context.
   - Architectural Blueprint Ingestion: Scan `docs/architecture/` (and `docs/architecture/blueprints/`) for existing architectural specifications, network blueprints, and safety frameworks (`*.md`). Ingest all system boundaries, subsystem mappings, and commercial toolchain hooks.
   - Reconcile customer interface schemas and architectural blueprints with system boundaries and MATLAB / Simulink / Stateflow control law synthesis hooks.

2. Ingestion & Analysis Scope:
   - Schema-derived operational envelope (physical boundaries, operating dynamics, environmental constraints, payload/actuator configurations).
   - Domain-specific operational lifecycle phases: Initialization, Normal Operation, Degraded/Contingency Modes, and Safe Shutdown/Transition.
   - Dynamic stakeholder roles derived from the system operational context (e.g., System Operators, Dispatchers/Supervisors, Field Maintenance Technicians, External Management/Telemetry Interfaces).
   - Domain-specific regulatory and safety classification relevant to the operational envelope.

3. Output Requirements:
   - Persist/validate `docs/conops/MISSION_INTENT.md` under `docs/conops/MISSION_INTENT.md` (if operating from prompt fallback or validating canonical format).
   - Generate `CONOPS.md` under `docs/conops/CONOPS.md` integrating all discovered intent, schema, and architectural constraints.
   - Ensure clear operational phase boundaries, system physical and functional boundaries, and environmental envelope constraints.
   - Include MATLAB / Simulink / Stateflow model integration baseline hooks for downstream control law synthesis.
   - KaTeX / LaTeX Math Formatting Mandate: All multi-line aligned equations MUST be enclosed in `\begin{aligned} ... \end{aligned}` within `$$` delimiters on dedicated lines. Bare alignment tabs `&` outside an alignment environment (`aligned`, `matrix`, `cases`) and `\begin{align*}` environments are strictly forbidden. Markdown Table Math Prohibition Rule: Strictly ban `$ ... $` and `$$ ... $$` LaTeX math delimiters inside table headers, rows, and cells; plain text and Unicode (e.g. `Initial S`, `ΔV`, `λ`, `°C`, `≥`, `≤`, `→`, `10⁻⁶`) must be used instead, with 1:1 column count match between header and delimiter rows.

PROCEED
```

#### 4.2.2 Worker 0B: STPA Hazard Analysis, FMECA & Domain Safety Assurer Prompt

```text
Execute `view_file` on `skills/spec-orchestrator/SKILL.md` as your very first step before taking any action.

Repository Classification: DOWNSTREAM_CUSTOMER_PROJECT (or UPSTREAM_SPEC_CORE_COMPILER depending on execution context)

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

#### 4.2.3 Worker 0C: SysML v2 Architectural & Safety Model Author Prompt

```text
Execute `view_file` on `skills/spec-orchestrator/SKILL.md` as your very first step before taking any action.

Repository Classification: DOWNSTREAM_CUSTOMER_PROJECT (or UPSTREAM_SPEC_CORE_COMPILER depending on execution context)

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

### 4.3 Pipeline 1 Agile Backlog Projection Prompts

Execute the following prompts to extract full Agile backlogs (Epics, Level 1C ICD Interface Matrices, BDD User Stories, and UML Use Cases) with closed-loop tracker synchronization:

#### 4.3.1 Worker 1A: Structural Spec Worker (Epics & Features) Prompt

```text
Execute `view_file` on `skills/schema-specification-engineering/SKILL.md` as your very first step before taking any action.

Repository Classification: DOWNSTREAM_CUSTOMER_PROJECT (or UPSTREAM_SPEC_CORE_COMPILER depending on execution context)

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
   - Execute the local model coverage linter: `./skills/spec-orchestrator/scripts/verify_model_coverage.py --spec-only --allow-missing-specs`.
   - Register Features first via `./skills/spec-orchestrator/scripts/create_issue.sh "<file>" "feature" "<title>"`.
   - Verify live published payload on the issue tracker (`gh issue view <ID> --json body` or `glab issue view <ID>`).
   - Inject verified Feature Issue IDs into Epic tasklists.
   - Register Epics via `./skills/spec-orchestrator/scripts/create_issue.sh "<file>" "epic" "<title>"`.

Defect Filing Directive:
If any compiler fault, schema inconsistency, or invariant violation is discovered, you are strictly forbidden from filing raw issues directly. You MUST dispatch a fresh context-isolated subagent with `skills/adversarial-code-auditor/SKILL.md` to perform the 5-pillar audit, generate the verified 7-section defect dossier, and submit it via `python3 scripts/file_defect.py`. Issue auto-closing keywords or issue close commands are strictly forbidden.

PROCEED
```

#### 4.3.2 Worker 1B: Interface Spec Worker (Logical ICD & Signal Dictionary) Prompt

```text
Execute `view_file` on `skills/spec-icd-engineering/SKILL.md` as your very first step before taking any action.

Repository Classification: DOWNSTREAM_CUSTOMER_PROJECT (or UPSTREAM_SPEC_CORE_COMPILER depending on execution context)

Role: Worker 1B -- Interface Specification Worker (Worker ICD)

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

#### 4.3.3 Worker 1C: Behavioral Spec Worker (User Stories & Statecharts) Prompt

```text
Execute `view_file` on `skills/spec-user-story-engineering/SKILL.md` as your very first step before taking any action.

Repository Classification: DOWNSTREAM_CUSTOMER_PROJECT (or UPSTREAM_SPEC_CORE_COMPILER depending on execution context)

Role: Worker 1C -- Behavioral Specification Worker (User Stories & Statecharts)

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
   - Execute local model coverage linter: `./skills/spec-orchestrator/scripts/verify_model_coverage.py --spec-only --allow-missing-specs`.
   - Register User Stories via `./skills/spec-orchestrator/scripts/create_issue.sh "<file>" "user-story" "<title>"`.
   - Verify live published payload on the issue tracker (`gh issue view <ID> --json body` or `glab issue view <ID>`).

Defect Filing Directive:
If any compiler fault, schema inconsistency, or invariant violation is discovered, you are strictly forbidden from filing raw issues directly. You MUST dispatch a fresh context-isolated subagent with `skills/adversarial-code-auditor/SKILL.md` to perform the 5-pillar audit, generate the verified 7-section defect dossier, and submit it via `python3 scripts/file_defect.py`. Issue auto-closing keywords or issue close commands are strictly forbidden.

PROCEED
```

#### 4.3.4 Worker 1D: System Interaction Spec Worker (UML Use Cases & Realization Matrix) Prompt

```text
Execute `view_file` on `skills/spec-usecase-engineering/SKILL.md` as your very first step before taking any action.

Repository Classification: DOWNSTREAM_CUSTOMER_PROJECT (or UPSTREAM_SPEC_CORE_COMPILER depending on execution context)

Role: Worker 1D -- System Interaction Specification Worker (UML Use Cases)

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
   - Execute local model coverage check: `./skills/spec-orchestrator/scripts/verify_model_coverage.py --spec-only --allow-missing-specs`.
   - Register Use Cases via `./skills/spec-orchestrator/scripts/create_issue.sh "<file>" "use-case" "<title>"`.
   - Verify live published payload on the issue tracker (`gh issue view <ID> --json body` or `glab issue view <ID>`).

Defect Filing Directive:
If any compiler fault, schema inconsistency, or invariant violation is discovered, you are strictly forbidden from filing raw issues directly. You MUST dispatch a fresh context-isolated subagent with `skills/adversarial-code-auditor/SKILL.md` to perform the 5-pillar audit, generate the verified 7-section defect dossier, and submit it via `python3 scripts/file_defect.py`. Issue auto-closing keywords or issue close commands are strictly forbidden.

PROCEED
```

### 4.4 Multi-Provider Backlog Reconciliation Commands

Execute backlog reconciliation and model parity verification across your target VCS platform or offline air-gapped environment:

#### 4.4.1 Option A: GitLab SaaS Reconciliation
```bash
./scripts/reconcile_backlog.py --provider gitlab
```

#### 4.4.2 Option B: GitLab Self-Managed / SCIF Air-Gapped Reconciliation
```bash
./scripts/reconcile_backlog.py --provider gitlab --gitlab-url https://gitlab.internal.defense.gov --project <group>/<project>
```

#### 4.4.3 Option C: GitHub Issues Reconciliation
```bash
./scripts/reconcile_backlog.py --provider github
```

#### 4.4.4 Option D: Offline Verification & 23-Gate Parity Lock
```bash
# Closed-loop reverse SysML v2 AST synchronization
python3 scripts/compile_sysml.py --reverse-sync

# Offline backlog checklist and status synchronization
./scripts/reconcile_backlog.py --offline

# 23-Gate Model Coverage & UML Compliance Lock
./skills/spec-orchestrator/scripts/verify_model_coverage.py schema docs/features --spec-only
```

### 4.5 Pipeline 2 Autonomous Feature Implementation Prompts

Execute the following prompts to drive feature implementation and two-path (dual-track) simulation verification through context-isolated TDD micro-tasks:

#### 4.5.1 Worker 2A / Synthesis Driver: Feature-Driven Implementation Prompt

```text
Execute `view_file` on `skills/feature-driven-implementation/SKILL.md` as your very first step before taking any action.

Repository Classification: DOWNSTREAM_CUSTOMER_PROJECT (or UPSTREAM_SPEC_CORE_COMPILER depending on execution context)

Role: Worker 2A -- Feature-Driven Implementation & Synthesis Driver

Primary Commercial Toolchain Integration Context:
This project explicitly declares MATLAB / Simulink / Stateflow / Embedded Coder as the Primary Tier-1 Commercial Toolchain Integration Context (Model-Based Design, Control Law Synthesis, DO-178C C/SPARK Ada code generation).

Governance Preamble & Execution Directive:
Adopt the feature-driven-implementation skill by reading `.pipeline/constitution.md` and the target platform profile (`.pipeline/profiles/<target-platform>.md`, e.g. `ros2_cpp.md`, `px4_module.md`, or `flutter.md`).

Implement prioritized Feature [Issue Number, e.g. #1] adhering strictly to the 3-Layer Definition of Done (DoD):
1. Layer 1: Domain Model / Safety Statechart -- Platform-independent domain entities, transition guards, mathematical invariants, and safety statecharts.
2. Layer 2: Safety Statechart / ViewModel -- State management, event handling, lifecycle hooks, and reactive telemetry bindings.
3. Layer 3: Interface Binding / Middleware & BDD Tests -- Platform interface bindings (ROS2 lifecycle nodes, PX4 uORB modules, or Flutter widgets) verified via automated BDD integration tests against live emulators / simulation harnesses.

Execution Standards:
- Execute TDD RED-GREEN-REFACTOR cycles using context-isolated subagents for each 2-5 minute micro-task.
- Dual-Track MBD Verification: Enforce Track A (Native MATLAB / Simulink / Stateflow synthesis) and Track B (Headless CI Digital Twin Engine) with numerical tolerance verification (error <= 10^-6) and zero license blockers.
- Zero-Mocking Live Persistence Mandate: Validate all transactions against live databases / emulators.
- Closed-Loop Payload Verification: Deliver cumulative solution walkthrough (`docs/designs/feat-<ID>-solution.md`), verify live published payload, comment on issue with walkthrough link, and apply `status:fixed-resolved` (GitHub) or `status::fixed-resolved` (GitLab). Leave issue open for Product Owner review.

Defect Filing Directive:
If any compiler fault, schema inconsistency, or invariant violation is discovered, you are strictly forbidden from filing raw issues directly. You MUST dispatch a fresh context-isolated subagent with `skills/adversarial-code-auditor/SKILL.md` to perform the 5-pillar audit, generate the verified 7-section defect dossier, and submit it via `python3 scripts/file_defect.py`. Issue auto-closing keywords or issue close commands are strictly forbidden.

PROCEED
```

#### 4.5.2 Worker 2B / Simulation Driver: Two-Path (Dual-Track) Simulation & Digital Twin Verification Prompt

```text
Execute `view_file` on `skills/feature-driven-implementation/SKILL.md` as your very first step before taking any action.

Repository Classification: DOWNSTREAM_CUSTOMER_PROJECT (or UPSTREAM_SPEC_CORE_COMPILER depending on execution context)

Role: Worker 2B -- Two-Path (Dual-Track) Simulation & Digital Twin Verification Driver

Primary Commercial Toolchain Integration Context:
This project explicitly declares MATLAB / Simulink / Stateflow / Embedded Coder as the Primary Tier-1 Commercial Toolchain Integration Context (Model-Based Design, Control Law Synthesis, DO-178C C/SPARK Ada code generation).

Governance Preamble & Execution Directive:
Adopt the feature-driven-implementation skill by reading `.pipeline/constitution.md`, `rules/dual-track-mbd-verification.md`, and `docs/architecture/blueprints/SYSML_SSOT_BIDIRECTIONAL_SYNCHRONIZATION_ARCHITECTURE.md`.

Execute Two-Path (Dual-Track) Model-Based Design (MBD) simulation synthesis and digital twin verification for Feature [Issue Number, e.g. #1]:

1. Track A (Native MATLAB / Simulink / Stateflow Synthesis):
   - Programmatic Model Construction: Deliver `models/scripts/build_<feature_slug>_model.m` to programmatically synthesize native `.slx` block diagrams and Stateflow charts using official MATLAB APIs.
   - Parameter & Signal Dictionaries: Deliver physical parameter dictionary `models/matlab/<feature_slug>_params.m` and Simulink Data Dictionary `models/matlab/<feature_slug>_data.sldd`.
   - Solver & Synthesis Baseline: Configure models for deterministic fixed-step discrete solvers (`FixedStepDiscrete`, $dt = 0.004\,\text{s}$ / 250 Hz) and Embedded Coder DO-178C C / SPARK Ada code synthesis.

2. Track B (Headless CI Digital Twin Engine):
   - License-Free Discrete Execution Engine: Deliver standalone Python simulation engine (`models/python/<feature_slug>_domain.py` and `models/python/<feature_slug>_engine.py`) executing at identical discrete loop rate ($dt$) with exact transition guards, polynomial transfer curves, and 6-DOF kinematics.
   - Zero License Blocker CI Harness: Deliver automated regression test suite `tests/test_<feature_slug>_simulation.py` running 100% offline in containerized CI environments without MathWorks licenses.

3. Mathematical & Discrete Equivalence Mandate:
   - Numerical Tolerance Verification: Guarantee state vector and output trajectory error between Track A reference and Track B digital twin satisfies $\|x_{\text{Simulink}} - x_{\text{DigitalTwin}}\|_\infty \le 10^{-6}$.
   - Formal DO-331 Verification Report: Generate comprehensive verification report `docs/reports/simulink_results/<FEATURE-ID>_simulation_results.md` detailing MC/DC coverage mapping, transition truth tables, fault-injection scenarios, and numerical parity logs.

Defect Filing Directive:
If any compiler fault, schema inconsistency, or invariant violation is discovered, you are strictly forbidden from filing raw issues directly. You MUST dispatch a fresh context-isolated subagent with `skills/adversarial-code-auditor/SKILL.md` to perform the 5-pillar audit, generate the verified 7-section defect dossier, and submit it via `python3 scripts/file_defect.py`. Issue auto-closing keywords or issue close commands are strictly forbidden.

PROCEED
```

#### 4.5.3 Two-Path MBD Artifact & Deliverable Hierarchy

Every feature containing control laws, flight dynamics, physical plant estimators, or safety state machines delivers the canonical two-path MBD artifact suite:

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

##### Dual-Track Artifact Descriptions:

1. **`models/scripts/build_<feature_slug>_model.m` (Track A Builder)**:
   Programmatically constructs native MATLAB / Simulink (`.slx`) block diagrams and Stateflow charts via official MATLAB APIs (`new_system`, `add_block`, `Stateflow.Data`, `Stateflow.State`, `Stateflow.Transition`). Configures deterministic discrete fixed-step solvers (`FixedStepDiscrete`) and Embedded Coder DO-178C C / SPARK Ada code synthesis.

2. **`models/matlab/<feature_slug>_params.m` & `.sldd` (Track A Dictionaries)**:
   Declares physical plant constants, control gains, rate limits, sensor noise variances, and discrete sample time ($dt = 0.004\,\text{s}$ / 250 Hz) in typed MATLAB structures and Simulink Data Dictionaries.

3. **`models/python/<feature_slug>_domain.py` & `_engine.py` (Track B Digital Twin)**:
   Pure Python, license-free, headless discrete simulation engine executing identical algebraic formulations, cubic polynomial blending curves ($\lambda(\tau) = 3\tau^2 - 2\tau^3$), and safety transition guards. Exposes typed state vectors and `step(dt, inputs) -> outputs` execution interface.

4. **`tests/test_<feature_slug>_simulation.py` (Automated CI Verification Suite)**:
   Pytest / Unittest test suite executing offline in CI/CD runners without MathWorks license blockers. Validates nominal control tracks, fault-injection responses, emergency safety transitions, and state invariants.

5. **`docs/reports/simulink_results/<FEATURE-ID>_simulation_results.md` (DO-331 Verification Report)**:
   Formal DO-178C / DO-331 verification deliverable documenting mathematical equivalence, step-by-step state transition logs, fault injection test results, and numerical tolerance parity ($\le 10^{-6}$).

---

## 5. Verification & Quality Gates

Execute baseline and safety governance verification:

```bash
# Run baseline tests
python3 -m pytest tests/

# Run downstream conformance gate
python3 scripts/verify_downstream_baseline.py --no-domain
```
EOF
fi

if [ ! -f "$TARGET_DIR/tests/test_baseline.py" ]; then
  cat << 'EOF' > "$TARGET_DIR/tests/test_baseline.py"
"""
Downstream Environment & Runtime Integrity Verification Suite.
/// Realises: [BaselineVerification]
"""
import sys
import os
import re
import subprocess
import tempfile
import pytest

def test_python_runtime_environment():
    """Verify Python runtime version and core interpreter executable exist and function."""
    assert sys.version_info >= (3, 8), f"Python version {sys.version} is below required 3.8+"
    assert os.path.exists(sys.executable), "Python interpreter path invalid"

def test_disk_io_and_permissions():
    """Verify local file system read, write, and permission capabilities."""
    with tempfile.NamedTemporaryFile(mode="w+", delete=True) as temp_file:
        test_payload = "DEAP_ENVIRONMENT_INTEGRITY_CHECK_PAYLOAD_2026"
        temp_file.write(test_payload)
        temp_file.seek(0)
        read_back = temp_file.read()
        assert read_back == test_payload, "Disk I/O payload mismatch during environment validation"

def test_schema_directory_accessible():
    """Verify schema directory exists and is accessible for domain specification contracts."""
    schema_dir = os.path.join(os.path.dirname(os.path.dirname(os.path.abspath(__file__))), "schema")
    assert os.path.isdir(schema_dir) or os.path.isdir("schema"), "Schema directory missing or inaccessible"

def test_latex_katex_integrity():
    """Verify KaTeX / LaTeX mathematical rendering syntax across all markdown files.

    Ensures:
    - Balanced $$ math blocks
    - No bare alignment operators & outside alignment environments (aligned, matrix, bmatrix, etc.)
    - No forbidden \\begin{align} or \\begin{align*} in math blocks (\\begin{aligned} must be used)
    - Balanced \\begin{aligned} and \\end{aligned} pairs
    """
    repo_root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
    if not os.path.isdir(repo_root):
        repo_root = os.getcwd()

    excluded_dirs = {".git", "node_modules", ".dart_tool", "build"}
    allowed_alignment_envs = {
        "aligned", "alignedat", "matrix", "pmatrix", "bmatrix", "Bmatrix",
        "vmatrix", "Vmatrix", "cases", "dcases", "rcases", "array",
        "split", "gathered", "gather", "subarray", "smallmatrix"
    }

    errors = []
    for root, dirs, files in os.walk(repo_root):
        dirs[:] = [d for d in dirs if d not in excluded_dirs]
        for f in files:
            if not f.endswith(".md"):
                continue
            file_path = os.path.join(root, f)
            rel_path = os.path.relpath(file_path, repo_root)
            try:
                with open(file_path, "r", encoding="utf-8") as md_file:
                    content = md_file.read()
            except Exception as e:
                errors.append(f"Failed to read {rel_path}: {e}")
                continue

            cleaned = re.sub(r"```.*?```|~~~.*?~~~", "", content, flags=re.DOTALL)
            cleaned = re.sub(r"`+.*?`+", "", cleaned)

            # a. Validate balanced $$ math blocks
            parts = cleaned.split("$$")
            if (len(parts) - 1) % 2 != 0:
                errors.append(f"Unbalanced $$ display math delimiters in {rel_path} (found {len(parts) - 1} delimiters).")
                continue

            # Check balanced \begin{aligned} and \end{aligned} globally in file
            num_begin_aligned_all = len(re.findall(r"\\begin\{aligned\}", cleaned))
            num_end_aligned_all = len(re.findall(r"\\end\{aligned\}", cleaned))
            if num_begin_aligned_all != num_end_aligned_all:
                errors.append(f"Unbalanced \\begin{{aligned}} ({num_begin_aligned_all}) and \\end{{aligned}} ({num_end_aligned_all}) pairs in {rel_path}.")

            # Validate each display math block
            for i in range(1, len(parts), 2):
                block = parts[i]

                # c. Detect top-level \begin{align} or \begin{align*}
                if re.search(r"\\begin\{align\*?\}", block):
                    errors.append(
                        f"Forbidden \\begin{{align}} or \\begin{{align*}} found in display math block in {rel_path}. "
                        f"In markdown KaTeX, \\begin{{aligned}} must be used instead."
                    )

                # d. Validate balanced \begin{aligned} and \end{aligned} pairs within the block
                num_begin_aligned = len(re.findall(r"\\begin\{aligned\}", block))
                num_end_aligned = len(re.findall(r"\\end\{aligned\}", block))
                if num_begin_aligned != num_end_aligned:
                    errors.append(
                        f"Unbalanced \\begin{{aligned}} ({num_begin_aligned}) and \\end{{aligned}} ({num_end_aligned}) in math block in {rel_path}."
                    )

                # b. Detect bare alignment operators & outside alignment environments
                token_pattern = re.compile(r"\\begin\{([a-zA-Z*]+)\}|\\end\{([a-zA-Z*]+)\}|\\&|&")
                env_stack = []
                for match in token_pattern.finditer(block):
                    token = match.group(0)
                    if token.startswith(r"\begin{"):
                        env_stack.append(match.group(1))
                    elif token.startswith(r"\end{"):
                        end_name = match.group(2)
                        if end_name in env_stack:
                            while env_stack:
                                popped = env_stack.pop()
                                if popped == end_name:
                                    break
                    elif token == r"\&":
                        continue
                    elif token == "&":
                        if not any(env in allowed_alignment_envs for env in env_stack):
                            snippet = block[max(0, match.start() - 20):min(len(block), match.end() + 20)].strip().replace("\n", " ")
                            errors.append(
                                f"Bare alignment operator '&' outside alignment environment in {rel_path}: \"...{snippet}...\""
                            )

    assert not errors, "KaTeX / LaTeX mathematical syntax violations found:\n" + "\n".join(errors)

def test_instructions_and_readme_accessible():
    """Verify README.md and agent instruction entrypoints exist and are accessible."""
    repo_root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
    if not os.path.isdir(repo_root):
        repo_root = os.getcwd()

    readme_path = os.path.join(repo_root, "README.md")
    assert os.path.isfile(readme_path), f"Root README.md missing in repository at {repo_root}"
    assert os.path.getsize(readme_path) > 0, f"Root README.md is empty in repository at {repo_root}"

    agent_entrypoints = [
        os.path.join(repo_root, "AGENTS.md"),
        os.path.join(repo_root, "CLAUDE.md"),
        os.path.join(repo_root, ".agents", "AGENTS.md"),
    ]
    valid_entrypoints = [p for p in agent_entrypoints if os.path.isfile(p) and os.path.getsize(p) > 0]
    assert len(valid_entrypoints) > 0, (
        f"No non-empty agent instruction entrypoint found at {repo_root} "
        f"(checked AGENTS.md, CLAUDE.md, .agents/AGENTS.md)"
    )

def test_reconcile_backlog_tooling_accessible():
    """Verify scripts/reconcile_backlog.py exists, is executable, and runs to completion."""
    repo_root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
    if not os.path.isdir(repo_root):
        repo_root = os.getcwd()

    reconcile_path = os.path.join(repo_root, "scripts", "reconcile_backlog.py")
    assert os.path.isfile(reconcile_path), f"scripts/reconcile_backlog.py missing at {repo_root}"
    assert os.path.getsize(reconcile_path) > 0, f"scripts/reconcile_backlog.py is empty at {repo_root}"
    assert os.access(reconcile_path, os.R_OK), f"scripts/reconcile_backlog.py is not readable at {repo_root}"

    res = subprocess.run([sys.executable, reconcile_path], cwd=repo_root, capture_output=True, text=True, timeout=60)
    assert res.returncode == 0, f"scripts/reconcile_backlog.py failed with exit code {res.returncode}:\nSTDOUT:\n{res.stdout}\nSTDERR:\n{res.stderr}"
    assert "Traceback" not in res.stderr, f"scripts/reconcile_backlog.py produced unhandled exception:\n{res.stderr}"

def test_sysml_ssot_completeness_rule_accessible():
    """Verify rules/sysml-ssot-completeness.md exists, is non-empty, and satisfies governance requirements."""
    repo_root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
    if not os.path.isdir(repo_root):
        repo_root = os.getcwd()

    rule_path = os.path.join(repo_root, "rules", "sysml-ssot-completeness.md")
    assert os.path.isfile(rule_path), f"rules/sysml-ssot-completeness.md missing at {repo_root}"
    assert os.path.getsize(rule_path) > 0, f"rules/sysml-ssot-completeness.md is empty at {repo_root}"

    with open(rule_path, "r", encoding="utf-8") as f:
        content = f.read()

    # Verify key architectural and governance markers
    required_phrases = [
        "SysML v2",
        "Single Source of Truth",
        "Primary Tier-1 Commercial Toolchain Integration Context",
        "MATLAB / Simulink / Stateflow / Embedded Coder",
        "use case def",
        "requirement def",
    ]
    for phrase in required_phrases:
        assert phrase in content, f"Missing required governance marker '{phrase}' in rules/sysml-ssot-completeness.md"

def test_upstream_template_clean_landing_zones():
    """Verify upstream template landing zones remain pristine with zero concrete specs.

    If repository is an upstream template (.pipeline/upstream/ exists), asserts that
    docs/conops/, docs/safety/, docs/epics/, docs/features/, docs/user-stories/,
    docs/use-cases/, and schema/ contain only .gitkeep and README.md, and zero concrete
    specification files or concrete .sysml domain models.
    """
    repo_root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
    if not os.path.isdir(repo_root):
        repo_root = os.getcwd()

    upstream_marker = os.path.join(repo_root, ".pipeline", "upstream")
    if not os.path.isdir(upstream_marker):
        pytest.skip("Downstream project detected -- skipping upstream landing zone clean check.")

    landing_zones = [
        os.path.join("docs", "conops"),
        os.path.join("docs", "safety"),
        os.path.join("docs", "epics"),
        os.path.join("docs", "features"),
        os.path.join("docs", "user-stories"),
        os.path.join("docs", "use-cases"),
        "schema",
    ]
    allowed_files = {".gitkeep", "README.md"}
    excluded_dirs = {".git", "node_modules", ".dart_tool", "build"}

    violations = []
    for zone in landing_zones:
        zone_path = os.path.join(repo_root, zone)
        if not os.path.isdir(zone_path):
            continue
        for root, dirs, files in os.walk(zone_path):
            dirs[:] = [d for d in dirs if d not in excluded_dirs]
            for f in files:
                if f not in allowed_files:
                    rel_path = os.path.relpath(os.path.join(root, f), repo_root)
                    violations.append(rel_path)

    assert not violations, (
        f"Upstream distribution template landing zones contain concrete specification files: {violations}"
    )


def test_operator_prompt_catalog_accessible():
    """Verify docs/OPERATOR_PROMPT_CATALOG.md exists, is non-empty, and contains headers for Pipeline 1 (Workers 1A-1D) and Pipeline 2."""
    repo_root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
    if not os.path.isdir(repo_root):
        repo_root = os.getcwd()

    catalog_path = os.path.join(repo_root, "docs", "OPERATOR_PROMPT_CATALOG.md")
    assert os.path.isfile(catalog_path), f"docs/OPERATOR_PROMPT_CATALOG.md missing at {repo_root}"
    assert os.path.getsize(catalog_path) > 0, f"docs/OPERATOR_PROMPT_CATALOG.md is empty at {repo_root}"

    with open(catalog_path, "r", encoding="utf-8") as f:
        content = f.read()

    required_headers = [
        "Pipeline 1",
        "Worker 1A",
        "Worker 1B",
        "Worker 1C",
        "Worker 1D",
        "Pipeline 2",
        "Synthesis Driver",
    ]
    for header in required_headers:
        assert header in content, f"Missing required header/section '{header}' in docs/OPERATOR_PROMPT_CATALOG.md"
EOF
fi

# Install-time safety fixture self-check: the safety integrity test suite consumes
# live fixture files under tests/fixtures/safety/; no synthetic content is generated here.
echo "Verifying safety integrity test fixtures..."
SAFETY_FIXTURES_MISSING=""
for fixture_name in complete_stpa_matrix.md truncated_uca_matrix.md missing_guideword_matrix.md incomplete_osos.md proof_missing_derivation.md complete_proof.md; do
  if [ ! -f "$TARGET_DIR/tests/fixtures/safety/$fixture_name" ]; then
    SAFETY_FIXTURES_MISSING="$SAFETY_FIXTURES_MISSING $fixture_name"
  fi
done
if [ -n "$SAFETY_FIXTURES_MISSING" ]; then
  echo "ERROR: safety integrity test fixtures missing under tests/fixtures/safety/:$SAFETY_FIXTURES_MISSING" >&2
  exit 1
fi
echo "Safety integrity test fixtures verified present (zero synthetic content generated)."

if [ -f "$TARGET_DIR/scripts/setup_git_hooks.py" ]; then
  (cd "$TARGET_DIR" && python3 scripts/setup_git_hooks.py --install) || true
fi

# Automatically bootstrap issue tracker label taxonomy
echo "Bootstrapping issue tracker label taxonomy..."
if [ -f "$TARGET_DIR/skills/spec-orchestrator/scripts/bootstrap_tracker_labels.py" ]; then
  python3 "$TARGET_DIR/skills/spec-orchestrator/scripts/bootstrap_tracker_labels.py" || {
    echo "Note: Tracker labels could not be provisioned automatically (e.g. offline or unauthenticated)."
    echo "You can re-run label provisioning anytime: python3 skills/spec-orchestrator/scripts/bootstrap_tracker_labels.py"
  }
fi

find "$TARGET_DIR" -name ".DS_Store" -delete 2>/dev/null || true

echo "==> Digital Pipeline Installation Complete. 0 manual steps remaining."

