# Implementation Plan: Option 1 — Full First-Class MBSE Dogfooding, Blueprint Ingestion & Specification Projection in `/Users/perkunas/jail/DEAP01-spec-core`

## 1. Objective & Scope
Enable `/Users/perkunas/jail/DEAP01-spec-core` (`UPSTREAM_SPEC_CORE_COMPILER`) to **fully dogfood its own MBSE specification pipeline** in `/Users/perkunas/jail/DEAP01-spec-core/schema/` and `/Users/perkunas/jail/DEAP01-spec-core/docs/`:
1. Unlock `Check 11` (`check_clean_landing_zones` in `/Users/perkunas/jail/DEAP01-spec-core/crates/verify-baseline/src/checks/structural.rs`) and governance rules so `/Users/perkunas/jail/DEAP01-spec-core` models and specifies itself (`package DeapSpecCoreCompiler`), while Tier 1 templates (`/Users/perkunas/jail/DEAP-profile-flutter-app`) retain `.gitkeep` landing zones and customer domain pollution remains forbidden.
2. Ingest `/Users/perkunas/jail/DEAP01-spec-core/docs/designs/zero-trust-deterministic-rust-c-abi-blueprint.md` (`#483`), `/Users/perkunas/jail/DEAP01-spec-core/docs/designs/option-1c-rust-app-glue-ui-contracts-and-anti-fabrication-blueprint.md` (`#489`), `/Users/perkunas/jail/DEAP01-spec-core/docs/architecture/blueprints/*.md`, and `/Users/perkunas/jail/DEAP01-spec-core/crates/*` via `/Users/perkunas/jail/DEAP01-spec-core/target/release/ingest-sysml` into `/Users/perkunas/jail/DEAP01-spec-core/schema/` and `/Users/perkunas/jail/DEAP01-spec-core/.pipeline/schema.sysml`.
3. Execute the complete `/Users/perkunas/jail/DEAP01-spec-core/.agents/skills/spec-orchestrator/SKILL.md` pipeline (`schema-specification-engineering`, Component Failure Mode Research Swarm, `compile-sysml` STPA/FMECA, `spec-conops-engineering` + `assemble-conops`, `spec-icd-engineering`, `spec-user-story-engineering`, `spec-usecase-engineering`, and `spec-wbs-engineering`) to generate all dogfooded Epics, Features, User Stories, Use Cases, Safety Matrices, ConOps, ICDs, and WBS in `/Users/perkunas/jail/DEAP01-spec-core/docs/`.
4. Project all generated Epics, Features, User Stories, and Use Cases into the `gintatkinson/DEAP01-spec-core` GitHub backlog via `/Users/perkunas/jail/DEAP01-spec-core/target/release/reconcile-backlog` and verify `/Users/perkunas/jail/DEAP01-spec-core/target/release/verify-baseline /Users/perkunas/jail/DEAP01-spec-core` passes **WITHOUT `--no-domain`**.

---

## 2. Work Package Breakdown (100% Subagent-Delegated in Bounded Waves `<= 5` After `PROCEED`)

