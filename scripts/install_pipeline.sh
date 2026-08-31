#!/usr/bin/env bash
set -e

INSTALLER_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
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
TARGET_DIR="$(cd "$TARGET_DIR" 2>/dev/null && pwd || echo "$TARGET_DIR")"

if [ "$TARGET_DIR" = "$INSTALLER_ROOT" ] && [ -e "$INSTALLER_ROOT/.pipeline/upstream" ]; then
  echo "REFUSING: target is the pipeline repository itself, not a downstream project." >&2
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
cp -P "$INSTALLER_ROOT/.gitlab-ci.yml" "$TARGET_DIR/" 2>/dev/null || true
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

# Scaffold downstream root AGENTS.md if missing
if [ ! -f "$TARGET_DIR/AGENTS.md" ]; then
  if [ -f "$TARGET_DIR/.agents/AGENTS.md" ]; then
    cp "$TARGET_DIR/.agents/AGENTS.md" "$TARGET_DIR/AGENTS.md"
  elif [ -f "$INSTALLER_ROOT/AGENTS.md" ]; then
    cp "$INSTALLER_ROOT/AGENTS.md" "$TARGET_DIR/AGENTS.md"
  fi
fi

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

## 4. Pipeline 0: Pre-Spec Safety Engineering Execution Workflow

Pipeline 0 (**Pre-Spec Safety Engineering Engine**) ingests mission flight envelopes and airspace constraints to produce normative safety specifications, STPA/FMECA analysis, SORA SAIL assurance models, and SysML v2 textual AST artifacts.

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

### 4.2 Pipeline 0 Command-Line Execution Prompts

Execute the following prompts in sequence using context-isolated subagents:

#### 4.2.1 Worker 0A: CONOPS & Mission Scenario Synthesis Prompt

```text
Role: Worker 0A — CONOPS & Mission Scenario Synthesizer

Primary Commercial Toolchain Integration Context:
This project explicitly declares MATLAB / Simulink / Stateflow / Embedded Coder as the Primary Tier-1 Commercial Toolchain Integration Context (Model-Based Design, Control Law Synthesis, DO-178C C/SPARK Ada code generation).

Directive:
Execute front-end CONOPS synthesis for the target UAS flight mission profile using Universal Multi-Document & Schema Ingestion:

1. Universal Multi-Document & Schema Discovery:
   - Operational Intent Discovery: Scan `docs/conops/` for all mission intent markdown files (`*.md`, excluding `README.md`). If present, ingest all as authoritative operational specifications. If `docs/conops/` contains no intent files, ingest prompt directives and auto-persist `docs/conops/MISSION_INTENT.md`.
   - Interface & Model Schema Ingestion: Scan `schema/` for pre-existing customer models and interface definitions (`*.sysml`, `*.proto`, `*.arxml`, `*.json`, `*.yaml`, `*.idl`). Ingest all port types, message structures, and subsystem definitions into the operational context.
   - Architectural Blueprint Ingestion: Scan `docs/architecture/` (and `docs/architecture/blueprints/`) for existing architectural specifications, network blueprints, and safety frameworks (`*.md`). Ingest all system boundaries, subsystem mappings, and commercial toolchain hooks.
   - Reconcile customer interface schemas and architectural blueprints with system boundaries and MATLAB / Simulink / Stateflow control law synthesis hooks.

2. Ingestion & Analysis Scope:
   - Operational mission envelope (flight altitude boundaries, max ground speed, payload configuration, population density, BVLOS vs VLOS flight operations).
   - Operational airspace constraints, regulatory classification (e.g., JARUS SORA, FAA Part 107/135, EASA Specific Category), and geographic boundaries.
   - Stakeholder role definitions (Remote Pilot in Command, Fleet Operations Manager, Command Center Lead, Air Traffic Management / UTM interface).
   - Flight operational phases (Pre-Flight Checkout, Launch/Takeoff, En-Route Cruise, Mission Execution, Approach & Landing, Fail-Safe Contingency RTL).

3. Output Requirements:
   - Persist/validate `docs/conops/MISSION_INTENT.md` under `docs/conops/MISSION_INTENT.md` (if operating from prompt fallback or validating canonical format).
   - Generate `CONOPS.md` under `docs/conops/CONOPS.md` integrating all discovered intent, schema, and architectural constraints.
   - Ensure clear operational phase boundaries, system physical and functional boundaries, and environmental envelope constraints.
   - Include MATLAB / Simulink / Stateflow model integration baseline hooks for downstream control law synthesis.
   - KaTeX / LaTeX Math Formatting Mandate: All multi-line aligned equations MUST be enclosed in `\begin{aligned} ... \end{aligned}` within `$$` delimiters on dedicated lines. Bare alignment tabs `&` outside an alignment environment (`aligned`, `matrix`, `cases`) and `\begin{align*}` environments are strictly forbidden. Markdown Table Math Prohibition Rule: Strictly ban `$ ... $` and `$$ ... $$` LaTeX math delimiters inside table headers, rows, and cells; plain text and Unicode (e.g. `Initial S`, `ΔV`, `λ`, `°C`, `≥`, `≤`, `→`, `10⁻⁶`) must be used instead, with 1:1 column count match between header and delimiter rows.

PROCEED
```

