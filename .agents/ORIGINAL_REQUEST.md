# Original User Request

## 2026-09-21T11:20:52Z

Fix the two-tier installation and propagation architecture in DEAP01-spec-core so that domain repositories scaffold self-contained install instructions pointing to themselves, enabling customer project repositories to install cleanly in isolation without references to spec-core or broken local paths.

Working directory: /Users/perkunas/jail/DEAP01-spec-core
Integrity mode: development

## Requirements

### R1. Two-Tier Architecture Alignment
- Distinguish Tier 1 (Upstream Compiler DEAP01-spec-core -> Domain Distribution Template DEAP-*) from Tier 2 (Domain Distribution Template DEAP-* -> Customer Application Workspace uav-*).
- Update DEAP01-spec-core/README.md to document this two-tier boundary cleanly, preventing maintainers and users from conflating compiler tooling propagation with end-user customer onboarding.

### R2. Parameterized Domain Installer & Scaffolding
- Update scripts/install_pipeline.sh so that when it scaffolds or updates a downstream domain repository's README.md, the installation section automatically embeds the domain repository's own git remote URL rather than defaulting to DEAP01-spec-core.
- In downstream domain repositories, the documented onboarding command for end-user customer projects must be a single self-contained command operating strictly inside the customer project directory with zero sibling path dependencies (../...):
  git clone <domain-repo-remote-url> ./.tmp-pipeline && bash ./.tmp-pipeline/scripts/install_pipeline.sh . && rm -rf ./.tmp-pipeline

### R3. Robust Model & Schema Copying
- Ensure scripts/install_pipeline.sh copies domain models and schemas from the domain repo into the target customer project directory even if an empty schema/ directory already exists.

## Acceptance Criteria

### Verification & Conformance
- [ ] python3 scripts/verify_downstream_baseline.py --no-domain passes all checks cleanly in DEAP01-spec-core.
- [ ] Code blocks in all updated markdown files contain pure, valid shell syntax with zero unescaped parentheses in comments and zero unquoted angle-bracket placeholders.
- [ ] Downstream README generation logic in scripts/install_pipeline.sh produces verified, self-contained onboarding instructions using the target domain's remote URL.

PROCEED

## 2026-09-21T11:40:40Z

Implement Level 0 OEM prose/markdown ingestion support in sysmlv2_ingest.py and update the Operator Prompt Catalog / Pipeline 0 sequence in DEAP01-spec-core so that downstream customer projects starting with unstructured/prose manuals (markdown, PDF, BOM tables) have a sanctioned, deterministic path to generate schema/model.sysml and satisfy the compilation gate without deadlocking.

Working directory: /Users/perkunas/jail/DEAP01-spec-core
Integrity mode: development

## Requirements

### R1. Level 0 Markdown / BOM Schema Ingestion in sysmlv2_ingest.py
- Extend skills/spec-orchestrator/scripts/sysmlv2_ingest.py to support --format markdown (or --format auto detection of markdown tables / BOM specifications in schema/ and schema/extracted/).
- Parse structured Markdown tables (e.g. components, BOMs, signal/port interfaces, parametric ranges) and translate them into a valid, canonical SysML v2 textual model (schema/model.sysml / .pipeline/schema.sysml) with complete package, part def, port definitions, and constraint blocks.

### R2. Operator Prompt Catalog & Pipeline 0 Sequence Remediation
- Update the Operator Prompt Catalog (in scripts/install_pipeline.sh, skills/spec-orchestrator/SKILL.md, and downstream README templates) to formalize Step 0.0: Level 0 OEM Ground Truth Ingestion:
  - Explicitly document the entrypoint for customer projects starting with prose/PDF documentation.
  - Clarify that extracting BOM and physical parameters into schema/extracted/ and synthesizing schema/model.sysml is authorized under Check 23 and is the required precursor to running compile_sysml.py --compile.
  - Provide a dedicated subagent prompt for Level 0 OEM Ground Truth extraction and SysML v2 model authoring.