### Work Package 1: Unlock First-Class Dogfooding in `/Users/perkunas/jail/DEAP01-spec-core` (`Check 11` & Governance Synchronization)
- **Assigned Subagent Role**: `Rust Verification & Governance Engineer -- Check 11 Dogfooding Gate`
- **Active Skill**: `/Users/perkunas/jail/DEAP01-spec-core/.agents/skills/project-constitution/SKILL.md`
- **Target Files**:
  - [/Users/perkunas/jail/DEAP01-spec-core/crates/verify-baseline/src/checks/structural.rs](file:///Users/perkunas/jail/DEAP01-spec-core/crates/verify-baseline/src/checks/structural.rs)
  - [/Users/perkunas/jail/DEAP01-spec-core/AGENTS.md](file:///Users/perkunas/jail/DEAP01-spec-core/AGENTS.md)
  - [/Users/perkunas/jail/DEAP01-spec-core/.agents/AGENTS.md](file:///Users/perkunas/jail/DEAP01-spec-core/.agents/AGENTS.md)
  - [/Users/perkunas/jail/DEAP01-spec-core/.pipeline/constitution.md](file:///Users/perkunas/jail/DEAP01-spec-core/.pipeline/constitution.md)
  - [/Users/perkunas/jail/DEAP01-spec-core/rules/codebase-compliance.md](file:///Users/perkunas/jail/DEAP01-spec-core/rules/codebase-compliance.md)
  - [/Users/perkunas/jail/DEAP01-spec-core/.pipeline/ACTIVE_RULES_BUNDLE.md](file:///Users/perkunas/jail/DEAP01-spec-core/.pipeline/ACTIVE_RULES_BUNDLE.md)
- **Deliverables**:
  1. Write failing RED unit tests in `/Users/perkunas/jail/DEAP01-spec-core/crates/verify-baseline/src/checks/structural.rs` and update `check_clean_landing_zones` (`Check 11`) so that:
     - **Tier 1 Application Profile Templates (`DEAP-profile-*`)**: Continue enforcing empty `.gitkeep` landing zones in `schema/`, `docs/epics/`, `docs/features/`, `docs/user-stories/`, and `docs/use-cases/`.
     - **Tier 0 Core Compiler (`UPSTREAM_SPEC_CORE_COMPILER` — `/Users/perkunas/jail/DEAP01-spec-core`, where `.pipeline/upstream/` exists)**: Permit and validate self-hosted compiler specifications (`package DeapSpecCoreCompiler`) in `schema/`, `.pipeline/schema.sysml`, `docs/epics/`, `docs/features/`, `docs/user-stories/`, and `docs/use-cases/`, while rejecting any customer domain specifications (C-UAS, UAV, telecom, medical, etc.).
  2. Update `/Users/perkunas/jail/DEAP01-spec-core/AGENTS.md`, `/Users/perkunas/jail/DEAP01-spec-core/.agents/AGENTS.md`, `/Users/perkunas/jail/DEAP01-spec-core/.pipeline/constitution.md`, `/Users/perkunas/jail/DEAP01-spec-core/rules/codebase-compliance.md`, and `/Users/perkunas/jail/DEAP01-spec-core/.pipeline/ACTIVE_RULES_BUNDLE.md` so `DEAP01-spec-core` runs full-domain `verify-baseline` (retiring `--no-domain` bypasses once dogfooded specs are generated).
  3. Rebuild `/Users/perkunas/jail/DEAP01-spec-core/target/release/verify-baseline` (`cargo build --release -p verify-baseline`) and verify `cargo test -p verify-baseline` passes.

---

### Work Package 2: Level 0 SysML v2 Ingestion of Compiler Design Blueprints & Crates into `/Users/perkunas/jail/DEAP01-spec-core/schema/` & `.pipeline/schema.sysml`
- **Assigned Subagent Role**: `SysML v2 Ingestion & Metamodel Engineer`
- **Active Skill**: `/Users/perkunas/jail/DEAP01-spec-core/.agents/skills/spec-orchestrator/SKILL.md`
- **Source Blueprints & Crates**:
  - [/Users/perkunas/jail/DEAP01-spec-core/docs/designs/zero-trust-deterministic-rust-c-abi-blueprint.md](file:///Users/perkunas/jail/DEAP01-spec-core/docs/designs/zero-trust-deterministic-rust-c-abi-blueprint.md) (`DEAP-BLUEPRINT-CABI-001`, `#483`)
  - [/Users/perkunas/jail/DEAP01-spec-core/docs/designs/option-1c-rust-app-glue-ui-contracts-and-anti-fabrication-blueprint.md](file:///Users/perkunas/jail/DEAP01-spec-core/docs/designs/option-1c-rust-app-glue-ui-contracts-and-anti-fabrication-blueprint.md) (`DEAP-BLUEPRINT-APP-GLUE-001`, `#489`)
  - `/Users/perkunas/jail/DEAP01-spec-core/docs/architecture/blueprints/*.md`
  - `/Users/perkunas/jail/DEAP01-spec-core/crates/{deap-core,compile-sysml,ingest-sysml,verify-baseline,assemble-conops,reconcile-backlog}`
- **Target Files**:
  - `/Users/perkunas/jail/DEAP01-spec-core/schema/01_deap_spec_core_compiler.sysml`
  - `/Users/perkunas/jail/DEAP01-spec-core/schema/02_cabi_solver_bridge_codegen.sysml`
  - `/Users/perkunas/jail/DEAP01-spec-core/schema/03_option_1c_app_glue_and_ast_gates.sysml`
  - `/Users/perkunas/jail/DEAP01-spec-core/.pipeline/schema.sysml`
- **Deliverables**:
  1. Invoke `/Users/perkunas/jail/DEAP01-spec-core/target/release/ingest-sysml` and `/Users/perkunas/jail/DEAP01-spec-core/target/release/compile-sysml` to ingest the design blueprints, canonical architecture blueprints, and compiler crate structures into `/Users/perkunas/jail/DEAP01-spec-core/schema/` and consolidated `/Users/perkunas/jail/DEAP01-spec-core/.pipeline/schema.sysml` (`package DeapSpecCoreCompiler`).
  2. Ensure 100% of structural and behavioral AST constructs (`part def`, `port def`, `item def`, `action def`, `state def`, `calc def`, `constraint def`, `requirement def`, `use case def`, `interaction def`, `test case def`) are modeled in `.pipeline/schema.sysml`.

---

### Work Package 3: Execute Full `spec-orchestrator` Pipeline to Dogfood Epics, Features, Failure Mode Registry, STPA/FMECA, ConOps, ICD, User Stories, Use Cases & WBS
- **Active Skills (Read as Step 1 by Each Single-Item Subagent)**:
  - [/Users/perkunas/jail/DEAP01-spec-core/.agents/skills/spec-orchestrator/SKILL.md](file:///Users/perkunas/jail/DEAP01-spec-core/.agents/skills/spec-orchestrator/SKILL.md)
  - [/Users/perkunas/jail/DEAP01-spec-core/.agents/skills/schema-specification-engineering/SKILL.md](file:///Users/perkunas/jail/DEAP01-spec-core/.agents/skills/schema-specification-engineering/SKILL.md)
  - [/Users/perkunas/jail/DEAP01-spec-core/.agents/skills/spec-conops-engineering/SKILL.md](file:///Users/perkunas/jail/DEAP01-spec-core/.agents/skills/spec-conops-engineering/SKILL.md)
  - [/Users/perkunas/jail/DEAP01-spec-core/.agents/skills/spec-icd-engineering/SKILL.md](file:///Users/perkunas/jail/DEAP01-spec-core/.agents/skills/spec-icd-engineering/SKILL.md)
  - [/Users/perkunas/jail/DEAP01-spec-core/.agents/skills/spec-user-story-engineering/SKILL.md](file:///Users/perkunas/jail/DEAP01-spec-core/.agents/skills/spec-user-story-engineering/SKILL.md)
  - [/Users/perkunas/jail/DEAP01-spec-core/.agents/skills/spec-usecase-engineering/SKILL.md](file:///Users/perkunas/jail/DEAP01-spec-core/.agents/skills/spec-usecase-engineering/SKILL.md)
  - [/Users/perkunas/jail/DEAP01-spec-core/.agents/skills/spec-wbs-engineering/SKILL.md](file:///Users/perkunas/jail/DEAP01-spec-core/.agents/skills/spec-wbs-engineering/SKILL.md)
- **Target Files**:
  - `/Users/perkunas/jail/DEAP01-spec-core/docs/epics/EPIC-*.md`
  - `/Users/perkunas/jail/DEAP01-spec-core/docs/features/FEAT-*.md`
  - `/Users/perkunas/jail/DEAP01-spec-core/docs/research/FAILURE_MODE_REGISTRY.md`
  - `/Users/perkunas/jail/DEAP01-spec-core/docs/safety/STPA_MATRIX.md`
  - `/Users/perkunas/jail/DEAP01-spec-core/docs/conops/CONOPS.md` and `/Users/perkunas/jail/DEAP01-spec-core/docs/conops/units/**`
  - `/Users/perkunas/jail/DEAP01-spec-core/docs/icd/ICD_01_SYSTEM_INTERFACE_MATRIX.md` and `/Users/perkunas/jail/DEAP01-spec-core/docs/icd/ICD_02_MASTER_SIGNAL_DICTIONARY.md`
  - `/Users/perkunas/jail/DEAP01-spec-core/docs/user-stories/US-*.md`
  - `/Users/perkunas/jail/DEAP01-spec-core/docs/use-cases/UC-*.md`
  - `/Users/perkunas/jail/DEAP01-spec-core/docs/wbs/**`
- **Deliverables (1 Spec Item per Subagent Dispatch in Bounded Waves of `<= 5` Subagents)**:
  1. **Step 1 (Epics & Features)**: Decompose `.pipeline/schema.sysml` and the blueprints into granular Epics (`3–15` Features each) and atomic, independently testable Features (`3–10` Given-When-Then BDD acceptance criteria per Feature, mandatory negative scenarios, and 100% constraint coverage).
  2. **Step 1.5 & Step 2 (Failure Mode Swarm & Safety/ConOps/ICD Synthesis)**:
     - Dispatch 1 `Component Failure Mode Researcher` subagent per AST `part def` to populate `/Users/perkunas/jail/DEAP01-spec-core/docs/research/FAILURE_MODE_REGISTRY.md` across the 4 Universal Failure Dimensions (Interface, State, Action, Resource).
     - Synthesize `/Users/perkunas/jail/DEAP01-spec-core/docs/safety/STPA_MATRIX.md` via `/Users/perkunas/jail/DEAP01-spec-core/target/release/compile-sysml`.
     - Synthesize ConOps units and assemble `/Users/perkunas/jail/DEAP01-spec-core/docs/conops/CONOPS.md` via `/Users/perkunas/jail/DEAP01-spec-core/target/release/assemble-conops`.
     - Synthesize `/Users/perkunas/jail/DEAP01-spec-core/docs/icd/ICD_01_SYSTEM_INTERFACE_MATRIX.md` and `ICD_02_MASTER_SIGNAL_DICTIONARY.md`.
  3. **Steps 3, 4 & WBS (User Stories, Use Cases & WBS)**:
     - Synthesize BDD User Stories (`docs/user-stories/US-*.md` with `test case def` verification bindings), UML System Use Cases (`docs/use-cases/UC-*.md` with Realization Matrices), and MIL-STD-881E WBS artifacts (`docs/wbs/`).

---

### Work Package 4: Project Specifications into the `gintatkinson/DEAP01-spec-core` GitHub Backlog & Verify Full-Domain Dogfooded Baseline (`Without --no-domain`)
- **Assigned Subagent Role**: `Backlog Reconciliation & Full-Domain Baseline Verification Engineer`
- **Target Files**:
  - All specification files in `/Users/perkunas/jail/DEAP01-spec-core/docs/{epics,features,user-stories,use-cases}/` (`Issue ID` bindings)
  - `/Users/perkunas/jail/DEAP01-spec-core/.pipeline/schema.sysml`
- **Deliverables**:
  1. Execute `/Users/perkunas/jail/DEAP01-spec-core/target/release/reconcile-backlog` to create/synchronize GitHub Issues in `gintatkinson/DEAP01-spec-core` for every generated Epic, Feature, User Story, and Use Case and bind their `Issue ID` values in the local markdown metadata tables.
  2. Run `/Users/perkunas/jail/DEAP01-spec-core/target/release/verify-baseline /Users/perkunas/jail/DEAP01-spec-core` **WITHOUT `--no-domain`** and `cargo test --workspace`, verifying `0` errors across all specification, model coverage (`Check 19`), traceability (`Check 20`, `Check 30`), UML (`Check 15`), Mermaid (`Check 13B`), STPA/FMECA (`Check 17`), ConOps, ICD, and WBS gates.
  3. Commit and push all dogfooded models and specifications in `/Users/perkunas/jail/DEAP01-spec-core` to `origin/main` and verify `git diff origin/main` is empty.
