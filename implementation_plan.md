# Implementation Plan -- Deep Ralph Semantic Convergence & Quality Gate Arming

**Objective**: Completely eliminate specification hollow stubs, redundant stuttering naming, canned diagrams, and cross-tier disconnects. Restore full SysML v2 AST behavioral fidelity (matching `3dgs-039` standard), arm `crates/verify-baseline` with non-bypassable semantic quality gates, regenerate all specifications from Ground 0, and project clean verified outputs to GitHub.

---

## 1. Problem Statement & Root Cause Analysis

### A. The "Bad Code" in the Compiler Toolchain (`crates/compile-sysml`)
1. **Grammar Parser Silently Discards Inner Doc Blocks** (`crates/compile-sysml/src/parser/grammar.rs:1101-1126`):
   - In `parse_use_case_decl`, the parser checks for `Subject`, `Actor`, `Objective`, `Step`.
   - When `doc /* ... */` is encountered inside `use case def { ... }`, it falls through to `else { self.skip_until_semi_or_brace(); }` and is **silently discarded**.
   - Result: All 12 rich Use Case specifications in `schema/subsystems/*/behavior.sysml` (containing multi-actor roles, triggers, preconditions, 6-step scenarios, and `5a`..`5j` exception flows) are completely lost during parsing.
2. **Canned Fallback Generator in `forward.rs`** (`crates/compile-sysml/src/sync/forward.rs:1151-1182`, `1243-1246`):
   - When `parsed_doc` is empty due to the parser bug, `forward.rs` falls back to canned 4-line boilerplate (`"Actor HumanEngineer initiates workflow..."`) and a dummy 2-node flowchart (`Actor["HumanEngineer"] --> Target["..."]`).
   - Violates `rules/uml-model-integrity.md` (missing system boundary subgraph, directed arrows instead of undirected associations, rectangle nodes instead of stadium shape `(["..."])`).
3. **Hardcoded Canned Epics** (`crates/compile-sysml/src/sync/forward.rs:835-868`):
   - Class diagrams hardcoded to only `DEAPCompilerSystem --> {primary_engine} : contains` (omitting all other constituent parts, ports, and attributes).
   - State machine diagrams hardcoded to identical `Bootstrapping -> Ingesting -> Compiling -> Verifying` across all 14 Epics, completely ignoring real subsystem FSMs in `behavior.sysml` (e.g. `StateSolverExecutionStatechart`).
4. **Hardcoded User Story Parent Epic & Canned Lifelines** (`crates/compile-sysml/src/sync/forward.rs:1061`):
   - All 12 User Stories hardcode `| **Parent Epic** | [EPIC-02-ConOps] |`, disconnecting stories from their actual subsystem Epics.
   - Ignores `interaction def` in `behavior.sysml` and generates fake lifelines `HumanEngineer -> DEAPCompilerSystem -> Engine`.
5. **Redundant Stuttering Titles & Filenames** (`crates/compile-sysml/src/sync/forward.rs:1107-1110`):
   - `format!("Use Case {:02}: {}", u_num, to_display_name(&uc_m.uc.name))` turns `UC_01_...` into `Use Case 01: UC 01 System Vision and Foundational Invariants`.
   - `format!("use-cases/UC-{:02}-{}.md", u_num, uc_slug)` turns into `UC-01-UC_01_...md`.
6. **Hardcoded Source References** (`crates/compile-sysml/src/sync/forward.rs:1237`):
   - Hardcoded `[schema/conops/activities.sysml]` for all use cases, even when defined in `schema/subsystems/<subsystem>/behavior.sysml`.

### B. Why `verify-baseline` Passed ("Checkbox Compliance")
- `crates/verify-baseline/src/spec_audit/sections.rs:365-374` only checked whether heading text tokens existed in the document.
- It never checked:
  - Title/filename stuttering (e.g., `Use Case 01: UC 01`).
  - Content word count or presence of canned fallback strings (`"initiates workflow"`, `"validates the operation"`).
  - Minimum scenario steps ($\ge 5$) or alternate flows ($\ge 3$).
  - Diagram topology: Use case flowcharts were never parsed or validated in `uml_validator.rs` (it only checked sequence and class diagrams).
  - Epic statechart variety: Never checked if all Epics have the identical dummy statechart.
  - Parent Epic parity: Never verified that a User Story's `Parent Epic` matches its subsystem package.