#### 4.2.2 Worker 0B: STPA Hazard Analysis, FMECA & SORA SAIL Assurer Prompt

```text
Role: Worker 0B — STPA Hazard Analysis, FMECA & SORA SAIL Assurer

Primary Commercial Toolchain Integration Context:
This project explicitly declares MATLAB / Simulink / Stateflow / Embedded Coder as the Primary Tier-1 Commercial Toolchain Integration Context (Model-Based Design, Control Law Synthesis, DO-178C C/SPARK Ada code generation).

Directive:
Perform STPA hazard analysis, FMECA failure mode criticality evaluation, and SORA SAIL I–VI risk assessment based on `docs/conops/CONOPS.md`.

1. Standards Compliance:
   - JARUS SORA v2.5 (SAIL I through SAIL VI risk mitigations, Ground Risk Class GRC, Air Risk Class ARC, Operational Safety Objectives OSO-01 through OSO-24).
   - ASTM F3269-17 (Run-Time Assurance Monitor Architecture & Safety Net switching).
   - RTCA DO-365B (Detect and Avoid DAA MOPS & TCAS II / ACAS sUAS alert & guidance).

2. Output Requirements:
   - Generate `STPA_MATRIX.md` under `docs/safety/STPA_MATRIX.md` adhering strictly to the 8-pillar schema:
     1. System Losses ($L-1..N$)
     2. System Hazards ($H-1..N$)
     3. Hierarchical Control Structure Topology (defining RPIC, Autopilot, ASTM F3269-17 RTA Monitor, Actuators, Sensors)
     4. Unsafe Control Actions ($UCA-1..N$) covering all 4 failure modes: (a) Not providing causes hazard, (b) Providing causes hazard, (c) Providing too early, too late, or out of order, (d) Stopped too soon or applied too long
     5. Loss Scenarios ($LS-1..N$) & Causal Factors
     6. Formal Safety Constraints ($SC-1..N$)
     7. FMECA Criticality Matrix: Component failure modes with 15+ rows, Severity ($S$), Occurrence ($O$), Detection ($D$), and Risk Priority Numbers ($\text{RPN} = S \times O \times D$)
     8. SORA SAIL Risk Mitigations & OSO Traceability Table: Final GRC, ARC, SAIL classification (SAIL I–VI), and comprehensive mapping of all 24 SORA OSOs (OSO-01 through OSO-24)
   - Include ASTM F3269-17 Run-Time Assurance (RTA) Safety Net monitor architecture.
   - Include MATLAB / Simulink / Stateflow / Embedded Coder model integration baseline hooks and SLDV formal proof properties.
   - KaTeX / LaTeX Math Formatting Mandate: All multi-line aligned equations MUST be enclosed in `\begin{aligned} ... \end{aligned}` within `$$` delimiters on dedicated lines. Bare alignment tabs `&` outside an alignment environment (`aligned`, `matrix`, `cases`) and `\begin{align*}` environments are strictly forbidden. Markdown Table Math Prohibition Rule: Strictly ban `$ ... $` and `$$ ... $$` LaTeX math delimiters inside table headers, rows, and cells; plain text and Unicode (e.g. `Initial S`, `ΔV`, `λ`, `°C`, `≥`, `≤`, `→`, `10⁻⁶`) must be used instead, with 1:1 column count match between header and delimiter rows.

PROCEED
```

