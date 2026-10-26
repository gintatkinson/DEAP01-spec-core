# ACTIVE RULES BUNDLE -- Consolidated Governance Manifest

> **Notice:** Consolidated governance manifest for DEAP specification core compiler.
> Aggregates the 20 active governance rules from `rules/` into a single source of truth.
> Autonomous agents MUST read this file in a single read to ingest active governance rules.

## Table of Contents

1. [behavioral-trigger-coverage.md](#rule-behavioral-trigger-coverage)
2. [codebase-compliance.md](#rule-codebase-compliance)
3. [conops-mission-intent-integrity.md](#rule-conops-mission-intent-integrity)
4. [constitution-first.md](#rule-constitution-first)
5. [document-references.md](#rule-document-references)
6. [domain-engineering-standards.md](#rule-domain-engineering-standards)
7. [dual-track-mbd-verification.md](#rule-dual-track-mbd-verification)
8. [latex-katex-integrity.md](#rule-latex-katex-integrity)
9. [no-browser-automation.md](#rule-no-browser-automation)
10. [platform-independence.md](#rule-platform-independence)
11. [role-boundary-lock.md](#rule-role-boundary-lock)
12. [serial-execution.md](#rule-serial-execution)
13. [specification-metadata-integrity.md](#rule-specification-metadata-integrity)
14. [subagent-dispatch-standards.md](#rule-subagent-dispatch-standards)
15. [sysml-ssot-completeness.md](#rule-sysml-ssot-completeness)
16. [tdd-mandate.md](#rule-tdd-mandate)
17. [tracker-source-of-truth.md](#rule-tracker-source-of-truth)
18. [uml-model-integrity.md](#rule-uml-model-integrity)
19. [user-authorization-lock.md](#rule-user-authorization-lock)
20. [verification-required.md](#rule-verification-required)

---

<a id="rule-behavioral-trigger-coverage"></a>
## 1. Rule: Behavioral Trigger Coverage (`rules/behavioral-trigger-coverage.md`)

**ALWAYS enforce:** When the workspace schema contains a node named by a behavioral trigger, specifications MUST cover that trigger's documented requirements.

### Hard Constraints
- **Active Trigger Coverage**: For every active trigger node in `rules/behavioral_triggers.json`, at least one specification file of the rule's `target_type` (`user-story` or `use-case`) MUST reference that node by name.
- **Trigger Rule Satisfaction**: Specifications referencing an active trigger node MUST satisfy that trigger's rule: required Mermaid block, match terms, body terms, and secondary body terms.
- **Enforcement**: Mechanically enforced offline by `./target/release/verify-baseline` (Check 21).

---

<a id="rule-codebase-compliance"></a>
## 2. Rule: Codebase Compliance (`rules/codebase-compliance.md`)

**ALWAYS enforce:** All generated and modified code must strictly comply with project architecture, layout, and coding standards.

### Hard Constraints
- **Clean Landing Zones**: Upstream template directories (`schema/`, `docs/epics/`, `docs/features/`, `docs/user-stories/`, `docs/use-cases/`) contain only `.gitkeep` files.
- **Directory Placement**: Code and specifications reside exclusively in designated project paths (`crates/`, `docs/designs/`, `docs/architecture/`). Root file writes are forbidden.
- **Workspace-Relative Paths**: Absolute workstation or user paths are strictly prohibited across all code and documentation (Check 12).
- **Zero Regex Invariant**: All Rust parser implementations must use deterministic string or token scanners; regex is forbidden across all crates (Check 34).

---

<a id="rule-conops-mission-intent-integrity"></a>
## 3. Rule: ConOps & Mission Intent Integrity (`rules/conops-mission-intent-integrity.md`)

**ALWAYS enforce:** Concept of Operations and Tactical Mission Intent units must conform to ISO/IEC/IEEE 29148:2018, INCOSE SE Handbook v5.0, ISO/IEC/IEEE 15288:2023, and OMG UAF v2.0.

### Hard Constraints
- **Pure Schema Contracts**: Units derive deterministically from schema AST without hardcoded assumptions or prose extrapolation.
- **Zero Truncation**: No truncation, omission markers (`[...]`), or partial generation in any ConOps or Mission Intent unit.
- **Deterministic Assembly**: All unit files must assemble deterministically via `./target/release/assemble-conops`.
- **Mandatory Metadata Table**: Every unit file opens with a CommonMark two-column metadata table declaring `Unit ID`, `Type`, `Title`, `Status`, `Traceability`, and `Schema References`.

---

<a id="rule-constitution-first"></a>
## 4. Rule: Constitution First (`rules/constitution-first.md`)

**ALWAYS enforce:** Before beginning any pipeline task, read the project constitution and operating rules.

### Hard Constraints
- **Direct-Path Reads**: Read root `AGENTS.md` (or optional alias `.agents/AGENTS.md`) and `.pipeline/constitution.md` directly by path before taking action. Both `.pipeline` and `.agents` are hidden directories that generic glob tools may exclude.
- **Scoped Profile Loading**: Platform profiles (`.pipeline/profiles/<platform>.md`) are read ONLY when implementing downstream features for the configured target platform. Spec workers and `UPSTREAM_SPEC_CORE_COMPILER` sessions MUST NOT read implementation profiles.
- **Curated Subagent Bypass**: Spawned single-item subagents receiving prompts ending in `---GOVERNANCE-END---` execute their assigned `SKILL.md` directly without repeating coordinator startup ceremonies.
- **Strict Planning Gate Overrides**: Root `AGENTS.md` states the strictest planning rule: an approved plan is required before modifying any files; authorization keywords alone are insufficient.
- **Turn-0 Canonical Initialization Receipt**: Coordinator MUST execute Phase A on startup and emit the canonical session initialization receipt table before any output or file modifications.

---

<a id="rule-document-references"></a>
## 5. Rule: Document References (`rules/document-references.md`)

**ALWAYS enforce:** All cross-document references, links, and citations must resolve to valid on-disk targets or authoritative external standards URLs.

### Hard Constraints
- **Verbatim Upstream Locators**: Schema and normative specification URLs in `Source References` must appear unchanged without rewriting to local repository paths.
- **Valid Relative Links**: Intra-repository markdown links must point to existing files and valid line or anchor targets.
- **Checklist Integrity**: Backlog checklist items must reference valid, registered tracker issue numbers or canonical specification titles.

---

<a id="rule-domain-engineering-standards"></a>
## 6. Rule: Domain Engineering Standards (`rules/domain-engineering-standards.md`)

**ALWAYS enforce:** The DEAP pipeline is an abstract MBSE compiler and verification framework. Hardcoding domain concepts into core compiler logic or templates is strictly prohibited.

### Hard Constraints
- **Abstract Metamodel Purity**: Pipeline logic operates purely on abstract M2 metamodel constructs (`PartDef`, `PortDef`, `ActionDef`, `StateDef`, `ConstraintDef`, `RequirementDef`).
- **Deterministic Derivation**: All downstream specification items derive deterministically from user-provided AST schemas in `schema/`.
- **Domain Template Boundary**: Domain-specific models belong in downstream workspaces, never in upstream distribution templates.

---

<a id="rule-dual-track-mbd-verification"></a>
## 7. Rule: Dual-Track MBD Verification (`rules/dual-track-mbd-verification.md`)

**ALWAYS enforce:** Maintain rigorous dual-track verification across Model-Based Design (MBD) and software implementation tiers.

### Hard Constraints
- **Primary Commercial Toolchain**: MATLAB / Simulink / Stateflow / Embedded Coder is declared as the Primary Tier-1 Commercial Toolchain Integration Context.
- **Traceability Hooks**: Safety constraints, invariants, and Stateflow transition triggers must maintain formal requirements verification bindings (`verify requirement`).
- **DO-178C / ISO 26262 Alignment**: All safety-critical control laws and software units must satisfy designated assurance levels (DO-178C Level A, ISO 26262 ASIL D).

---

<a id="rule-latex-katex-integrity"></a>
## 8. Rule: LaTeX & KaTeX Mathematical Integrity (`rules/latex-katex-integrity.md`)

**ALWAYS enforce:** Mathematical formulations and formal constraints must render cleanly in KaTeX without syntax or parser errors.

### Hard Constraints
- **Isolated Display Math**: Display math delimiters `$$` must reside on their own separate, isolated lines.
- **Aligned Environments**: Multi-line aligned formulas inside `$$` MUST use `\begin{aligned} ... \end{aligned}`. Top-level `\begin{align}` is strictly forbidden.
- **No Non-Math Delimiters**: Non-mathematical identifiers, requirement IDs, hazard IDs, or text tokens MUST NOT use `$...$` delimiters; use bold or code spans.
- **Escaped Special Characters**: Underscores in text blocks within math environments must be escaped (`\_`).

---

<a id="rule-no-browser-automation"></a>
## 9. Rule: No Browser Automation (`rules/no-browser-automation.md`)

**ALWAYS enforce:** Never use headless browser automation when lightweight CLI tools or HTTP requests can perform the task.

### Hard Constraints
- **Prohibited Tools**: Puppeteer, Selenium, Playwright, and Chrome DevTools Protocol are strictly prohibited.
- **Approved Alternatives**: Use `read_url_content` for web pages, `gh` for GitHub, `glab` for GitLab, or native CLI utilities.

---

<a id="rule-platform-independence"></a>
## 10. Rule: Platform Independence & Mermaid Syntax (`rules/platform-independence.md`)

**ALWAYS enforce:** Specifications must remain platform-independent, and all embedded Mermaid diagrams must comply strictly with syntax rules E1-E5.

### Hard Constraints (Mermaid Rules E1-E5)
- **E1**: Header must be declared on the very first non-comment line inside ```` ```mermaid ````.
- **E2**: Top-level flowchart diagrams MUST use `flowchart TD`. Subgraphs MUST declare `direction TB`.
- **E3**: Maximum of 3 columns in flowchart topology.
- **E4**: Node labels must not exceed 35 characters per line; break longer labels with `<br/>`.
- **E5**: Labels containing special characters (`/`, `:`, `()`, `[]`, `<>`) must be enclosed in double quotes. Unquoted `<` and `>` characters are strictly forbidden across ALL diagram types. Curly braces `{}` and secondary colons are prohibited in class members.

---

<a id="rule-role-boundary-lock"></a>
## 11. Rule: Role Boundary Lock & Phase Enforcement (`rules/role-boundary-lock.md`)

**ALWAYS enforce:** Enforce strict role boundaries between specification, research, and implementation phases. Coordinators must never directly write codebase files or specifications, and subagents must only write within their designated domain boundaries.

### Hard Constraints
- **Coordinator Direct Writing & Research Lock**: The coordinator agent is strictly forbidden from directly writing or modifying target functional specifications or codebase source files, and is locked from directly conducting technology stack research. All codebase write and update operations must be delegated to spawned implementer subagents, and failure mode research delegated to context-isolated subagents per component.
- **Specification Phase Boundary**: Specification workers, spec subagents, and `UPSTREAM_SPEC_CORE_COMPILER` workers are strictly forbidden from reading, writing, or referencing implementation profiles, implementation plans, or target source code files.
- **Implementation Phase Boundary**: Implementation workers must not modify upstream specifications without synchronized backlog reconciliation.
- **Strict Subagent Tool Locking**: Spawned subagents must only execute tools that fall within their explicit domain.
- **Subagent Cleanup**: Every spawned subagent must be terminated or confirmed reclaimed immediately upon task completion.
- **Scope of Coordinator Writing Lock**: The delegation duty binds for all repository source, governance, tooling, rule, test, and documentation writes. The coordinator direct file-writing lock applies strictly to `walkthrough.md` (which is exclusively a post-implementation verification artifact) and shell file mutations (`cp`, `mv`, `tee`, or redirection), which cannot be used to bypass subagent delegation.
- **Mandatory Full Compilation Build**: Full application compilation build must be executed before completing any implementation phase.

---

<a id="rule-serial-execution"></a>
## 12. Rule: Serial Execution (`rules/serial-execution.md`)

**ALWAYS enforce:** Micro-task implementations must proceed serially with verified review gates between steps.

### Hard Constraints
- **No Concurrent File Edits**: Never execute parallel writes or overlapping edits to the same files.
- **Step-by-Step Verification**: Each micro-task must complete its test and lint gates before the next micro-task begins.

---

<a id="rule-specification-metadata-integrity"></a>
## 13. Rule: Specification Metadata Integrity (`rules/specification-metadata-integrity.md`)

**ALWAYS enforce:** Every specification document MUST begin with a standardized CommonMark two-column Metadata Table.

### Hard Constraints
- **Header Placement**: Lines 1-10 MUST contain a CommonMark two-column Metadata Table (`| Attribute | Specification Detail |`).
- **Mandatory Attributes**: The table must declare `Issue ID`, `Title`, `Type`, `Parent Epic`, `Status`, `Generation Mode: subagent`, and document-relative `Specification Source`.
- **Zero Raw YAML**: In downstream distribution workspaces, raw YAML frontmatter is superseded by the CommonMark table format.

---

<a id="rule-subagent-dispatch-standards"></a>
## 14. Rule: Subagent Dispatch Standards (`rules/subagent-dispatch-standards.md`)

**ALWAYS enforce:** Subagent dispatches must adhere strictly to canonical prompt standards, single-item scoping, target-file read locks, and pre-flight self-rejection.

### Hard Constraints
- **Step 1 Skill Directive & Surgical Edit Fast-Path**: Dispatch prompt MUST direct `view_file` on `SKILL.md` by explicit path as its first step for pipeline skill workflows. For surgical edits where exact `Target`/`Replacement` blocks are already approved in `implementation_plan.md`, reading any `SKILL.md` is prohibited fake work; the subagent must apply the exact edits and exit immediately.
- **Strict Target-File Read & Verification Boundary**: Subagents are strictly forbidden from reading, searching, or inspecting any file outside the explicit target file list in their prompt, and must never redundantly run `verify-baseline` inside single-item edit dispatches.
- **Repository Classification**: Prompt MUST state the verbatim repository classification (e.g. `UPSTREAM_SPEC_CORE_COMPILER`).
- **Zero Leading Steering**: Coordinators are forbidden from injecting pseudo-code or pre-digested code before the skill read.
- **Single-Item Scope**: Every dispatch must target at most 1 item (max 1 Epic, Feature, Story, Use Case, or bounded micro-task).
- **Curated Execution Bypass**: Subagents receiving prompts ending in `---GOVERNANCE-END---` execute their assigned skill directly without repeating coordinator Turn-1 startup ceremonies.
- **Trailing Authorization**: Coordinator must append `PROCEED` to authorize execution in subagent context.

---

<a id="rule-sysml-ssot-completeness"></a>
## 15. Rule: SysML v2 Model-as-SSOT Completeness (`rules/sysml-ssot-completeness.md`)

**ALWAYS enforce:** The SysML v2 textual model is the authoritative Single Source of Truth for all structural and behavioral derivations.

### Hard Constraints
- **Pure AST Derivations**: All Epics, Features, Stories, and Use Cases derive deterministically from SysML v2 AST nodes (`part def`, `port def`, `item def`, `action def`, `state def`, `use case def`).
- **Bidirectional Parity**: Synchronize model and backlog changes via `./target/release/compile-sysml` and `./target/release/reconcile-backlog`.
- **Zero Phantom Artifacts**: Specifications without backing AST nodes in `.pipeline/schema.sysml` or `schema/` are strictly prohibited.

---

<a id="rule-tdd-mandate"></a>
## 16. Rule: Test-Driven Development Mandate (`rules/tdd-mandate.md`)

**ALWAYS enforce:** All software modifications and bug fixes MUST follow the strict RED-GREEN-REFACTOR TDD cycle.

### Hard Constraints
- **RED Demonstration First**: Every fix or feature must ship a failing unit or mutation test demonstrated RED before implementation code is written.
- **Native Rust Tests**: All tests under `crates/` must be native Rust `#[test]` functions. Legacy Python/pytest is strictly prohibited under Check 33; shell scripts are prohibited under Check 32.
- **Workspace Test Gate**: `cargo test --workspace` MUST pass with zero failures before declaring completion.

---

<a id="rule-tracker-source-of-truth"></a>
## 17. Rule: Tracker Source of Truth (`rules/tracker-source-of-truth.md`)

**ALWAYS enforce:** The repository issue tracker is the single source of truth for work items, task states, and delivery status.

### Hard Constraints
- **Normalized Title Uniqueness**: All specification items of the same type must have unique normalized titles to prevent collision during reconciliation.
- **Non-Closure Commit Invariant**: Auto-closing keywords (`fix #`, `closes #`, `resolve #`) are strictly forbidden in commit messages; use neutral citations `(refs #<id>)` or `(#<id>)` exclusively.
- **Agent Lifecycle Limit**: Agents may transition issues to `status:fixed-resolved` with empirical evidence comments; closing issues requires Product Owner validation and is forbidden to agents.

---

<a id="rule-uml-model-integrity"></a>
## 18. Rule: UML Model Integrity (`rules/uml-model-integrity.md`)

**ALWAYS enforce:** UML diagrams embedded in specifications must be structurally sound and valid Mermaid representations.

### Hard Constraints
- **Sequence Diagram Mappings**: Every message in a sequence diagram must map to an active operation or signal on the classifier interface.
- **State Machine Transitions**: Every trigger, event, or guard on a state machine transition must correspond to declared operations or signals.
- **Matching Fences**: Every diagram block must have valid opening and closing fences on their own lines.

---

<a id="rule-user-authorization-lock"></a>
## 19. Rule: User Authorization Lock & Planning Gate (`rules/user-authorization-lock.md`)

**ALWAYS enforce:** The agent must run the 5-Point Thought-Block Compliance Check in every thought block and lock all modifying tools until authorized by the keyword `PROCEED`.

### Hard Constraints
- **5-Point Compliance Check**: Every agent thought block MUST begin with the standardized 5-point check:
  1. `Turn-0 Initialization Gate`: Verify Phase A execution (detecting `.pipeline/upstream/`, loading `.pipeline/constitution.md` and `.pipeline/ACTIVE_RULES_BUNDLE.md`, running `git status --short --branch`) and emission of the Canonical Session Initialization Receipt Table, or prompt ends with `---GOVERNANCE-END---`.
  2. `Intent Classification`: Classify message as inquiry vs direct command.
  3. `User Authorization Lock`: Verify exact target files are documented in an approved `implementation_plan.md` AND user message explicitly contains `PROCEED` (or `Approved` / `Approve plan`).
  4. `Zero Silent Assumptions`: Confirm no silent assumptions about intent or workspace state.
  5. `Coordinator Delegation & Shell Mutation Lock`: If active skill mandates subagent dispatch or turn mutates repository files other than `implementation_plan.md`, direct coordinator writing and shell mutations (`cp`, `mv`, `rm`, `tee`, output redirects) are strictly locked; writes must be delegated to subagents after `PROCEED`.
- **Turn-0 Read-Only Whitelist**: Read-only commands (`git status`, `git status --short --branch`, `git branch --show-current`, `git rev-parse --show-toplevel`, `git diff`, `ls`, `test -d`, `test -e`) are authorized without prior `PROCEED`.
- **State-Mutating Shell Lock**: Mutating commands (`cp`, `mv`, `rm`, `tee`, `sed -i`, `>`, `>>`) and build/test commands are strictly locked until plan approval with `PROCEED`.
- **Prohibition of Pre-Approval `walkthrough.md` Edits**: `walkthrough.md` is exclusively a post-implementation verification artifact and must never be created or edited prior to plan approval and subagent completion.
- **Mandatory Empirical Physical Path Verification**: Before marking any issue resolved, verify target directories via `git status` and `git rev-parse --show-toplevel`.
- **Plan Continuity**: Established plans cannot be abandoned, altered, or replaced without explicit user authorization.
- **Single-Task Atomic `implementation_plan.md` & Anti-Snowballing Invariant**: `implementation_plan.md` MUST contain **at most 1 atomic work package** at any time. Dumping multi-wave issue backlogs into `implementation_plan.md` or appending/prepending a new user task onto a previous `implementation_plan.md` is strictly forbidden; backlog queues reside exclusively in the issue tracker (`gh issue list`). `Prohibition of Unapproved Plan Abandonment & Mandatory Plan Continuity` applies strictly **within the currently active in-flight atomic task** (forbidding mid-task abandonment when hitting test/linter friction), and MUST NEVER be used as an excuse to retain completed or unrelated work packages inside `implementation_plan.md` when planning a new atomic task.
- **Downstream Binary-Only Distribution & Source Firewall Lock**: Upstream Tier-0 compiler source code (`crates/{deap-core,ingest-sysml,compile-sysml,verify-baseline,reconcile-backlog,assemble-conops}`, `Cargo.toml`, `Cargo.lock`), `.pipeline/upstream/`, and `docs/architecture/blueprints/` reside **exclusively** in `DEAP01-spec-core` (`UPSTREAM_SPEC_CORE_COMPILER`). Agents (and subagents) are strictly forbidden from proposing, planning (`implementation_plan.md`), or executing any copy, sync, or commit of Tier-0 compiler `crates/*` source code into any downstream repository (`DOWNSTREAM_CUSTOMER_PROJECT`), and strictly forbidden from running `cargo build` to recompile already-verified `target/release/*` binaries during downstream distribution. Downstream distribution is restricted exclusively to: (1) pre-built `target/release/{compile-sysml,verify-baseline,ingest-sysml,reconcile-backlog,assemble-conops}` binary images, (2) `AGENTS.md` / `.agents/AGENTS.md`, (3) `.pipeline/constitution.md` / `.pipeline/ACTIVE_RULES_BUNDLE.md`, (4) `rules/*.md`, and (5) `skills/**`. Any subagent prompt requesting to copy `crates/*` to a downstream repository MUST immediately self-reject and halt.

---

<a id="rule-verification-required"></a>
## 20. Rule: Verification Required (`rules/verification-required.md`)

**Enforce on code/model modifications only (never on startup or read-only turns):** Never assert state or completion of code/model modifications without direct, empirical verification.

### Hard Constraints
- **Pre-Commit Quality Gates (Code/Model Edits Only)**: Only when tracked source or model files were modified in the current turn, run `cargo test --workspace` and `./target/release/verify-baseline . --no-domain` once before committing (never on session initialization or read-only turns).
- **Multi-Provider Remote Sync Verification (GitHub & GitLab)**: Commit all verified changes and push all commits to the configured remote tracking branch (`origin/<branch>` on GitHub or GitLab), verifying `git diff origin/<branch>` is empty.
- **Forensic Verification Evidence**: Document raw terminal outputs, test results, commit hashes, and issue transitions in handoff reports and issue comments.
