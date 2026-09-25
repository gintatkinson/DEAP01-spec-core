#!/usr/bin/env bash
set -e

INSTALLER_ROOT="$(cd -P "$(dirname "${BASH_SOURCE[0]}")/.." && pwd -P)"
TARGET_DIR=""
PROVIDER="auto"
GITLAB_URL="https://gitlab.com"
GITLAB_GROUP=""
GITHUB_ORG=""
JIRA_URL="https://your-domain.atlassian.net"
JIRA_PROJECT=""
JIRA_EMAIL=""
DOMAIN_URL=""
DOMAIN_NAME=""
CLI_ROLE=""

show_help() {
  cat << 'EOF'
Usage: install_pipeline.sh [OPTIONS] [TARGET_DIR]

Installs the DEAP safety-critical engineering pipeline and governance baseline into a downstream project repository.

Primary Commercial Toolchain Integration Context:
  MATLAB / Simulink / Stateflow / Embedded Coder (Model-Based Design, Control Law Synthesis, DO-178C C/SPARK Ada code generation).

Arguments:
  TARGET_DIR                 Target project directory (default: current directory '.')

Options:
  -r, --role ROLE            Target repository role: 'domain-template' or 'customer-project' (auto-detected if omitted)
  -p, --provider PROVIDER    Target issue tracker and CI/CD provider: 'github', 'gitlab', 'jira', or 'auto' (default: 'auto')
  -t, --tracker TRACKER      Alias for --provider: 'github', 'gitlab', 'jira', or 'auto'
      --platform PLATFORM    Alias for --provider: 'github', 'gitlab', 'jira', or 'auto'
      --gitlab-url URL       GitLab instance base URL (default: 'https://gitlab.com')
      --gitlab-group GROUP   GitLab namespace/group path (e.g. 'uas-safety', auto-detected from git remote if omitted)
      --github-org ORG       GitHub organization/user (auto-detected from git remote if omitted)
      --jira-url URL         Jira instance base URL (default: 'https://your-domain.atlassian.net')
      --jira-project PROJECT Jira project key code (e.g. 'UAS')
      --jira-email EMAIL     Jira account email address (for Jira Cloud Basic Auth)
      --domain-url URL       Explicit remote URL for upstream domain template repository
      --domain-name NAME     Domain template or project name (e.g. 'DEAP-uas-infrastructure-safety')
  -h, --help                 Display this help documentation and exit

Examples:
  ./scripts/install_pipeline.sh
  ./scripts/install_pipeline.sh .
  ./scripts/install_pipeline.sh /path/to/downstream-project
  ./scripts/install_pipeline.sh --role domain-template
  ./scripts/install_pipeline.sh --role customer-project
  ./scripts/install_pipeline.sh --platform gitlab
  ./scripts/install_pipeline.sh --provider gitlab --gitlab-url https://gitlab.internal.defense.gov
  ./scripts/install_pipeline.sh --tracker jira --jira-url https://my-org.atlassian.net --jira-project PROJ
  ./scripts/install_pipeline.sh --provider github
EOF
}

# Parse CLI options and arguments
while [[ $# -gt 0 ]]; do
  case "$1" in
    -h|--help)
      show_help
      exit 0
      ;;
    -r|--role)
      if [[ -z "$2" || "$2" == -* ]]; then
        echo "Error: $1 requires a role argument ('domain-template' or 'customer-project')." >&2
        exit 1
      fi
      CLI_ROLE="$2"
      shift 2
      ;;
    --role=*)
      CLI_ROLE="${1#*=}"
      shift
      ;;
    -r=*)
      CLI_ROLE="${1#*=}"
      shift
      ;;
    -p|--provider|-t|--tracker|--platform)
      if [[ -z "$2" || "$2" == -* ]]; then
        echo "Error: $1 requires an argument ('github', 'gitlab', 'jira', or 'auto')." >&2
        exit 1
      fi
      PROVIDER="$2"
      shift 2
      ;;
    --provider=*|--tracker=*|--platform=*)
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
    --github-org)
      if [[ -z "$2" || "$2" == -* ]]; then
        echo "Error: --github-org requires an organization/user argument." >&2
        exit 1
      fi
      GITHUB_ORG="$2"
      shift 2
      ;;
    --github-org=*)
      GITHUB_ORG="${1#*=}"
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
    --domain-url)
      if [[ -z "$2" || "$2" == -* ]]; then
        echo "Error: --domain-url requires a URL argument." >&2
        exit 1
      fi
      DOMAIN_URL="$2"
      shift 2
      ;;
    --domain-url=*)
      DOMAIN_URL="${1#*=}"
      shift
      ;;
    --domain-name)
      if [[ -z "$2" || "$2" == -* ]]; then
        echo "Error: --domain-name requires a name argument." >&2
        exit 1
      fi
      DOMAIN_NAME="$2"
      shift 2
      ;;
    --domain-name=*)
      DOMAIN_NAME="${1#*=}"
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

mkdir -p "$TARGET_DIR"
chmod u+w "$TARGET_DIR" 2>/dev/null || true
TARGET_DIR="$(cd -P "$TARGET_DIR" 2>/dev/null && pwd -P || echo "$TARGET_DIR")"
DIR_BASE="$(basename "$TARGET_DIR")"

TARGET_ROLE=""
if [ -n "$CLI_ROLE" ]; then
  case "$(echo "$CLI_ROLE" | tr '[:upper:]' '[:lower:]' | tr '-' '_')" in
    domain_template|domain|domain_distribution_template)
      TARGET_ROLE="DOMAIN_DISTRIBUTION_TEMPLATE"
      ;;
    customer_project|customer|downstream_customer_project|downstream_application_workspace|workspace)
      TARGET_ROLE="DOWNSTREAM_CUSTOMER_PROJECT"
      ;;
    *)
      echo "Error: Invalid --role '$CLI_ROLE'. Valid values: 'domain-template', 'customer-project', 'DOMAIN_DISTRIBUTION_TEMPLATE', 'DOWNSTREAM_CUSTOMER_PROJECT'." >&2
      exit 1
      ;;
  esac
fi

