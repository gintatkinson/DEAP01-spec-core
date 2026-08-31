---
title: "Defect Inventory — full-tree sweep"
date: "2026-08-05"
scope: "DEAP-spec-core @ 4a4e2b5"
method: "read-only sweep; no fixes applied in this pass"
---

# Defect Inventory — 2026-08-05

Full-tree sweep, commissioned after an earlier count of 31 was judged too low. It was.

## Method

- Both Python suites executed, exit codes captured **without pipes**.
- Flutter suite and `flutter analyze` executed (not run at all earlier in the session).
- `verify_model_coverage.py` executed and its **real** exit code captured.
- AST sweep of all test functions for tautological assertions, empty bodies, absent
  assertions, unconditional skips.
- `py_compile` across every pipeline script.
- Directory census of the backlog corpus and schema inputs.

## Summary

| | Count |
|---|---|
| Previously reported | 31 |
| **New in this sweep** | **19** |
| **Total identified** | **50** |
| Fixed to date | 18 |
| **Outstanding** | **32** |
| Filed as issues | **0** |

## NEW — Critical

**N1. The pipeline's own coverage linter fails.**
`./skills/spec-orchestrator/scripts/verify_model_coverage.py --spec-only` exits **1**
with 16 findings. This is the gate every worker skill calls a mandatory pre-commit check
("fix all reported errors until the linter passes with exit code 0"). The repository that
ships it does not pass it.

**N2. The backlog corpus is empty.**

| Directory | Files |
|---|---|
| `docs/epics/` | 0 |
| `docs/user-stories/` | 0 |
| `docs/use-cases/` | 0 |
| `docs/features/` | 4 |

A specification pipeline ships with no Epics, no User Stories and no Use Cases. Every
traceability rule in `.pipeline/constitution.md` — Epic→Feature, Story→Feature,
UseCase→Story — is vacuously satisfied because there is nothing to trace.

**N3. There is no input schema.** `schema/` contains only `.gitkeep`. Phase 0 and Phase 1
of the orchestrator take a structural schema as their input. There is none.

**N4. Broken links in shipped Features point at a non-existent branch and non-existent
files.** `feat-28` and `feat-13` link to `.../blob/master/docs/epics/epic-28-...` and
`epic-13-...`. The branch is `main`, not `master`, and `docs/epics/` is empty. These are
published to the tracker where they render as live links.

## NEW — Major

**N5. `README.md` violates the standard-agnostic rule it enforces.** Line 267 names
`IETF / 3GPP`. `verify_model_coverage.py` flags it: *"Documentation file 'README.md'
contains hardcoded reference to '3GPP'. Target profiles and READMEs must remain strictly
standard-agnostic."* The rule is in `rules/platform-independence.md`
§ *Hardcoded Standard Reference Rules*.

**N6-N17. Twelve Mermaid syntax violations in shipped documentation**, all flagged by the
validator this repository authors:

| File | Count | Kind |
|---|---|---|
| `docs/feat-hardware-decoupled-persistence-design.md` | 6 | unquoted `<`, unquoted node labels |
| `docs/feat-firestore-persistence-adapter-design.md` | 3 | unquoted node labels |
| `docs/decisions/audits/pipeline_integration_critique.md` | 2 | unquoted node labels |
| `docs/designs/feat-epic-template-mandate-plan.md` | 1 | unquoted `>` |

**N18. The React suite is never run by any documented command.** `web_react/package.json`
defines `test` (`vitest run`) and `build`. Neither `install-guide.md` § 3 nor any CI step
invokes them. One test file exists for the whole application.

**N19. My own exit-code masking.** During this sweep I ran the coverage linter piped to
`tail`, reported `EXIT=0`, and had to re-run it to discover it exits 1. This is precisely
the defect `tests/test_process_discipline_gates.py` § 3 exists to prevent, which I had
cited earlier in the same session.

## Verified clean

Not everything is broken; these were checked and pass.

- Flutter suite: **350 passed, 1 skipped**.
- `flutter analyze`: **0 issues**.
- Python suites: **569 passed / 306 passed**, 0 failed.
- No tautological assertions, empty test bodies or unconditional skips remain.
- Every pipeline script compiles.

## Carried forward — the 13 previously reported and still unfixed

`tests/` absent from the installer copy list; `install-guide.md:73` prescribes a command
that cannot work; the template route ships `.pipeline/upstream/`; no update procedure that
preserves customer code; `DEAP_HANDOFF.md` amends the constitution via `git add -A`, uses
the prohibited `.agents/skills/` prefix and records a stale HEAD;
`verify_downstream_baseline.py` hard-codes a macOS build and writes an 83 MB zip to the
repository root; 126 tests assert prose only; no end-to-end test exists;
`test_skill_path_references.py` validates only `skills/` paths; no harness has a
permission floor.

## Assessment

The engineering that exists is sound: three test suites pass, static analysis is clean,
the validators work. What is absent is the subject. The pipeline has no schema, no Epics,
no Stories, no Use Cases, four Features, and its own mandatory gate fails on the
documentation it ships.

The gap between "the linter passes" and "the system is verified" is the whole of N1-N4.

## Not filed

Zero of these 50 are filed as tracker issues, in breach of `.agents/AGENTS.md`
§ *Mandatory Upstream Tooling Bug Reporting*.