### R3. Pipeline 0 Compilation Gate Fallback & Helpful Error Messages
- Update scripts/compile_sysml.py so that when no .sysml file is found in schema/, the error message does not just raise an unhelpful FileNotFoundError, but provides clear remediation instructions directing the user/agent to run Step 0.0 / sysmlv2_ingest.py.

## Acceptance Criteria

### Verification & Conformance
- [ ] Unit tests added in tests/ verifying markdown table / BOM ingestion into SysML v2 AST.
- [ ] python3 skills/spec-orchestrator/scripts/sysmlv2_ingest.py --schema <sample_markdown> --format markdown --out .pipeline/schema.sysml successfully produces a valid SysML v2 AST parseable by compile_sysml.py.
- [ ] python3 scripts/verify_downstream_baseline.py --no-domain passes all checks cleanly in DEAP01-spec-core.
- [ ] All updated markdown code blocks contain valid shell syntax with zero unescaped parentheses in comments.


## 2026-09-21T12:56:23Z

Execute an adversarial code audit on `scripts/install_pipeline.sh` for the domain template URL synthesis bug, submit the verified 7-section defect dossier upstream via `python3 scripts/file_defect.py`, and implement the verified fix across compiler and downstream templates.

Working directory: /Users/perkunas/jail/DEAP01-spec-core
Integrity mode: development

## Requirements

### R1. Adversarial 5-Pillar Code Audit & Defect Submission
- Dispatch an adversarial audit subagent adhering to `skills/adversarial-code-auditor/SKILL.md`.
- Perform a 5-pillar forensic audit on the fallback logic in `scripts/install_pipeline.sh` (lines 626–633) where the script synthesizes a `gitlab.com` URL for upstream domain repositories (`DEAP-uas-infrastructure-safety`) when the downstream project provider is GitLab:
  - Error: `remote: The project you were looking for could not be found or you don't have permission to view it. fatal: repository 'https://gitlab.com/gintatkinson/DEAP-uas-infrastructure-safety.git/' not found`
  - Cause: Conflating target customer project issue tracker / git host (`$PROVIDER`) with the host platform of the upstream domain template repository.
- Generate the verified 7-section defect dossier and submit it upstream using `python3 scripts/file_defect.py --repo gintatkinson/DEAP01-spec-core --title "Tooling Bug: install_pipeline.sh synthesizes non-existent GitLab URLs for GitHub domain templates" --body-file <payload_path> --label "bug"`.

### R2. Grounded Code Remediation in `scripts/install_pipeline.sh`
- Fix the defect in `scripts/install_pipeline.sh`:
  - Decouple the customer project's provider (`$PROVIDER`, e.g. `gitlab`) from the host platform of the upstream domain template.
  - Add explicit `--domain-url <URL>` CLI parameter support.
  - Ensure canonical domain template repositories (`DEAP-uas-infrastructure-safety`, etc.) resolve to their authoritative host (GitHub) and never fabricate non-existent GitLab URLs.
  - Ensure the customer onboarding command generated in downstream `README.md` files always targets the verified working domain remote URL:
    ```bash
    git clone https://github.com/gintatkinson/DEAP-uas-infrastructure-safety.git ./.tmp-pipeline && bash ./.tmp-pipeline/scripts/install_pipeline.sh . && rm -rf ./.tmp-pipeline
    ```

### R3. Downstream Propagation & Remote Synchronization
- Verify the fix with `python3 scripts/verify_downstream_baseline.py --no-domain`.
- Propagate the updated installer to the domain repository (`DEAP-uas-infrastructure-safety`) and customer project (`uav-011`).
- Verify that `git diff origin/main` is clean and pushed.

## Acceptance Criteria

### Audit & Verification Gates
- [ ] 7-section defect dossier generated and successfully filed on upstream GitHub tracker via `scripts/file_defect.py`.
- [ ] Unit/regression tests added verifying that `install_pipeline.sh` with `--provider gitlab` produces valid GitHub clone commands for domain templates and zero non-existent `gitlab.com` domain URLs.
- [ ] `python3 scripts/verify_downstream_baseline.py --no-domain` passes all 30 checks cleanly.
- [ ] Remote branches (`DEAP01-spec-core`, `DEAP-uas-infrastructure-safety`, `uav-011`) are fully synchronized.

