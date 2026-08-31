#!/bin/bash
# Gated issue creation with mandatory linter pass precondition.
# Usage: ./create_issue.sh <body-file> <label> <title> <repo>
set -euo pipefail

LOCAL_FILE="$1"
LABEL="$2"
TITLE="$3"
REPO="${4:-}"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
LINTER="$SCRIPT_DIR/verify_model_coverage.py"

if [ ! -f "$LOCAL_FILE" ]; then
    echo "FATAL: Body file not found: $LOCAL_FILE" >&2
    exit 1
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


# Issue #330 — fail closed. This block previously warned and carried on, so the gate
# vanished in exactly the circumstance where it most needed to hold: a partial or broken
# checkout with no checker present. A gate that yields when its checker is absent is not
# a gate, it is a comment.
if [ ! -f "$LINTER" ]; then
    echo "FATAL: Linter not found at $LINTER." >&2
    echo "       The specification gate cannot be satisfied, so no issue will be filed." >&2
    exit 1
fi

# Issue #331 — the explicit --allow-missing-specs flag was removed from this invocation.
# It was a no-op: cli.py declares it with default=True, so passing it changed nothing,
# and its strict counterpart --no-allow-missing-specs is not used anywhere in the
# repository. It also does not gate the 100% *model coverage* invariant as #331 states;
# it gates whether an open tracker issue lacking a local spec file is fatal. Narrowing
# the gate's scope to the item being filed remains an open design question recorded on
# #331 and #321 — two views of one scoping defect that must be resolved together.
# Issue #331 + #321 — the gate is scoped to the item being filed. Whole-corpus
# invariants still run (uniqueness and cross-references cannot be checked per file),
# but only findings naming this specification are reported. That is what makes
# strictness affordable: the permissive --allow-missing-specs flag is gone, and an
# unrelated work-in-progress draft no longer blocks this filing.
echo "[GATE] Running linter: $LINTER --spec-only --only $(basename "$LOCAL_FILE")"
if ! python3 "$LINTER" --spec-only --only "$(basename "$LOCAL_FILE")"; then
    echo "FATAL: Linter failed. Fix all specification violations before filing issues." >&2
    exit 1
fi
echo "[GATE] Linter passed."

REPO_FLAG=""
if [ -n "$REPO" ]; then
    REPO_FLAG="--repo $REPO"
fi

# Issue #332 — idempotency. Re-running filed a second issue with the same title. Since
# #314/#316 the reconciler resolves specs by canonical issue_id, but a duplicate still
# pollutes the tracker and re-triggers downstream automation, so the cheapest place to
# stop it is before creation. Exact match on the title column, not a substring:
# `gh issue list` emits TSV as number<TAB>title<TAB>labels<TAB>state.
EXISTING=$(gh issue list --state all --search "in:title \"$TITLE\"" $REPO_FLAG 2>/dev/null \
    | awk -F'\t' -v t="$TITLE" '$2 == t { print $1; exit }')
if [ -n "$EXISTING" ]; then
    echo "[IDEMPOTENT] Issue #$EXISTING already carries the title '$TITLE'. Not filing a duplicate."
    exit 0
fi

# Issue #332 — the label precondition used `grep -Fq "$LABEL"`, a substring match, so an
# existing `feature-request` satisfied the check for `feature` and the real label was
# never created. Exact match on the name column instead.
if ! gh label list $REPO_FLAG 2>/dev/null \
    | awk -F'\t' -v l="$LABEL" '$1 == l { found = 1 } END { exit !found }'; then
    echo "[GATE] Label '$LABEL' not found. Creating..."
    gh label create "$LABEL" $REPO_FLAG --color "0366d6" --description "${LABEL} specification"
fi

if [ -n "$REPO" ]; then
    gh issue create --repo "$REPO" --title "$TITLE" --label "$LABEL" --body-file "$LOCAL_FILE"
else
    gh issue create --title "$TITLE" --label "$LABEL" --body-file "$LOCAL_FILE"
fi
