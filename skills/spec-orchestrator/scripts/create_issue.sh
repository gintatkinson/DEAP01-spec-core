#!/bin/bash
# Gated issue creation with mandatory linter pass precondition.
# Usage: ./create_issue.sh <body-file> <label> <title> <repo>
set -euo pipefail

CLI_PROVIDER=""
REOPEN_ANALYSIS_FILE=""
REPO=""
POSITIONAL_ARGS=()

while [ $# -gt 0 ]; do
    case "$1" in
        --provider)
            if [ $# -lt 2 ]; then
                echo "FATAL: --provider requires an argument" >&2
                exit 1
            fi
            CLI_PROVIDER="$2"
            shift 2
            ;;
        --provider=*)
            CLI_PROVIDER="${1#*=}"
            shift
            ;;
        --reopen-with-analysis)
            if [ $# -lt 2 ]; then
                echo "FATAL: --reopen-with-analysis requires an argument" >&2
                exit 1
            fi
            REOPEN_ANALYSIS_FILE="$2"
            shift 2
            ;;
        --reopen-with-analysis=*)
            REOPEN_ANALYSIS_FILE="${1#*=}"
            shift
            ;;
        --repo)
            if [ $# -lt 2 ]; then
                echo "FATAL: --repo requires an argument" >&2
                exit 1
            fi
            REPO="$2"
            shift 2
            ;;
        --repo=*)
            REPO="${1#*=}"
            shift
            ;;
        *)
            POSITIONAL_ARGS+=("$1")
            shift
            ;;
    esac
done

if [ "${#POSITIONAL_ARGS[@]}" -lt 3 ]; then
    echo "Usage: $0 <body-file> <label> <title> [repo]" >&2
    exit 1
fi

LOCAL_FILE="${POSITIONAL_ARGS[0]}"
LABEL="${POSITIONAL_ARGS[1]}"
TITLE="${POSITIONAL_ARGS[2]}"
if [ -z "$REPO" ] && [ "${#POSITIONAL_ARGS[@]}" -ge 4 ]; then
    REPO="${POSITIONAL_ARGS[3]}"
fi
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
LINTER="$SCRIPT_DIR/verify_model_coverage.py"

if [ ! -f "$LOCAL_FILE" ]; then
    echo "FATAL: Body file not found: $LOCAL_FILE" >&2
    exit 1
fi

if [ -n "$REOPEN_ANALYSIS_FILE" ] && [ ! -f "$REOPEN_ANALYSIS_FILE" ]; then
    echo "FATAL: Reopen analysis file not found: $REOPEN_ANALYSIS_FILE" >&2
    exit 1
fi

# Provider Auto-Detection:
# Check --provider <github|gitlab> CLI flag, or TRACKER_PROVIDER env var, or inspect git remote URL
PROVIDER="${CLI_PROVIDER:-${TRACKER_PROVIDER:-}}"
if [ -z "$PROVIDER" ] || [ "$PROVIDER" = "auto" ]; then
    REMOTE_URL=$(git remote get-url origin 2>/dev/null || git config --get remote.origin.url 2>/dev/null || true)
    if [[ "$REMOTE_URL" =~ gitlab ]] || [ -n "${GITLAB_CI:-}" ]; then
        PROVIDER="gitlab"
    else
        PROVIDER="github"
    fi
else
    PROVIDER=$(echo "$PROVIDER" | tr '[:upper:]' '[:lower:]')
fi

normalize_spec_slug() {
    # Standardized slugification that preserves stop words
    # Usage: slug=$(normalize_spec_slug "us-29-fiber-cable-and-strand-inventory")
    local title="$1"
    if [ -z "$title" ]; then
        echo ""
        return
    fi
    # Strip quotes and leading/trailing whitespace
    title=$(echo "$title" | sed -e 's/^[[:space:]]*//' -e 's/[[:space:]]*$//' -e "s/^['\"]*//" -e "s/['\"]*$//")
    # Strip common prefixes
    local stripped
    stripped=$(echo "$title" | sed -E 's/^(epic|feature|feat|user[- ]story|use[- ]case|us|uc)[s]?([- ]*[0-9]+)?[[:space:]]*[:-]?[[:space:]]*//i')
    if [ -n "$(echo "$stripped" | sed -e 's/^[[:space:]]*//' -e 's/[[:space:]]*$//')" ]; then
        title="$stripped"
    fi
    # Normalize hyphens to spaces, strip punctuation, convert back to hyphens and lowercase
    title=$(echo "$title" | tr '-' ' ' | sed 's/[^a-zA-Z0-9 ]//g' | awk '{ $1=$1; print }' | tr ' ' '-' | tr '[:upper:]' '[:lower:]')
    echo "$title"
}


