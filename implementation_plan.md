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
- R4 (RESOLVED): the coordinator session exposes the context-isolated dispatch capability (task tool, subagent types general/explore) and every write in this plan was executed through fresh dispatches; the earlier blocker report was a runtime misreading and was overcome by re-dispatch clarification.

## 11. Wave-2 addendum (approved under PROCEED continuous execution)

Gate-blocking parity-auditor findings discovered during WP-A/E verification, resolved as follows:
- WP-E2: quote slash-containing Mermaid node labels in docs/architecture/MASTER_OPENCODE_AGENT_HANDOVER.md (flagged lines 25-31) so the Mermaid Syntax stage reports zero violations for that file.
- WP-H: convert dangling template links in skills/spec-icd-engineering/SKILL.md (the three occurrences of a markdown link whose text is "schema/model.sysml" and whose target is the repo-relative path to schema/model.sysml) to plain code spans — upstream `schema/` is a clean landing zone by design; a live link that 404s is worse than no link (rules/document-references.md).
- WP-I: parity auditor upstream-mode — auto-detect `UPSTREAM_SPEC_CORE_COMPILER` (`.pipeline/upstream/` sentinel) and skip inapplicable stages (missing-local-spec-file for tooling features, empty-codebase profile/test-completeness scans) while keeping Mermaid/link/metadata/KaTeX/traceability gates active; add regression tests in the parity_auditor test suite; all 82 existing tests must stay green. Mirrors the #68 reconciler exemption class.

Success criterion (gate exit 0 for the upstream workspace):
`PYTHONPATH=skills/spec-orchestrator/parity_auditor/src python3 -m parity_auditor.cli --workspace . --allow-missing-specs`

## Phase 2 — Domain Chain Materialization & Downstream Tooling Sync

> Status: APPROVED (PROCEED, 2026-09-01; A3 CI drift-check included)

### 2.0 Verified facts (live GitHub API, 2026-09-01)

| Repo | Role | Created | Fork? | Tooling state |
| --- | --- | --- | --- | --- |
| gintatkinson/DEAP01-spec-core | upstream core (this repo) | — | — | fixed: main b5bc0b6..2716a7a |
| gintatkinson/DEAP-avionic-flight-safety | intended domain parent | 2026-08-06 18:31:52Z | no | pre-fix (no MarkdownTableASTParser); README wrongly declares identifier DEAP-uas-infrastructure-safety |
| gintatkinson/DEAP-uas-infrastructure-safety | intended domain child | 2026-08-06 18:32:14Z | no | pre-fix; README self-consistent |
| UAS-001 / UAS-002 | leaf customer workspaces | — | private | — |

GitHub has no API to link existing repos as forks: lineage is materialized via managed metadata + a derived-by-bootstrap protocol (not destructive re-forking).

### 2.1 Target architecture (user-confirmed domain split)

- **Tier 0**: DEAP01-spec-core — abstract MBSE compiler core, zero domain content (hard invariant).
- **Tier 1**: DEAP-avionic-flight-safety — COMMON AVIATION SAFETY domain distribution template. Home of general aviation standards inventory and shared domain artifacts (DO-178C/DO-254, ARP4754A/ARP4761, generic MIL-STD-882E hazard practice, common flight-safety control patterns, generic FMECA/CTMC proof taxonomies). Parent of the UAS chain and of any future aviation-domain children.
- **Tier 2**: DEAP-uas-infrastructure-safety — UAS-specific distribution template, derived from the parent. Home of UAS-specific standards (JARUS SORA v2.5, ASTM F3269-17 RTA, ASTM F3411-22a Remote ID, RTCA DO-365B DAA), UAS schemas, and parameter catalog frameworks. This is the template `install_pipeline.sh` installs into customer workspaces (UAS-001, UAS-002, and the future leaf project).
- **Propagation rule**: domain-independent tooling flows core → parent → child → leaf installs. Common-aviation content lives in the parent, UAS-only content in the child. Neither domain content nor customer data ever flows back into the core.

### 2.2 Work packages (strictly serial: L0 → L1 → L2 → S1 → S2 → C → T → V)

All work happens in fresh clones under the system scratch path (`/var/folders/.../opencode/domain-sync/`), never inside this workspace (forbidden test-workspace rule). Every work package is executed by a fresh context-isolated implementer subagent with the mandatory skill-read directive, governance preamble, and PROCEED token; coordinator performs verification, not implementation.

