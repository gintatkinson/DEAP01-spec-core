# BRIEFING — 2026-09-21T11:28:00Z

## Mission
Investigate Work Package 3 (R3: Robust Model & Schema Copying) and Baseline Verification (`scripts/verify_downstream_baseline.py --no-domain`) for `scripts/install_pipeline.sh` and downstream installation.

## 🔒 My Identity
- Archetype: explorer
- Roles: [explorer, synthesis]
- Working directory: /Users/perkunas/jail/DEAP01-spec-core/.agents/explorer_r1_3
- Original parent: f3cf7ab0-b3f5-46dd-b0c6-a615bbe571ed
- Milestone: Investigation & Analysis Complete

## 🔒 Key Constraints
- Read-only investigation — do NOT implement
- Strictly read-only on codebase and specification files
- Only write metadata in `/Users/perkunas/jail/DEAP01-spec-core/.agents/explorer_r1_3/`
- Zero hardcoded domain concepts

## Current Parent
- Conversation ID: f3cf7ab0-b3f5-46dd-b0c6-a615bbe571ed
- Updated: 2026-09-21T11:28:00Z

## Investigation State
- **Explored paths**: `scripts/install_pipeline.sh` (lines 283-289, 475-645), `scripts/verify_downstream_baseline.py`, `README.md`, `.agents/ORIGINAL_REQUEST.md`, `implementation_plan.md`
- **Key findings**:
  1. Lines 283-289 in `scripts/install_pipeline.sh` skip schema copying entirely when `$TARGET_DIR/schema` already exists due to `[ ! -e "$TARGET_DIR/schema" ]`.
  2. In 4 out of 5 standard edge case scenarios, models are lost.
  3. Direct fix using POSIX standard `cp -RP "$INSTALLER_ROOT/schema/." "$TARGET_DIR/schema/"` preceded by `mkdir -p "$TARGET_DIR/schema"` passes 100% of scenarios without directory nesting.
  4. `scripts/verify_downstream_baseline.py --no-domain` runs Checks 10 through 30; Check 14 requires `"Operator Prompt Catalog"` to remain in `README.md`, Check 16 requires clean upstream landing zones, and Check 13 enforces KaTeX/markdown syntax without unquoted angle brackets or unescaped parentheses.
- **Unexplored areas**: None within the scope of WP3 and Baseline Verification.

## Key Decisions Made
- Formulated exact drop-in replacement for lines 283-289 of `scripts/install_pipeline.sh`.
- Validated behavior across BSD (macOS) and POSIX cp standards in external sandbox.
- Documented failure modes and constraints for downstream baseline verification.

## Artifact Index
- `DISPATCH.md` — incoming dispatch records
- `progress.md` — liveness heartbeat and step tracking
- `handoff.md` — comprehensive 5-component handoff report