# Issue #330 -- fail closed. This block previously warned and carried on, so the gate
# vanished in exactly the circumstance where it most needed to hold: a partial or broken
# checkout with no checker present. A gate that yields when its checker is absent is not
# a gate, it is a comment.
if [ ! -f "$LINTER" ]; then
    echo "FATAL: Linter not found at $LINTER." >&2
    echo "       The specification gate cannot be satisfied, so no issue will be filed." >&2
    exit 1
fi

# Issue #331 -- the explicit --allow-missing-specs flag was removed from this invocation.
# It was a no-op: cli.py declares it with default=True, so passing it changed nothing,
# and its strict counterpart --no-allow-missing-specs is not used anywhere in the
# repository. It also does not gate the 100% *model coverage* invariant as #331 states;
# it gates whether an open tracker issue lacking a local spec file is fatal. Narrowing
# the gate's scope to the item being filed remains an open design question recorded on
# #331 and #321 -- two views of one scoping defect that must be resolved together.
# Issue #331 + #321 -- the gate is scoped to the item being filed. Whole-corpus
# invariants still run (uniqueness and cross-references cannot be checked per file),
# but only findings naming this specification are reported. That is what makes
# strictness affordable: the permissive --allow-missing-specs flag is gone, and an
# unrelated work-in-progress draft no longer blocks this filing.
echo "[GATE] Running linter: $LINTER --spec-only --only $(basename "$LOCAL_FILE") --provider $PROVIDER"
if ! python3 "$LINTER" --spec-only --only "$(basename "$LOCAL_FILE")" --provider "$PROVIDER"; then
    echo "FATAL: Linter failed. Fix all specification violations before filing issues." >&2
    exit 1
fi
echo "[GATE] Linter passed."

REPO_FLAG=""
if [ -n "$REPO" ]; then
    REPO_FLAG="--repo $REPO"
fi

# Duplicate search and idempotency
if [ "$PROVIDER" = "gitlab" ]; then
    EXISTING=$(glab issue list $REPO_FLAG --all --search "$TITLE" 2>/dev/null \
        | awk -F'\t' -v t="$TITLE" '$2 == t { print $1; exit }')
else
    # Issue #332 -- idempotency. Exact match on the title column, not a substring:
    # `gh issue list` emits TSV as number<TAB>title<TAB>labels<TAB>state.
    EXISTING=$(gh issue list --state all --search "in:title \"$TITLE\"" $REPO_FLAG 2>/dev/null \
        | awk -F'\t' -v t="$TITLE" '$2 == t { print $1; exit }')
fi

if [ -n "$EXISTING" ]; then
    EXISTING="${EXISTING#\#}"
    if [ -n "$REOPEN_ANALYSIS_FILE" ]; then
        echo "[REOPEN] Issue #$EXISTING already exists. Reopening with analysis from $REOPEN_ANALYSIS_FILE..."
        if [ "$PROVIDER" = "gitlab" ]; then
            glab issue reopen "$EXISTING" $REPO_FLAG
            glab issue note "$EXISTING" $REPO_FLAG --message "$(< "$REOPEN_ANALYSIS_FILE")"
        else
            gh issue reopen "$EXISTING" $REPO_FLAG
            gh issue comment "$EXISTING" $REPO_FLAG --body "$(< "$REOPEN_ANALYSIS_FILE")"
        fi
        exit 0
    else
        echo "[IDEMPOTENT] Issue #$EXISTING already carries the title '$TITLE'. Not filing a duplicate."
        exit 0
    fi