### C. Where the SysML and UML Semantic Requirements Reside
- `.pipeline/constitution.md`:
  - **Universal Model Consistency Rules** (lines 77-83): Dynamic-to-Static Alignment; every lifeline must resolve to a defined classifier; every message must map to an active Operation; every transition trigger must be defined in the metamodel.
  - **Quality Gates** (lines 181-204): Gate 1 (UML Model Integrity), Gate 21 (Semantic Diagram-to-AST Topology Parity), Gate 23 (Factual Grounding), Gate 30 (Architecture Viewpoint & Diagram Completeness).
- `rules/sysml-ssot-completeness.md` (lines 1185-1250 of `.pipeline/ACTIVE_RULES_BUNDLE.md`):
  - SysML v2 AST is 100% SSOT; formal mappings of `part def`, `item def`, `action def`, `state def`, `use case def`.
  - Zero heuristic prose parsing; pure token AST lowering.
- `rules/uml-model-integrity.md` (lines 1501-1680 of `.pipeline/ACTIVE_RULES_BUNDLE.md`):
  - Use Cases Must Declare A System Boundary Subgraph.
  - Actors Must Sit Outside The System Boundary.
  - Use Case Nodes Must Sit Inside The System Boundary using Stadium Shape `(["..."])`.
  - Actor Associations Must Be Undirected (`---`).
  - Use Cases Must Carry Detailed Alternate and Exception Flows.
  - Statecharts Must Form Deterministic Automata with Triggers and Bracketed Guards.

---

## 2. Work Packages Decomposition

### Work Package 1: Compiler Parser Fix (`crates/compile-sysml/src/parser/grammar.rs`)
- In `parse_use_case_decl`, handle `TokenKind::Doc` and `TokenKind::DocComment` inside `use case def { ... }` braces.
- Store inner doc comments on `UseCaseDef.doc` so that structured sections (`Actors:`, `Subject:`, `Preconditions:`, `Trigger:`, `Main Success Scenario:`, `Alternate and Exception Flows:`, `Postconditions:`) are retained in the AST.

### Work Package 2: Generator Overhaul for Full Model Fidelity (`crates/compile-sysml/src/sync/forward.rs`)
1. **Use Case Clean Naming & Paths**:
   - Implement `clean_use_case_name(name: &str) -> &str` stripping leading `UC_XX_`, `UC-XX-`, or `UCXX_`.
   - Emit clean titles (`Use Case {:02}: {}`) and clean filenames (`use-cases/UC-{:02}-{}.md`).
   - Update Epic checklists in `forward.rs:748-756` to link to clean titles and filenames.
   - Update `resolve_issue_id` to resolve IDs across renames.
2. **Use Case UML Diagram Compliance**:
   - Replace the dummy 2-node flowchart with a valid UML Use Case diagram conforming to `rules/uml-model-integrity.md`:
     - System boundary subgraph enclosing use case nodes in stadium shape `(["..."])`.
     - Primary and secondary actors outside the boundary using `((Actor))` or `["Actor"]`.
     - Undirected associations (`---`) between actors and use cases.
     - Include/extend dependency connectors where applicable.
3. **Dynamic Source References & Full Doc Lowering**:
   - Bind `Specification Source` to `[schema/subsystems/{subsys_dir}/behavior.sysml](../../schema/subsystems/{subsys_dir}/behavior.sysml)`.
   - Lower all 6+ numbered main scenario steps and `5a`..`5j` exception flows directly from the parsed doc block.
4. **Epic Model Parity**:
   - Replace canned state machine with genuine subsystem `state def` from `behavior.sysml` (e.g. `StateSolverExecutionStatechart`).
   - Expand class diagrams to include all constituent `part def`, `port def`, and attributes from `architecture.sysml`.
