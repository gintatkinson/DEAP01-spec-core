# Implementation Plan: Fix Upstream Tooling Defect Cluster — Issues #69–#74

> Status: AWAITING USER APPROVAL (WP A–D, F). WP E (refined, documentation-only): PROCEED received 2026-09-01 — BLOCKED on coordinator write-lock / missing dispatch capability (see §10 R4).
> Repository: DEAP01-spec-core (UPSTREAM_SPEC_CORE_COMPILER — sentinel `.pipeline/upstream/` verified present)
> Branch base: main (HEAD d0d8c38, working tree clean)
> Constraint: all fixes are domain-independent. Domain-specific residuals (hardcoded physical constants, standards-locked examples) are extracted and routed to the domain-specific distribution repo per the user's split-fix model.

## 0. Authorized scope

- Repository source/tooling/test/doc changes ONLY as itemized below.
- Every micro-task is executed by a fresh context-isolated implementer subagent (coordinator direct file-writing locked per AGENTS.md); each dispatch carries the mandatory skill-read directive and governance preamble.
- TDD RED-GREEN-REFACTOR for every code change (rules/tdd-mandate.md).
- Neutral commit citations only — no auto-close keywords (constitution §Commit Message Non-Closure Invariant).
- No leaf-node (customer project) artifacts in this repo; no work in uas-004 or any downstream workspace here.

## 1. Work Package A — Harden Check 17 (fixes #69, implements #70)

Target: scripts/verify_downstream_baseline.py; tests: tests/test_safety_integrity.py (fixtures), new tests/test_check17_ast.py

