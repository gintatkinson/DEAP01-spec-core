<!-- Copyright Gint Atkinson, gint.atkinson@gmail.com -->

# Rule: Codebase Compliance

**ALWAYS enforce:** generated and hand-written source in a downstream workspace must
satisfy the compliance constraints below, and specification files must not leak
presentation detail into Tier 1.

## Scope, and why these are here rather than in a platform profile

Every rule below is **configuration-driven**. The validator reads the directories,
file extensions, keywords and patterns it matches from `codebase_rules.json` — for
example `flutter_rules.ffi_keywords`, `flutter_rules.write_lock_keywords`,
`spec_rules.dom_leak_patterns`. The *rule* is therefore platform-independent and only its
vocabulary is platform-specific, so this file is its normative home and
`.pipeline/profiles/<platform>.md` supplies the vocabulary. That division follows
`rules/platform-independence.md` § *Where platform-specific details belong*.

Enforced offline by `parity_auditor/validators/codebase.py`. Every rule below was
enforced before issue #304 and stated in no document — the orphan-enforcement shape
recorded as issue #299 — so nothing generating code into a downstream workspace could
have been told what it had to satisfy.

## Workspace configuration

- **Configured Platform Directory Must Exist**: if `target_directories` names a platform
  directory, that directory MUST exist whenever the workspace actually contains source
  files of that platform's extensions. A configured directory that is absent while
  matching sources sit elsewhere means every platform check silently scans nothing — a
  compliance bypass that reads as a clean run.
- **Design Tokens Path Must Be Configured**: `spec_rules.design_tokens_path` MUST be set.
  It is the authority for which colours are forbidden in source, so without it the
  hardcoded-colour rules below cannot run at all.
- **Design Tokens File Must Exist**: the configured path MUST resolve on disk.
- **Design Tokens File Must Be Loadable**: the file MUST parse as JSON.
- **Design Tokens Must Declare Colours**: at least one hex colour MUST be extractable
  from it. A token file with no colours makes every hardcoded-colour check vacuous, and a
  vacuous check is indistinguishable from a compliant codebase — which is why the four
  rules above are reported as failures rather than skipped preconditions.

## Source file integrity

- **Source Files Must Be Valid UTF-8**: a file with a platform source extension MUST
  decode as UTF-8. A binary blob wearing a source extension is skipped by every
  content-based check below, so it is reported rather than passed over.
- **Source Files Must Be Readable**: a source file that cannot be opened is reported. The
  alternative — continuing silently — shrinks the audited corpus without saying so.

## Presentation and design tokens

- **Hardcoded Design Token Colours Are Forbidden**: no source file may embed a colour
  literal equal to a colour declared in the design tokens file. Reference the theme or
  the tokens configuration instead. A duplicated literal is the value that stops tracking
  the token when the token changes.
- **Python Source Must Not Hardcode Token Constants**: the same constraint, checked
  through the Python AST rather than by pattern, so a colour assembled from constants is
  caught as well as one written literally.

## Interaction and concurrency

- **Selection Setters Require An Event Echo Guard**: a file that both sets selection state
  and emits a change notification MUST carry a loop-guard flag (`userInitiated`,
  `programmatic` or an equivalent from `loop_guard_keywords`). Without one, the
  notification re-enters the setter and the selection oscillates.
- **UI Layers Must Not Import Banned Libraries**: files under the configured UI
  directories MUST NOT import the libraries listed in `forbidden_words`. Heavy
  computation belongs in a background isolate or worker; importing it into a view puts it
  on the frame-rendering thread.
- **Network Gateways Require A Write Lock**: files matching the configured network
  gateway patterns MUST define a write-lock control, so that egress mutations are blocked
  during timeline playback or scrubbing. Replaying history while writing to the live
  system is how a diagnostic view becomes an outage.
- **Viewports Require Playhead Rate Clamps**: files matching the viewport patterns MUST
  implement the configured playhead rate clamps. An unclamped rate lets a 4D
  spatial-temporal viewport request frames faster than the source can supply them.

## Foreign function interfaces

- **FFI Boundaries Require A Native Finalizer**: a file using the configured FFI keywords
  MUST register a native finalizer. Without one, native allocations outlive the managed
  objects that own them and the process leaks for as long as it runs.
- **FFI Boundaries Require Reference Counting**: the same files MUST implement reference
  counting over native allocations. A finalizer alone frees on collection; reference
  counting is what makes shared native memory safe to free at all.

## Specification files

- **Specifications Must Not Leak DOM Attributes**: files listed in `spec_rules.spec_files`
  MUST NOT contain DOM or accessibility attributes such as `aria-*` or `role="..."`.
  These name a specific rendering technology, which is the Tier 1 contamination
  `rules/platform-independence.md` exists to prevent, reaching the logical component
  specification rather than the backlog.
- **Specifications Must Not Hardcode Pixel Dimensions**: the same files MUST NOT contain
  literal pixel dimensions. Express size through configuration tokens, so one functional
  specification can drive implementations with different density and scaling.
- **Specification Files Must Be Readable**: a configured specification file that cannot be
  read is reported rather than skipped, for the same reason as the source-file rule above.

## Why

The two-tier architecture only holds if something checks it. These constraints are the
ones whose violation is invisible in review — a duplicated colour literal, a missing loop
guard, an unclamped rate, an `aria-label` in a logical specification — and each of them
degrades silently rather than failing loudly.