PROCEED

## 2026-09-21T16:32:10Z

Overhaul the installation instructions and README templates across all three repository tiers to eliminate broken, contradictory, and circular onboarding instructions:
1. Upstream Spec Core Compiler (DEAP01-spec-core)
2. Domain Distribution Templates (DEAP-*, e.g. DEAP-uas-infrastructure-safety)
3. Customer Application Workspaces (uav-*, e.g. uav-011)

Working directory: /Users/perkunas/jail/DEAP01-spec-core
Integrity mode: development

## Requirements

### R1. Clean, Purpose-Driven Upstream Compiler README (DEAP01-spec-core/README.md)
- Purge Broken Manual Snippets: Delete Section 5.4's 80-line fragile inline Python monkeypatching script and manual cp loops from README.md.
- Compiler-Centric Focus: The compiler README.md must document how to run and verify the compiler itself (python3 scripts/compile_sysml.py, pytest), and provide the single clean command for maintainers to propagate compiler tooling into a domain distribution template repository:
  bash scripts/install_pipeline.sh <path-to-domain-template>
  or via remote bootstrap:
  git clone https://github.com/gintatkinson/DEAP01-spec-core.git /tmp/deap_compiler && bash /tmp/deap_compiler/scripts/install_pipeline.sh . && rm -rf /tmp/deap_compiler
- Remove Conflated Domain Content: Remove hardcoded customer onboarding commands that clone domain repositories from the upstream compiler's quickstart.

### R2. Proper Domain Distribution Template README Scaffolding (DEAP-*)
- In scripts/install_pipeline.sh, distinguish when installing into a Domain Distribution Template (DOMAIN_DISTRIBUTION_TEMPLATE) vs. a Customer Application Workspace (DOWNSTREAM_CUSTOMER_PROJECT).
- For Domain Distribution Templates (DEAP-*):
  - Repository role must be declared as DOMAIN_DISTRIBUTION_TEMPLATE.
  - Maintain the clean landing zone invariant in docs.
  - Document the authoritative, single-line onboarding command for customers to clone from this domain template into their customer project:
    git clone <this-domain-template-remote-url> ./.tmp-pipeline && bash ./.tmp-pipeline/scripts/install_pipeline.sh . && rm -rf ./.tmp-pipeline

### R3. Proper Customer Workspace README Scaffolding (uav-*)
- For Customer Application Workspaces (uav-*):
  - Repository role must be declared as DOWNSTREAM_CUSTOMER_PROJECT.
  - The README should NOT contain circular instructions telling the customer to clone uav-011 to install a pipeline into uav-011.
  - Instead, document project-specific commands: how to run baseline verification (python3 scripts/verify_downstream_baseline.py), how to execute Step 0.0 Level 0 ingestion, and how to update local pipeline tooling in-place:
    bash scripts/install_pipeline.sh .

## Acceptance Criteria

### Verification & Documentation Hygiene
- [ ] DEAP01-spec-core/README.md contains zero inline multi-line Python scripts and zero hardcoded domain repository clone commands in its installation sections.
- [ ] Scaffolding in scripts/install_pipeline.sh detects repository role dynamically and generates distinct, accurate READMEs for Domain Templates vs Customer Workspaces with zero circular clone commands.
- [ ] python3 scripts/verify_downstream_baseline.py --no-domain passes cleanly in DEAP01-spec-core.
- [ ] All code fences contain pure, executable shell syntax with zero unescaped parentheses in comments and zero unquoted angle-bracket placeholders.

PROCEED

## 2026-09-24T15:26:00Z

Full multi-stage team: Adversarial auditor diagnoses, files upstream defect issue, implementer builds the fix, and verifier tests.

Execute an adversarial code audit on the downstream onboarding and rule-ingestion pipeline, file a formal verified defect report to `gintatkinson/DEAP01-spec-core`, and execute the debug protocol to implement and verify a deterministic fix preventing LLM agents from taking ingestion shortcuts.