#### 4.2.3 Worker 0C: SysML v2 Architectural & Safety Model Author Prompt

```text
Role: Worker 0C — SysML v2 Architectural & Safety Model Author

Primary Commercial Toolchain Integration Context:
This project explicitly declares MATLAB / Simulink / Stateflow / Embedded Coder as the Primary Tier-1 Commercial Toolchain Integration Context (Model-Based Design, Control Law Synthesis, DO-178C C/SPARK Ada code generation).

Directive:
Formalize the CONOPS (`CONOPS.md`), STPA hazard matrices, FMECA ratings, and SORA SAIL requirements (`STPA_MATRIX.md`) into a normative SysML v2 textual model and serialized AST handoff contract.

1. Model Engineering Mandate:
   - Construct `DEAP_MODEL.sysml` conforming to SysML v2 textual specification standards (`package`, `req`, `part`, `port`, `state`, `satisfy`, `verify`).
   - Define safety statecharts for Run-Time Assurance (RTA) switching logic, contingency flight modes, and fail-safe Return-to-Launch (RTL) transitions.
   - Establish MATLAB / Simulink / Stateflow export compatibility for DO-178C C/SPARK Ada code synthesis.
   - KaTeX / LaTeX Math Formatting Mandate: Ensure any statechart/mathematical transition guards and formal expressions follow standard escaping and valid KaTeX blocks (all multi-line aligned equations MUST be enclosed in `\begin{aligned} ... \end{aligned}` within `$$` delimiters on dedicated lines; bare alignment tabs `&` outside an alignment environment and `\begin{align*}` are strictly forbidden). Markdown Table Math Prohibition Rule: Strictly ban `$ ... $` and `$$ ... $$` LaTeX math delimiters inside table headers, rows, and cells; plain text and Unicode (e.g. `Initial S`, `ΔV`, `λ`, `°C`, `≥`, `≤`, `→`, `10⁻⁶`) must be used instead, with 1:1 column count match between header and delimiter rows.

2. Output Requirements:
   - Generate `DEAP_MODEL.sysml` under `docs/architecture/blueprints/DEAP_MODEL.sysml`.
   - Generate `pipeline0_handoff_contract.json` under `.pipeline/contracts/pipeline0_handoff_contract.json` for downstream Pipeline 1 Agile projection and Pipeline 2 code generation.

PROCEED
```

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
        pytest.skip("Downstream project detected — skipping upstream landing zone clean check.")

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

if [ ! -f "$TARGET_DIR/tests/test_safety_integrity.py" ]; then
  cat << 'EOF' > "$TARGET_DIR/tests/test_safety_integrity.py"
"""
Safety Integrity Quality Gate & SORA OSO-01..24 Completeness Verification Suite.
/// Realises: [SafetyIntegrityQualityGate, SORACompleteness, ASTM_F3269_RTA]
"""
import os
import sys
import tempfile
import pytest

# Ensure scripts directory is in sys.path
repo_root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
if repo_root not in sys.path:
    sys.path.insert(0, repo_root)

from scripts.verify_downstream_baseline import (
    count_fmeca_rows,
    check_uca_categories,
    check_sora_osos,
    validate_safety_matrix_content,
    check_safety_integrity_and_sora_completeness,
)


