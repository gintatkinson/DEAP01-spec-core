### Verification Evidence — Resolved in 90fe8fa

The ARG_MAX argument list buffer overflow defect reported in #374 has been fully remediated and verified:

1. **File Payload Passing via `--description-file`**:
   - In `skills/spec-orchestrator/scripts/create_issue.sh` (line 305), `glab issue create` now receives `--description-file "$TMP_EXPANDED_BODY"` instead of expanding large specification markdown bodies inline into the command line argument array (`--description "$(...)"`).
   - GitHub mode (line 307) identically passes `--body-file "$TMP_EXPANDED_BODY"`.
2. **Protection Against E2BIG / ARG_MAX Overflows**:
   - Monolithic specification documents (such as large ConOps matrices or interface dictionaries) are streamed directly from disk via CLI file options, completely eliminating `E2BIG (Argument list too long)` shell execution failures.
3. **Empirical Test Verification**:
   - `tests/test_create_issue_dual_provider.py:775` asserts `--description-file "$TMP_EXPANDED_BODY"` contract compliance.
   - `python3 -m pytest tests/test_create_issue_dual_provider.py` passed 31/31 in 20.44s (exit code 0).

Status: `Fixed / Resolved` (marking with `status:fixed-resolved` label; left open for Product Owner review per .pipeline/constitution.md:161).