- **WP-L0 — Recon & sync manifest**: in both domain-repo clones, read AGENTS.md/README.md/.pipeline layout, diff key tooling files vs upstream (sha/size + symbol probes: MarkdownTableASTParser, generate_valid_stpa_matrix_content, is_upstream_compiler_repo), enumerate every divergent file. Output: `sync-manifest.md` (scratch only). No repo writes.
- **WP-L1 — Parent identity & lineage (avionic-flight-safety)**: replace README identifier/role block: identifier DEAP-avionic-flight-safety, role PARENT_DOMAIN_DISTRIBUTION_TEMPLATE, domain = Common Aviation Safety Standards (list the general frameworks above), remove the UAS-repo identifier copy; add `.pipeline/lineage.json` = {self, tier: 1, role: domain_parent, upstream: DEAP01-spec-core, upstream_sync_ref: <sha at sync>, children: [DEAP-uas-infrastructure-safety]}; add `docs/domain/DOMAIN_SCOPE.md` (standards inventory, zero customer content). Verify repo's own pre-sync gates still pass (no regression from doc-only change). Neutral commit, push.
- **WP-L2 — Child identity & lineage (uas-infrastructure-safety)**: keep UAS README identity; add "Parent domain: DEAP-avionic-flight-safety (common aviation standards)" derived-from note; add `.pipeline/lineage.json` = {self, tier: 2, role: domain_child, parent: DEAP-avionic-flight-safety, upstream: DEAP01-spec-core (via parent)}. Gate check, neutral commit, push.
- **WP-S1 — Core→parent tooling sync**: propagate ONLY tooling + shared governance files (not upstream-specific docs): scripts/verify_downstream_baseline.py, scripts/compile_sysml.py, scripts/install_pipeline.sh, tests/test_safety_integrity.py, tests/test_check17_ast.py, tests/test_mock_generator_purge.py, tests/test_compile_sysml_stpa_transpiler.py, tests/fixtures/, skills/spec-orchestrator/parity_auditor/** (incl. __main__.py + upstream-mode + tests), skills/spec-icd-engineering/SKILL.md. Method: cherry-pick upstream commits 5027eb5, a0b8f7e, 3c8deff, f7e58b9, and the relevant doc portions where those files exist identically pre-fix. Conflicts must be resolved surgically; no weakening of gates. Then apply the wave-4 style additive metadata sweep to THAT repo's flagged docs. Verification in the parent clone: scoped pytest green, verify_downstream_baseline.py checks 10–19 exit 0, parity audit (upstream mode now shipping) exit 0, reconcile offline clean. Neutral commit, push.
- **WP-S2 — Parent→child tooling sync**: identical fixed set applied to uas-infrastructure-safety (source = parent repo post-WP-S1), same verification battery, neutral commit, push.
- **WP-C — Domain residual catalog split** (issue #72 residuals):
  - Common-aviation items (generic standards mappings, ARP4754A/4761 process artifacts, generic FMECA/CTMC and 6-DOF proof taxonomy) → documented as abstract catalogs in DEAP-avionic-flight-safety `docs/domain/`.
  - UAS-specific items (SORA GRC/ARC + 24-OSO mapping frameworks, F3269-17 RTA/Simplex patterns, F3411 Remote ID, DO-365B DAA, STANAG 4187 safety-reference notes) → DEAP-uas-infrastructure-safety `docs/domain/`.
  - Concrete vehicle numeric catalogs (masses, voltages, envelopes) → NEVER in any template repo; they arrive with each customer's schema at install time.
- **WP-T — Tracker operations**: open issues in avionic-flight-safety ("lineage materialization", "tooling sync from core <sha-range>", "README identity correction") and in uas-infrastructure-safety (lineage + sync + parent linkage), each with evidence summary; no issue is closed; labels per repo conventions; duplicate check before filing.
- **WP-V — Cross-repo Definition of Done verification** (coordinator): batteries run in all three repos: upstream remains green; parent and child each show scoped pytest green, baseline exit 0, parity audit exit 0, `git diff origin/main` empty, lineage.json present, README identifiers correct, landing zones clean. Final walkthrough report.

### 2.3 Guardrails

- Neutral commit citations in every repo; no auto-close keywords; no issue closures (PO authority).
- Serial execution enforced (user-authorization-lock / serial-execution); no parallel implementers touching the same repo checkout.
- Item-level subagent isolation; retry protocol (two consecutive failures → escalate).
- Scoped pytest only (never bare `unittest discover` in repos with tracker-touching tests).

## Phase 3 — `.agents/skills` Symlink Remediation (approved-under-PROCEED, 2026-09-01)

> Status: APPROVED via user directive (full step list + PROCEED, 2026-09-01). Scope: `.agents/skills/` tree conversion + one upstream tooling issue. `skills/` content untouched. No unittest discover.

Defect: `.agents/skills/` is a stale divergent copy of `skills/` (older SKILL.md variants; outdated `parity_auditor` hardcodes `gintatkinson/uas-003` at `.agents/skills/spec-orchestrator/parity_auditor/src/parity_auditor/cli.py`). `rules/document-references.md` documents `.agents/skills` as a tracked symlink (git mode 120000 → `../skills`).

Work packages:
- P3-1 RED evidence: `diff -rq skills .agents/skills | head -20`; `rg -n 'uas-003' .agents/skills`.
- P3-2 Convert to symlink: `git rm -r --cached .agents/skills`; `rm -rf .agents/skills`; `ln -s ../skills .agents/skills`; `git add .agents/skills`. Verify `git ls-files -s` mode 120000, `ls -la .agents/`, SKILL.md line-count equality, zero `uas-003` hits.
- P3-3 Gates: `python3 -m pytest tests/test_skill_path_references.py -q`; `python3 scripts/verify_downstream_baseline.py`; `PYTHONPATH=skills/spec-orchestrator/parity_auditor/src python3 -m parity_auditor.cli --workspace . --allow-missing-specs`. Fallback if a test pins non-symlink mode: revert to byte-identical rsync copy and commit that instead.
- P3-4 Report-only issue for the latent `cardinality_validator` SysML parser fallback binding `None` (not fixed here).
- P3-5 Commit + push: neutral citations, verify `git diff origin/main` empty.

### 2.4 Approval questions

- A1 (default: yes): lineage = metadata + derived-by-bootstrap protocol + periodic sync, without destructive re-forking.
- A2 (default: yes): propagate the full upstream tooling set to BOTH domain repos (parent first).
- A3 (open): add a GitHub Actions drift-check workflow in the domain repos comparing pinned tooling shas against core (recommended — prevents the 10:51Z-vs-14:10Z staleness we found from recurring). In scope or deferred?
- A4 (default: as §2.2 WP-C): confirm the residual split table (common aviation → parent; UAS-specific → child; vehicle numerics → customer schema only).

Approval required: reply PROCEED (or request plan changes) to authorize execution. On approval, all work packages run continuously to completion per AGENTS.md Continuous Execution Gate.