def generate_valid_stpa_matrix_content(fmeca_row_count=16, include_all_osos=True):
    """Generate a fully conforming 8-pillar STPA_MATRIX.md string."""
    fmeca_rows = []
    for i in range(1, fmeca_row_count + 1):
        fmeca_rows.append(
            f"| FM-{i:02d} | Subsystem-{i} | Failure Mode {i} | Local Effect {i} | System Loss L-1 | 4 | 2 | 2 | 16 | Redundant Channel {i} |"
        )
    fmeca_table_str = "\n".join(fmeca_rows)

    osos_list = [f"- **OSO-{i:02d}**: Robustness Level High / Satisfied via Architecture" for i in range(1, 25)]
    if not include_all_osos:
        osos_list = osos_list[:-2]  # Remove OSO-23 and OSO-24
    osos_str = "\n".join(osos_list)

    header_suffix = "(OSO-01 through OSO-24)" if include_all_osos else "(Partial OSO Set)"
    return rf"""# STPA Safety Analysis, FMECA Matrix & SORA SAIL Assessment

> **Primary Commercial Toolchain Integration Context:** MATLAB / Simulink / Stateflow / Embedded Coder  
> **Safety Standards:** JARUS SORA v2.5 | ASTM F3269-17 RTA | RTCA DO-365B  

---

## 1. System Losses (**L-1..N**)

- **L-1**: Loss of human life or severe ground fatal injury.
- **L-2**: Mid-air collision with crewed aircraft.
- **L-3**: Total loss of UAS airframe and critical infrastructure payload.

---

## 2. System Hazards (**H-1..N**)

- **H-1**: Aircraft breaches 3D operational containment geofence boundary.
- **H-2**: Aircraft violates RTCA DO-365B DAA well-clear safety separation.
- **H-3**: Uncontrolled flight termination due to propulsion/actuator loss.

---

## 3. Hierarchical Control Structure Topology

The control structure consists of the Remote Pilot in Command (RPIC), Autopilot Flight Controller, ASTM F3269-17 Run-Time Assurance (RTA) Safety Net Monitor, Actuator Servos, and Telemetry Sensor Suite.

```mermaid
flowchart TD
    RPIC["Remote Pilot in Command"] --> Autopilot["Autopilot Flight Controller"]
    Autopilot --> RTA["ASTM F3269-17 RTA Monitor"]
    RTA --> Actuator["Actuator Servos / Flight Surfaces"]
    Sensors["IMU / GPS / DAA Sensors"] --> RTA
    Sensors --> Autopilot
```

---

## 4. Unsafe Control Actions (**UCA-1..N**)

Systematic identification across 4 STPA guide words / failure mode categories:

1. **Not providing causes hazard**:
   - `UCA-01`: Not providing emergency parachute deployment command when uncontrolled descent detected.
2. **Providing causes hazard**:
   - `UCA-02`: Providing motor cutoff command during active low-altitude hover over populated area.
3. **Providing too early, too late, or out of order**:
   - `UCA-03`: Providing collision avoidance maneuver too late after DAA boundary violation.
4. **Stopped too soon or applied too long**:
   - `UCA-04`: Stopped too soon contingency Return-to-Launch climb before reaching minimum safe altitude.

---

## 5. Loss Scenarios (**LS-1..N**) & Causal Factors

- **LS-1**: Primary GNSS spoofing causes false position estimation, leading to geofence boundary breach (**H-1**, **L-1**).
- **LS-2**: Actuator telemetry packet loss stalls flight control surface transition.

---

## 6. Formal Safety Constraints (**SC-1..N**)

- **SC-1**: The flight control system shall enforce pitch limits between $-15^\circ$ and $+25^\circ$ under all operating conditions.
- **SC-2**: The ASTM F3269-17 RTA Safety Net shall switch to certified safe-state recovery within 50ms of barrier violation.

---

## 7. FMECA Criticality Matrix

| Failure ID | Component / Subsystem | Failure Mode | Local Effect | System Effect | S | O | D | RPN | Mitigating Design Control |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- |
{fmeca_table_str}

---

## 8. SORA SAIL Risk Mitigations & OSO Traceability Table

- **Ground Risk Class (GRC):** Final GRC = 4 (Initial GRC = 5, M1/M2 mitigations applied).
- **Air Risk Class (ARC):** Final ARC-c.
- **Specific Assurance and Integrity Level (SAIL):** SAIL III.

### Operational Safety Objectives {header_suffix}

{osos_str}

---

## 9. ASTM F3269-17 Run-Time Assurance (RTA) & Commercial Toolchain Architecture

The safety net monitor architecture complies with **ASTM F3269-17** Run-Time Assurance (RTA) for Aircraft Systems. Formal invariant proofs and Stateflow recovery supervisors are synthesized directly into **MATLAB / Simulink / Stateflow / Embedded Coder** and verified with Simulink Design Verifier (SLDV).
"""


