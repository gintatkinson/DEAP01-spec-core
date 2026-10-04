# Implementation Plan: Reference Blueprint Decontamination & Master Multi-Repository Execution Plan

## 1. Executive Summary & Objective

This implementation plan governs the complete decontamination, modernization, and atomic materialization of the reference architecture blueprints and master multi-repository execution plan in `DEAP01-spec-core`:
1. **Master Multi-Repository Execution Plan**: Establish `docs/architecture/MASTER_EXECUTION_PLAN.md` (1,009 lines) orchestrating Work Packages WP-01 through WP-09 across `DEAP01-spec-core`, `deap-compiler-spec`, and `DEAP02-spec-core`.
2. **Reference Architecture Blueprints**: Modernize and decontaminate all 14 reference architecture blueprints in `docs/architecture/blueprints/` to `status: "APPROVED / PRODUCTION-GRADE"`.
3. **Pure Schema-Driven Compiler Invariant**: Eradicate all physical hardware, kinetic, defense, munition, and vehicle terms (ESAD, squib, warhead, fuze, fuzing, arrestor, standoff, wing, aileron, rudder, propulsion, throttle, motor, rotor, glide, drone, airspeed, cfs, medical, chassis). Replace concrete safety matrices with purely symbolic mathematical formulations of Cartesian product expansion $(U = A \times G)$.
4. **Institutional Regulatory Agency Decontamination**: Eradicate all regulatory agency acronyms (FAA, EASA, FDA, NHTSA, IMO, CFR, CS-25), replacing them with domain-independent formal compliance abstractions.
5. **KaTeX & Markdown Hygiene**: Fix LaTeX syntax in Markdown tables (e.g. line 198 of `MASTER_EXECUTION_PLAN.md` and token bounds in table cells). Zero Unicode em dashes (`\u2014`); ASCII `--` or `-` exclusively. Zero unresolved markers (TODO, TBD, FIXME, draft, pending).
6. **Pure Rust Workspace Architecture Codification**: Document the modular Cargo workspace crates (`deap-core`, `deap-ast`, `deap-codegen`, `deap-harness`, `deap-cli`), strongly-typed AST slicing mechanics ($\le 4,096$ tokens), and dual-LLM air-gapped agent dispatch (DeepSeek-R1 CoT reasoning slot streaming to `.pipeline/diagnostics/cot_audit_log.json` -> Qwen-2.5-Coder execution slot).
7. **Downstream Baseline Verification**: Execute `python3 scripts/verify_downstream_baseline.py .` and achieve exit code 0 across all 31 checks.
8. **Git Commit & Remote Push**: Commit with neutral citation `docs(architecture): decontaminate reference blueprints and establish master execution plan (refs #405, refs #406)` and push to `origin main`, ensuring `git diff origin/main` returns exactly 0 bytes.

---

## 2. Work Packages

### WP-01: Baseline Blueprint Decontamination & Master Execution Plan Materialization
- **Assigned Worker**: `worker_commit_restorer` (Implementer, QA, Specialist)
- **Target Files**:
  - `docs/architecture/MASTER_EXECUTION_PLAN.md`
  - `docs/architecture/blueprints/DEAP_DEEPSEEK_HARNESS_INTEGRATION_BLUEPRINT.md`
  - `docs/architecture/blueprints/DEAP_DETERMINISTIC_SAFETY_SPECIFICATION_COMPILER_BLUEPRINT.md`
  - `docs/architecture/blueprints/DEAP_DOMAIN_AGNOSTIC_MISSION_LIFECYCLE_ARCHITECTURE.md`
  - `docs/architecture/blueprints/DEAP_DOMAIN_INDEPENDENT_FDIR_BLUEPRINT.md`
  - `docs/architecture/blueprints/DEAP_LOCAL_AIRGAPPED_DEEPSEEK_WORKSTATION_BLUEPRINT.md`
  - `docs/architecture/blueprints/DEAP_LOGICAL_INTERFACE_SPECIFICATION_BLUEPRINT.md`
  - `docs/architecture/blueprints/DEAP_MULTI_TOOLCHAIN_SYNTHESIS_ARCHITECTURE.md`
  - `docs/architecture/blueprints/DEAP_SYSML_V2_INGESTION_ENGINE_BLUEPRINT.md`
  - `docs/architecture/blueprints/MULTI_PROVIDER_GITLAB_INFRASTRUCTURE_ARCHITECTURE.md`
  - `docs/architecture/blueprints/PERSISTENCE_ARCHITECTURE.md`
  - `docs/architecture/blueprints/RUNTIME_METADATA_ENGINE.md`
  - `docs/architecture/blueprints/SAFETY_CRITICAL_REALTIME_UI_FRAMEWORK.md`
  - `docs/architecture/blueprints/SPECKIT_NATIVE_INTEGRATION.md`
  - `docs/architecture/blueprints/SYSML_SSOT_BIDIRECTIONAL_SYNCHRONIZATION_ARCHITECTURE.md`
  - `implementation_plan.md`
- **Actions**:
  1. Materialize all 15 files with exact required production-grade content.
  2. Apply KaTeX table formatting fixes:
     - In `MASTER_EXECUTION_PLAN.md` line 198: replace LaTeX inside table pipes with `(U = A x G)`.
     - In `DEAP_LOCAL_AIRGAPPED_DEEPSEEK_WORKSTATION_BLUEPRINT.md` and `DEAP_DEEPSEEK_HARNESS_INTEGRATION_BLUEPRINT.md`: table rows with `(\le 4,096 tokens)` use `(<= 4,096 tokens)`.
  3. Validate static invariants across all 15 files:
     - Zero prohibited terms: ESAD, squib, warhead, fuze, fuzing, arrestor, standoff, wing, aileron, rudder, propulsion, throttle, motor, rotor, glide, drone, airspeed, cfs, medical, chassis.
     - Zero regulatory agency acronyms: FAA, EASA, FDA, NHTSA, IMO, CFR, CS-25.
     - Zero Unicode em dashes (`\u2014`).
     - Zero unresolved markers: TODO, TBD, FIXME, draft, pending.
     - Valid YAML frontmatter: `status: "APPROVED / PRODUCTION-GRADE"`.
     - Cargo workspace crates documented across all blueprints.
  4. Run `python3 scripts/verify_downstream_baseline.py .` and verify 31/31 checks pass with exit code 0.
  5. Commit and push:
     - `git add docs/architecture/ implementation_plan.md .agents/`
     - `git commit -m "docs(architecture): decontaminate reference blueprints and establish master execution plan (refs #405, refs #406)"`
     - `git push origin main`
  6. Verify `git diff origin/main` returns exactly 0 bytes.

---

## 3. Verification Gates Matrix

| Gate | Verification Command | Acceptance Standard |
|---|---|---|
| **Blueprint & Plan Count** | `ls docs/architecture/blueprints/*.md docs/architecture/MASTER_EXECUTION_PLAN.md \| wc -l` | Exactly `15` files |
| **Master Execution Plan Lines** | `wc -l docs/architecture/MASTER_EXECUTION_PLAN.md` | Exactly `1009` lines |
| **Static Invariant Verification** | Python verification script checking 15 files | `0` errors across all invariants |
| **Baseline Conformance Gate** | `python3 scripts/verify_downstream_baseline.py .` | Exit code `0`, all 31 checks pass |
| **Commit Neutrality Gate** | `python3 scripts/verify_commit_messages.py --head` | Exit code `0`, neutral citations only |
| **Remote Synchronization Gate** | `git diff origin/main` | Exactly `0` bytes |
