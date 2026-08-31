---
title: "Solution — Upstream Repository Repair and Installer Guard"
type: "solution-walkthrough"
date: "2026-08-05"
branch: "docs/part-q-r-governance-record"
---

# Solution: Upstream Repository Repair and Installer Guard

Two deliverables from one root cause: the damage was repaired, and the defect that
caused it was closed so it cannot recur.

## 1. Root Cause

The **downstream** installation block published in `README.md` and `install-guide.md`
was executed inside the **upstream** repository. Its cleanup steps are correct for a
downstream project and destructive here.

Three steps in that block are unsafe in this repository:

| Step | Effect when run here |
|---|---|
| `rm -rf ./skills ./rules ./.pipeline ./.agents ./scripts ...` | Deletes the pipeline before copying it back. Uncommitted work is lost. |
| `rm -rf ./.pipeline/upstream` | Deletes the upstream-only tooling profile that this repository owns. |
| `cat ./.tmp-pipeline/.gitignore >> ./.gitignore` | Concatenates `.gitignore` onto itself. |

### Evidence

- `git status --porcelain` reported exactly two entries: ` M .gitignore` and
  `D  .pipeline/upstream/pipeline-tooling.md` (deletion staged).
- `git show HEAD:.gitignore` is 34 lines; the working file was 80. Lines 35-68 were a
  byte-identical duplicate of the committed 34, confirmed by `diff` producing no output.
- `git rev-parse HEAD origin/main` returned the same SHA, so the damage was local-only
  divergence from a synchronised remote.

### Third consequence, initially missed

The duplicated `.gitignore` ended with broad re-inclusions (`!/.pipeline/**`,
`!/scripts/**`, `!/skills/**`) which sit *after* the ignore rules and therefore win.
`git check-ignore -v` named the deciding lines:

```
.gitignore:76:!/.pipeline/**   .pipeline/diagnostics/repro_payload_...json
.gitignore:80:!/scripts/**     scripts/__pycache__/
```

`HEAD:.gitignore` ignores both (`__pycache__/` line 2, `.pipeline/diagnostics/` line 28).
Under the corrupted file they were untracked-and-visible, so any routine `git add .`
would have committed `__pycache__/` trees and diagnostic payloads as if they were
source.

## 2. Impact

The deletion had a dedicated gate, which was failing:

- `tests/test_upstream_profile_containment.py::test_profile_exists_in_upstream_dir`
- `tests/test_upstream_profile_containment.py::test_profile_declares_upstream_only_scope`

Suite before repair: **13 failed, 532 passed, 18 skipped**.
Suite after repair: **11 failed, 536 passed, 16 skipped**.

The delta of exactly 2 is those tests. The remaining 11 failures are pre-existing,
unrelated to this event, and out of scope.

## 3. The Repair

Both changes were reverts to `HEAD`. No content was authored — the 197-line profile was
intact in git history and its frontmatter already declared `scope: upstream-only`, which
is what the second failing test asserts.

```
git restore --staged --worktree .pipeline/upstream/pipeline-tooling.md
git restore .gitignore
```

Because the repair restored files to their committed state, it produced no diff to
commit. It landed as an absence of change.

## 4. The Guard

Added as the first statement of the installation block in both documents, ahead of every
destructive step:

```bash
if [ -e ./.pipeline/upstream ]; then
  echo "REFUSING: this is the pipeline repository, not a downstream project." >&2
  exit 1
fi
```

`.pipeline/upstream/` exists only in the upstream repository — the installer deletes it
from every downstream copy — so its presence is a reliable discriminator.

`test -e` is used rather than `find -type f` because `rules/document-references.md`
§ *Existence Claims Must Use Commands That Observe Symlinks* requires existence checks to
observe symlinks and their targets.

The guard is plain POSIX shell. It introduces no dependency on any particular agent
harness, configuration format, or tool.

## 5. Verification

Guard behaviour, both directions:

```
$ (in DEAP-spec-core)
REFUSING: this is the pipeline repository, not a downstream project.
exit=1

$ (in an empty directory)
would proceed to clone
exit=0
```

Ordering constraints inside the block are pinned by two pre-existing tests, both of which
still pass after the edit:

```
.venv/bin/python -m pytest tests/test_upstream_profile_containment.py tests/test_label_bootstrap_issue323.py -q
12 passed
```

Full suite:

```
.venv/bin/python -m pytest tests/ -q
11 failed, 536 passed, 16 skipped
```

The 11 are the documented pre-existing set; no new failure was introduced.

## 6. Code Realization Table

| Element | File | Detail |
|---|---|---|
| Installer guard (primary docs) | `README.md` | First statement of the Direct Copy Installation block |
| Installer guard (setup guide) | `install-guide.md` | First statement of § 2 Direct Copy Installation Workflow |
| Restored artefact | `.pipeline/upstream/pipeline-tooling.md` | 197 lines, `scope: upstream-only` frontmatter |
| Restored artefact | `.gitignore` | 34 lines, duplicate block removed |
| Deletion gate | `tests/test_upstream_profile_containment.py` | `test_profile_exists_in_upstream_dir`, `test_profile_declares_upstream_only_scope` |
| Block-ordering gate | `tests/test_upstream_profile_containment.py` | `test_readme_direct_copy_deletes_the_upstream_dir`, `test_readme_deletion_comes_after_the_pipeline_copy` |
| Block-ordering gate | `tests/test_label_bootstrap_issue323.py` | Asserts `cp -RP ./.tmp-pipeline/skills` precedes the bootstrap invocation |

## 7. Residual Risk

- **The GitHub template route is not covered.** `gh repo create <name> --template
  gintatkinson/DEAP-implementation-driver` copies the whole tree including
  `.pipeline/upstream/`. The `export-ignore` attribute in `.gitattributes` applies to
  `git archive`, not to template instantiation, and the guard added here protects only
  the Direct Copy block. A downstream project created by template therefore still
  receives the upstream-only profile.
- **The guard is advisory if the block is not pasted whole.** An operator running the
  individual lines by hand skips it.
- **Eleven pre-existing test failures remain**, concentrated in
  `tests/test_rule_contracts.py`, `tests/test_title_namespacing_issue317.py` and
  `tests/test_validator_findings_migration_issue304.py`. Unrelated to this work and
  untouched by it.

## 8. Source References

Governing rules: `rules/document-references.md` § *Existence Claims Must Use Commands
That Observe Symlinks*, `rules/verification-required.md`.
Containment rationale: `.pipeline/upstream/pipeline-tooling.md` (upstream-only scoping).
Plan of record: `implementation_plan.md`, the repair section and its executed change
record.
