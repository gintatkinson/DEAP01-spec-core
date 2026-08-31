<!-- Copyright Gint Atkinson, gint.atkinson@gmail.com -->

# Rule: Tracker as Source of Truth

**ALWAYS enforce:** Use the issue tracker's CLI commands resolved from configuration to query the tracker for authoritative issue state. Never trust local files or checklist documentation alone.

## Hard constraints

- Before working on a feature, verify its status using the tracker's configured issue view command.
- Before creating issues, check for duplicates using the tracker's configured issue query command.
- All issue lifecycle operations (create, close, edit, transition) MUST use the tracker's configured issue management commands — never modify local files as the primary operation.
- Every local specification file MUST include the tracker issue ID in its YAML frontmatter (`issue_id: <int>`).
- When referencing Issue IDs in markdown, use the live tracker Issue number — never hard-code or assume numbers.
- When constructing links to files in issue descriptions, dynamically determine the remote URL from the repository settings (e.g., `meta.upstream_repository`). Never use relative paths like `../features/...` in issue bodies.
- **Registered Issues Must Have A Local Specification**: every tracker issue carrying the Epic or Feature label MUST have a corresponding local specification file. A registered issue with no file means the branch baseline is incomplete, and the reconciler will write checklist state back to an issue whose specification nobody can read.
- **Local Indices Must Not Collide With Registered Issues**: a local specification whose ordinal (`epic-02`, `feat-07`) is already claimed on the tracker by a *differently titled* issue is a collision and MUST be renumbered. Reconciliation addresses specifications by ordinal, so the ambiguity silently retargets updates at the wrong issue.
- **Local Specification Titles Must Be Unique Within A Spec Type**: no two Epics, no two Features, no two User Stories and no two Use Cases may declare titles that normalise to the same key. Uniqueness is scoped **per spec type, not globally**: an Epic naming a theme and a Feature delivering part of it may legitimately share a subject, and the tracker treats `(spec type, normalised title)` as the identity for exactly that reason. Reconciliation builds its issue lookup on the normalised title, so two specifications of one type sharing a key resolve to whichever issue was seen last — one body is published over the other and the loser is orphaned. Enforced offline by `parity_auditor/validators/spec_title_uniqueness_validator.py`, using the same normalisation as `reconcile_backlog.py` so the gate collides in exactly the space the reconciler collides in.
- **Generated Item Titles Must Be Namespaced To Their Source Module**: a subagent drafting one item from one schema node MUST prefix the generated Epic, Feature, User Story and Use Case title with a bracketed short-code identifying the bounded context it came from — for example `[NI-Location] Geo Location` rather than `Geo Location`. Item subagents draft in isolation and never see one another's output, so a node name that recurs across modules (`geo-location`, `status`, `interface` are the standard cases) produces the same title twice and neither subagent is in a position to notice. The namespace is what makes the item identifiable in a shared backlog without opening it. What is mechanically gated is the *effect* — the uniqueness rule above; the gate does not check the shape of the prefix, because a prefix-shape check would reject every specification written before this rule and the invariant that actually protects the tracker is uniqueness rather than any particular spelling. That gap is recorded in `tests/rule_contracts.py` under `KNOWN_UNREGISTERED_FAMILIES` rather than left silent.

## What local files are for

- Local markdown files (`docs/epics/`, `docs/features/`, etc.) are the specification source, but their checklist states may be stale.
- Always sync local state with the tracker state using the backlog reconciliation script (e.g., `reconcile_backlog.py`).
- The authoritative "done" state lives in the issue tracker, not in local frontmatter.

## Multi-Provider Issue Tracker Standards (GitHub & GitLab)

DEAP supports multi-provider issue tracking across GitHub (SaaS/Enterprise) and GitLab (GitLab.com, Self-Hosted EE/CE, and Air-Gapped / SCIF defense enclaves). All issue lifecycle management operations adhere to unified semantics across providers.

### GitLab Scoped Label Taxonomy

