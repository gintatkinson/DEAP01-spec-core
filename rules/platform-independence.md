<!-- Copyright Gint Atkinson, gint.atkinson@gmail.com -->

# Rule: Platform Independence in Specifications

**ALWAYS enforce:** Epics, Features, User Stories, and Use Cases must be purely functional and platform-independent.

## Hard constraints

- Specification documents MUST describe *what* the system does, never *how* it is built.
- Feature specs MUST NOT contain framework-specific component names (e.g., no `<Drawer>`, no `showModalBottomSheet`).
- Feature specs MUST NOT contain a `platform` field in their YAML frontmatter.
- Acceptance criteria MUST be platform-independent (e.g., "the detail view displays the address" — not "the React Drawer component renders the address").
- The `Interface Requirements` section describes data, payloads, layout, or protocols logically, without referencing specific frameworks or transport libraries.
- Every Mermaid diagram or code block MUST be strictly and explicitly closed using matching closing fences (e.g. ```` ``` ```` on a new line) to prevent layout/parser leakage.
- **Mermaid Class Diagram Syntax Rules**: Colons are strictly prohibited inside Mermaid class member strings (e.g., do not use `+methodName() : ReturnType` or `+methodName(arg : Type)`), as secondary colons confuse the parser and break rendering. Use standard spacing instead (e.g., `+ReturnType methodName(Type arg)`).
- **Mermaid Class Naming Rules**: Double quotes MUST NOT be used in class names. Colons are explicitly forbidden in unbackticked class names; any class names containing colons must be enclosed in backticks (e.g., `` class `Nw:network` ``). Use backticks (e.g., `class `My Class` {`) or label brackets if names contain special characters or spaces.
- **Mermaid Note Rules**: Colons are strictly prohibited inside Mermaid class diagram note strings (e.g., do not use `note "Status: Active"` or `note for ClassA : "Status: Active"`), as colons in notes confuse the parser and break rendering.
- **Mermaid Relationship Rules**: Double angle brackets, stereotypes, or HTML entities representing stereotypes (e.g., `<<`, `>>`, `&lt;&lt;`, `&gt;&gt;`, `«`, `»`) are strictly prohibited on class diagram relationship lines. Relationship labels must be plain strings without stereotypes (e.g., use `-->` with a label like `references` instead of `<<references>>`).
- **Mermaid Relationship Label Rules**: Relationship labels containing spaces MUST be enclosed in double quotes (e.g., use `RootContainer *-- NestedContainer : "contains a nested container"`, not `RootContainer *-- NestedContainer : contains a nested container`). An unquoted multi-word label is a parse error. A single unquoted word is permitted (e.g., `A --> B : references`).
- **Mermaid Relationship Label Colon Rules**: Colons are strictly prohibited inside relationship labels, **quoted or not**. This rule previously grouped colons with spaces as characters that quoting makes safe. That is true of spaces and false of colons: Mermaid parses `:` as a statement separator wherever it occurs, so a label such as `"augments nw:node"` ends the statement mid-label and GitHub fails with `Parse error: Expecting 'NEWLINE', 'EOF', got 'LABEL'`. The offline gate passed it because the label satisfied the quoting rule, so nothing caught it until a renderer refused the diagram (issue #333). Drop the namespace prefix or replace the colon — `: "augments nw node"`.
- **Mermaid Semicolon Rules**: Semicolons (`;`) are strictly prohibited inside Mermaid `Note` statements and inside message text (e.g., do not write `Note over Val: n = len(x); error when n > 1` or `A->>B: do a thing; then another`). Mermaid parses `;` as a statement separator, so the remainder of the line becomes a new statement and collides with whatever follows, breaking rendering. Replace semicolons with commas, dashes, or spaces.
- **Mermaid Empty Class Body Rules**: An empty class body MUST NOT be written on a single line — do not write `class ParentContainer {}`. Put the opening brace at the end of the declaration line and the closing brace on its own line, or omit the braces entirely (`class ParentContainer`). Attribute-less classes themselves remain legal and are in places mandatory: an ancestor container node on a schema containment path carries the containment relationship and often has no attributes of its own, and the canonical Feature template ships exactly that. The prohibition is on the spelling, not on the empty class. Reason: a class block is opened by `class X {` and closed only by a line consisting of `}`, so a same-line closing brace never closes the block; a later `}` — the one closing a `namespace`, for instance — pops the leaked class block instead, and every following class is silently attached to the wrong namespace with no parse error raised. Note that this is a distinct rule from the prohibition on *isolated* classes (a class with no relationships), which is enforced by the UML validator.
- **Mermaid Unquoted Bracket Rules**: Unquoted `<` and `>` characters are strictly prohibited in **every** diagram type — `graph`, `flowchart`, `sequenceDiagram` and `stateDiagram-v2` alike. A transition, label or guard containing a comparison operator or bracket MUST enclose the whole label in double quotes (e.g. `ActiveCounting --> ActiveCounting: "incrementCounter [value < maxBound] / updateValue"`). Unquoted, Mermaid reads `<` as the start of markup and the diagram fails to render. This rule was enforced by `mermaid_syntax_validator.py` and stated only in `.agents/AGENTS.md`, not in this file — its normative home — so an author working from the Mermaid rules alone could breach it (the #289 fragmentation shape).
- **Mermaid Node Label Quoting Rules**: In `graph` and `flowchart` diagrams, a node label containing a slash, colon, parenthesis or bracket MUST be enclosed in double quotes (e.g. `Node["Save/Restore (Local DB)"]`). Those characters are node-shape and edge syntax to the parser, so an unquoted label is read as structure rather than text. Same provenance as the rule above: enforced, and until now documented outside its normative home.
- **Mermaid Subgraph Title Quoting Rules**: A `subgraph` title containing spaces or hyphens MUST be enclosed in double quotes (e.g. `subgraph "System Boundary"`). Unquoted, Mermaid reads only the first word as the title and the remainder as syntax it cannot parse.
- **Mermaid Quote Balance Rules**: Double quotes inside a diagram line MUST be balanced. An unclosed quote swallows the remainder of the line — and often the following lines — into a single string literal, so the diagram either fails to parse or renders with silently missing elements. This is the failure mode the three quoting rules above create when applied halfway.
- **Mermaid Class Member Brace Rules**: Curly braces (`{` `}`) are strictly prohibited inside Mermaid class member lines (e.g., do not write `+Decimal64 dim_0 {range = "-90.0..90.0"}`), as they crash GitHub and Mermaid CLI renderers. Use parentheses or simple brackets instead, e.g., `(default earth)` or `[default earth]`.
- **Mermaid Sequence Diagram Participant Alias Rules**: Mermaid reserved keywords (`link`, `links`, `actor`, `participant`, `loop`, `opt`, `alt`, `rect`, `note`, `end`, `par`, `and`, `critical`, `option`, `break`, `activate`, `deactivate`, `autonumber`, `box`, `create`, `destroy`) MUST NOT be used as participant aliases or IDs in sequence diagrams (e.g. `participant link as Link Service` or `actor link as Link Interface`). The parser interprets reserved keywords as structural sequence grammar, breaking diagram rendering.

## Document integrity constraints

These are the non-Mermaid constraints on the same corpus, enforced offline by
`parity_auditor/validators/docs.py`. They are stated here because each one exists to keep
a Tier 1 document functional and standard-agnostic, or to keep it parseable by the tools
that read it — the same subject as the rules above. The three fence rules are deliberately
**distinct** from `mermaid-fence-must-be-closed` in the Mermaid syntax checker: that rule
governs a `mermaid` fence inside a diagram under audit, these govern the enclosing
Markdown document, and conflating them would make a grouped multi-workspace report unable
to say which checker fired.

- **Obsolete Token Namespace Rules**: documentation and specification files MUST NOT
  reference retired design-token namespaces (`color.alarm`, `alarm.cleared`,
  `alarm.minor`, `alarm.critical`). Use the standard-agnostic status mappings instead. A
  retired namespace resolves to nothing, so the document describes styling that cannot be
  applied.
- **Hardcoded Standard Reference Rules**: no README, implementation profile or backlog
  specification may name a standard listed in `spec_rules.forbidden_standards_blocklist`.
  Naming the standard binds the specification to it, which is the contamination this rule
  file exists to prevent, one layer up from framework names.
- **Backlog Standard And Platform Leak Rules**: backlog specifications additionally MUST
  NOT contain a Code Realization Table, an `Error Handling & Codes` or
  `Protocol & Endpoint Definitions` header, a literal HTTP status code, or a reference to
  an implementation source file. Each belongs to Tier 3 and states *how* rather than
  *what*. Code Realization Tables belong in walkthroughs; the others have no Tier 1 home
  at all.
- **Markdown Construct Leak Rules**: a Markdown heading, blockquote, list bullet, link,
  inline code span, image or table row appearing *inside* an open `mermaid` fence means
  the fence leaked — the diagram was never closed and the prose after it is being parsed
  as diagram source. Reported at the line the block opened on, because that is where the
  missing fence belongs.
- **Code Fence Closure Rules**: every fenced block in a specification MUST be closed,
  whatever its language. This is broader than the Mermaid rule above: an unclosed
  `json` or `bash` fence swallows the remainder of the document just as effectively, and
  the reader sees a truncated specification rather than an error.
- **Diagram Fence Parity Rules**: the count of `mermaid` fence markers in a document MUST
  be even. Parity catches the case the leak scan above cannot see — a `mermaid` block
  closed by nothing at all, with no stray Markdown construct following it to betray the
  omission, typically at the end of a file.

## Normative home & enforcement

**This file is the single normative home for Mermaid syntax constraints.** Skills that emit Mermaid MUST reference this section rather than restating their own subset. These rules were previously fragmented across four files with disjoint subsets, so an author working from one file could breach a constraint documented in another — see issue #289.

These rules are mechanically enforced, offline, by `parity_auditor/validators/mermaid_syntax_validator.py`. That checker is a rule checker, not a full Mermaid grammar parser: a clean result means no documented rule was violated, which is not proof that a diagram renders. Blocking gates must not call remote renderers — see `.pipeline/upstream/pipeline-tooling.md` § *Validation Gates*.



## Where platform-specific details belong

- Implementation profiles: `.pipeline/profiles/<platform>.md`
- Implementation plans: `implementation_plan.md` created during The Grill (Step 2 of feature-driven-implementation)
- Solution walkthroughs: paths defined by design and implementation guidelines (e.g. `<walkthrough_dir>/feat-<N>-solution.md` or as configured)

## Why

A single set of functional specs can drive implementations on React, Flutter, .NET, or any other platform. Contaminating specs with platform details forces re-specification when targeting a new platform and violates the two-tier constitution architecture.
