<!-- Copyright Gint Atkinson, gint.atkinson@gmail.com -->

# Rule: Document References Must Resolve

**ALWAYS enforce:** A governance document that refers to a path, a step, or another
document must refer to something that exists.

## Hard constraints

- **Repository-Relative Skill Paths**: Governance documents MUST cite skill files through
  the repository-relative `skills/` prefix, never through the `.agents/skills/` symlink.
  The symlink is tracked (git mode `120000`, pointing at `../skills`) and resolves under a
  normal checkout, but it is not guaranteed to be materialised under archive extraction,
  `core.symlinks=false`, or a filesystem without symlink support. The failure is
  asymmetric and confusing: a `skills/` reference on one line still resolves while an
  `.agents/skills/` reference on the next fails, pointing debugging at the wrong subsystem.
- **Cited Paths Must Resolve**: Every repository path named in a governance document MUST
  exist on disk. A dangling path is worse than no reference, because it sends the reader —
  or a dispatched subagent instructed to read it — somewhere empty, and the resulting
  silence looks like an absence of instructions rather than a broken link.
- **Markdown Links Must Resolve**: Every relative markdown link in a backlog specification
  MUST point at a file that exists. This is the specification-corpus counterpart of the
  rule above — that one governs paths named in governance prose, this one governs
  `[text](target)` links in Epics, Features, User Stories and Use Cases, and it is
  enforced by a different checker (`link_validator.py`, offline, no network). A broken
  link inside a specification is worse than a broken prose reference, because the
  specification is published to the tracker where the link renders as live and resolves
  to a 404 for every reader.
- **Cited Steps Must Resolve**: Every cross-document citation of another document's numbered
  step MUST name a step that exists in the cited document. A citation to a step that is not
  there defeats verification: a reader checking whether an override or cross-reference
  actually covers the text it claims to cover cannot locate the target, and any step the
  citation omits is silently excluded from whatever the citing document asserts.
- **Runtime Tool Names Belong In The Dispatch Table**: A normative sentence that directs an
  agent to dispatch, spawn or terminate a subagent MUST state the **capability** required and
  MUST NOT name the concrete tool that provides it. Concrete per-runtime tool names belong
  only in the dispatch table in `.agents/AGENTS.md` § *Mandatory Subagent Dispatch for
  Research, Specification & Implementation Loops*, which is the single place a change of
  runtime has to be reflected. A directive naming a tool the active runtime does not expose
  is unexecutable, and an agent facing it does the work itself rather than halting — the
  failure recorded in issue #312, where the coordinator wrote every file directly for an
  entire session. This is the same shape as the three constraints above: a reference to
  something that is not there, differing only in that the missing referent is a tool rather
  than a path or a step. **Naming a tool in order to prohibit its use remains permitted** —
  a prohibition that no longer matches the runtime becomes inert, not unexecutable.

- **Authoritative Source Locators Must Be Preserved Verbatim**: A `Source References`
  entry describing an external artefact — a structural schema or a normative
  specification — MUST carry the authoritative upstream URL exactly as supplied, and MUST
  NOT be rewritten to point at this repository. Those artefacts are external by
  definition, so a self-referential locator means the upstream URL was replaced during
  drafting, breaking the traceability the reference exists to provide. Generative models
  bias heavily toward local paths, so this needs stating rather than assuming (issue
  #322). Enforcement is structural and offline: reachability is deliberately NOT checked,
  because `.pipeline/upstream/pipeline-tooling.md` § *Validation Gates* forbids network
  egress in a blocking gate and forbids sending specification content to a third party.
- **Existence Claims Must Use Commands That Observe Symlinks**: A claim of path existence or absence MUST be derived from a command capable of observing symlinks and symlink targets (such as `test -e`, `ls`, or `find` without `-type f`). `find -type f` lists neither symlinks nor their targets, producing false absence claims (as occurred in #305 and #294).

## Scope

These constraints apply to every document under `rules/`, `skills/`, `.agents/` and
`.pipeline/` — the corpus scanned by `tests/test_skill_path_references.py`. Hidden
directories are explicitly in scope; omitting them is how the prefix rule went unenforced
against the only document that violated it (issue #305).

## Enforcement

Mechanically enforced, offline, by `tests/test_skill_path_references.py`. The pairing
between each constraint above and its enforcing assertion is registered in
`tests/rule_contracts.py`, so removing either side fails the suite.

## Why

Until issue #310 these three rules were enforced by tests and stated in no document.
`rules/constitution-first.md` requires agents to read `rules/` before any task, so a rule
absent from `rules/` is invisible to the process meant to guarantee compliance — an agent
could follow every documented rule and still fail the suite. It also left the rules
unamendable, because there was no text to propose a change against.

That is the **orphan enforcement** failure mode `tests/rule_contracts.py` exists to detect,
and the same class as issue #299, where the Mermaid parser rejected unquoted relationship
labels that `rules/platform-independence.md` never mentioned.

The fourth constraint was added afterwards, and has the reverse provenance. Issue #312
already established it twice — `.agents/AGENTS.md` § *Mandatory Subagent Dispatch* states
that concrete tool names belong in its dispatch table "and nowhere else in this document",
and `rules/user-authorization-lock.md` restates it for the authorization lock. Both
statements are scoped to their own file, so `skills/` was covered by neither, and the #312
sweep left one live violation behind in `skills/feature-driven-implementation/SKILL.md`.
Stating the rule here gives it a normative home with repository-wide scope and a mechanical
gate, rather than two file-local assertions that no document generalises and no test checks.
