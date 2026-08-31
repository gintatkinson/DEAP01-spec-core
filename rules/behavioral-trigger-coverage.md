<!-- Copyright Gint Atkinson, gint.atkinson@gmail.com -->

# Rule: Behavioural Trigger Coverage

**ALWAYS enforce:** when the workspace schema contains a node named by a behavioural
trigger, the specifications MUST cover that trigger's documented requirements.

## What a behavioural trigger is

`rules/behavioral_triggers.json` declares a list of triggers. Each carries
`trigger_nodes` — schema node names that arm it — and `rules`, each of which names a
`target_type` (`user-story` or `use-case`), the terms or Mermaid block the specification
must contain, and the `error_message` reported when it does not.

A trigger is **active** only when one of its `trigger_nodes` is present in the parsed
schema modules. An inactive trigger imposes nothing. This is deliberate: the requirement
is conditional on the system actually having the capability, so a project that never
streams high-frequency data is not asked to specify concurrency control for it.

## Hard constraints

- **Active Trigger Nodes Must Be Covered**: for every active trigger node, at least one
  specification file of the rule's `target_type` MUST reference that node by name. A
  trigger armed by the schema and mentioned in no User Story or Use Case is an
  unspecified capability, and coverage by a *different* node's file does not satisfy it —
  each active node is evaluated independently so one documented node cannot stand in for
  another.
- **Trigger Rules Must Be Satisfied**: a specification file that references an active
  trigger node MUST also satisfy that trigger's rule — the required Mermaid block and its
  match terms, the required body terms, and the required secondary body terms. Naming the
  node without specifying the behaviour the trigger exists to require is the failure mode
  this rule catches: the file looks like coverage and asserts nothing.

## Normative home & enforcement

**This file is the single normative home for behavioural trigger coverage.** It applies
across both specification types, so it is stated here rather than restated in
`spec-user-story-engineering/SKILL.md` and `spec-usecase-engineering/SKILL.md` — the
fragmentation issue #289 fixed for the Mermaid rules. The per-trigger requirements
themselves are data, not prose, and live in `rules/behavioral_triggers.json`.

These rules are mechanically enforced, offline, by
`parity_auditor/validators/behavioral.py`. Before issue #304 they were enforced and
stated in no document at all — the orphan-enforcement shape recorded as #299 — so a
subagent drafting a User Story could not have known the requirement existed.

## Why

Behavioural requirements are the ones most easily lost between a structural schema and a
functional specification. The schema records that a capability exists; nothing in the
schema records that its concurrency, temporal or failure behaviour has been specified.
The trigger list is how the pipeline carries that obligation forward, and it is only
worth carrying if it is both enforced and readable by whoever writes the specification.