def test_upstream_safety_landing_zone_clean():
    """Verify that upstream distribution templates enforce clean docs/safety/ landing zone."""
    if os.path.isdir(os.path.join(repo_root, ".pipeline", "upstream")):
        safety_dir = os.path.join(repo_root, "docs", "safety")
        if os.path.isdir(safety_dir):
            allowed = {".gitkeep", "README.md"}
            for f in os.listdir(safety_dir):
                assert f in allowed, f"Upstream template contains non-template file in docs/safety/: {f}"


def test_upstream_safety_landing_zone_dirty_fails():
    """Verify check_safety_integrity_and_sora_completeness rejects dirty upstream safety landing zones."""
    with tempfile.TemporaryDirectory() as tmpdir:
        os.makedirs(os.path.join(tmpdir, ".pipeline", "upstream"), exist_ok=True)
        os.makedirs(os.path.join(tmpdir, "docs", "safety"), exist_ok=True)

        # Write allowed README.md
        with open(os.path.join(tmpdir, "docs", "safety", "README.md"), "w") as f:
            f.write("# Safety Directory\n")

        # Write concrete spec file (violation)
        with open(os.path.join(tmpdir, "docs", "safety", "STPA_MATRIX.md"), "w") as f:
            f.write("# Concrete STPA Matrix\n")

        with pytest.raises(SystemExit) as exc_info:
            check_safety_integrity_and_sora_completeness(tmpdir)
        assert exc_info.value.code == 1


def test_downstream_8_pillar_passing():
    """Verify that a complete 8-pillar STPA matrix passes validation with zero errors."""
    valid_content = generate_valid_stpa_matrix_content(fmeca_row_count=16, include_all_osos=True)
    errors = validate_safety_matrix_content(valid_content)
    assert not errors, f"Expected 0 errors for valid 8-pillar STPA matrix, got:\n{errors}"


def test_sora_oso_01_to_24_validation():
    """Verify all 24 SORA OSOs (OSO-01 through OSO-24) are rigorously validated."""
    # Test complete list
    all_osos_text = " ".join([f"OSO-{i:02d}" for i in range(1, 25)])
    assert check_sora_osos(all_osos_text) == []

    # Test missing OSO-07 and OSO-24
    partial_osos_text = " ".join([f"OSO-{i:02d}" for i in range(1, 25) if i not in (7, 24)])
    missing = check_sora_osos(partial_osos_text)
    assert missing == ["OSO-07", "OSO-24"]

    # Test within full document
    incomplete_content = generate_valid_stpa_matrix_content(include_all_osos=False)
    errors = validate_safety_matrix_content(incomplete_content)
    assert any("OSO-23" in err and "OSO-24" in err for err in errors), f"Expected missing OSOs error, got:\n{errors}"


def test_fmeca_row_count_validation():
    """Verify FMECA matrix row count requires at least 15 component rows."""
    valid_content_16 = generate_valid_stpa_matrix_content(fmeca_row_count=16)
    assert count_fmeca_rows(valid_content_16) >= 15
    assert validate_safety_matrix_content(valid_content_16) == []

    # Exactly 15 rows
    valid_content_15 = generate_valid_stpa_matrix_content(fmeca_row_count=15)
    assert count_fmeca_rows(valid_content_15) == 15
    assert validate_safety_matrix_content(valid_content_15) == []

    # Less than 15 rows (e.g. 5 rows)
    invalid_content_5 = generate_valid_stpa_matrix_content(fmeca_row_count=5)
    assert count_fmeca_rows(invalid_content_5) == 5
    errors = validate_safety_matrix_content(invalid_content_5)
    assert any("FMECA Criticality Matrix contains 5 row(s); minimum required is 15 rows" in err for err in errors)


