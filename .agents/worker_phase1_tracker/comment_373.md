### Verification Evidence — Resolved in 90fe8fa

The duplicate issue detection defect reported in #373 has been fully remediated and verified:

1. **Title Column Indexing Parity**:
   - In `skills/spec-orchestrator/scripts/create_issue.sh` (line 195), duplicate issue detection now parses column 3 of GitHub CLI TSV output:
     `EXISTING=$(printf '%s\n' "$GH_LIST_OUT" | awk -F'\t' '$3 == ENVIRON["TITLE"] { print $1; exit }')`
   - In GitLab mode (line 180), column 2 continues to match `glab issue list` TSV (`number<TAB>title<TAB>labels...`):
     `EXISTING=$(printf '%s\n' "$GLAB_LIST_OUT" | awk -F'\t' '$2 == ENVIRON["TITLE"] { print $1; exit }')`
2. **Environment Variable Export & Quoting**:
   - `export TITLE` is declared at line 64.
   - Title quotes are escaped at line 184 (`${TITLE//\"/\\\"}`).
3. **Empirical Test Verification**:
   - `tests/test_create_issue_dual_provider.py` contains dedicated tests `test_github_idempotency_avoids_duplicate` and `test_create_issue_script_contract_assertions` asserting exact column 3 matching.
   - `python3 -m pytest tests/test_create_issue_dual_provider.py` passed 31/31 in 20.44s (exit code 0).

Status: `Fixed / Resolved` (marking with `status:fixed-resolved` label; left open for Product Owner review per .pipeline/constitution.md:161).