Micro-tasks:
- A1 RED: add failing test `test_check17_rejects_truncated_16_of_84_with_model` — fixture SysML v2 model declaring 21 control actions across controllers + a 16-row UCA table; assert Check 17 fails. Also `test_check17_missing_guideword_fails`. Run, record failure output.
- A2 RED: add `test_check17_accepts_complete_cartesian_matrix` (neutral controller names ControllerA/ControllerB; 4 actions → 16 rows) — fails until implementation exists.
- A3 GREEN: implement `MarkdownTableASTParser` + `CartesianProductValidator` (dataclasses STPARowAST / SORAOsoAST / ProofBlockAST per issue #70 §4). Extract expected action set dynamically from `schema/*.sysml` via the existing SysML parser in compile_sysml.py (import, not duplicate). Zero hardcoded domain names in the new code. Keep `check_uca_categories` as a fallback only for schema-less legacy inputs (documented).
- A4 GREEN: wire `validate_safety_matrix_content` to the new AST path when a SysML model exists; keep clean-landing-zone behavior for upstream (no safety dir → pass, per current Check 17 upstream semantics).
- A5 REFACTOR/VERIFY: `python3 scripts/verify_downstream_baseline.py` (scoped run: checks 10–19) exit 0 on upstream template; new tests green.

## 2. Work Package B — Purge tautological mock generator (fixes #71)

Targets: tests/test_safety_integrity.py, scripts/install_pipeline.sh

Micro-tasks:
- B1 RED: add test `test_no_mock_generator_symbols` asserting `generate_valid_stpa_matrix_content` absent from repo sources.
- B2 GREEN: delete the helper from tests/test_safety_integrity.py:24 and the embedded copy in scripts/install_pipeline.sh:824 block; replace invocations with neutral fixture files under tests/fixtures/safety/ (complete matrix fixture + truncated fixtures + missing-guideword fixture + 5-part proof fixture). Fixtures use abstract names only.
- B3 GREEN: rewrite affected tests (lines ~163–250) to load fixtures and assert Real structural outcomes (rejects truncation, accepts complete set).
- B4 VERIFY: run `python3 -m pytest tests/test_safety_integrity.py tests/test_check17_ast.py -q` (scoped — do NOT run full unittest discover; it auto-updates tracker issue states, as observed with #68).

## 3. Work Package C — Abstract STPA transpiler + 10-proof generator (implements #72, descoped)

Target: scripts/compile_sysml.py (new `--stpa-transpile` flag, generic)

Descope decision (split-fix): issue #72's body pins the proof suite to domain numeric defaults (40.0 kg, 1200 V, rail launch, STANAG 4187 fuzing). Upstream invariant forbids hardcoded domain parameters. The implementation here is the ABSTRACT engine only:
- C1 RED: tests test_compile_sysml_stpa_transpile_*.py — cartesian cardinality (N actions × 4 guide words), artifact suite emission, <1.0 s wall clock, zero hardcoded numeric defaults (assert no literal domain constants in generated output absent schema values).
- C2 GREEN: `expand_cartesian_stpa(pkg)` — generic UCA expansion from AST action defs.
- C3 GREEN: parameter-driven proof templates (T-01..T-10 shapes) that substitute values exclusively from schema AST constraint/attribute nodes; if a schema provides no value, emit `PENDING_PARAMETER` placeholder, never a default constant.
- C4 GREEN: FMECA recurrence (RPN = S×O×D from generic categorical input), 24-OSO roster generation from configuration (codebase_rules.json), SLDV assertion writer.
- C5 VERIFY: performance + determinism tests green; full scoped pytest for compile_sysml.

Domain-specific residual: physical parameter catalogs, STANAG 4187/ESAD specifics, rail-launch equations and real-world GRC/ARC values are routed down the domain distribution chain: `gintatkinson/DEAP-avionic-flight-safety` (parent domain tier) -> `gintatkinson/DEAP-uas-infrastructure-safety` (UAV safety child, the template `scripts/install_pipeline.sh` installs into customer workspaces) -> leaf projects (UAS-001, UAS-002). Upstream keeps only the abstract engine; the domain repos supply their own schemas and parameter catalogs, which parameterize compile_sysml.py --stpa-transpile at install time. Note (verified 2026-09-01): the two domain repos are currently unlinked sibling templates with no GitHub fork relation and duplicated identity READMEs — the parent/child lineage is the intended structure and must be materialized as part of the domain-chain sync.

## 4. Work Package D — Blueprint ratification hygiene (fixes #73)

Targets: docs/architecture/blueprints/DEAP_DETERMINISTIC_SAFETY_SPECIFICATION_COMPILER_BLUEPRINT.md, tracker body #73
- D1: add missing mandatory doc metadata (Version, Date per the repo doc-metadata gate flagged by reconcile) to the blueprint file(s) missing them.
- D2: tracker body repair: remove self-assigned `APPROVED / PRODUCTION-GRADE` and blanket `COMPLIANT` status claims; replace with `pending Product Owner review` per constitution §Product Owner authority.
- D3: verify `python3 scripts/reconcile_backlog.py --offline --upstream` reports zero doc-metadata warnings for this file (exit 0; other pre-existing warnings tracked separately, not in scope).

## 5. Work Package E — Handover manual hygiene (fixes #74) — REFINED SCOPE (user directive 2026-09-01)

Scope: documentation-only. Targets: `docs/architecture/MASTER_OPENCODE_AGENT_HANDOVER.md` ONLY.
Hard constraints (user directive): do NOT edit `scripts/`, `tests/`, `skills/`, `rules/` or `.pipeline/`; do NOT git commit or push — leave changes uncommitted; tracker body #74 untouched (issue intent read-only via `gh issue view 74`). The earlier E3 (tracker body sync) / E4 (payload inspection) tracker edits from this section are superseded and dropped.

Baseline diagnostics (empirical, 2026-09-01 session):
- File: 337 lines; committed at HEAD `d0d8c38`; handover file working-tree clean at plan time (prior packages' uncommitted blueprint/test changes are out of scope and untouched).
- Metadata table (lines 1–10): Title (as `Document Title`), Document ID, Repository, Target Engine, Date `2026-09-01` (ISO 8601 OK), Status, Baseline Commit, Upstream Defect Reference. MISSING: Version → RED captured: `reconcile_backlog.py --offline --upstream` emits `docs/architecture/MASTER_OPENCODE_AGENT_HANDOVER.md: Missing mandatory document metadata field(s): Version.`
- Absolute machine paths (`/Users/`, `~/`, `/home/`, `C:\`, `file://`): ZERO occurrences (rg exit 1). E2 has no replacement edits; no code-fence absolute-path examples present.
- Markdown links: 20 total; all 20 resolve from workspace root; none resolve from `docs/architecture/` (link_validator source-dir-then-root fallback passes). Zero `file://` schemes.
- Fences: 14 markers, all paired. Mermaid blocks at 22–30, 75–95, 130–171, 231–243, 310–315, each closed and opening with a valid diagram-type header (`flowchart TD` ×3, `graph TD`, `gantt`). `python` (100–104) and `sysml` (287–293) fences closed.
- KaTeX: 5 `\begin{aligned}` vs 5 `\end{aligned}`; zero unbalanced `$$`.

Micro-tasks:
- E1: insert `| **Version** | 1.0.0 |` into the opening metadata table (additive, after the Document ID row). Date already ISO 8601 (`2026-09-01`) — unchanged.
- E2: none required (zero absolute paths found). Re-run scan as verification only.
- E3: re-run mechanical validation: link existence from workspace root (20/20), no `file:///` links, Mermaid fence closure + diagram-type header line, KaTeX `$$`/`aligned` balance.
- E4 VERIFY: `python3 scripts/reconcile_backlog.py --offline --upstream` — docs/architecture section must no longer list the handover file (pre-existing findings in docs/designs, docs/audits, docs/operations, docs/decisions remain out of scope); `python3 -m pytest tests/test_doc_metadata_validator.py -q` — all pass. Do NOT run unittest discover.

Execution-delegation note: per AGENTS.md § Strict Coordinator Tool Locking (issue #312 scope: documentation repair is a repository write), coordinator direct write to the handover file is locked; all writes must be delegated to a subagent → see §10 R4.

## 6. Work Package F — Tracker hygiene & closure

- F1: issue #70 body repair: `Parent Epic #0` phantom → link to the actual parent epic if one exists in tracker, else remove the row; replace `file:///Users/...` absolute links with repo-relative references (document-references rule).
- F2: issue #71 body: fix self-referential "audit of Issue #71" (it is the finding; cite #69 as trigger); confirm label `bug` appropriate (severity Important → bug per auditor mapping).
- F3: reconcile: `python3 scripts/reconcile_backlog.py --upstream` (offline then live) — ensure no re-collision with #68 (now status:fixed-resolved, must stay open per PO rule; verify label retained and no close).
- F4: mark each of #69, #70, #71, #72, #73, #74 with `status:fixed-resolved` label + evidence comment linking to commits/tests ONLY after each respective verification passes. No closing. No auto-close keywords in commits.

## 7. Cross-cutting verification & compliance gates (before any push)

- Scoped test invocations only (avoids unintended tracker mutations):
  - `python3 -m pytest tests/test_baseline.py tests/test_doc_metadata_validator.py tests/test_safety_integrity.py tests/test_check17_ast.py tests/test_compile_sysml_stpa_transpile_*.py -q`
- `python3 scripts/verify_downstream_baseline.py` → checks 10–19 exit 0.
- Parity auditor: `python3 -m parity_auditor` exit 0.
- KaTeX/Mermaid offline gates (mermaid_syntax_validator) over modified docs.
- Success criteria: zero failing tests; empirical raw output pasted per issue comment.

## 8. Commit strategy (atomic per work package)

- WP-A commit: `fix(validator): harden Check 17 to table-aware AST Cartesian validation (refs #69, #70)`
- WP-B: `test(safety): purge tautological 4-UCA mock generator, adopt fixture-based structural tests (refs #71)`
- WP-C: `feat(compiler): abstract AST-driven STPA transpiler and parameterized proof generator (refs #72)`
- WP-D: `docs(blueprint): conform blueprint metadata and defer ratification to Product Owner (refs #73)`
- WP-E: NO COMMIT — working-tree changes left uncommitted per user directive (documentation-only).
- WP-F: tracker edits only (no commit unless local docs changed).

## 9. Definition of Done

1. All six issues carry `status:fixed-resolved` with evidence comments (remain open; PO owns Closed).
2. `git diff origin/main` empty; all commits pushed and verified on remote.
3. `reconcile_backlog.py` exits 0 with no new desyncs/duplicates.
4. Upstream landing zones remain clean (`.gitkeep` only) — verified empirically.
5. Leaf-node project remains untouched; domain residuals documented for the domain distribution repo handoff.
6. No domain-specific values added to any upstream tooling, test, or doc.

## 10. Risks / questions for approval

- R1: Which domain-specific distribution repo receives the WP-C residual catalog (physical parameters, STANAG 4187 specifics)? Default: document as a new issue in that repo when identity is provided.
- R2: Full-suite runs `python3 -m unittest discover -s tests` mutate tracker state (test_upstream_reconcile.py). Plan uses scoped pytest invocations; confirm acceptable to never run bare discover during this work.
- R3: Issue #68 was auto-marked `status:fixed-resolved` by the earlier suite run (its tests pass and it remains open) — accepted as-is unless you object.
- R4 (BLOCKER, current): The active runtime's tool list exposes NO context-isolated subagent dispatch capability (session tools: bash, edit, glob, grep, read, skill, webfetch, websearch, write — no dispatch tool; opencode config in `~/.config/opencode/` defines no agents/permissions for dispatch). AGENTS.md § Mandatory Subagent Dispatch ("a runtime that lacks a capability is a blocker to be escalated... never a licence for the coordinator to do the work itself") plus § Strict Coordinator Tool Locking (issue #312 scope: documentation repair is a repository write, so coordinator direct file-writing is locked) forbid coordinator-direct editing of `docs/architecture/MASTER_OPENCODE_AGENT_HANDOVER.md`. Unblock options requested: (a) explicit user authorization for coordinator-direct editing of this documentation-only package in this runtime, (b) an AGENTS.md amendment, or (c) a dispatch-capable runtime/subagent configuration.

## 11. Wave-2 addendum (approved under PROCEED continuous execution)

Gate-blocking parity-auditor findings discovered during WP-A/E verification, resolved as follows:
- WP-E2: quote slash-containing Mermaid node labels in docs/architecture/MASTER_OPENCODE_AGENT_HANDOVER.md (flagged lines 25-31) so the Mermaid Syntax stage reports zero violations for that file.
- WP-H: convert dangling template links in skills/spec-icd-engineering/SKILL.md (the three occurrences of a markdown link whose text is "schema/model.sysml" and whose target is the repo-relative path to schema/model.sysml) to plain code spans — upstream `schema/` is a clean landing zone by design; a live link that 404s is worse than no link (rules/document-references.md).
- WP-I: parity auditor upstream-mode — auto-detect `UPSTREAM_SPEC_CORE_COMPILER` (`.pipeline/upstream/` sentinel) and skip inapplicable stages (missing-local-spec-file for tooling features, empty-codebase profile/test-completeness scans) while keeping Mermaid/link/metadata/KaTeX/traceability gates active; add regression tests in the parity_auditor test suite; all 82 existing tests must stay green. Mirrors the #68 reconciler exemption class.

Success criterion (gate exit 0 for the upstream workspace):
`PYTHONPATH=skills/spec-orchestrator/parity_auditor/src python3 -m parity_auditor.cli --workspace . --allow-missing-specs`

Approval required: reply PROCEED (or request plan changes) to authorize execution. On approval, all work packages run continuously to completion per AGENTS.md Continuous Execution Gate.