def test_uca_failure_mode_categories():
    """Verify all 4 STPA UCA failure mode categories are required."""
    all_cats_text = (
        "1. Not providing causes hazard\n"
        "2. Providing causes hazard\n"
        "3. Providing too early, too late, or out of order\n"
        "4. Stopped too soon or applied too long"
    )
    assert check_uca_categories(all_cats_text) == []

    # Missing "Not providing"
    no_omission = (
        "2. Providing causes hazard\n"
        "3. Providing too early, too late, or out of order\n"
        "4. Stopped too soon or applied too long"
    )
    missing = check_uca_categories(no_omission)
    assert any("Not providing" in m for m in missing)


def test_astm_f3269_rta_and_commercial_toolchain_hooks():
    """Verify ASTM F3269-17 RTA and MATLAB/Simulink hooks are strictly enforced."""
    base_content = generate_valid_stpa_matrix_content()

    # Strip ASTM F3269-17
    no_rta = base_content.replace("ASTM F3269-17", "").replace("ASTM F3269", "")
    errors = validate_safety_matrix_content(no_rta)
    assert any("ASTM F3269-17" in err for err in errors)

    # Strip MATLAB / Simulink
    no_matlab = base_content.replace("MATLAB", "").replace("Simulink", "").replace("Stateflow", "").replace("Embedded Coder", "").replace("SLDV", "")
    errors = validate_safety_matrix_content(no_matlab)
    assert any("MATLAB / Simulink" in err for err in errors)


def test_end_to_end_check_17_downstream_integration():
    """Verify end-to-end Check 17 execution on downstream project directory."""
    with tempfile.TemporaryDirectory() as tmpdir:
        # Downstream project (no .pipeline/upstream)
        safety_dir = os.path.join(tmpdir, "docs", "safety")
        os.makedirs(safety_dir, exist_ok=True)

        stpa_file = os.path.join(safety_dir, "STPA_MATRIX.md")
        valid_content = generate_valid_stpa_matrix_content(fmeca_row_count=16, include_all_osos=True)

        with open(stpa_file, "w", encoding="utf-8") as f:
            f.write(valid_content)

        # Should pass with no exception
        check_safety_integrity_and_sora_completeness(tmpdir)

        # Corrupt file with violation (drop OSO-24)
        corrupted_content = valid_content.replace("OSO-24", "INVALID-REF")
        with open(stpa_file, "w", encoding="utf-8") as f:
            f.write(corrupted_content)

        with pytest.raises(SystemExit) as exc_info:
            check_safety_integrity_and_sora_completeness(tmpdir)
        assert exc_info.value.code == 1
EOF
fi

if [ -f "$TARGET_DIR/scripts/setup_git_hooks.py" ]; then
  (cd "$TARGET_DIR" && python3 scripts/setup_git_hooks.py) || true
fi

# Automatically bootstrap issue tracker label taxonomy
echo "Bootstrapping issue tracker label taxonomy..."
if [ -f "$TARGET_DIR/skills/spec-orchestrator/scripts/bootstrap_tracker_labels.py" ]; then
  python3 "$TARGET_DIR/skills/spec-orchestrator/scripts/bootstrap_tracker_labels.py" || {
    echo "Note: Tracker labels could not be provisioned automatically (e.g. offline or unauthenticated)."
    echo "You can re-run label provisioning anytime: python3 skills/spec-orchestrator/scripts/bootstrap_tracker_labels.py"
  }
fi

echo "==> Digital Pipeline Installation Complete. 0 manual steps remaining."