GitLab issue tracking uses native scoped labels (`key::value`) to ensure mutual exclusivity and deterministic state tracking:

- **Artifact Type Scoped Labels**:
  - `type::epic`: Architectural Epic specifications.
  - `type::feature`: Subsystem / Component Feature specifications.
  - `type::user-story`: Behavioral User Story specifications.
  - `type::use-case`: System Interaction UML Use Case specifications.
- **Lifecycle Status Scoped Labels**:
  - `status::ready-for-review`: Specification elaboration complete and ready for automated / peer review.
  - `status::fixed-resolved`: Verification gates passed; implementation / specification completed.

Applying a scoped label (e.g., `status::fixed-resolved`) automatically supersedes any conflicting label in the same scope (such as `status::ready-for-review` or `status::in-progress`).

### Environment Token Hierarchy & Authentication

Authentication tokens for issue tracker API callers and CLI tooling are resolved in the following priority order:

1. **GitLab Provider**:
   - `GITLAB_TOKEN`: Primary Personal Access Token (PAT) for GitLab REST API v4 operations.
   - `GL_TOKEN`: Standard CLI token for `glab` operations.
   - `CI_JOB_TOKEN`: Ephemeral pipeline execution token within GitLab CI/CD jobs.
2. **GitHub Provider**:
   - `GITHUB_TOKEN` / `GH_TOKEN`: Personal Access Token or workflow installation token for GitHub (`gh`) operations.

If no valid token is present in the environment for the detected or configured provider, automation scripts halt immediately to prevent unauthenticated drift.

### Product Owner Issue Closure Authority

Product Owner (PO) issue closure authority applies **identically across both GitHub and GitLab**. Automated agents, subagents, and background scripts are strictly permitted to transition issues only up to `status::fixed-resolved` (or `status:fixed-resolved` on GitHub) with verified evidence comments. Transitioning an issue to `Closed` (or applying `status::closed`) requires formal Product Owner / Certification Authority validation.

### Commit Message Issue Reference Rule (No Auto-Close Keywords)

Because GitHub and GitLab automatically close referenced issues when commit messages containing trigger keywords (`fix #<id>`, `fixes #<id>`, `fixed #<id>`, `close #<id>`, `closes #<id>`, `closed #<id>`, `resolve #<id>`, `resolves #<id>`, `resolved #<id>`) are merged or pushed to the default branch, **agents and scripts are strictly forbidden from using auto-closing trigger keywords in commit messages**.

All issue references in git commit messages MUST use neutral reference syntax:
- **Permitted**: `feat(compiler): multi-mode FMECA extraction (#50, #51)` or `fix(auditor): harden LinkValidator (refs #50)`
- **Strictly Forbidden**: `(fix #50, fix #51)`, `(closes #50)`, `(resolved #51)`, or any closing keyword preceding `#<id>`.


## Relationship to other rules

- See `rules/platform-independence.md` for specification content rules (WHAT vs HOW) and for all Mermaid syntax constraints.
- See `.pipeline/constitution.md` § *Unique Backlog Identifiers* for the issue ID frontmatter mandate.
- See `.pipeline/constitution.md` § *Agent Behavior* for commit and branch naming conventions referencing issue numbers.
- See `.pipeline/constitution.md` § *CMMI Level 3 & Scrum Issue Lifecycle Rules* for the states an agent may set. An agent may reach `Fixed / Resolved`; `Closed` requires Product Owner validation.

> **Consolidation note (issue #284):** this file supersedes the former
> `rules/github-source-of-truth.md`, which stated the same mandate in
> provider-specific wording. That file has been deleted and its unique clauses —
> the `issue_id` frontmatter mandate and this cross-reference section — merged here.
> Do not reintroduce a second statement of this rule; the fork drifted last time.

## Why

Local files can be contaminated, outdated, or contain broken links. The configured issue tracker is the canonical state store. Querying it directly prevents agents from working on closed issues, creating duplicates, or referencing stale data.