5. **User Story Model Parity**:
   - Lower genuine `interaction def` lifelines and messages from `behavior.sysml` into sequence diagrams.
   - Bind `Parent Epic` to the declaring subsystem Epic instead of hardcoded `[EPIC-02-ConOps]`.

### Work Package 3: Arming `verify-baseline` with Semantic Quality Gates (`crates/verify-baseline`)
1. **Anti-Stutter Gate**:
   - In `spec_audit/metadata.rs`, fail if title or filename contains stuttering prefixes (e.g. `Use Case 01: UC 01` or `UC-01-UC_01_`).
2. **Anti-Boilerplate / Minimum Scenario Gate**:
   - In `spec_audit/sections.rs` / `uml_validator.rs`, scan Use Case content:
     - Reject documents containing canned fallback strings (`"initiates workflow execution"`, `"processes structural elements and validates semantic constraints"`).
     - Require Main Success Scenario to have $\ge 5$ numbered steps.
     - Require Alternate and Exception Flows to have $\ge 3$ concrete exception cases.
3. **Use Case Diagram Compliance Gate**:
   - In `spec_audit/diagram_ast.rs` and `uml_validator.rs`:
     - Reject Use Case diagrams missing a system boundary subgraph.
     - Reject Use Case diagrams with directed arrows (`-->`) connecting actors to use cases.
     - Reject Use Case diagrams with non-stadium nodes inside the boundary.
     - Reject 2-node trivial diagrams (`Actor --> Target`).
4. **Epic Statechart Heterogeneity Gate**:
   - Verify that Epics do not share identical canned state machine diagrams; each must reflect its subsystem's actual behavioral states.
5. **Parent Epic Alignment Gate**:
   - Verify that User Stories declare a Parent Epic corresponding to their owning package, not hardcoded `EPIC-02-ConOps`.

### Work Package 4: Ground 0 Pipeline Recompilation & Purge
- Rebuild workspace release binaries: `cargo build --release --workspace`.
- Purge legacy misnamed files: `git rm docs/use-cases/UC-*-UC_*.md`.
- Run Ground 0 compilation:
  - `./target/release/compile-sysml --compile`
  - `./target/release/compile-sysml --stpa-transpile`
  - `./target/release/compile-sysml --forward-sync --force`
- Audit regenerated files against reference `target/reference/3dgs-039/docs/use-cases/` to ensure 100% convergence.

### Work Package 5: Verification, GitHub Projection & Remote Sync
- Execute `./target/release/verify-baseline . --spec-only` (must pass with 0 findings).
- Execute `./target/release/verify-baseline . --no-domain` (must pass all 32 quality gates).
- Project cleaned specifications to GitHub: `./target/release/reconcile-backlog --sync-bodies`.
- Commit changes using neutral citations: `(refs #115)`.
- Push to `origin/main` and verify `git diff origin/main` is empty.

---

## 3. Acceptance Criteria

- [ ] `crates/compile-sysml/src/parser/grammar.rs` captures inner `doc /* ... */` in `use case def`.
- [ ] `crates/compile-sysml/src/sync/forward.rs` generates clean Use Case titles, clean filenames, rich scenarios, valid UML Use Case diagrams, and dynamic source paths.
- [ ] Epics carry genuine subsystem statecharts and multi-part class diagrams from `schema/subsystems/*/`.
- [ ] User Stories carry genuine sequence diagrams and bind to their respective parent Epics.
- [ ] `crates/verify-baseline` fails mechanically on stuttering titles, canned boilerplate, 2-node flowcharts, canned epic statecharts, and cross-tier epic mismatches.
- [ ] All 93 specification files pass the armed `verify-baseline` with 0 findings.
- [ ] All 32 quality gates pass with exit code 0.
- [ ] All updated specifications projected to GitHub issues via `reconcile-backlog --sync-bodies`.
- [ ] All changes committed with neutral citations and pushed cleanly to remote.

---

## 4. Human Authorization Gate

Per `AGENTS.md` § **Strict Planning Gate**:
To authorize execution of this implementation plan, please reply with **PROCEED**.
