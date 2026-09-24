# Milestone 1 (R1) Changes Report: Upstream Compiler README.md

**Subagent ID:** `worker_m1_1`  
**Role:** Implementer / QA / Specialist  
**Target File:** `/Users/perkunas/jail/DEAP01-spec-core/README.md`  
**Date:** 2026-09-21  

---

## 1. Summary of Changes

Milestone 1 implements the overhaul of `/Users/perkunas/jail/DEAP01-spec-core/README.md` to establish an unambiguous, compiler-centric specification and maintainer guide while eliminating broken manual copy loops, inline monkeypatching scripts, and conflated downstream domain onboarding content.

### 1.1 Overview & Architecture Hygiene (Section 1)
- **Line 17:** Removed obsolete reference to "or Direct Copy" and updated phrasing to state that downstream customer application workspaces (`uav-*`) are onboarded via domain distribution templates (`DEAP-*`).
- **Lines 124 & 153:** Added explicit `text` language tags to directory tree code blocks to ensure markdown linter compliance and prevent misinterpretation as shell code.

### 1.2 Purge of Section 5.4 Manual Snippets & Inline Monkeypatching
- **Deleted Section 5.4 (`### 5.4 Direct Copy / Manual Setup`):** Excised 90 lines of fragile manual shell and Python monkeypatching commands (formerly lines 273–362):
  - 31 lines of raw directory `cp`/`rm`/`mkdir` loops.
  - 30 lines of brittle inline Python regex and string replacement attempting to monkeypatch `AGENTS.md` headers.
  - 12 lines of inline Python mutating `.pipeline/codebase_rules.json` to hardcode GitLab tracker labels.
  - Manual workspace cleanup scripts bypassing the verified turnkey installer.

### 1.3 Upstream Compiler-Centric Quickstart (Section 5.2)
- Added dedicated documentation in Section 5.2 instructing developers and maintainers on how to run and verify the compiler itself:
  1. **Pipeline 0 Compilation Gate:** `python3 scripts/compile_sysml.py --compile` (compiles SysML v2 AST into `.pipeline/schema.sysml` and `.pipeline/schema-digest.json`).
  2. **Regression & Unit Test Suite:** `python3 -m pytest tests/` (executes the pytest suite covering AST parsing, coverage linters, and verification harnesses).
  3. **Compiler Conformance Baseline:** `python3 scripts/verify_downstream_baseline.py --no-domain` (validates all 30 abstract MBSE compiler baseline checks).

### 1.4 Clean Maintainer Propagation Commands (Section 5.3)
- Reorganized Section 5.3 around clean, canonical maintainer workflows to propagate compiler tooling to domain distribution templates:
  - **Local Sibling Propagation:** `bash scripts/install_pipeline.sh "<path-to-domain-template>"` (run from `DEAP01-spec-core` root with quoted placeholder).
  - **Remote Bootstrap Propagation:** `git clone https://github.com/gintatkinson/DEAP01-spec-core.git /tmp/deap_compiler && bash /tmp/deap_compiler/scripts/install_pipeline.sh . && rm -rf /tmp/deap_compiler` (uses `/tmp/deap_compiler` sandbox).
  - **In-Place Governance Updates:** `bash scripts/install_pipeline.sh .` (updates an initialized domain template repository in-place).

### 1.5 Architectural Boundary & Removal of Conflated Domain Content (Section 5.4)
- **Excised Hardcoded Domain Clone Commands:** Removed hardcoded customer onboarding commands cloning `DEAP-uas-infrastructure-safety` (formerly lines 251–264 in Section 5.3).
- **Formalized Architectural Boundary:** Added Section 5.4 clarifying that end-user customer projects (`uav-*`) clone from their respective **Domain Distribution Template** (`DEAP-*`), not from the upstream compiler (`DEAP01-spec-core`).
- Added a clean ASCII structural diagram mapping the propagation and onboarding flow across the tiers.

### 1.6 Governance & Agent Initialization Updates (Sections 5.5 - 5.9)
- **Section 5.5:** Clarified role detection for `UPSTREAM_SPEC_CORE_COMPILER` when `.pipeline/upstream/` is present; sanitized instruction fence with `text` tag.
- **Section 5.6:** Replaced the full `DOWNSTREAM_CUSTOMER_PROJECT` sample template with a concise explanation of how `install_pipeline.sh` configures role classifications across repository tiers.
- **Section 5.7:** Sanitized shell comment in Claude Code setup.
- **Section 5.9:** Retitled to `Compiler Verification & Baseline Gate` reflecting its role in `DEAP01-spec-core`.

---

## 2. Verification Results

1. `python3 scripts/verify_downstream_baseline.py --no-domain`
   - **Result:** Exit code 0. All 30 baseline checks verified successfully.
2. `python3 -m pytest tests/`
   - **Result:** Exit code 0. All 23 tests passed in 6.83s.
3. Code fence and syntax hygiene:
   - Zero unescaped parentheses in shell comments.
   - All angle-bracket placeholders quoted (`"<path-to-domain-template>"`).
   - Zero multi-line inline Python scripts in installation sections.
