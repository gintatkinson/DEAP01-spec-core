<!-- Copyright Gint Atkinson, gint.atkinson@gmail.com -->

# Rule: Constitution First

**ALWAYS enforce:** Before beginning any pipeline task, read the project constitution.

## Required reads

1. **Agent operating rules** (root `AGENTS.md`; `.agents/AGENTS.md` is an optional alias) -- project-scoped agent rules, including the **Strict Planning Gate** that governs when file writes are permitted. Read this FIRST, before the constitution. It states the strictest authorization rule in the repository, and omitting it leads agents to act on weaker rules found elsewhere.
2. **Functional constitution** (located at the path resolved from configuration, e.g., `.pipeline/constitution.md`) -- domain rules, spec standards, agent behavior, quality gates. Read this before ANY task (specification or implementation).
3. **Implementation profile** (located under the profiles directory, e.g., `.pipeline/profiles/<platform>.md`) -- platform-specific coding standards, testing mandates, build config. Read this ONLY when implementing features, not during spec generation.

## Hard constraints

- If the functional constitution file exists in the repository, you MUST read it before starting work. Do not skip it.
- If you are implementing a feature and no implementation profile exists for the target platform, HALT and ask the human to create one.
- If a proposed change conflicts with any constitution document, HALT and escalate to the human.
- Specification workers/modules and `UPSTREAM_SPEC_CORE_COMPILER` workers MUST NOT read implementation profiles -- they operate on functional specs and compiler internals only.
- Curated single-item subagents receiving prompts ending in `---GOVERNANCE-END---` execute their assigned `SKILL.md` directly without repeating full coordinator initialization ceremonies.
- You MUST read root `AGENTS.md` (or its optional alias `.agents/AGENTS.md` where present) before writing or modifying any file. Its *Strict Planning Gate* requires an approved implementation plan and overrides any rule that treats an authorization keyword alone as sufficient. See `rules/user-authorization-lock.md` § *Precedence*.
- `.pipeline` and `.agents` are hidden directories and generic glob tools may exclude them. You MUST read root `AGENTS.md`, `.pipeline/constitution.md` and `.pipeline/profiles/` directly by their explicit paths. A file absent from this list and hidden from glob is effectively invisible -- that is how the strictest authorization rule came to be missed.
- Baseline conformance is verified exclusively with the native Rust binary `./target/release/verify-baseline . --no-domain`; the retired `scripts/verify_downstream_baseline.py` MUST NOT be invoked (Check 33).

## Silent Once-Per-Session Lazy Loading (Zero Receipt Ceremony)

- **Zero Startup Ceremony on Conversational/Read-Only Turns**: Agents MUST NOT interrupt or halt a user's prompt to run startup ceremonies, MUST NOT re-initialize across turns, and MUST NOT emit session initialization receipt tables or `"Ready for instructions"` prompts unless explicitly requested.
- **Once-Per-Session Pipeline Context Load**: Before executing a pipeline specification or feature-implementation skill workflow for the first time in a session, the coordinator silently verifies `.pipeline/upstream/` presence, `git status --short --branch`, `.pipeline/constitution.md`, and `.pipeline/ACTIVE_RULES_BUNDLE.md` at most once per session (never re-reading files already present in conversation context, and skipping `.pipeline/constitution.md` / `.pipeline/ACTIVE_RULES_BUNDLE.md` during diagnostic/read-only turns and bounded surgical edits).

## Why

The constitution captures non-negotiable project constraints. Without reading it, agents may violate platform rules, skip required test types, use forbidden dependencies, or produce specs that don't conform to domain standards.
