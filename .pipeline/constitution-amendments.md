# Constitution Amendment Log

Append-only record of every change to `.pipeline/constitution.md`.

`tests/test_constitution_integrity.py` asserts that the SHA-256 of the constitution
matches the newest entry below. **The constitution therefore cannot be changed by
anyone — human or agent — without a logged entry.** An unlogged edit fails the suite.

## Why this exists

`.agents/AGENTS.md:60` permits rewriting `constitution.md` only when *"every line of
the replacement has been explicitly approved by the user in the current conversation
turn"*, and `skills/project-constitution/SKILL.md` Mandate 4 forbids modifying it
*autonomously*. Both allow an approved amendment; neither described **how**.

`project-constitution` Step 7 gave implementation profiles a full lifecycle — add,
update, remove, list — while the constitution, the higher-authority Tier 1 document,
had a single sentence at line 285. The stronger document had the weaker process, so
the safe default became refusal, and two known defects stayed unfixed as a result.
See Step 9 of that skill for the procedure this log serves.

## Entry format

Every entry MUST carry all fields. The integrity test rejects a partial entry.

- **Date** — the date of the constitution change, matching the file's `last_updated`.
- **Logged** — when this entry was written.
- **Motivating issue** — the issue that justifies the change, or `n/a` with a reason.
- **Approved by** — verbatim quote of the human's approval, or `n/a` for the baseline.
- **Destructive** — `no` if the change is purely additive or a refinement in place.
  `yes` requires a justification paragraph; `project-constitution` Mandate 3 requires
  amendments to be cumulative and never destructive.
- **Line count** — lines in the resulting file. A non-destructive entry may not
  reduce it below the previous entry.
- **Resulting SHA-256** — checksum of `.pipeline/constitution.md` after the change.

---

## AMEND-0000 — Baseline

- **Date:** 2026-06-29
- **Logged:** 2026-07-31
- **Motivating issue:** n/a — protocol adoption, no content change
- **Approved by:** n/a — baseline record; the constitution was not modified
- **Destructive:** no
- **Line count:** 161
- **Resulting SHA-256:** `ae68494ed6190919dc342612d27f873d01f273750b2d1418aa3516913dac24e9`

Records the constitution's state at the moment the amendment protocol was adopted, so
the integrity test has a starting point. No content was changed by this entry. The
`Date` is the constitution's existing `last_updated` value rather than the date this
entry was written, because it records the state as of the last actual change.

### Amendments known to be pending at baseline

Two divergences between the constitution and the implemented, enforced behaviour were
outstanding when this log was created. Both are registered in
`tests/rule_contracts.py` `KNOWN_DOC_DIVERGENCES` and both require line-by-line human
approval before they can be applied through Step 9:

