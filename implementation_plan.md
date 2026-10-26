# Implementation Plan: Harden Agent Atomic Planning & Downstream Binary-Only Distribution Locks

## 1. Objective (Single Atomic Work Package)
Eliminate two systemic agent governance failures across `AGENTS.md`, `.pipeline/constitution.md`, `rules/user-authorization-lock.md`, and `.pipeline/ACTIVE_RULES_BUNDLE.md`:
1. **Stop Plan Snowballing & Commingling (`Plan Continuity` vs. `Atomic Work Execution`)**:
   - Clarify that `implementation_plan.md` MUST contain **at most 1 atomic work package** at any time (never a multi-wave backlog dump or appended unrelated tasks).
   - Clarify that `Prohibition of Unapproved Plan Abandonment & Mandatory Plan Continuity` applies strictly **within the currently active in-flight atomic task** (forbidding mid-task abandonment on test/linter friction), and explicitly **forbids** appending or prepending a new user task onto a previous `implementation_plan.md`.
2. **Enforce Downstream Binary-Only Distribution Lock (Zero Compiler Source Leakage & Zero Post-Gate Rebuilds)**:
   - Strictly forbid agents from proposing, planning (`implementation_plan.md`), or executing any copy of `crates/*`, `Cargo.toml`, `Cargo.lock`, `.pipeline/upstream/*`, or `docs/architecture/blueprints/*` into any downstream repository (`DOWNSTREAM_CUSTOMER_PROJECT`), and forbid rebuilding `target/release/*` during downstream distribution.
   - Restrict downstream distribution exclusively to pre-built `target/release/{compile-sysml,verify-baseline,ingest-sysml,reconcile-backlog,assemble-conops}` binary images plus `AGENTS.md`, `.agents/AGENTS.md`, `.pipeline/constitution.md`, `.pipeline/ACTIVE_RULES_BUNDLE.md`, `rules/*.md`, and `skills/**`.

## 2. Target Files
- `AGENTS.md` (and symlink `.agents/AGENTS.md`)
- `.pipeline/constitution.md`
- `rules/user-authorization-lock.md`
- `.pipeline/ACTIVE_RULES_BUNDLE.md`

## 3. Verification Gates
1. `./target/release/verify-baseline . --no-domain` exits `0`.
2. Commit and push to `origin/main`, verifying `git diff origin/main` is empty.
