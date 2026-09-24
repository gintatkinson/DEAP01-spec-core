## 2026-09-24T18:41:00Z

Execute `view_file` on `/Users/perkunas/jail/DEAP01-spec-core/skills/adversarial-code-auditor/SKILL.md` as your very first step before executing any file edits or commands, and strictly follow its formatting templates and instruction guidelines.

Repository Classification: UPSTREAM_SPEC_CORE_COMPILER
Primary Commercial Toolchain Integration Context: MATLAB / Simulink / Stateflow / Embedded Coder

You are the Implementation Pipeline Worker. Your working directory is `/Users/perkunas/jail/DEAP01-spec-core/.agents/worker_r3`.
Your task is to implement the debug protocol and root-cause remediation for upstream Issue #368:
"Tooling Defect: Downstream Onboarding Rule-Shortcutting & Lack of Consolidated Rule Ingestion Bundle"

Target Files:
1. `scripts/install_pipeline.sh`
2. `tests/test_readme_scaffolding.py`

Detailed Requirements:
1. **Active Governance Rule Bundling in `scripts/install_pipeline.sh`**:
   - During pipeline installation (into `$TARGET_DIR`), automatically compile/bundle all active rule files from `$INSTALLER_ROOT/rules/*.md` into `$TARGET_DIR/.pipeline/ACTIVE_RULES_BUNDLE.md`.
   - Ensure the bundle file is generated for all repository tiers (Domain Distribution Template and Customer Application Workspace).
   - Structure of `.pipeline/ACTIVE_RULES_BUNDLE.md`:
     - Header: Explaining this is the consolidated governance manifest compiled automatically at installation time by `scripts/install_pipeline.sh`.
     - Table of Contents: Alphabetical or ordered markdown list linking to each rule anchor: `- [rule-file.md](#rule-slug)`.
     - Rule Sections: Each rule has an anchor (`<a id="rule-slug"></a>` or `## Rule: rule-file.md`), followed by its complete, unabridged content from `rules/<file>.md`.
     - Clean delimiters between rules (e.g. `---`).
     - Must contain 100% of the content from all files in `rules/*.md` (all 21 rules).

2. **Operator Prompt Catalog Scaffolding Update in `scripts/install_pipeline.sh`**:
   - Update the README generation logic in `scripts/install_pipeline.sh` (e.g. `scaffold_customer_project_readme` and `scaffold_domain_template_readme` and any prompt templates):
   - Replace open-ended instructions like "Ingest AGENTS.md and rules/ to enforce..." with explicit instructions:
     `3. **Load Governance Rules**: Execute `view_file` on `.pipeline/ACTIVE_RULES_BUNDLE.md` to ingest the complete, consolidated suite of active governance rules in a single read (covering dual-track MBD, SysML SSOT completeness, role boundary locks, and TDD mandates).`
   - Ensure downstream operator prompt catalogs in generated READMEs mandate reading `.pipeline/ACTIVE_RULES_BUNDLE.md` directly.
   - Eliminate prompt templates pointing to isolated rule subsets (e.g. ensure prompts reference the consolidated bundle `.pipeline/ACTIVE_RULES_BUNDLE.md` as the mandatory governance entry point).

3. **Comprehensive Regression Tests in `tests/test_readme_scaffolding.py`**:
   - Add tests verifying:
     a) Installation generates `.pipeline/ACTIVE_RULES_BUNDLE.md` containing all rules from `rules/` (verify file existence, TOC, and that every rule from `rules/*.md` is included with its exact content).
     b) Generated `README.md` operator prompts for both domain templates and customer projects explicitly mandate reading `.pipeline/ACTIVE_RULES_BUNDLE.md`.
     c) No prompt templates in generated READMEs point to isolated rule subsets or bare `rules/` directory reads.
   - Run `python3 -m unittest tests/test_readme_scaffolding.py` and verify all tests pass (at least 18/18+ tests passing).
   - Run `python3 scripts/verify_downstream_baseline.py --no-domain` and verify all checks pass cleanly.

MANDATORY INTEGRITY WARNING:
DO NOT CHEAT. All implementations must be genuine. DO NOT hardcode test results, create dummy/facade implementations, or circumvent the intended task. A auditor will independently verify your work. Integrity violations WILL be detected and your work WILL be rejected. Commit messages referencing Issue #368 MUST use neutral citations: `(#368)` or `(refs #368)`, never auto-closing keywords like `fixes #368`.

Write your completion report in `/Users/perkunas/jail/DEAP01-spec-core/.agents/worker_r3/handoff.md`, send message to parent orchestrator with test results and summary of changes.

PROCEED