if [ "$TARGET_DIR" = "$INSTALLER_ROOT" ]; then
  if [ -e "$INSTALLER_ROOT/.pipeline/upstream" ]; then
    echo "REFUSING: target is the pipeline repository itself, not a downstream project." >&2
    exit 1
  fi
  echo "Operating in-place on initialized downstream repository: $TARGET_DIR"
fi

# Auto-detect platform and namespace/group from git remote in TARGET_DIR if omitted or auto
REMOTE_URL=$(git -C "$TARGET_DIR" remote get-url origin 2>/dev/null || git -C "$TARGET_DIR" config --get remote.origin.url 2>/dev/null || true)

if [ -n "$REMOTE_URL" ]; then
  DETECTED_INFO=$(python3 -c "
import re, urllib.parse, sys

url = '''$REMOTE_URL'''.strip()
if url.endswith('.git'):
    url = url[:-4]

if '://' in url:
    parsed = urllib.parse.urlsplit(url)
    netloc = parsed.netloc
    host = netloc.split('@')[-1].split(':')[0]
    scheme = parsed.scheme if parsed.scheme in ('http', 'https') else 'https'
    server_url = f'{scheme}://{host}'
    path = parsed.path.strip('/')
else:
    match = re.match(r'^(?:[^@]+@)?([^:/]+):?(?:\d+)?(?:/|:)?(.*)$', url)
    if match:
        host = match.group(1)
        path = match.group(2).strip('/')
        server_url = f'https://{host}'
    else:
        host = ''
        path = url.strip('/')
        server_url = ''

parts = [p for p in path.split('/') if p]
project = parts[-1] if parts else ''
namespace = '/'.join(parts[:-1]) if len(parts) > 1 else ''

platform = 'unknown'
if 'gitlab' in host.lower() or 'gitlab' in url.lower():
    platform = 'gitlab'
elif 'github' in host.lower() or 'github' in url.lower():
    platform = 'github'

print(f'{platform}\t{server_url}\t{namespace}\t{project}')
" 2>/dev/null || true)

  if [ -n "$DETECTED_INFO" ]; then
    DETECTED_PLATFORM=$(echo "$DETECTED_INFO" | cut -f1)
    DETECTED_SERVER_URL=$(echo "$DETECTED_INFO" | cut -f2)
    DETECTED_NAMESPACE=$(echo "$DETECTED_INFO" | cut -f3)
    DETECTED_PROJECT=$(echo "$DETECTED_INFO" | cut -f4)

    # Auto-detect provider/platform if not specified or set to auto
    if [ "$PROVIDER" = "auto" ] && [ "$DETECTED_PLATFORM" != "unknown" ]; then
      PROVIDER="$DETECTED_PLATFORM"
      echo "Auto-detected platform '$PROVIDER' from git remote: $REMOTE_URL"
    fi

    # Auto-detect group/namespace if platform is gitlab
    if [ "$PROVIDER" = "gitlab" ]; then
      if [ -z "$GITLAB_GROUP" ] && [ -n "$DETECTED_NAMESPACE" ]; then
        GITLAB_GROUP="$DETECTED_NAMESPACE"
        echo "Auto-detected GitLab group '$GITLAB_GROUP' from git remote"
      fi
      if [ "$GITLAB_URL" = "https://gitlab.com" ] && [ -n "$DETECTED_SERVER_URL" ] && [ "$DETECTED_SERVER_URL" != "https://gitlab.com" ]; then
        GITLAB_URL="$DETECTED_SERVER_URL"
        echo "Auto-detected GitLab server URL '$GITLAB_URL' from git remote"
      fi
    fi

    # Auto-detect org/namespace if platform is github
    if [ "$PROVIDER" = "github" ]; then
      if [ -z "$GITHUB_ORG" ] && [ -n "$DETECTED_NAMESPACE" ]; then
        GITHUB_ORG="$DETECTED_NAMESPACE"
        echo "Auto-detected GitHub organization '$GITHUB_ORG' from git remote"
      fi
    fi
  fi
fi

# If provider is still 'auto' (e.g. no git remote or unknown host), default to 'github'
if [ "$PROVIDER" = "auto" ]; then
  PROVIDER="github"
fi

if [[ "$PROVIDER" != "github" && "$PROVIDER" != "gitlab" && "$PROVIDER" != "jira" ]]; then
  echo "Error: Invalid provider '$PROVIDER'. Must be one of 'github', 'gitlab', 'jira', or 'auto'." >&2
  exit 1
fi

# Determine TARGET_ROLE if not explicitly specified via --role
if [ -z "$TARGET_ROLE" ]; then
  if [[ "$DIR_BASE" == uav-* ]] || [[ "$DETECTED_PROJECT" == uav-* ]]; then
    TARGET_ROLE="DOWNSTREAM_CUSTOMER_PROJECT"
  elif [ -n "$DOMAIN_URL" ] || \
       [ -n "$DOMAIN_NAME" ] || \
       [[ "$DIR_BASE" == DEAP-* ]] || \
       [[ "$REMOTE_URL" == *DEAP-* ]] || \
       [[ "$DETECTED_PROJECT" == DEAP-* ]]; then
    TARGET_ROLE="DOMAIN_DISTRIBUTION_TEMPLATE"
  elif [ -f "$TARGET_DIR/.pipeline/lineage.json" ]; then
    META_ROLE=$(python3 -c "import json; data=json.load(open('$TARGET_DIR/.pipeline/lineage.json')); print(data.get('role') or data.get('classification') or '')" 2>/dev/null || true)
    if [ "$META_ROLE" = "DOMAIN_DISTRIBUTION_TEMPLATE" ] || [ "$META_ROLE" = "DOWNSTREAM_CUSTOMER_PROJECT" ]; then
      TARGET_ROLE="$META_ROLE"
    else
      TARGET_ROLE="DOWNSTREAM_CUSTOMER_PROJECT"
    fi
  else
    TARGET_ROLE="DOWNSTREAM_CUSTOMER_PROJECT"
  fi
fi

echo "Target repository role: $TARGET_ROLE"

# Preserve any existing downstream project metadata or configuration
PRESERVED_METADATA=""
PRESERVED_PROFILE_CONFIG=""
if [ -f "$TARGET_DIR/.pipeline/project_metadata.json" ]; then
  PRESERVED_METADATA=$(cat "$TARGET_DIR/.pipeline/project_metadata.json")
fi
if [ -f "$TARGET_DIR/.pipeline/profile_config.json" ]; then
  PRESERVED_PROFILE_CONFIG=$(cat "$TARGET_DIR/.pipeline/profile_config.json")
fi

if [ "$TARGET_DIR" != "$INSTALLER_ROOT" ]; then
  chmod -R u+w "$TARGET_DIR/skills" "$TARGET_DIR/rules" "$TARGET_DIR/.pipeline" "$TARGET_DIR/.agents" "$TARGET_DIR/scripts" 2>/dev/null || true
  rm -rf "$TARGET_DIR/skills" "$TARGET_DIR/rules" "$TARGET_DIR/.pipeline" "$TARGET_DIR/.agents" "$TARGET_DIR/scripts"
  cp -RPf "$INSTALLER_ROOT/skills" "$TARGET_DIR/"
  cp -RPf "$INSTALLER_ROOT/rules" "$TARGET_DIR/"
  cp -RPf "$INSTALLER_ROOT/.pipeline" "$TARGET_DIR/"
  rm -rf "$TARGET_DIR/.pipeline/upstream"
  rm -rf "$TARGET_DIR/.pipeline/diagnostics"

  if [ -n "$PRESERVED_METADATA" ]; then
    chmod u+w "$TARGET_DIR/.pipeline/project_metadata.json" 2>/dev/null || true
    echo "$PRESERVED_METADATA" > "$TARGET_DIR/.pipeline/project_metadata.json"
  fi
  if [ -n "$PRESERVED_PROFILE_CONFIG" ]; then
    chmod u+w "$TARGET_DIR/.pipeline/profile_config.json" 2>/dev/null || true
    echo "$PRESERVED_PROFILE_CONFIG" > "$TARGET_DIR/.pipeline/profile_config.json"
  fi
  cp -RPf "$INSTALLER_ROOT/.agents" "$TARGET_DIR/"
  cp -RPf "$INSTALLER_ROOT/scripts" "$TARGET_DIR/"
  mkdir -p "$TARGET_DIR/schema"
  chmod -R u+w "$TARGET_DIR/schema" 2>/dev/null || true
  if [ -d "$INSTALLER_ROOT/schema" ]; then
    cp -RPf "$INSTALLER_ROOT/schema/." "$TARGET_DIR/schema/"
  fi
  chmod u+w "$TARGET_DIR/requirements.txt" "$TARGET_DIR/pyproject.toml" 2>/dev/null || true
  cp -Pf "$INSTALLER_ROOT/requirements.txt" "$TARGET_DIR/" 2>/dev/null || true
  cp -Pf "$INSTALLER_ROOT/pyproject.toml" "$TARGET_DIR/" 2>/dev/null || true
  chmod u+w "$TARGET_DIR/.gitignore" 2>/dev/null || true
  if [ -f "$TARGET_DIR/.gitignore" ]; then
    cat "$INSTALLER_ROOT/.gitignore" >> "$TARGET_DIR/.gitignore"
    # Deduplicate lines in .gitignore
    sort -u "$TARGET_DIR/.gitignore" -o "$TARGET_DIR/.gitignore"
  elif [ -f "$INSTALLER_ROOT/.gitignore" ]; then
    cp -Pf "$INSTALLER_ROOT/.gitignore" "$TARGET_DIR/"
  fi
  mkdir -p "$TARGET_DIR/tests"
  chmod -R u+w "$TARGET_DIR/tests" 2>/dev/null || true
  cp -RPf "$INSTALLER_ROOT/tests/fixtures" "$TARGET_DIR/tests/" 2>/dev/null || true
else
  # Running in-place in an initialized downstream project
  chmod -R u+w "$TARGET_DIR/.pipeline" 2>/dev/null || true
  rm -rf "$TARGET_DIR/.pipeline/upstream"
fi

if [ ! -e "$TARGET_DIR/schema" ]; then
  mkdir -p "$TARGET_DIR/schema"
fi
mkdir -p "$TARGET_DIR/tests"
mkdir -p "$TARGET_DIR/docs" "$TARGET_DIR/docs/conops" "$TARGET_DIR/docs/conops/units/conops" "$TARGET_DIR/docs/conops/units/mission_intent" "$TARGET_DIR/docs/interfaces" "$TARGET_DIR/docs/safety" "$TARGET_DIR/docs/architecture/blueprints" "$TARGET_DIR/docs/epics" "$TARGET_DIR/docs/features" "$TARGET_DIR/docs/user-stories" "$TARGET_DIR/docs/use-cases" "$TARGET_DIR/docs/management"
chmod -R u+w "$TARGET_DIR/docs" 2>/dev/null || true
touch "$TARGET_DIR/docs/management/.gitkeep" "$TARGET_DIR/docs/epics/.gitkeep" "$TARGET_DIR/docs/features/.gitkeep" "$TARGET_DIR/docs/user-stories/.gitkeep" "$TARGET_DIR/docs/use-cases/.gitkeep" "$TARGET_DIR/schema/.gitkeep" "$TARGET_DIR/docs/conops/.gitkeep" "$TARGET_DIR/docs/safety/.gitkeep"
if [ "$TARGET_ROLE" = "DOMAIN_DISTRIBUTION_TEMPLATE" ]; then
  rm -f "$TARGET_DIR/schema/README.md" "$TARGET_DIR/docs/conops/README.md" "$TARGET_DIR/docs/safety/README.md"
fi
if [ "$TARGET_DIR" != "$INSTALLER_ROOT" ]; then
  if [ -f "$INSTALLER_ROOT/docs/conops/README.md" ]; then
    cp -Pf "$INSTALLER_ROOT/docs/conops/README.md" "$TARGET_DIR/docs/conops/"
  fi
  if [ -f "$INSTALLER_ROOT/docs/safety/README.md" ]; then
    cp -Pf "$INSTALLER_ROOT/docs/safety/README.md" "$TARGET_DIR/docs/safety/"
  fi
  if [ -f "$INSTALLER_ROOT/docs/OPERATOR_PROMPT_CATALOG.md" ]; then
    cp -Pf "$INSTALLER_ROOT/docs/OPERATOR_PROMPT_CATALOG.md" "$TARGET_DIR/docs/"
  fi
  if [ -f "$INSTALLER_ROOT/docs/JIRA_INTEGRATION_GUIDE.md" ]; then
    cp -Pf "$INSTALLER_ROOT/docs/JIRA_INTEGRATION_GUIDE.md" "$TARGET_DIR/docs/"
  fi
fi
mkdir -p "$TARGET_DIR/.pipeline/contracts" "$TARGET_DIR/.pipeline/domain_specs" "$TARGET_DIR/.pipeline/profiles"
chmod -R u+w "$TARGET_DIR/.pipeline" 2>/dev/null || true
chmod +x "$TARGET_DIR"/scripts/*.sh "$TARGET_DIR"/scripts/*.py 2>/dev/null || true

# Compile consolidated active governance rules manifest into .pipeline/ACTIVE_RULES_BUNDLE.md
echo "Compiling active governance rules into .pipeline/ACTIVE_RULES_BUNDLE.md..."
BUNDLE_FILE="$TARGET_DIR/.pipeline/ACTIVE_RULES_BUNDLE.md"
mkdir -p "$TARGET_DIR/.pipeline"
chmod u+w "$BUNDLE_FILE" 2>/dev/null || true

cat << 'EOF' > "$BUNDLE_FILE"
# ACTIVE RULES BUNDLE — Consolidated Governance Manifest

> **Notice:** This consolidated governance manifest is compiled automatically at installation time by `scripts/install_pipeline.sh`.
> It aggregates 100% of the active governance rules from `rules/` into a single, unified source of truth.
> Autonomous agents (Antigravity, Claude Code, Gemini CLI, Cursor) MUST execute `view_file` on this file to ingest the full suite of active governance rules in a single read before executing any implementation or orchestration tasks.

## Table of Contents

EOF

RULES_SRC="$INSTALLER_ROOT/rules"
if [ ! -d "$RULES_SRC" ] && [ -d "$TARGET_DIR/rules" ]; then
  RULES_SRC="$TARGET_DIR/rules"
fi

for rule_file in "$RULES_SRC"/*.md; do
  [ -f "$rule_file" ] || continue
  rule_base=$(basename "$rule_file")
  rule_slug=$(echo "$rule_base" | tr '[:upper:]' '[:lower:]' | sed -e 's/[^a-z0-9]/-/g' -e 's/--*/-/g' -e 's/^-//' -e 's/-$//')
  echo "- [${rule_base}](#rule-${rule_slug})" >> "$BUNDLE_FILE"
done

echo "" >> "$BUNDLE_FILE"
echo "---" >> "$BUNDLE_FILE"
echo "" >> "$BUNDLE_FILE"

for rule_file in "$RULES_SRC"/*.md; do
  [ -f "$rule_file" ] || continue
  rule_base=$(basename "$rule_file")
  rule_slug=$(echo "$rule_base" | tr '[:upper:]' '[:lower:]' | sed -e 's/[^a-z0-9]/-/g' -e 's/--*/-/g' -e 's/^-//' -e 's/-$//')
  rule_stem="${rule_base%.md}"
  rule_stem_slug=$(echo "$rule_stem" | tr '[:upper:]' '[:lower:]' | sed -e 's/[^a-z0-9]/-/g' -e 's/--*/-/g' -e 's/^-//' -e 's/-$//')
  echo "<a id=\"${rule_slug}\"></a>" >> "$BUNDLE_FILE"
  echo "<a id=\"rule-${rule_slug}\"></a>" >> "$BUNDLE_FILE"
  echo "<a id=\"${rule_stem_slug}\"></a>" >> "$BUNDLE_FILE"
  echo "<a id=\"rule-${rule_stem_slug}\"></a>" >> "$BUNDLE_FILE"
  echo "## Rule: ${rule_base}" >> "$BUNDLE_FILE"
  echo "" >> "$BUNDLE_FILE"
  cat "$rule_file" >> "$BUNDLE_FILE"
  echo "" >> "$BUNDLE_FILE"
  echo "---" >> "$BUNDLE_FILE"
  echo "" >> "$BUNDLE_FILE"
done

# Apply provider configurations if specified
if [ "$PROVIDER" = "gitlab" ] || [ -n "$GITLAB_GROUP" ] || [ "$GITLAB_URL" != "https://gitlab.com" ]; then
  chmod u+w "$TARGET_DIR/.gitlab-ci.yml" 2>/dev/null || true
  if [ -f "$INSTALLER_ROOT/.pipeline/templates/.gitlab-ci.yml" ]; then
    cp -Pf "$INSTALLER_ROOT/.pipeline/templates/.gitlab-ci.yml" "$TARGET_DIR/.gitlab-ci.yml"
  elif [ -f "$INSTALLER_ROOT/.pipeline/.gitlab-ci.yml" ]; then
    cp -Pf "$INSTALLER_ROOT/.pipeline/.gitlab-ci.yml" "$TARGET_DIR/.gitlab-ci.yml"
  elif [ -f "$TARGET_DIR/.pipeline/templates/.gitlab-ci.yml" ]; then
    cp -Pf "$TARGET_DIR/.pipeline/templates/.gitlab-ci.yml" "$TARGET_DIR/.gitlab-ci.yml"
  elif [ -f "$TARGET_DIR/.pipeline/.gitlab-ci.yml" ]; then
    cp -Pf "$TARGET_DIR/.pipeline/.gitlab-ci.yml" "$TARGET_DIR/.gitlab-ci.yml"
  fi
  for rules_file in "$TARGET_DIR/.pipeline/logical-ui/codebase_rules.json" "$TARGET_DIR/codebase_rules.json"; do
    if [ -f "$rules_file" ]; then
      chmod u+w "$rules_file" 2>/dev/null || true
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
      chmod u+w "$rules_file" 2>/dev/null || true
      python3 -c "
import json, sys
path = '$rules_file'
with open(path, 'r', encoding='utf-8') as f:
    data = json.load(f)
if 'tracker_rules' not in data:
    data['tracker_rules'] = {}
data['tracker_rules']['provider'] = 'github'
if '$GITHUB_ORG':
    data['tracker_rules']['owner'] = '$GITHUB_ORG'
with open(path, 'w', encoding='utf-8') as f:
    json.dump(data, f, indent=2)
" 2>/dev/null || true
    fi
  done
fi

# Generate .env.template in target workspace
chmod u+w "$TARGET_DIR/.env.template" 2>/dev/null || true
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
chmod u+w "$TARGET_DIR/AGENTS.md" "$TARGET_DIR/.agents/AGENTS.md" "$TARGET_DIR/CLAUDE.md" "$TARGET_DIR/README.md" 2>/dev/null || true
python3 "$INSTALLER_ROOT/scripts/scaffold_downstream_agents.py" "$INSTALLER_ROOT" "$TARGET_DIR"


# Determine whether downstream root README.md needs scaffolding
SHOULD_SCAFFOLD_README=false

if [ ! -f "$TARGET_DIR/README.md" ]; then
  SHOULD_SCAFFOLD_README=true
elif grep -qE "Getting started with GitLab|To make it easy for you to get started" "$TARGET_DIR/README.md"; then
  SHOULD_SCAFFOLD_README=true
elif ! grep -qE "Multi-Pipeline Operator Prompt Catalog|Operator Prompt Catalog" "$TARGET_DIR/README.md"; then
  SHOULD_SCAFFOLD_README=true
elif [ "$TARGET_ROLE" = "DOMAIN_DISTRIBUTION_TEMPLATE" ]; then
  if ! grep -qE "Customer Project Onboarding|\.tmp-pipeline" "$TARGET_DIR/README.md" || \
     ! grep -qE "DOMAIN_DISTRIBUTION_TEMPLATE" "$TARGET_DIR/README.md" || \
     ! grep -q "ACTIVE_RULES_BUNDLE.md" "$TARGET_DIR/README.md" || \
     grep -q "rules/dual-track-mbd-verification.md" "$TARGET_DIR/README.md" || \
     grep -qE " -- Downstream.* -- Downstream" "$TARGET_DIR/README.md"; then
    SHOULD_SCAFFOLD_README=true
  fi
elif [ "$TARGET_ROLE" = "DOWNSTREAM_CUSTOMER_PROJECT" ]; then
  if grep -qE "git clone.*\.tmp-pipeline" "$TARGET_DIR/README.md" || \
     ! grep -qE "DOWNSTREAM_CUSTOMER_PROJECT" "$TARGET_DIR/README.md" || \
     ! grep -qE "Project Lifecycle & Tooling Maintenance" "$TARGET_DIR/README.md" || \
     ! grep -q "ACTIVE_RULES_BUNDLE.md" "$TARGET_DIR/README.md" || \
     grep -q "rules/dual-track-mbd-verification.md" "$TARGET_DIR/README.md" || \
     grep -qE " -- Downstream.* -- Downstream" "$TARGET_DIR/README.md"; then
    SHOULD_SCAFFOLD_README=true
  fi
fi

if [ "$SHOULD_SCAFFOLD_README" = true ]; then

  # Extract domain project metadata from existing project artifacts
  DOMAIN_PROJECT_NAME=""
  DOMAIN_PROJECT_DESC=""
  DOMAIN_TECH_PROFILE=""
  DOMAIN_REGULATORY=""

  # 0. Check explicit CLI --domain-name parameter
  if [ -n "$DOMAIN_NAME" ]; then
    DOMAIN_PROJECT_NAME="$DOMAIN_NAME"
  fi

  # 1. Check existing README.md before overwriting (e.g. GitLab initial commit: "# <project_name>\n\nGetting started with GitLab...")
  if [ -z "$DOMAIN_PROJECT_NAME" ] && [ -f "$TARGET_DIR/README.md" ]; then
    CANDIDATE_TITLE=$(grep -E '^# ' "$TARGET_DIR/README.md" | head -n 1 | sed 's/^# *//' | tr -d '\r\n')
    CLEAN_CANDIDATE=$(echo "$CANDIDATE_TITLE" | sed -E 's/( -- Downstream Cyber-Physical Infrastructure Safety Project)+$//; s/( -- Downstream Safety-Critical Engineering Project)+$//')
    if ! echo "$CLEAN_CANDIDATE" | grep -qE "Getting started with GitLab|Downstream Cyber-Physical Infrastructure Safety Project|Downstream Safety-Critical Engineering Project" && [ -n "$CLEAN_CANDIDATE" ]; then
      DOMAIN_PROJECT_NAME="$CLEAN_CANDIDATE"
    fi
  fi

  # 2. Check .pipeline/project_metadata.json if present
  if [ -f "$TARGET_DIR/.pipeline/project_metadata.json" ]; then
    META_NAME=$(python3 -c "import json; data=json.load(open('$TARGET_DIR/.pipeline/project_metadata.json')); print(data.get('project_name') or data.get('name') or '')" 2>/dev/null || true)
    if [ -n "$META_NAME" ] && [ -z "$DOMAIN_PROJECT_NAME" ]; then
      DOMAIN_PROJECT_NAME="$META_NAME"
    fi
    META_DESC=$(python3 -c "import json; data=json.load(open('$TARGET_DIR/.pipeline/project_metadata.json')); print(data.get('description') or '')" 2>/dev/null || true)
    if [ -n "$META_DESC" ]; then
      DOMAIN_PROJECT_DESC="$META_DESC"
    fi
    META_PROF=$(python3 -c "import json; data=json.load(open('$TARGET_DIR/.pipeline/project_metadata.json')); print(data.get('technology_profile') or data.get('profile') or '')" 2>/dev/null || true)
    if [ -n "$META_PROF" ]; then
      DOMAIN_TECH_PROFILE="$META_PROF"
    fi
    META_REG=$(python3 -c "import json; data=json.load(open('$TARGET_DIR/.pipeline/project_metadata.json')); print(data.get('regulatory_frameworks') or '')" 2>/dev/null || true)
    if [ -n "$META_REG" ]; then
      DOMAIN_REGULATORY="$META_REG"
    fi
  fi

  # 3. Check codebase_rules.json if present
  if [ -z "$DOMAIN_PROJECT_NAME" ] && [ -f "$TARGET_DIR/codebase_rules.json" ]; then
    CR_NAME=$(python3 -c "import json; data=json.load(open('$TARGET_DIR/codebase_rules.json')); print(data.get('project_name') or data.get('name') or '')" 2>/dev/null || true)
    if [ -n "$CR_NAME" ]; then
      DOMAIN_PROJECT_NAME="$CR_NAME"
    fi
  fi

  # 4. Check pubspec.yaml (Flutter / Dart)
  if [ -f "$TARGET_DIR/pubspec.yaml" ]; then
    if [ -z "$DOMAIN_PROJECT_NAME" ]; then
      PUB_NAME=$(grep -E '^name:' "$TARGET_DIR/pubspec.yaml" | head -n 1 | sed 's/^name:[[:space:]]*//' | tr -d '\r\n')
      [ -n "$PUB_NAME" ] && DOMAIN_PROJECT_NAME="$PUB_NAME"
    fi
    if [ -z "$DOMAIN_PROJECT_DESC" ]; then
      PUB_DESC=$(grep -E '^description:' "$TARGET_DIR/pubspec.yaml" | head -n 1 | sed 's/^description:[[:space:]]*//' | tr -d '\r\n')
      [ -n "$PUB_DESC" ] && DOMAIN_PROJECT_DESC="$PUB_DESC"
    fi
    [ -z "$DOMAIN_TECH_PROFILE" ] && DOMAIN_TECH_PROFILE="\`Flutter / Dart Embedded & Operator Console Profile\`"
  fi

  # 5. Check package.json (React / TypeScript Web)
  if [ -f "$TARGET_DIR/package.json" ]; then
    if [ -z "$DOMAIN_PROJECT_NAME" ]; then
      PKG_NAME=$(python3 -c "import json; data=json.load(open('$TARGET_DIR/package.json')); print(data.get('name') or '')" 2>/dev/null || true)
      [ -n "$PKG_NAME" ] && DOMAIN_PROJECT_NAME="$PKG_NAME"
    fi
    if [ -z "$DOMAIN_PROJECT_DESC" ]; then
      PKG_DESC=$(python3 -c "import json; data=json.load(open('$TARGET_DIR/package.json')); print(data.get('description') or '')" 2>/dev/null || true)
      [ -n "$PKG_DESC" ] && DOMAIN_PROJECT_DESC="$PKG_DESC"
    fi
    [ -z "$DOMAIN_TECH_PROFILE" ] && DOMAIN_TECH_PROFILE="\`React / TypeScript Web Operator Profile\`"
  fi

  # 6. Check pyproject.toml
  if [ -f "$TARGET_DIR/pyproject.toml" ]; then
    if [ -z "$DOMAIN_PROJECT_NAME" ]; then
      PYP_NAME=$(grep -E '^name[[:space:]]*=' "$TARGET_DIR/pyproject.toml" | head -n 1 | sed -E 's/^name[[:space:]]*=[[:space:]]*["\x27]([^"\x27]+)["\x27]/\1/' | tr -d '\r\n')
      if [ -n "$PYP_NAME" ] && [ "$PYP_NAME" != "deap01-spec-core" ]; then
        DOMAIN_PROJECT_NAME="$PYP_NAME"
      fi
    fi
  fi

  # 7. Check active platform profile configuration
  if [ -z "$DOMAIN_TECH_PROFILE" ]; then
    if [ -f "$TARGET_DIR/.pipeline/profile_config.json" ]; then
      CONF_PROF=$(python3 -c "import json; data=json.load(open('$TARGET_DIR/.pipeline/profile_config.json')); print(data.get('active_profile') or '')" 2>/dev/null || true)
      [ -n "$CONF_PROF" ] && DOMAIN_TECH_PROFILE="\`$CONF_PROF\`"
    elif [ -f "$TARGET_DIR/CMakeLists.txt" ] || [ -f "$TARGET_DIR/package.xml" ]; then
      DOMAIN_TECH_PROFILE="\`ROS2 C++ Real-Time\` | \`Target Embedded Platform Execution Profile\`"
    fi
  fi

  # Fallbacks if still empty
  if [ -z "$DOMAIN_PROJECT_NAME" ]; then
    DIR_BASE=$(basename "$(cd "$TARGET_DIR" 2>/dev/null && pwd || echo "$TARGET_DIR")")
    if [ -n "$DIR_BASE" ] && [ "$DIR_BASE" != "." ] && [ "$DIR_BASE" != "/" ]; then
      DOMAIN_PROJECT_NAME="$DIR_BASE"
    else
      DOMAIN_PROJECT_NAME="Downstream Cyber-Physical Infrastructure Safety Project"
    fi
  fi

  if [ -z "$DOMAIN_PROJECT_DESC" ]; then
    DOMAIN_PROJECT_DESC="This repository is an installed downstream implementation workspace governed by the **Digital Engineering Agent Platform (DEAP)** for cyber-physical infrastructure safety, real-time control, run-time assurance (RTA), and autonomous operations."
  fi

  if [ -z "$DOMAIN_TECH_PROFILE" ]; then
    DOMAIN_TECH_PROFILE="\`ROS2 C++ Real-Time\` | \`Target Embedded Platform Execution Profile\`"
  fi

  if [ -z "$DOMAIN_REGULATORY" ]; then
    DOMAIN_REGULATORY="\`JARUS SORA v2.5 (SAIL I–VI)\` | \`ASTM F3269-17 RTA\` | \`ASTM F3411-22a Remote ID\` | \`RTCA DO-365B DAA\`"
  fi

  # Strip any redundant trailing suffix from DOMAIN_PROJECT_NAME to avoid duplicate title suffixes
  DOMAIN_PROJECT_NAME=$(echo "$DOMAIN_PROJECT_NAME" | sed -E 's/( -- Downstream Cyber-Physical Infrastructure Safety Project)+$//; s/( -- Downstream Safety-Critical Engineering Project)+$//')

  if [ "$DOMAIN_PROJECT_NAME" = "Downstream Cyber-Physical Infrastructure Safety Project" ] || [ -z "$DOMAIN_PROJECT_NAME" ]; then
    README_TITLE="# Downstream Cyber-Physical Infrastructure Safety Project"
  else
    README_TITLE="# ${DOMAIN_PROJECT_NAME} -- Downstream Cyber-Physical Infrastructure Safety Project"
  fi

  # Resolve domain repository git remote URL for customer onboarding instructions
  DOMAIN_REMOTE_URL=""
  if [ -n "$DOMAIN_URL" ]; then
    DOMAIN_REMOTE_URL="$DOMAIN_URL"
  elif [ -n "$REMOTE_URL" ] && ! echo "$REMOTE_URL" | grep -q "DEAP01-spec-core"; then
    DOMAIN_REMOTE_URL="$REMOTE_URL"
  elif [ -n "$DETECTED_SERVER_URL" ] && [ -n "$DETECTED_NAMESPACE" ] && [ -n "$DETECTED_PROJECT" ] && [ "$DETECTED_PROJECT" != "DEAP01-spec-core" ]; then
    DOMAIN_REMOTE_URL="${DETECTED_SERVER_URL}/${DETECTED_NAMESPACE}/${DETECTED_PROJECT}.git"
  elif [ -n "$DETECTED_SERVER_URL" ] && [ -n "$DETECTED_NAMESPACE" ] && [ -n "$DOMAIN_PROJECT_NAME" ] && [ "$DOMAIN_PROJECT_NAME" != "Downstream Cyber-Physical Infrastructure Safety Project" ]; then
    CLEAN_NAME=$(echo "$DOMAIN_PROJECT_NAME" | tr ' ' '-')
    DOMAIN_REMOTE_URL="${DETECTED_SERVER_URL}/${DETECTED_NAMESPACE}/${CLEAN_NAME}.git"
  elif [ ! -e "$INSTALLER_ROOT/.pipeline/upstream" ]; then
    INSTALLER_REMOTE=$(git -C "$INSTALLER_ROOT" remote get-url origin 2>/dev/null || git -C "$INSTALLER_ROOT" config --get remote.origin.url 2>/dev/null || true)
    if [ -n "$INSTALLER_REMOTE" ] && ! echo "$INSTALLER_REMOTE" | grep -q "DEAP01-spec-core"; then
      DOMAIN_REMOTE_URL="$INSTALLER_REMOTE"
    fi
  fi

  if [ -z "$DOMAIN_REMOTE_URL" ]; then
    CLEAN_NAME=$(echo "${DOMAIN_PROJECT_NAME:-downstream-project}" | tr ' ' '-')
    DOMAIN_REMOTE_URL="https://github.com/${GITHUB_ORG:-gintatkinson}/${CLEAN_NAME}.git"
  fi

  chmod u+w "$TARGET_DIR/README.md" 2>/dev/null || true
  if [ "$TARGET_ROLE" = "DOMAIN_DISTRIBUTION_TEMPLATE" ]; then
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
- \`.pipeline/\`: Constitution (\`constitution.md\`), active governance rules bundle (\`ACTIVE_RULES_BUNDLE.md\`), domain specifications, and execution profiles (\`profiles/ros2_cpp.md\`, \`profiles/px4_module.md\`).
- \`rules/\` & \`skills/\`: Platform engineering rules and agent workflow skills.
- \`schema/\`: Contract definitions and SysML v2 schemas.
- \`tests/\`: Automated baseline verification and safety compliance tests.

---

## 3. Customer Project Onboarding & Agent Initialization Sequence

### 3.1 Turnkey Customer Project Onboarding

To install this domain pipeline and its engineering baseline into an end-user customer application workspace, run the following turnkey command from your customer project root directory:

\`\`\`bash
# Onboard customer application workspace
git clone ${DOMAIN_REMOTE_URL} ./.tmp-pipeline && bash ./.tmp-pipeline/scripts/install_pipeline.sh . && rm -rf ./.tmp-pipeline
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
3. **Load Governance Rules**: Execute \`view_file\` on \`.pipeline/ACTIVE_RULES_BUNDLE.md\` to ingest the complete, consolidated suite of active governance rules in a single read (covering dual-track MBD, SysML SSOT completeness, role boundary locks, and TDD mandates).
4. **Load Platform Profile**: Read the target platform execution profile (\`.pipeline/profiles/flutter.md\`, \`.pipeline/profiles/react.md\`, \`.pipeline/profiles/ros2_cpp.md\`, or \`.pipeline/profiles/px4_module.md\`) to establish platform-specific build, test, and lifecycle constraints.
5. **Bootstrap Tracker Labels & Verify Baseline**: Verify that repository issue tracker labels and baseline conformance pass by running \`python3 scripts/verify_downstream_baseline.py --no-domain\`.

---

EOF
  else
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
- \`.pipeline/\`: Constitution (\`constitution.md\`), active governance rules bundle (\`ACTIVE_RULES_BUNDLE.md\`), domain specifications, and execution profiles (\`profiles/ros2_cpp.md\`, \`profiles/px4_module.md\`).
- \`rules/\` & \`skills/\`: Platform engineering rules and agent workflow skills.
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
3. **Load Governance Rules**: Execute \`view_file\` on \`.pipeline/ACTIVE_RULES_BUNDLE.md\` to ingest the complete, consolidated suite of active governance rules in a single read (covering dual-track MBD, SysML SSOT completeness, role boundary locks, and TDD mandates).
4. **Load Platform Profile**: Read the target platform execution profile (\`.pipeline/profiles/flutter.md\`, \`.pipeline/profiles/react.md\`, \`.pipeline/profiles/ros2_cpp.md\`, or \`.pipeline/profiles/px4_module.md\`) to establish platform-specific build, test, and lifecycle constraints.
5. **Bootstrap Tracker Labels & Verify Baseline**: Verify that repository issue tracker labels and baseline conformance pass by running \`python3 scripts/verify_downstream_baseline.py --no-domain\`.

---

EOF
  fi

  cat << 'EOF' >> "$TARGET_DIR/README.md"
## 4. Multi-Pipeline Operator Prompt Catalog & Autonomous Execution Workflows

This catalog contains the complete, unabridged, copy-pasteable operator prompt suite for executing all stages of the Digital Engineering Agent Platform (DEAP) lifecycle across context-isolated subagents in Antigravity, Claude Code, Gemini CLI, Cursor, and Cascade.

### 4.1 Master-Worker Subagent Topology

```mermaid
flowchart LR
    Step00["Step 0.0: Level 0 OEM Ground Truth Ingestion (sysmlv2_ingest.py)"] --> Step0["Step 0: SysML Model Ingestion & Compilation Gate (python3 scripts/compile_sysml.py --compile)"]
    Step0 -->|"Compiled AST"| Worker_0A["Worker 0A: CONOPS Synthesizer"]
    Worker_0A -->|"docs/conops/CONOPS.md"| Worker_0B["Worker 0B: STPA / FMECA Assurer"]
    Worker_0B -->|"docs/safety/STPA_MATRIX.md"| Step3["Step 3: Level 1C ICD Extraction & Level 2 Specifications"]
```

### 4.2 Pipeline 0 Execution Prompts

Execute the following prompts in sequence using context-isolated subagents to transform unstructured intent, operational scenarios, and interface schemas into formal CONOPS, STPA hazard matrices, and SysML v2 AST models:

#### 4.2.0 Worker 00: OEM Prose / BOM Ingestion & Model Synthesis Prompt (Step 0.0)

**Step 0.0 Entrypoint for Unstructured / Prose Customer Documentation:**
For customer projects starting with unstructured OEM prose manuals, PDF documentation, markdown tables, or Bill of Materials (BOM) specifications, Worker 00 provides the sanctioned, deterministic entrypoint. Extracting OEM Bill of Materials (BOM) and physical parameters into `schema/extracted/` and synthesizing canonical SysML v2 textual models in `schema/model.sysml` (or `.pipeline/schema.sysml`) is fully authorized under Check 23 (Factual Grounding & Numeric Provenance Gate) and serves as the mandatory precursor to executing the Step 0 compilation gate (`python3 scripts/compile_sysml.py --compile`).

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

#### 4.2.1 Worker 0A: CONOPS & Operational Scenario Synthesis Prompt

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

#### 4.2.2 Worker 0B: STPA Hazard Analysis, FMECA & Domain Safety Assurer Prompt

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

#### 4.2.3 Worker 0C: SysML v2 Architectural & Safety Model Author Prompt

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

#### 4.2.4 Worker 0D: Interface Specification Worker (Logical ICD & Signal Dictionary) Prompt

```text
Execute `view_file` on `skills/spec-icd-engineering/SKILL.md` as your very first step before taking any action.

Repository Classification: UPSTREAM_SPEC_CORE_COMPILER (or DOWNSTREAM_CUSTOMER_PROJECT depending on execution context)

Role: Worker 0D -- Interface Specification Worker (Logical ICD & Signal Dictionary)

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
   - Execute the local model coverage linter: `./skills/spec-orchestrator/scripts/verify_model_coverage.py --spec-only --allow-missing-specs --only <spec_file>`.
   - Register Features first via `./skills/spec-orchestrator/scripts/create_issue.sh "<file>" "feature" "<title>"`.
   - Verify live published payload on the issue tracker (`gh issue view <ID> --json body` or `glab issue view <ID>`).
   - Inject verified Feature Issue IDs into Epic tasklists.
   - Register Epics via `./skills/spec-orchestrator/scripts/create_issue.sh "<file>" "epic" "<title>"`.

Defect Filing Directive:
If any compiler fault, schema inconsistency, or invariant violation is discovered, you are strictly forbidden from filing raw issues directly. You MUST dispatch a fresh context-isolated subagent with `skills/adversarial-code-auditor/SKILL.md` to perform the 5-pillar audit, generate the verified 7-section defect dossier, and submit it via `python3 scripts/file_defect.py`. Issue auto-closing keywords or issue close commands are strictly forbidden.

PROCEED
```


#### 4.3.2 Worker 1B: Behavioral Spec Worker (User Stories & Statecharts) Prompt

```text
Execute `view_file` on `skills/spec-user-story-engineering/SKILL.md` as your very first step before taking any action.

Repository Classification: DOWNSTREAM_CUSTOMER_PROJECT (or UPSTREAM_SPEC_CORE_COMPILER depending on execution context)

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

#### 4.3.3 Worker 1C: Operational Spec Worker (Use Cases & Realization Matrices) Prompt

```text
Execute `view_file` on `skills/spec-usecase-engineering/SKILL.md` as your very first step before taking any action.

Repository Classification: DOWNSTREAM_CUSTOMER_PROJECT (or UPSTREAM_SPEC_CORE_COMPILER depending on execution context)

Role: Worker 1C -- Operational Spec Worker (Use Cases & Realization Matrices)

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

#### 4.3.4 Worker 1D: WBS & Work Package Decomposition Spec Worker Prompt

```text
Execute `view_file` on `skills/spec-wbs-engineering/SKILL.md` as your very first step before taking any action.

Repository Classification: DOWNSTREAM_CUSTOMER_PROJECT (or UPSTREAM_SPEC_CORE_COMPILER depending on execution context)

Role: Worker 1D -- WBS & Work Package Decomposition Spec Worker

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

### 4.4 Multi-Provider Backlog Reconciliation Commands

Execute backlog reconciliation and model parity verification across your target VCS platform or offline air-gapped environment:

#### 4.4.1 Option A: GitLab SaaS Reconciliation
```bash
./scripts/reconcile_backlog.py --provider gitlab
```

#### 4.4.2 Option B: GitLab Self-Managed / SCIF Air-Gapped Reconciliation
```bash
./scripts/reconcile_backlog.py --provider gitlab --gitlab-url https://gitlab.internal.defense.gov --project uas-safety/uav-010
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
Adopt the feature-driven-implementation skill by reading `.pipeline/constitution.md`, `.pipeline/ACTIVE_RULES_BUNDLE.md`, and the target platform profile (`.pipeline/profiles/<target-platform>.md`, e.g. `ros2_cpp.md`, `px4_module.md`, or `flutter.md`).

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
Adopt the feature-driven-implementation skill by reading `.pipeline/constitution.md`, `.pipeline/ACTIVE_RULES_BUNDLE.md`, and `docs/architecture/blueprints/SYSML_SSOT_BIDIRECTIONAL_SYNCHRONIZATION_ARCHITECTURE.md`.

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
# Run downstream conformance gate
python3 scripts/verify_downstream_baseline.py --no-domain
```
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