fi

# Label check and creation
if [ "$PROVIDER" = "gitlab" ]; then
    if ! glab label list $REPO_FLAG 2>/dev/null \
        | awk -v l="$LABEL" '($1 == l || $0 ~ ("^" l "([[:space:]]|$)")) { found = 1 } END { exit !found }'; then
        echo "[GATE] Label '$LABEL' not found. Creating..."
        glab label create "$LABEL" $REPO_FLAG --color "#0366d6" --description "${LABEL} specification"
    fi
else
    # Issue #332 -- the label precondition used `grep -Fq "$LABEL"`, a substring match, so an
    # existing `feature-request` satisfied the check for `feature` and the real label was
    # never created. Exact match on the name column instead.
    if ! gh label list $REPO_FLAG 2>/dev/null \
        | awk -F'\t' -v l="$LABEL" '$1 == l { found = 1 } END { exit !found }'; then
        echo "[GATE] Label '$LABEL' not found. Creating..."
        gh label create "$LABEL" $REPO_FLAG --color "0366d6" --description "${LABEL} specification"
    fi
fi

# Issue #244 -- expand relative markdown links to full blob URLs before creating the issue.
TMP_EXPANDED_BODY=$(mktemp /tmp/expanded_body_XXXXXX)
cleanup() {
    rm -f "$TMP_EXPANDED_BODY"
}
trap cleanup EXIT

python3 - "$LOCAL_FILE" "$TMP_EXPANDED_BODY" "$REPO" "$SCRIPT_DIR" "$PROVIDER" <<'PYEOF'
import sys, os

local_file = sys.argv[1]
tmp_out = sys.argv[2]
repo = sys.argv[3] if len(sys.argv) > 3 and sys.argv[3] else None
script_dir = sys.argv[4] if len(sys.argv) > 4 and sys.argv[4] else ""
provider = sys.argv[5] if len(sys.argv) > 5 and sys.argv[5] else ""

# Ensure scripts directories are on sys.path
for path in [
    script_dir,
    os.path.join(script_dir, "..", "..", "..", "scripts"),
    os.path.join(os.path.dirname(os.path.abspath(local_file)), "..", "..", "scripts"),
]:
    abs_p = os.path.abspath(path)
    if os.path.isdir(abs_p) and abs_p not in sys.path:
        sys.path.insert(0, abs_p)

try:
    from reconcile_backlog import expand_relative_links_for_tracker, load_codebase_rules, find_workspace_dir
    workspace_dir = find_workspace_dir(os.path.dirname(os.path.abspath(local_file))) or find_workspace_dir(os.getcwd())
    rules = load_codebase_rules(workspace_dir) if workspace_dir else {}
    if not rules:
        rules = {}
    if repo:
        rules.setdefault("meta", {})["upstream_repository"] = repo
        rules.setdefault("tracker_rules", {})["project_id"] = repo
        rules.setdefault("tracker_rules", {})["project_key"] = repo

    if provider:
        rules.setdefault("tracker_rules", {})["provider"] = provider
        os.environ["TRACKER_PROVIDER"] = provider

    with open(local_file, "r", encoding="utf-8") as f:
        content = f.read()

    expanded = expand_relative_links_for_tracker(content, filepath=local_file, rules=rules, workspace_dir=workspace_dir)
    with open(tmp_out, "w", encoding="utf-8") as f:
        f.write(expanded)
except Exception:
    import shutil
    shutil.copyfile(local_file, tmp_out)
PYEOF

if [ "$PROVIDER" = "gitlab" ]; then
    glab issue create $REPO_FLAG --title "$TITLE" --label "$LABEL" --description "$(< "$TMP_EXPANDED_BODY")"
else
    if [ -n "$REPO" ]; then
        gh issue create --repo "$REPO" --title "$TITLE" --label "$LABEL" --body-file "$TMP_EXPANDED_BODY"
    else
        gh issue create --title "$TITLE" --label "$LABEL" --body-file "$TMP_EXPANDED_BODY"
    fi
fi