1. **Line 41 — external actor exemption (issue #277).** The constitution requires
   *every* sequence-diagram lifeline to resolve to a defined Class or Component. The
   enforced rule exempts lifelines declared as external UML actors, which are outside
   the system boundary and correctly absent from the structural models.
2. **Line 120 — authorization sufficiency (issue #295).** The constitution states that
   typing "Proceed" is sufficient authorization. `.agents/AGENTS.md:7` states a
   keyword is explicitly insufficient without an approved implementation plan, and
   #295 unified on the stricter reading. The constitution sentence remains weaker.

---

## AMEND-0001 — External actor exemption for sequence-diagram lifelines

- **Date:** 2026-07-31
- **Logged:** 2026-07-31
- **Motivating issue:** #277 (implemented), #298 (divergence class)
- **Approved by:** "approve both" — in response to Amendments A and B quoted verbatim as current-versus-proposed text, per Step 9 items 2 and 3.
- **Destructive:** no
- **Line count:** 161
- **Resulting SHA-256:** `98434ea59d1fdba780cf2aea430004658c0dc0776519f0374c96a809e5152a6e`

### Change

Section *Universal Model Consistency Rules*, line 41.

Before:

> - Every lifeline in a sequence diagram MUST represent an instance of a defined logical Class or Component.

After:

> - Every lifeline in a sequence diagram MUST represent an instance of a defined logical Class or Component, except lifelines declared as external actors (UML `actor`), which represent entities outside the system boundary and are therefore not defined in the structural models. Every non-actor lifeline MUST resolve to a defined classifier.

### Rationale

Issue #277 replaced a name-suffix bypass in `validators/uml.py` with an exemption keyed
on UML role. The prior rule exempted classifiers by spelling — `PaymentManager` passed
while `PaymentHandler` did not — and, because an exempt classifier never entered the
global class registry, it also silently disabled operation-signature validation for
every message sent to that lifeline.

Deleting the bypass outright was tested and rejected: an ordinary human actor
`payer : Payer` was reported undefined, which is the false-positive class that commit
`a5de5f8` had introduced the bypass to remove. A UML `actor` denotes an entity outside
the system boundary and is correctly absent from the structural models.

The sentence as written required *every* lifeline to resolve, so the implemented rule
was narrower than the constitution. Left unamended this would have been an eighth
instance of the defect class #298 exists to detect — a documented contract diverging
from the enforced one — created while closing the third.

Non-destructive: the original requirement is preserved in full and an exemption is
carved out. The second clause restates the requirement for all non-actor lifelines so
no obligation is weakened by implication.

---

## AMEND-0002 — Authorization requires an approved plan, not only a keyword

- **Date:** 2026-07-31
- **Logged:** 2026-07-31
- **Motivating issue:** #295
- **Approved by:** "approve both" — in response to Amendments A and B quoted verbatim as current-versus-proposed text, per Step 9 items 2 and 3.
- **Destructive:** no
- **Line count:** 161
- **Resulting SHA-256:** `5dc3da88e38fa9333e1dd297701fdd4082fda48b343363f850e1ab20a26e5e50`

### Change

Section *Strict Planning Mode Gate (Insurmountable Approval Gate)*, line 120. Line 121
is untouched.

Before:

> - Under NO circumstances may the agent invoke any file-writing, file-modifying, or command-running tools that alter the codebase/repository files unless the user has explicitly typed "Proceed", "Approved", or "Approve plan" in the conversation history of the current turn sequence.

After:

> - Under NO circumstances may the agent invoke any file-writing, file-modifying, or command-running tools that alter the codebase/repository files unless BOTH of the following hold: (1) the specific file and its exact changes are documented in an approved implementation plan, AND (2) the user has explicitly typed "Proceed", "Approved", or "Approve plan" in the conversation history of the current turn sequence. An authorization keyword alone is NOT sufficient. See `.agents/AGENTS.md` § Strict Planning Gate, which takes precedence, and `rules/user-authorization-lock.md` § Precedence.

### Rationale

Issue #295 found three documents stating three different rules for what authorizes a
file write. This sentence and `rules/user-authorization-lock.md:9` both treated an
authorization keyword as sufficient, while `.agents/AGENTS.md:7` states explicitly that
a keyword is **not** sufficient without an approved implementation plan.

The decisive defect was reading order: `rules/constitution-first.md` enumerated the
mandatory reads without listing `.agents/AGENTS.md`, and `.agents/` is a hidden
directory that glob and ripgrep skip. An agent complying fully with constitution-first
therefore read only the two keyword-sufficient documents and never saw the strictest
rule. That is exactly what happened during this session: two commits reached `main`
before `AGENTS.md` was discovered.

#295 unified on the strictest reading, on the grounds that under-authorizing costs one
redundant question whereas over-authorizing causes unapproved writes. This amendment
brings the Tier 1 document into agreement with that resolution, so the highest-authority
statement is no longer the weakest.

**This amendment tightens a constraint on the agent.** It removes an authorization path
the agent previously had, and adds no capability.

Non-destructive: no principle is removed. Condition (2) preserves the original clause
verbatim, and condition (1) is added alongside it.

---

## AMEND-0003 — Standardize product name to Digital Engineering Agent Platform (DEAP)

- **Date:** 2026-08-06
- **Logged:** 2026-08-06
- **Motivating issue:** n/a — product name standardization across repository
- **Approved by:** "PROCEED" — approved implementation plan to standardize official product name to Digital Engineering Agent Platform (DEAP).
- **Destructive:** no
- **Line count:** 161
- **Resulting SHA-256:** `952397210c5163672e05bac9b1afcaa1351522e2ad6a3c18c09525cdc6cae896`

### Change

Frontmatter line 3 and main title line 9.

Before:

> project: "Digital Systems Engineering Pipeline"
> # Project Constitution: Digital Systems Engineering Pipeline

After:

> project: "Digital Engineering Agent Platform (DEAP)"
> # Project Constitution: Digital Engineering Agent Platform (DEAP)

### Rationale

Standardize the official product name to Digital Engineering Agent Platform (DEAP) across the repository in accordance with the approved implementation plan.

Non-destructive: product name updated, governance rules unchanged.

---

## AMEND-0004 — Clarify Logical UI (LUI) platform-independence and canonical avionics patterns

- **Date:** 2026-08-06
- **Logged:** 2026-08-07
- **Motivating issue:** Evolved LUI Architecture and Safety-Critical Real-Time UI Framework Blueprint
- **Approved by:** "PROCEED" — approved implementation plan to evolve LUI architecture to support ARINC 661, FSM symbology, and safety-critical real-time UI framework.
- **Destructive:** no
- **Line count:** 161
- **Resulting SHA-256:** `79141cff13372778f6f3e2243478512a47026584bdd1f86da6942ade07390e1a`

### Change

Section *Standard & Platform Parameter Isolation*, Tier 1 Functional Layer, line 53.

Before:

> 1. **Tier 1: Functional Layer (Abstract Specification)**: Epics, Features, User Stories, Use Cases, and Logical UI specifications. Must be platform-independent and standard-agnostic. No framework keywords, specific standards designations, or hardcoded visual values allowed.

After:

> 1. **Tier 1: Functional Layer (Abstract Specification)**: Epics, Features, User Stories, Use Cases, and Logical UI (LUI) specifications. Logical UI is 100% platform-independent and UI-framework-agnostic, supporting 3 canonical architectural patterns: (A) ARINC 661 Cockpit Display Systems (UA Parameter Buffer -> CDS Widget Definition -> Display Kernel Render), (B) Real-Time Safety Statecharts & Symbology (Discrete Event -> Safety Statechart/FSM State -> Symbology/Alarm Render), and (C) Decoupled Operator Consoles & EFBs (Operator Action -> ViewModel/State Holder -> GUI Component Binding). Must be platform-independent and standard-agnostic. No framework keywords, specific standards designations, or hardcoded visual values allowed.

### Rationale

Clarify that Logical UI is 100% platform-independent and supports 3 canonical avionics display patterns (ARINC 661 CDS, Real-Time FSM Symbology, Decoupled Operator Consoles/EFBs).

Non-destructive: additive clarification of Tier 1 LUI scope, governance rules unchanged.

---

## AMEND-0005 — Transform BDD and LUI specifications to Evolved 3-Layer Aerospace Semantics

- **Date:** 2026-08-07
- **Logged:** 2026-08-07
- **Motivating issue:** Aerospace & Real-Time Control Semantic Transformation (Evolved 3-Layer Chain & Canonical Aerospace BDD Templates)
- **Approved by:** "PROCEED" — approved implementation plan for Aerospace & Real-Time Control Semantic Transformation.
- **Destructive:** no
- **Line count:** 164
- **Resulting SHA-256:** `d3c6ef70323acb045f080bca88d482a609768aaec1680695daef3a4474a1734c`

### Change

Section *Standard & Platform Parameter Isolation*, Tier 1 Functional Layer (line 53), and Section *BDD Scenario Format* (lines 81-88).

Before:

> - All acceptance criteria MUST use Given-When-Then format.

After:

> - All acceptance criteria MUST use Given-When-Then format adhering to canonical aerospace BDD templates:
>   - **Pattern A (ARINC 661 Cockpit Display Systems)**: `Given [UA Parameter Buffer State], When [ARINC 661 Binary Command Received], Then [CDS Widget State & Display Kernel Render Updated]`.
>   - **Pattern B (Real-Time Safety Statechart / Flight Control)**: `Given [Aircraft State Vector / Discrete Event], When [Safety FSM Transition Triggered], Then [Actuator Command / Symbology Graphic Rendered]`.
>   - **Pattern C (Decoupled Operator Console)**: `Given [Console Domain Model State], When [Operator Action Initiated], Then [ViewModel State & GUI Component Binding Updated]`.

### Rationale

Transform legacy BDD/LUI wording to lock in the Evolved 3-Layer Semantic Chain (Domain State & Signal Model -> Logic & Safety State Management -> Display & Actuator Interface Binding) and canonical aerospace BDD templates across ARINC 661, FSM safety statecharts, and operator console patterns.

Non-destructive: additive refinement of Tier 1 LUI and BDD scenario standards, governance rules unchanged.

---

## AMEND-0006 — Explicitly name enforcing validator paths for traceability mandates

- **Date:** 2026-08-08
- **Logged:** 2026-08-08
- **Motivating issue:** #378
- **Approved by:** "PROCEED" — approved implementation plan to explicitly name enforcing validator paths for all four traceability rules in constitution.md.
- **Destructive:** no
- **Line count:** 164
- **Resulting SHA-256:** `8a83667fd5594b03b5a3b9ad5f1357b389f89681b5e0c6e46306e7159d348b57`

### Change

Section *Traceability*, lines 47-50.

Before:

> - Every Epic MUST reference the specification section(s) it covers.
> - Every Feature MUST include a "Source References" section with verbatim specification clause numbers and schema paths.
> - Every User Story MUST link to the Features it validates.
> - Every Use Case MUST link to the User Stories and Features it realizes.

After:

> - Every Epic MUST reference the specification section(s) it covers. Enforced by parity_auditor/validators/uml.py via required sections configuration.
> - Every Feature MUST include a "Source References" section with verbatim specification clause numbers and schema paths. Enforced by parity_auditor/validators/uml.py and source_reference_validator.py.
> - Every User Story MUST link to the Features it validates. Enforced by parity_auditor/validators/uml.py via Required Features Matrix validation.
> - Every Use Case MUST link to the User Stories and Features it realizes. Enforced by parity_auditor/validators/uml.py via Realization Matrix validation.

### Rationale

Update each of the four traceability mandates in the Tier 1 constitution to explicitly name its enforcing validator paths (`parity_auditor/validators/uml.py` and `source_reference_validator.py`), closing document-enforcement traceability gaps as required by Issue #378.

Non-destructive: additive clarification of enforcing validator paths, governance rules unchanged.

---

## AMEND-0007 — Title normalization primary selector requirement for backlog reconciliation

- **Date:** 2026-08-08
- **Logged:** 2026-08-08
- **Motivating issue:** #377
- **Approved by:** "PROCEED" — approved implementation plan to replace title normalization prohibition with reconciliation primary selector requirement in constitution.md.
- **Destructive:** no
- **Line count:** 164
- **Resulting SHA-256:** `d763c600eda7ef7fa2447bcb732ec3e148c803d94bbebddf1824134f8dd2d016`

### Change

Section *Unique Backlog Identifiers*, line 59.

Before:

> - Matching by title normalization is prohibited as a primary selector.

After:

> - Matching by title normalization is the primary selector used by the backlog reconciliation tool. To prevent collisions, all specification files of the same spec type MUST have unique normalised titles, as enforced by parity_auditor/validators/spec_title_uniqueness_validator.py and rules/tracker-source-of-truth.md.

### Rationale

Update `.pipeline/constitution.md` under `### Unique Backlog Identifiers` to reflect that matching by title normalization is the primary selector used by the backlog reconciliation tool (`reconcile_backlog.py`), resolving governance document contradiction as required by Issue #377.

Non-destructive: title normalization rule updated to match enforced reconciliation primary selector, governance rules aligned.

---

## AMEND-0008 — Promote Three-Tier Platform Isolation architecture to top-level section

- **Date:** 2026-08-08
- **Logged:** 2026-08-08
- **Motivating issue:** #384
- **Approved by:** "PROCEED" — approved implementation plan to promote three-tier platform isolation architecture to top-level section in constitution.md.
- **Destructive:** no
- **Line count:** 195
- **Resulting SHA-256:** `f0d3ab82d4658f5798e3e228a3b5ec324f54be94dfb66762a1a0733ee85075f1`

### Change

Promote Three-Tier Platform Isolation architecture from `### Standard & Platform Parameter Isolation` under `## Domain Rules` to a top-level section (`## Architecture: Three-Tier Platform Isolation`) under Functional Layer Governance. Add Mermaid graph TD diagram with Tier 1, Tier 2, Tier 3 subgraphs and relationships, and explicit tier boundary guidelines.

Before:

> ### Standard & Platform Parameter Isolation
> 1. **Tier 1: Functional Layer (Abstract Specification)**: Epics, Features, User Stories, Use Cases, and Logical UI (LUI) specifications...
> 2. **Tier 2: Runtime Configuration Parameters (Dynamic Context)**: Design tokens...
> 3. **Tier 3: Platform Implementation Profiles (Technical Execution)**: `.pipeline/profiles/<platform>.md`...

After:

> ## Architecture: Three-Tier Platform Isolation
>
> The pipeline enforces a strict three-tier platform isolation architecture to decouple abstract functional specifications from dynamic runtime parameters and platform-specific execution details.
> [Mermaid graph TD diagram]
> ### Tier Boundary Guidelines
> 1. **Tier 1: Functional Layer (Abstract Specification)**...
> 2. **Tier 2: Runtime Configuration Parameters (Dynamic Context)**...
> 3. **Tier 3: Platform Implementation Profiles (Technical Execution)**...

### Rationale

Promote the three-tier platform isolation architecture to a top-level section with Mermaid diagram visualization to elevate foundational architectural isolation principles, resolving audit finding #384.

Non-destructive: Three-tier architecture principles promoted and expanded, line count increased from 164 to 195 lines, governance rules preserved.

---

## AMEND-0009 — Mandate Source References across all four specification types

- **Date:** 2026-08-08
- **Logged:** 2026-08-08
- **Motivating issue:** #383
- **Approved by:** "PROCEED" — approved implementation plan to update Source References mandate in constitution.md to explicitly cover Epics, Features, User Stories, and Use Cases.
- **Destructive:** no
- **Line count:** 195
- **Resulting SHA-256:** `89060b0d78264c603570fe54c7f2a36c3d91ea869c868fefaa0fc7989f2afb6d`

### Change

Section *Traceability*, line 81.

Before:

> - Every Feature MUST include a "Source References" section with verbatim specification clause numbers and schema paths. Enforced by parity_auditor/validators/uml.py and source_reference_validator.py.

After:

> - Every Feature MUST include a 'Source References' section with verbatim specification clause numbers and schema paths. Every Epic, User Story, and Use Case MUST also carry a 'Source References' section (or Realization / Target Features Matrix linking to upstream sources). Enforced by parity_auditor/validators/uml.py via required_sections configuration in codebase_rules.json.

### Rationale

Update `.pipeline/constitution.md` under `### Traceability` to explicitly mandate that every Epic, Feature, User Story, and Use Case carry a 'Source References' section (or Realization / Target Features Matrix linking to upstream sources), enforced via `required_sections` configuration in `codebase_rules.json`, resolving audit finding #383.

Non-destructive: additive clarification of Source References requirement across all four specification types, line count preserved at 195 lines, governance rules preserved.

---

## AMEND-0010 — Expand Quality Gates section with comprehensive table of all 15 enforced quality gates

- **Date:** 2026-08-08
- **Logged:** 2026-08-08
- **Motivating issue:** #382
- **Approved by:** "PROCEED" — approved implementation plan to expand Quality Gates section in constitution.md with comprehensive table of 15 active enforced quality gates.
- **Destructive:** no
- **Line count:** 216
- **Resulting SHA-256:** `b01ef7d5da9ebcfdc6d0244f94d2a78ff140e3161b8157d26dc233c9490ba52c`

### Change

Section *Universal Quality Gates*, line 157.

Before:

> ## Universal Quality Gates
>
> ### Specification Validation Gates
> - Post schema extraction: Every schema node maps to at least one Feature. Coverage = 100%.

After:

> ## Universal Quality Gates
>
> ### Quality Gates & Verification Standards
> The pipeline mechanically enforces 15 active quality gates that halt execution on failure. All agents MUST ensure deliverables comply with these gates before declaring completion:
>
> | Quality Gate | Enforcing Validator Path | Documentation Reference |
> |---|---|---|
> | Specification Validation | `validators/spec_validator.py` | `rules/platform-independence.md` |
> | Model Coverage Verification | `scripts/verify_model_coverage.py` | `rules/platform-independence.md` |
> | Cross-Reference Integrity | `validators/link_validator.py` | `rules/document-references.md` |
> | Human Approval | `rules/user-authorization-lock.md` | `.pipeline/constitution.md` |
> | Downstream Conformance | `scripts/verify_downstream_baseline.py` | `rules/downstream-conformance.md` |
> | UML Model Integrity | `validators/uml.py` | `rules/uml-model-integrity.md` |
> | Mermaid Syntax Constraints | `validators/mermaid_syntax_validator.py` | `rules/platform-independence.md` |
> | Behavioral Trigger Coverage | `validators/behavioral.py` | `rules/behavioral-trigger-coverage.md` |
> | Codebase Compliance | `validators/codebase.py` | `rules/codebase-compliance.md` |
> | Document Cross-Reference Integrity | `tests/test_skill_path_references.py` | `rules/document-references.md` |
> | Constitution Amendment Integrity | `tests/test_constitution_integrity.py` | `.pipeline/constitution-amendments.md` |
> | Specification File Integrity | `validators/docs.py` | `rules/platform-independence.md` |
> | Spec Title Uniqueness | `validators/spec_title_uniqueness_validator.py` | `rules/tracker-source-of-truth.md` |
> | Source Reference Integrity | `validators/source_reference_validator.py` | `rules/codebase-compliance.md` |
> | Logical UI Validation | `validators/logical_ui_validator.py` | `rules/platform-independence.md` |
>
> ### Specification Validation Gates

### Rationale

Expand `.pipeline/constitution.md` under `## Universal Quality Gates` with a `### Quality Gates & Verification Standards` table listing all 15 active enforced quality gates, their enforcing validator paths, and documentation references, resolving audit finding #382.

Non-destructive: additive table detailing all active quality gates, line count increased from 195 to 216 lines, governance rules preserved.

---

## AMEND-0011 — Update labeling taxonomy to explicitly include operational and state labels

- **Date:** 2026-08-08
- **Logged:** 2026-08-08
- **Motivating issue:** #381
- **Approved by:** "PROCEED" — approved user request for Issue #381 Tier 1 constitution amendment.
- **Destructive:** no
- **Line count:** 219
- **Resulting SHA-256:** `8c6a24269a55b1312c53799b68f72c8fc7511d6816ddf459d93ab958237a8dc0`

### Change

Section *Labeling Taxonomy*, lines 127-132.

Before:

> ### Labeling Taxonomy
> - Exactly four label types: `epic`, `feature`, `user-story`, `use-case`, or as defined by the issue tracker configuration.
> - Labels are bootstrapped via the configured label bootstrap command to ensure idempotency.

After:

> ### Labeling Taxonomy
> - Issue tracking labels are defined with `codebase_rules.json` acting as the authoritative label registry, categorized into specification, operational, and state labels:
>   - Specification labels: `epic`, `feature`, `user-story`, `use-case`.
>   - Operational labels: `bug`, `enhancement`, `chore`.
>   - State labels: `status:fixed-resolved`.
> - Labels are bootstrapped via the configured label bootstrap command to ensure idempotency.

### Rationale

Update `.pipeline/constitution.md` under `### Labeling Taxonomy` to explicitly list specification labels (`epic`, `feature`, `user-story`, `use-case`), operational labels (`bug`, `enhancement`, `chore`), state labels (`status:fixed-resolved`), and reference `codebase_rules.json` as the authoritative label registry, resolving audit finding #381.

Non-destructive: additive taxonomy expansion, line count updated from 216 to 219 lines, governance rules preserved.

---

## AMEND-0012 — Add CMMI Level 3 Process Area Mapping table explicitly substantiating process alignment

- **Date:** 2026-08-08
- **Logged:** 2026-08-08
- **Motivating issue:** #380
- **Approved by:** "PROCEED" — approved user request for Issue #380 Tier 1 constitution amendment.
- **Destructive:** no
- **Line count:** 233
- **Resulting SHA-256:** `65a9614cc4e0238acbabdb19fa07db52ff9a3b32c6b4e3ec71c71d4b826b239b`

### Change

Section *CMMI Level 3 & Scrum Issue Lifecycle Rules*, added subsection `### CMMI Level 3 Process Area Mapping`.

Before:

> ## CMMI Level 3 & Scrum Issue Lifecycle Rules
>
> ### Separation of Verification and Validation

After:

> ## CMMI Level 3 & Scrum Issue Lifecycle Rules
>
> ### CMMI Level 3 Process Area Mapping
> The pipeline explicitly substantiates CMMI Level 3 alignment across key engineering and management process areas:
>
> | Process Area (CMMI Acronym) | Enforcing Mechanisms & Pipeline Artifacts |
> |---|---|
> | Requirements Management (REQM) | `tracker-source-of-truth.md`, `reconcile_backlog.py` |
> | Verification (VER) | `verify_model_coverage.py`, `parity_auditor` validators |
> | Validation (VAL) | Product Owner `Closed` state transition & verification walkthroughs |
> | Configuration Management (CM) | Git-tracked specification files, `constitution-amendments.md` |
> | Technical Solution (TS) | 3-Layer LUI Definition of Done & implementation profiles |
> | Product Integration (PI) | Automated baseline verification `verify_downstream_baseline.py` |
>
> ### Separation of Verification and Validation

### Rationale

Add a CMMI Level 3 Process Area Mapping table under Section introducing CMMI Level 3 & Scrum Issue Lifecycle Rules in `.pipeline/constitution.md` explicitly substantiating CMMI Level 3 alignment across key process areas (REQM, VER, VAL, CM, TS, PI), resolving audit finding #380.

Non-destructive: additive table substantiating CMMI Level 3 alignment, line count increased from 219 to 233 lines, governance rules preserved.

---

## AMEND-0013 — Update Granularity Bounds to normative RFC 2119 language with explicit enforcers

- **Date:** 2026-08-08
- **Logged:** 2026-08-08
- **Motivating issue:** #379
- **Approved by:** "PROCEED" — approved user request for Issue #379 Tier 1 constitution amendment.
- **Destructive:** no
- **Line count:** 235
- **Resulting SHA-256:** `14d2e2dfc339f51e924e12908a6e7b1b9de1d58c2e48c523ed45b1aa2e1ee9c8`

### Change

Section *Specification Standards*, added subsection `### Granularity Bounds` and updated Epic and Feature Granularity rules.

Before:

> ### Epic Granularity
> - One Epic per major functional domain or protocol module.
> - An Epic should contain 3-15 Features. Fewer than 3 means the Epic is too narrow; more than 15 means it should be split.
> - Epic titles use the format: `[Module/Domain]: [Functional Area]`.
>
> ### Feature Granularity
> - A Feature represents a single, independently testable functional capability.
> - A Feature should have 3-10 acceptance criteria. Fewer means it lacks specificity; more than 10 means it should be split.
> - Features MUST be platform-independent and standard-agnostic.
> - Feature titles use the format: `[Verb] [Object] [Qualifier]`.

After:

> ### Granularity Bounds
> - An Epic SHOULD contain 3-15 Features. Epics exceeding 15 Features MUST be split by the schema-specification-engineering worker during Step 1 decomposition; Epics with fewer than 3 Features MUST be reviewed for consolidation. Enforced by schema-specification-engineering decomposition heuristics.
> - A Feature SHOULD carry 3-10 acceptance criteria. Features exceeding 10 acceptance criteria MUST be split into targeted sub-features; Features with fewer than 3 acceptance criteria MUST be expanded to ensure full scenario coverage. Enforced by parity_auditor/validators/cardinality_validator.py and spec worker review gates.
>
> ### Epic Granularity
> - One Epic per major functional domain or protocol module.
> - Epic titles use the format: `[Module/Domain]: [Functional Area]`.
>
> ### Feature Granularity
> - A Feature represents a single, independently testable functional capability.
> - Features MUST be platform-independent and standard-agnostic.
> - Feature titles use the format: `[Verb] [Object] [Qualifier]`.

### Rationale

Update `.pipeline/constitution.md` under `### Granularity Bounds` to use normative RFC 2119 SHOULD/MUST language with explicit enforcing mechanisms (`schema-specification-engineering` decomposition heuristics and `parity_auditor/validators/cardinality_validator.py`), resolving audit finding #379.

Non-destructive: additive Granularity Bounds subsection with normative language and explicit enforcers, line count increased from 233 to 235 lines, governance rules preserved.

---

## AMEND-0014 — Universal LUMI (Logical User & Machine Interface) Framework Integration

- **Date:** 2026-08-09
- **Logged:** 2026-08-09
- **Motivating issue:** Universal LUMI Framework Integration
- **Approved by:** "PROCEED" — approved user prompt directive to implement Multi-Interface Bindings (LUMI Framework) code, skill, constitution, and validator updates.
- **Destructive:** no
- **Line count:** 235
- **Resulting SHA-256:** `6b4a547ca9afe379bda2e185e455c46e9a1868f367fc8f4b5f1520ce1d7cc7cf`

### Change

Section *Architecture: Three-Tier Platform Isolation*, line 22 and line 44.

Before:

> T1_LUI["Logical UI (LUI) & 3-Layer Semantic Chain"]
> 1. **Tier 1: Functional Layer (Abstract Specification)**: Epics, Features, User Stories, Use Cases, and Logical UI (LUI) specifications...

After:

> T1_LUI["Logical User & Machine Interface (LUMI) & 3-Layer Semantic Chain"]
> 1. **Tier 1: Functional Layer (Abstract Specification)**: Epics, Features, User Stories, Use Cases, and Logical User & Machine Interface (LUMI) specifications. LUMI is 100% platform-independent and framework-agnostic, covering three primary interface categories: Visual GUI (`gui`), Machine-to-Machine API (`mcp`/`api`), and Hardware Bus (`hardware`). LUMI supports the Evolved 3-Layer Semantic Chain (Domain State & Signal Model -> Logic & Safety State Management -> Display & Actuator Interface Binding) across canonical architectural patterns (ARINC 661 Cockpit Display Systems, Real-Time Safety Statecharts & Flight Control, Decoupled Operator Consoles & EFBs, Automated M2M Agentic Tooling, and Hardware Bus Register Mapping). Must be platform-independent and standard-agnostic. No framework keywords, specific standards designations, or hardcoded visual values allowed.

### Rationale

Update `.pipeline/constitution.md` under `## Architecture: Three-Tier Platform Isolation` to define the Universal LUMI (Logical User & Machine Interface) Framework covering Visual GUI (`gui`), Machine-to-Machine API (`mcp`/`api`), and Hardware Bus (`hardware`) categories.

Non-destructive: additive definition of LUMI framework across Visual GUI, M2M API, and Hardware Bus modalities, line count preserved at 236 lines, governance rules preserved.

---

## AMEND-0015 — Standardize product name to Digital Engineering Agent Platform (DEAP)

- **Date:** 2026-08-10
- **Logged:** 2026-08-10
- **Motivating issue:** Product Name Standardization
- **Approved by:** "PROCEED" — approved user prompt directive to standardize official product name to Digital Engineering Agent Platform (DEAP).
- **Destructive:** no
- **Line count:** 235
- **Resulting SHA-256:** `77b040805aa4ada8c71a7e5265e90df34f09b1b4bc4b368aeb9fdd72f0b09b5b`

### Change

Frontmatter line 3 and main title line 9.

Before:

> project: "Digital Engineering Agentic Pipeline (DEAP)"
> # Project Constitution: Digital Engineering Agentic Pipeline (DEAP)

After:

> project: "Digital Engineering Agent Platform (DEAP)"
> # Project Constitution: Digital Engineering Agent Platform (DEAP)

### Rationale

Standardize official product name to Digital Engineering Agent Platform (DEAP) across `.pipeline/constitution.md` and repository artifacts per user instruction and debug-protocol skill execution.

Non-destructive: product name updated to Digital Engineering Agent Platform (DEAP), line count preserved at 236 lines, governance rules preserved.

---

## AMEND-0016 — Add Downstream Single Source of Truth (SSOT) & Clean Baseline Mandate

- **Date:** 2026-08-11
- **Logged:** 2026-08-11
- **Motivating issue:** 5-Whys Root Cause Remediation for Downstream Single Source of Truth & Clean Baseline Linters
- **Approved by:** "PROCEED" — approved implementation plan for 5-Whys root cause remediation.
- **Destructive:** no
- **Line count:** 239
- **Resulting SHA-256:** `76f73f60b1b5c2dd3be5c6327537ddd4c2fda17cabd3985b550534311a71ec56`

### Change

Section *Downstream Conformance Gates*, added subsection `### Downstream Single Source of Truth (SSOT) & Clean Baseline Mandate`.

Before:

> ### Downstream Conformance Gates
> - Prior to integrating any downstream application implementation, the project MUST be bootstrapped and verified.
> - The downstream project must be initialized using the configured bootstrap script.
> - Baseline conformance must be verified using the configured verification script, which asserts that all baseline files are present, validates type compatibility, and compiles/tests the project with a clean exit code.

After:

> ### Downstream Conformance Gates
> - Prior to integrating any downstream application implementation, the project MUST be bootstrapped and verified.
> - The downstream project must be initialized using the configured bootstrap script.
> - Baseline conformance must be verified using the configured verification script, which asserts that all baseline files are present, validates type compatibility, and compiles/tests the project with a clean exit code.
>
> ### Downstream Single Source of Truth (SSOT) & Clean Baseline Mandate
> - **Single Source of Truth (SSOT)**: Master architecture blueprints (`DEAP_MASTER_ARCHITECTURE.md`, `THREE_TIER_GOVERNANCE_BLUEPRINT.md`, `DEAP_SYSML_V2_SAFETY_MODEL_SPECIFICATION.sysml`) reside exclusively in the central upstream specification repository (`DEAP-spec-core`). Downstream repositories are strictly forbidden from copying or duplicating master blueprints locally. All downstream projects MUST reference upstream specifications via URLs or relative submodule paths.
> - **Clean Baseline & Version Control Mandate**: Every downstream repository MUST maintain a valid `.gitignore` file in its root directory. OS-generated metadata files (such as `.DS_Store`) and build artifacts are strictly forbidden from being committed or present in the working tree or git index.

### Rationale

Add explicit Downstream Single Source of Truth (SSOT) & Clean Baseline Mandate under `### Downstream Conformance Gates` in `.pipeline/constitution.md`, forbidding master blueprint duplication in downstream projects and requiring a root `.gitignore` file.

Non-destructive: additive subsection, line count increased from 235 to 239 lines, governance rules preserved.

---

## AMEND-0017 — Add Pure Schema-Driven Compiler and Upstream Clean Landing Zone Invariants

- **Date:** 2026-08-23
- **Logged:** 2026-08-23
- **Motivating issue:** Pure Schema-Driven Compiler and Upstream Clean Landing Zone Invariants
- **Approved by:** "PROCEED" — approved user prompt directive to enforce pure schema-driven compilation and clean landing zone invariants.
- **Destructive:** no
- **Line count:** 244
- **Resulting SHA-256:** `c8d785e713c91c638477801af52f61f8726e1a55ac3f8e258b6770bf1b0cdf02`

### Change

Section *Architecture: Three-Tier Platform Isolation*, added subsection `### Core System Boundaries & Invariants`.

Before:

> ### Tier Boundary Guidelines
> 
> 1. **Tier 1: Functional Layer (Abstract Specification)**: Epics, Features, User Stories, Use Cases, and Logical User & Machine Interface (LUMI) specifications...
> 2. **Tier 2: Runtime Configuration Parameters (Dynamic Context)**: Design tokens, dynamic mapping configurations, translation files...
> 3. **Tier 3: Platform Implementation Profiles (Technical Execution)**: `.pipeline/profiles/<platform>.md` and codebase implementations...
> 
> ## Domain Rules

After:

> ### Tier Boundary Guidelines
> 
> 1. **Tier 1: Functional Layer (Abstract Specification)**: Epics, Features, User Stories, Use Cases, and Logical User & Machine Interface (LUMI) specifications...
> 2. **Tier 2: Runtime Configuration Parameters (Dynamic Context)**: Design tokens, dynamic mapping configurations, translation files...
> 3. **Tier 3: Platform Implementation Profiles (Technical Execution)**: `.pipeline/profiles/<platform>.md` and codebase implementations...
> 
> ### Core System Boundaries & Invariants
> 
> - **Pure Schema-Driven Compiler**: The pipeline is an abstract compiler and verification framework. Agents are strictly forbidden from inventing, proposing, or hardcoding domain-specific concepts into pipeline logic or templates.
> - **Upstream Clean Landing Zone Invariant**: Upstream distribution templates (`DEAP-*`) contain only the abstract compiler, linters, and empty landing zones (`schema/`, `docs/epics/`, `docs/features/`, `docs/user-stories/`, `docs/use-cases/` containing only `.gitkeep`). Concrete project schemas and specifications reside exclusively in downstream application workspaces installed via `scripts/install_pipeline.sh`.
> 
> ## Domain Rules

### Rationale

Add formal constitutional mandate for Pure Schema-Driven Compiler and Upstream Clean Landing Zone Invariants under `### Core System Boundaries & Invariants` in `.pipeline/constitution.md`.

Non-destructive: additive subsection, line count increased from 240 to 244 lines, governance rules preserved.

---

## AMEND-0018 — Register Gate 16 (Safety Integrity Quality Gate) and Phase 0 Airworthiness Gate

- **Date:** 2026-08-24
- **Logged:** 2026-08-24
- **Motivating issue:** #38 — Missing Safety Integrity Quality Gate and SORA OSO-01..24 Completeness Verification
- **Approved by:** "PROCEED" — approved implementation plan for Safety Integrity Quality Gate and SORA OSO-01..24 Completeness Verification.
- **Destructive:** no
- **Line count:** 257
- **Resulting SHA-256:** `4779d7c72630a7ff08b6a814e1fd99ff64c1e944bb8d76b47d5977351a81c852`

### Change

Section *Universal Quality Gates*, updated `### Quality Gates & Verification Standards` table count to 16 active quality gates and registered Safety Integrity Quality Gate; added subsection `### Phase 0 Safety Engineering Airworthiness Gate`.

Before:

> ### Quality Gates & Verification Standards
> The pipeline mechanically enforces 15 active quality gates that halt execution on failure. All agents MUST ensure deliverables comply with these gates before declaring completion:
> 
> | Quality Gate | Enforcing Validator Path | Documentation Reference |
> |---|---|---|
> ...
> | Logical UI Validation | `validators/logical_ui_validator.py` | `rules/platform-independence.md` |
> 
> ### Specification Validation Gates

After:

> ### Quality Gates & Verification Standards
> The pipeline mechanically enforces 16 active quality gates that halt execution on failure. All agents MUST ensure deliverables comply with these gates before declaring completion:
> 
> | Quality Gate | Enforcing Validator Path | Documentation Reference |
> |---|---|---|
> ...
> | Logical UI Validation | `validators/logical_ui_validator.py` | `rules/platform-independence.md` |
> | Safety Integrity Quality Gate | `scripts/verify_downstream_baseline.py` | `docs/safety/README.md` |
> 
> ### Phase 0 Safety Engineering Airworthiness Gate
> - **8-Pillar Safety Specification Schema**: All downstream safety engineering deliverables (`docs/safety/STPA_MATRIX.md`) MUST conform to the 8-pillar STPA, FMECA, and SORA schema:
>   1. System Losses ($L-1..N$)
>   2. System Hazards ($H-1..N$)
>   3. Hierarchical Control Structure Topology
>   4. Unsafe Control Actions ($UCA-1..N$) covering all 4 failure modes (Not providing, Providing, Too early/too late/out of order, Stopped too soon/applied too long)
>   5. Loss Scenarios ($LS-1..N$) & Causal Factors
>   6. Formal Safety Constraints ($SC-1..N$)
>   7. FMECA Criticality Matrix with at least 15 component failure mode rows ($15+$ rows) and RPN calculations
>   8. SORA SAIL Risk Mitigations & OSO Traceability Table determining Final GRC, ARC, SAIL classification, and complete coverage of all 24 Operational Safety Objectives (OSO-01 through OSO-24).
> - **ASTM F3269-17 RTA & Model-Based Design Hook Mandate**: Formal safety constraints and Run-Time Assurance (RTA) Safety Net monitors MUST specify direct integration hooks for MATLAB / Simulink / Stateflow / Embedded Coder control law synthesis and Simulink Design Verifier (SLDV) invariant proving.
> 
> ### Specification Validation Gates

### Rationale

Register Gate 16 (Safety Integrity Quality Gate) in the Universal Quality Gates table and add the Phase 0 Safety Engineering Airworthiness Gate subsection to `.pipeline/constitution.md` to formally enforce the 8-pillar schema, all 24 SORA OSOs, 15+ FMECA rows, 4 UCA categories, ASTM F3269-17 RTA architecture, and MATLAB/Simulink hooks.

Non-destructive: additive quality gate registration and airworthiness governance subsection, line count increased from 245 to 257 lines, governance rules preserved.




---

## AMEND-0014 — Domain-Agnostic Tier 1 Semantic Pattern Generalization & Governance Alignment

- **Date:** 2026-08-29
- **Logged:** 2026-08-29
- **Motivating issue:** DEAP-spec-core Upstream Specification Core Compiler Domain Neutrality Alignment
- **Approved by:** "PROCEED" — approved implementation plan for clean domain remediation.
- **Destructive:** no
- **Line count:** 257
- **Resulting SHA-256:** `de61ca04dbf1cbfdbb53709b14722863eb6f3ba7649fade49782d2655e29a010`

### Change

Updated frontmatter `last_updated` date to `2026-08-29` and generalized the 3-layer LUI semantic BDD patterns in Section *Requirements & Specifications Standards* to platform-independent domain models while preserving the mandatory 3-layer semantic chain.

### Rationale

Reinforce the pure schema-driven compiler invariant in Tier 1 functional governance for `DEAP-spec-core` (Upstream Specification Core Compiler), ensuring all examples use generic domain and interface primitives while downstream distribution templates provide domain-specific regulatory and platform bindings.

Non-destructive: semantic chain rules and all 16 quality gates preserved, line count maintained.

### Amendment: Commit Message Non-Closure Invariant (#382)
- Codified strict prohibition on auto-closing trigger keywords (`fix #`, `closes #`, `resolve #`) in git commit messages to guarantee that issues remain in `status:fixed-resolved` for Product Owner validation and are never closed by server-side push automation.