Working directory: /Users/perkunas/jail/DEAP01-spec-core
Integrity mode: development

## Problem Context
When downstream customer projects are onboarded and executed via `scripts/install_pipeline.sh` and the generated operator prompt catalog:
1. LLM agents treat directory-wide directives (e.g., `rules/` or `skills/`) as directory listings (`list_dir`) rather than executing sequential reads (`view_file`) on all 21 rule files.
2. Agents sample 1-2 representative files (e.g., `rules/role-boundary-lock.md`), falsely assuming they cover all governance, skipping critical domain constraints like `rules/dual-track-mbd-verification.md` and `rules/sysml-ssot-completeness.md`.
3. Downstream onboarding prompts in `README.md` selectively name only 1 or 2 rules, reinforcing lazy evaluation.
4. No consolidated active rule manifest exists, forcing agents into 21 round-trip tool calls which triggers subconscious token-conservation shortcuts.

## Requirements

### R1. Adversarial Audit & 5-Pillar Vulnerability Diagnosis
- Dispatch an adversarial audit adhering to `skills/adversarial-code-auditor/SKILL.md`.
- Inspect `scripts/install_pipeline.sh`, `scripts/scaffold_downstream_agents.py`, `tests/test_readme_scaffolding.py`, and the generated operator prompt templates in `README.md`.
- Produce a comprehensive 7-section defect dossier diagnosing:
  - Token-conservation bias triggers in current prompt templates.
  - Lack of an installation-time consolidated governance bundle (`.pipeline/ACTIVE_RULES_BUNDLE.md`).
  - Fragility of open-ended folder directives across downstream customer workspaces.

### R2. Upstream Defect Submission
- Submit the verified defect report via `python3 scripts/file_defect.py` to `gintatkinson/DEAP01-spec-core` with label `bug` and title:
  `Tooling Defect: Downstream Onboarding Rule-Shortcutting & Lack of Consolidated Rule Ingestion Bundle`.
- Capture and log the created issue URL and number.

### R3. Debug Protocol & Root-Cause Remediation
- **Active Governance Rule Bundling**:
  - Update `scripts/install_pipeline.sh` to automatically compile/bundle all active rule files from `rules/*.md` into a single consolidated, strongly formatted governance manifest at `.pipeline/ACTIVE_RULES_BUNDLE.md`.
  - Ensure the bundle includes table-of-contents, origin file headers, and full unabridged rule bodies.
- **Operator Prompt Catalog Update**:
  - Update `scripts/install_pipeline.sh` README scaffolding logic so downstream prompt catalogs direct agents to execute `view_file` directly on `.pipeline/ACTIVE_RULES_BUNDLE.md`, eliminating the need for 21 separate round trips while guaranteeing 100% rule coverage in a single read.
- **Regression Test Suite**:
  - Add comprehensive regression tests in `tests/test_readme_scaffolding.py` verifying:
    1. Installation generates `.pipeline/ACTIVE_RULES_BUNDLE.md` containing all rules from `rules/`.
    2. Generated `README.md` operator prompts mandate reading `.pipeline/ACTIVE_RULES_BUNDLE.md`.
    3. No prompt templates point to isolated rule subsets.

## Acceptance Criteria

### Objective Verification
- [ ] 7-section adversarial defect dossier is generated and filed upstream to `gintatkinson/DEAP01-spec-core` via `python3 scripts/file_defect.py`.
- [ ] `scripts/install_pipeline.sh` generates `.pipeline/ACTIVE_RULES_BUNDLE.md` during installation with 100% content from `rules/*.md`.
- [ ] Downstream `README.md` operator prompt catalog directs agents to read `.pipeline/ACTIVE_RULES_BUNDLE.md` as their mandatory governance entry point.
- [ ] Unit test suite passes with zero regressions: `python3 -m unittest tests/test_readme_scaffolding.py` (18/18+ passing).
- [ ] Zero uncommitted or unstaged changes; git diff against remote tracking branch is verified clean.

PROCEED
