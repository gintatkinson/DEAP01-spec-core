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

## 2026-09-26T16:47:37Z

Execute view_file on skills/spec-orchestrator/SKILL.md as your very first step before executing any file edits or commands, and strictly follow its formatting templates and instruction guidelines.

Repository Classification: UPSTREAM_SPEC_CORE_COMPILER
Working directory: /Users/perkunas/jail/DEAP01-spec-core
Primary Native Skill: skills/spec-orchestrator/SKILL.md

Task: Author authoritative operational handoff DEAP-HANDOFF-ROOT-006 in HANDOFF.md, strictly maintaining the upstream specification compiler boundary and purging all hardcoded downstream customer concepts.

## Requirements

### R1. Upstream Compiler Scope & Boundary Enforcement
- Strictly adhere to the Pure Schema-Driven Compiler Invariant: DEAP01-spec-core is an abstract MBSE compiler and verification framework.
- Purge all concrete downstream customer domain concepts (such as Avenger 5, specific aircraft mass/inertia bounds, or UAS flight controller implementations) from HANDOFF.md. Downstream project roadmaps belong exclusively in customer workspaces (e.g. uav-009).
- Document upstream compiler deliverables: installer hardening (preserving customer compiled schemas in scripts/install_pipeline.sh), dual-provider architecture (GitHub and GitLab CLI engines in create_issue.sh and reconcile_backlog.py), CommonMark AST validation, and clean landing zones.

### R2. Complete 13 Failure Modes Retrospective
- Retain Failure Modes 1 through 9 verbatim (ensuring zero downstream drone schema filenames; replace any reference to schema/avenger5_system.sysml in Failure Mode 2 with abstract schema/*.sysml or downstream customer SysML model).
- Detail Failure Modes 10 through 13 in full depth:
  * Failure Mode 10: Regex & Substring Heuristics vs. AST / Schema Validation (Anti-Regex Invariant).
  * Failure Mode 11: Attempting to Clobber Downstream Customer Workspaces instead of Hardening Upstream Compiler Tooling.
  * Failure Mode 12: Collapsing Teamwork-Preview into Self-Auditing Single Workers.
  * Failure Mode 13: Coordinator Context Bloat via Verbose Terminal Diagnostics, Repeated Test Runs, and Blurring Upstream/Downstream Boundaries.

### R3. Fleet Synchronization & Remote Baseline Matrix
- Record the verified baseline commits across the fleet:
  * Upstream DEAP01-spec-core: commit dd7638c / 6188e52 (GitHub origin/main, clean 0-byte diff).
  * Customer uav-009: commit faff825 (GitLab origin/main, clean 0-byte diff).
  * Customer uav-011: commit c2826b9 (GitLab origin/main, clean 0-byte diff).
  * Template DEAP-uas-infrastructure-safety: clean landing zones (.gitkeep only).

### R4. Remote Synchronization & Commit Mandate
- Stage and commit HANDOFF.md using neutral citation:
  git commit -am "docs(handoff): update HANDOFF.md to DEAP-HANDOFF-ROOT-006 (refs #371)"
- Push to GitHub origin/main and verify git diff origin/main is 0 bytes.
- CONSTRAINT: Run ZERO tests. Do not invoke test runners, linters, or baseline verification scripts.

## Acceptance Criteria
- [ ] HANDOFF.md contains 0 references to concrete downstream drone schemas (e.g. schema/avenger5_system.sysml does not exist here and is not cited as an upstream schema).
- [ ] HANDOFF.md documents all 13 unvarnished failure modes.
- [ ] Section 4 and Section 5 focus on the upstream specification compiler roadmap and abstract pipeline orchestration, directing downstream application work to run in downstream application repositories.
- [ ] git diff origin/main is 0 bytes on DEAP01-spec-core.

PROCEED

## 2026-09-26T19:37:44Z

Execute view_file on skills/spec-orchestrator/SKILL.md as your very first step before executing any file edits or commands, and strictly follow its formatting templates and instruction guidelines.

Repository Classification: UPSTREAM_SPEC_CORE_COMPILER
Target Workspace: /Users/perkunas/jail/DEAP01-spec-core
Primary Role: Abstract MBSE Specification Compiler & Tooling Maintenance
Primary Commercial Toolchain Integration Context: MATLAB / Simulink / Stateflow / Embedded Coder

Execute a phased audit, triage, and comprehensive resolution of all 17 open and unfixed defect issues in DEAP01-spec-core across AST factual grounding, dual-provider tooling, baseline validator masking, and test mock elimination.

Working directory: /Users/perkunas/jail/DEAP01-spec-core
Integrity mode: development

## Requirements

### R1. Comprehensive Triage & Evidence Audit (Phase 1)
Audit all 17 open issues (#378, #377, #376, #375, #374, #373, #372, #368, #366, #365, #364, #363, #362, #361, #360, #349, #286) against the current codebase state and recent git commit log (d0e1bf0 down to dd7638c):
- Identify which issues have already been remediated by recent commits (e.g. #368 consolidated rules bundle, #363 template URLs, #373 duplicate checks).
- For each verified remediated issue, post an empirical verification evidence comment via gh issue comment and transition the issue label to status:fixed-resolved (retaining issue open status per tracker non-closure invariant).
- Formally catalog the remaining active defects into thematic clusters for Phase 2 execution.

### R2. Positive AST Provenance & Anti-Regex Hardening (Phase 2 - Cluster A)
Remediate grounding evasion defects (#378, #377, #376, #364):
- Replace negative-string regex heuristics and exemption tag bypasses in factual_grounding_validator.py with positive closed-world AST provenance validation against the SysML v2 AST and typed parameter dictionaries.
- Ensure Mermaid sequence diagrams and code fences do not bypass numeric grounding.

### R3. Dual-Provider Tooling & Installer Hardening (Phase 2 - Cluster B)
Remediate tooling automation defects (#374, #373, #372, #363):
- Fix skills/spec-orchestrator/scripts/create_issue.sh to prevent ARG_MAX buffer overflow by supporting body file payloads (--body-file), and ensure duplicate issue detection indexes the title column correctly.
- Ensure scripts/install_pipeline.sh correctly resolves domain template repository URLs between GitHub and GitLab without synthesizing non-existent routes.

### R4. Baseline Gate Masking & SSOT Parity (Phase 2 - Cluster C)
Remediate validator masking and Green Test Trap defects (#375, #366, #365, #362, #361):
- Fix scripts/verify_downstream_baseline.py Checks 17, 20, 23 and architecture_viewpoint_validator.py Gate 30 so that missing architecture models or specifications fail closed rather than silently returning exit code 0 when allow_missing_specs=False.
- Implement dual-schema SSOT parity verification and update README.md documentation harnesses.

### R5. Synthetic Mock Elimination in Safety & Parity Tests (Phase 2 - Cluster D)
Remediate mock violations (#360, #349, #286):
- Replace synthetic in-memory string mocks in safety validation and diagram parity tests with genuine schema/AST structures from test fixtures, enforcing closed-world model verification and eliminating citation fraud.

## Acceptance Criteria

### Automated Gate Verification
- [ ] Phase 1 Triage Report completed with empirical evidence for all 17 issues.
- [ ] All remediated issues have verification evidence posted and carry status:fixed-resolved.
- [ ] Remaining active defects have verified automated unit/integration tests in tests/.
- [ ] pytest tests/ runs with 100% pass rate (0 failures, 0 regressions).
- [ ] python3 scripts/verify_downstream_baseline.py passes all baseline checks with exit code 0.
- [ ] python3 scripts/verify_commit_messages.py --head passes with neutral citations (refs #<id>) and zero auto-closing verbs.
- [ ] Clean working tree with git diff origin/main returning 0 bytes after remote push.

PROCEED

## 2026-09-27T07:04:27Z

Execute view_file on skills/spec-orchestrator/SKILL.md as your very first step before executing any file edits or commands, and strictly follow its formatting templates and instruction guidelines.

Repository Classification: UPSTREAM_SPEC_CORE_COMPILER
Target Workspace: /Users/perkunas/jail/DEAP01-spec-core
Primary Role: Abstract MBSE Specification Compiler & Tooling Maintenance
Primary Commercial Toolchain Integration Context: MATLAB / Simulink / Stateflow / Embedded Coder

Execute fleet-wide pipeline propagation and parity verification from upstream DEAP01-spec-core to active downstream workspaces uav-009 and uav-011, verify all 31 baseline gates pass, synchronize with remote tracking branches at 0 bytes diff, and update the upstream handoff baseline matrix.

Working directory: /Users/perkunas/jail/DEAP01-spec-core
Integrity mode: development

## Requirements

### R1. Propagate to Customer Workspace uav-009 (Preserving Customer SSOT & Specs)
- Execute bash /Users/perkunas/jail/DEAP01-spec-core/scripts/install_pipeline.sh /Users/perkunas/jail/uav-009.
- In strict adherence to Failure Mode 11, verify that customer SysML models (schema/avenger5_system.sysml), compiled ASTs (.pipeline/schema.sysml), defect dossiers (docs/audit/), and all 75 published specifications in docs/ are 100% preserved (zero clobbering).

### R2. Automated Baseline Gate Verification for uav-009
- Run python3 /Users/perkunas/jail/DEAP01-spec-core/scripts/verify_downstream_baseline.py /Users/perkunas/jail/uav-009.
- Verify that all baseline checks (Checks 10 through 31, including Check 31 Dual-Schema SSOT Parity Gate) pass with exit code 0.

### R3. Git Stage, Commit & Remote Push for uav-009
- Stage updated pipeline framework assets in /Users/perkunas/jail/uav-009.
- Commit with neutral citation:
  chore(pipeline): propagate upstream spec-core fixes and Check 31 SSOT parity gate (refs #378, refs #377, refs #376, refs #375, refs #372, refs #366, refs #365, refs #364, refs #362, refs #361, refs #360, refs #349, refs #286)
- Verify commit neutrality via python3 /Users/perkunas/jail/uav-009/scripts/verify_commit_messages.py --head.
- Push to remote tracking branch: git -C /Users/perkunas/jail/uav-009 push origin main.
- Confirm git -C /Users/perkunas/jail/uav-009 diff origin/main is 0 bytes and working tree is clean.

### R4. Propagate to Application Workspace uav-011 (Preserving Clean Landing Zones)
- Execute bash /Users/perkunas/jail/DEAP01-spec-core/scripts/install_pipeline.sh /Users/perkunas/jail/uav-011.
- Verify that landing zones (schema/, docs/epics/, docs/features/, docs/user-stories/, docs/use-cases/) maintain 100% clean .gitkeep state.

### R5. Automated Baseline Gate Verification for uav-011
- Run python3 /Users/perkunas/jail/DEAP01-spec-core/scripts/verify_downstream_baseline.py /Users/perkunas/jail/uav-011.
- Verify that all baseline checks (Checks 10 through 31) pass with exit code 0.

### R6. Git Stage, Commit & Remote Push for uav-011
- Stage updated pipeline framework assets in /Users/perkunas/jail/uav-011.
- Commit with neutral citation:
  chore(pipeline): propagate upstream spec-core fixes and Check 31 SSOT parity gate (refs #378, refs #377, refs #376, refs #375, refs #372, refs #366, refs #365, refs #364, refs #362, refs #361, refs #360, refs #349, refs #286)
- Verify commit neutrality via python3 /Users/perkunas/jail/uav-011/scripts/verify_commit_messages.py --head.
- Push to remote tracking branch: git -C /Users/perkunas/jail/uav-011 push origin main.
- Confirm git -C /Users/perkunas/jail/uav-011 diff origin/main is 0 bytes and working tree is clean.

### R7. Update Fleet Parity Matrix in DEAP01-spec-core/HANDOFF.md
- Record the newly verified baseline commit hashes of uav-009 and uav-011 in Section 2.1 of HANDOFF.md.
- Commit with neutral citation: docs(handoff): update fleet parity baseline commit matrix (refs #372).
- Push to GitHub origin/main and verify git diff origin/main is 0 bytes.

## Acceptance Criteria

### Automated Gate Verification
- [ ] uav-009: All 31 baseline checks pass with exit code 0 under verify_downstream_baseline.py.
- [ ] uav-009: Pushed to GitLab origin/main, clean working tree, 0-byte remote diff.
- [ ] uav-009: Customer models (schema/avenger5_system.sysml), compiled ASTs, and 75 specifications are 100% intact.
- [ ] uav-011: All 31 baseline checks pass with exit code 0 under verify_downstream_baseline.py.
- [ ] uav-011: Pushed to GitLab origin/main, clean working tree, 0-byte remote diff.
- [ ] DEAP01-spec-core: HANDOFF.md Section 2.1 updated, pushed to GitHub origin/main, 0-byte remote diff.
- [ ] Zero auto-closing verbs across all commit messages; verified by verify_commit_messages.py --head.

PROCEED

## 2026-09-27T15:38:55Z

Execute view_file on skills/spec-orchestrator/SKILL.md as your very first step before executing any file edits or commands, and strictly follow its formatting templates and instruction guidelines.

Repository Classification: UPSTREAM_SPEC_CORE_COMPILER
Target Workspace: /Users/perkunas/jail/DEAP01-spec-core
Primary Native Skill: skills/spec-orchestrator/SKILL.md
Primary Commercial Toolchain Integration Context: MATLAB / Simulink / Stateflow / Embedded Coder

Perform a multi-agent adversarial audit and remediation of README.md and associated installer scaffolding templates in DEAP01-spec-core, resolving architecture tier numbering contradictions, heading ordering defects, and upstream vs. downstream repository execution boundaries.

Working directory: /Users/perkunas/jail/DEAP01-spec-core
Integrity mode: development

## Requirements

### R1. Forensic Audit of README.md & Installer Scaffolding
- Deploy an adversarial auditor subagent to perform a comprehensive audit of `README.md` and the README scaffolding logic in `scripts/install_pipeline.sh`.
- Catalog all structural defects, including contradictory tier numbering, inverted heading hierarchies, broken/outdated test citations, and repository classification boundary ambiguities.

### R2. Architecture Tier Hierarchy & Heading Normalization
- Remediate the tier numbering contradictions in `README.md` (Sections 1.2 & 5.4) and ASCII topology diagrams where both the Upstream Compiler and Domain Distribution Templates are labeled "Tier 1":
  * Normalize to a clean, unambiguous three-tier architecture:
    - Tier 1: Upstream Specification Core Compiler (`DEAP01-spec-core`)
    - Tier 2: Domain Distribution Templates (`DEAP-*`)
    - Tier 3: Customer Application Workspaces (`uav-*`)
  * Alternatively, clearly distinguish the two propagation boundaries (Tier 1 Compiler Propagation vs. Tier 2 Customer Onboarding) from repository classifications.
- Correct the out-of-order heading sequence in Section 1 so `1.1 Primary Commercial Toolchain Integration` logically precedes `1.2 System Architecture & Repository Boundaries`.

### R3. Upstream Compiler vs. Downstream Prompt Boundary Hardening
- Clarify Section 9.4 (Pipeline 2 Operator Prompts):
  * Explicitly specify that Pipeline 2 (Autonomous Feature Implementation for Flutter/ROS2/PX4 and Digital Twin Simulation) is strictly intended for execution within downstream customer application workspaces (`DOWNSTREAM_CUSTOMER_PROJECT`, e.g. `uav-*`).
  * Remove ambiguous statements implying that concrete application features or simulation drivers may be executed directly in `UPSTREAM_SPEC_CORE_COMPILER`.

### R4. Automated Regression & Scaffolding Test Verification
- Run `python3 -m unittest tests/test_readme_scaffolding.py` and ensure all scaffolding tests pass with exit code 0.
- Update `tests/test_readme_scaffolding.py` to assert the corrected heading hierarchy, tier definitions, and absence of contradictory tier labels.
- Run `python3 scripts/verify_downstream_baseline.py --no-domain` and verify all baseline checks pass cleanly with exit code 0.

### R5. Git Stage, Neutral Citation Commit & Remote Synchronization
- Verify zero issue-closing verbs across commit messages via `python3 scripts/verify_commit_messages.py --head`.
- Stage all changes, commit using neutral issue citations:
  `git commit -m "docs(readme): normalize architecture tiers, fix heading sequence, and harden repository boundary (refs #371, refs #368)"`
- Push to GitHub `origin/main` and verify that `git diff origin/main` returns exactly 0 bytes.
- Have the Victory Auditor independently confirm the 0-byte remote diff and gate passes before declaring victory.

PROCEED

## Acceptance Criteria

### Documentation & Scaffolding Integrity
- [ ] No contradictory "Tier 1" labels remain across `README.md` and `scripts/install_pipeline.sh`; tiers 1, 2, and 3 are distinctly and consistently defined.
- [ ] Section 1.1 precedes Section 1.2 in `README.md`.
- [ ] Pipeline 2 execution guidance explicitly restricts autonomous feature implementation to downstream customer workspaces.

### Verification & Automated Gates
- [ ] `python3 -m unittest tests/test_readme_scaffolding.py` passes with exit code 0.
- [ ] `python3 scripts/verify_downstream_baseline.py --no-domain` passes with exit code 0.
- [ ] `python3 scripts/verify_commit_messages.py --head` verifies zero issue-closing verbs.

### Git & Remote Synchronization
- [ ] Commit message strictly matches `docs(readme): normalize architecture tiers, fix heading sequence, and harden repository boundary (refs #371, refs #368)`.
- [ ] Changes are successfully pushed to `origin/main`.
- [ ] `git diff origin/main` returns exactly 0 bytes.

## 2026-09-27T17:36:23Z

Execute view_file on skills/spec-orchestrator/SKILL.md as your very first step before executing any file edits or commands, and strictly follow its formatting templates and instruction guidelines.

Repository Classification: UPSTREAM_SPEC_CORE_COMPILER
Working directory: /Users/perkunas/jail/DEAP01-spec-core/.agents/orchestrator_12
Target Workspace: /Users/perkunas/jail/DEAP01-spec-core
Primary Native Skill: skills/spec-orchestrator/SKILL.md
Primary Commercial Toolchain Integration Context: MATLAB / Simulink / Stateflow / Embedded Coder

Your mission is to fix defect #395 according to the constitution and methods of the repo in totality:
1. Complete the defect fix for #395 in full accordance with the repository constitution and governance methods (.pipeline/constitution.md and AGENTS.md).
2. Clean up untracked working tree scratch files and artifacts.
3. Commit the verified changes with a message referencing (fixes #395) or neutral citation per repo rules.
4. Remote Synchronization Mandate: push the commit to origin/main and verify git diff origin/main is completely empty.
5. Mandatory Tracker Issue Transition Gate: apply label status:fixed-resolved to issue #395, post verification evidence comment, and close issue #395 via gh issue.
6. Verify all test suites pass with exit code 0.

Follow repository constitution (.pipeline/constitution.md) and AGENTS.md in totality. Maintain BRIEFING.md and progress.md in your working directory. Dispatch context-isolated workers/reviewers as required. When complete, report victory with a structured handoff.
PROCEED

## 2026-10-03T18:17:22Z

Requested team: Full multi-agent team (dedicated architect, lead implementer, adversarial security auditor, and acceptance test engineer)

Deploy a full multi-agent team to refactor `scripts/install_pipeline.sh` from a monolithic, brittle shell script into a modular Python package (`scripts/installer/`) with atomic directory staging, safe error rollbacks, zero inline shell-interpolated Python snippets, and rigorous semantic acceptance verification.

Working directory: `/Users/perkunas/jail/DEAP01-spec-core`
Integrity mode: development

## Requirements

### R1. Structured Python Package & Lightweight Bootstrap Wrapper
- Deconstruct the monolithic logic of `scripts/install_pipeline.sh` into a structured Python package under `scripts/installer/` with clean functional separation (e.g. `cli.py`, `staging.py`, `tracker.py`, `scaffolding.py`, `metadata.py`).
- Reduce `scripts/install_pipeline.sh` to a concise, lightweight bootstrap wrapper (<150 lines) that checks host prerequisites (Python 3.10+, Git) and delegates execution to `python3 -m scripts.installer.cli "$@"`.
- Completely eliminate inline `python3 -c "..."` script blocks from shell scripts. All parameter handling, URL parsing, and JSON merging must be executed inside Python using `argparse` and type-safe data structures.

### R2. Atomic Directory Staging & State Preservation
- Eliminate destructive `rm -rf` operations followed by manual environment restoration.
- Use atomic staging directories (`tempfile.TemporaryDirectory()`) to prepare pipeline assets, rule bundles, and templates before committing changes to the target downstream workspace.
- Ensure all existing downstream configuration files (e.g., `project_metadata.json`, `profile_config.json`, git remote definitions, custom `.gitignore` entries) are automatically detected, safely merged, and preserved without data loss.

### R3. Robust Error Trapping, Rollback, and Security Posture
- Implement comprehensive exit/signal trapping (`trap cleanup EXIT ERR INT`) in `install_pipeline.sh` and structured exception handling in Python to guarantee automatic cleanup of staging directories and atomic rollback on failure.
- Ensure all remote URLs, namespaces, and provider strings are parsed with strict input sanitization to eliminate shell injection and quoting collisions.

### R4. Exclusive Semantic Acceptance Testing Mandate
- Strictly adhere to the repository invariant: **Unit tests (`tests/`, `test_*.py`, `pytest`) are strictly forbidden**.
- Verification must be performed exclusively via end-to-end semantic acceptance testing:
  1. Cold install against a real temporary git repository: verify exit code 0, complete asset tree generation, and zero unescaped template tokens.
  2. Upgrade test against an existing simulated downstream repository: verify exit code 0 and 100% preservation of custom metadata/configs.
  3. Downstream baseline verification: execute `python3 scripts/verify_downstream_baseline.py <target_dir>` against the installed directory and assert exit code 0 across all checks.
  4. Repository baseline verification: execute `python3 scripts/verify_downstream_baseline.py .` on `DEAP01-spec-core` and assert exit code 0.

## Acceptance Criteria

### Modularity & Code Quality
- [ ] `scripts/install_pipeline.sh` is a clean bootstrap script (<150 lines) with zero inline Python blocks (`python3 -c`).
- [ ] `scripts/installer/` is established as a clean Python package containing modular subcomponents with standard CLI entrypoint `cli.py`.
- [ ] All configuration, git URL parsing, and metadata logic utilize `argparse` and native Python libraries (`urllib.parse`, `json`, `pathlib`, `shutil`).

### Safety & Atomic Rollback
- [ ] Interrupted or failed installations trigger automated cleanup and leave the target directory in its original uncorrupted state.
- [ ] Existing downstream `project_metadata.json` and `profile_config.json` are preserved across consecutive install/upgrade runs.
- [ ] Zero shell injection vulnerabilities: all subprocess invocations use list-style arguments (`subprocess.run(["git", ...])`).

### Semantic Acceptance Verification
- [ ] Cold installation test into a temporary workspace executes cleanly with exit code 0.
- [ ] `python3 scripts/verify_downstream_baseline.py <temporary_workspace>` passes all baseline checks with exit code 0.
- [ ] `python3 scripts/verify_downstream_baseline.py .` passes on the compiler repo with exit code 0.
- [ ] Zero `test_*.py` unit test files exist in the repository (`find . -name "test_*.py" -not -path "./.git/*"` returns only the existing validator in `parity_auditor/`).
- [ ] `git diff origin/main` is clean after changes are committed and pushed.



## 2026-10-04T12:20:52Z

<USER_REQUEST>
Modernize the DEAP specification core compiler from procedural Python to a modular, mathematically deterministic Rust compiler workspace coordinating atomic, context-isolated LLM agent passes over strongly-typed AST slices. Decontaminate and modernize reference blueprints and establish the multi-repository master execution plan across DEAP01-spec-core, deap-compiler-spec, and DEAP02-spec-core.

Working directory: /Users/perkunas/jail/DEAP01-spec-core
Integrity mode: development

## 1. High-Assurance Production Reality & Mission
- **Platform**: Digital Engineering Agent Platform (DEAP).
- **Assurance Context**: Active, mission-critical production compiler supporting safety-critical engineering under RTCA DO-178C (DAL A/B), DO-254, SAE ARP4754A/4761, and MIL-STD-882E standards.
- **Mission**: Modernize the specification core compiler from legacy procedural Python to a modular, mathematically deterministic Rust compiler workspace coordinating atomic, context-isolated LLM agent passes over strongly-typed AST slices.
- **Tri-Repository Architecture**:
  1. `DEAP01-spec-core` (`/Users/perkunas/jail/DEAP01-spec-core`): Upstream specification core compiler repository. Reference blueprints authority and frozen black-box regression oracle.
  2. `deap-compiler-spec` (`/Users/perkunas/jail/deap-compiler-spec`): Downstream customer specification project workspace where the new Rust compiler architecture is modeled in SysML v2 and compiled via DEAP01 into formal specifications.
  3. `DEAP02-spec-core` (`/Users/perkunas/jail/DEAP02-spec-core`): Clean-slate upstream landing repository (to be created and initialized under `/Users/perkunas/jail/DEAP02-spec-core` during WP-04) where the modular pure Rust compiler workspace is built and verified.

---

## 2. The Non-Negotiable Prime Directive: Zero Domain Impressions from Ground Zero

The DEAP specification compiler is an abstract Model-Based Systems Engineering (MBSE) compiler (analogous to `rustc` or `llvm`). It is **NOT** a domain-specific modeler.

1. **Absolute Prohibition of Physical Domain Concepts**:
   - The compiler core, its reference blueprints, and its execution plans must contain **ZERO** physical hardware or kinetic concepts (no aircraft, wings, throttles, rudders, ailerons, propulsion, missiles, warheads, ESADs, squibs, arrestor wires, medical devices, or automotive chassis).
   - All models and illustrations must be purely symbolic computer science primitives: `PartDef_Alpha`, `Subsystem_01`, `Port_A`, `Action_01`, `Constraint_Invariant`.
2. **Absolute Prohibition of Institutional Regulatory Agency Names**:
   - The compiler core must contain **ZERO** institutional agency acronyms (no `FAA`, `EASA`, `FDA`, `NHTSA`, `IMO`, `CFR`, `CS-25`).
   - The compiler deals exclusively with abstract mathematical assurance tiers (e.g. `AssuranceLevel::Tier1..5`), formal bipartite verification graphs ($G = (V_{\text{req}}, V_{\text{test}}, E)$), and cryptographic reasoning audit records.
   - Agency-specific mappings belong **strictly and exclusively** in downstream customer domain repositories and platform profiles, never in upstream core code or blueprints.
3. **Absolute Prohibition of Hardcoded Domain Dictionaries**:
   - Zero hardcoded domain vectors, dictionaries, or static string lists in Python or Rust source code.
   - Domain cleanliness must be enforced through **closed-world positive type theory**: the compiler parses strictly against the OMG KerML / SysML v2 metamodel, and any undefined identifier is an immediate compiler error.

---

## 3. Strict Director Governance Invariants
- **Role Boundary**: The user is the Director. The Director sets high-level intent. The agent does the systems engineering work and does NOT push low-level design questions or "how to proceed" back to the Director.
- **Anti-Contamination Mandate**: Legacy Python code serves strictly as an external black-box regression oracle (verifying AST and gate parity in CI), never an architectural design source.
- **Dogfooding Protocol**: The compiler architecture is modeled in SysML v2 in `deap-compiler-spec`, generated via DEAP01 into formal specifications, and implemented in pure Rust in `DEAP02-spec-core`.
- **Zero Forbidden Unit Tests**: Unit tests (`tests/`, `test_*.py`, `pytest`) are strictly forbidden across upstream repositories. All verification is performed exclusively via end-to-end semantic acceptance testing and baseline gate checks.
- **Zero Em Dash Invariant**: Unicode em dashes (`\u2014`) are strictly forbidden across all files, commit messages, and outputs. Use ASCII `--` or `-` exclusively.
- **Adversarial Audit Mandate**: Do not trust shallow regex scanner passes. Verify every claim by empirical inspection.

---

## Requirements

### R1. Complete Decontamination & Modernization of Reference Blueprints (`DEAP01-spec-core`)
Audit and thoroughly rewrite the reference architecture blueprints in `docs/architecture/blueprints/` to `status: APPROVED / PRODUCTION-GRADE`:
- **Purge Concrete Project Leaks**: Eradicate all concrete defense, munition, and drone safety matrices (specifically the 84-row ESAD fuzing, squib trigger, and carrier arrestor tables in `DEAP_DETERMINISTIC_SAFETY_SPECIFICATION_COMPILER_BLUEPRINT.md`). Replace with purely symbolic mathematical formulations of Cartesian product expansion ($\mathcal{U} = \mathcal{A} \times \mathcal{G}$).
- **Purge Agency Citations**: Replace all occurrences of `FAA`, `EASA`, `FDA` with domain-independent formal compliance abstractions (`safety-critical regulatory traceability`, `multi-authority auditability`).
- **Codify the Pure Rust Workspace**: Formally document the modular Cargo workspace architecture (`deap-core`, `deap-ast`, `deap-codegen`, `deap-harness`, `deap-cli`), strongly-typed AST slicing mechanics ($\le 4,096$ tokens), and dual-LLM air-gapped agent dispatch (DeepSeek-R1 CoT reasoning slot streaming to `.pipeline/diagnostics/cot_audit_log.json` -> Qwen-2.5-Coder execution slot).

### R2. Author Master Multi-Repository Execution Plan (`DEAP01-spec-core`)
Author `docs/architecture/MASTER_EXECUTION_PLAN.md` orchestrating Work Packages WP-01 through WP-09:
- **WP-01**: Baseline Blueprint Decontamination & Abstract Rust Architecture Definition (`DEAP01-spec-core`).
- **WP-02**: SysML v2 Metamodeling of the Pure Rust Compiler Workspace (`deap-compiler-spec/schema/compiler_architecture.sysml`).
- **WP-03**: Specification Compilation & Agile Backlog Generation (`deap-compiler-spec`).
- **WP-04**: Rust Cargo Workspace Initialization & AST Metamodel Crates (`DEAP02-spec-core`).
- **WP-05**: Strongly-Typed AST Slicing Engine & Atomic Agent Dispatch Protocol (`DEAP02-spec-core`).
- **WP-06**: Deterministic Combinatorial Safety & Constraint Synthesizer (`DEAP02-spec-core`).
- **WP-07**: Dual-LLM Air-Gapped Agent Orchestration Harness (`deap-harness`) & CoT Audit Logger (`DEAP02-spec-core`).
- **WP-08**: Parity Verification Harness against DEAP01 Python Oracle (`DEAP01` <-> `DEAP02`).
- **WP-09**: Production Cutover, Packaging & Single Static Binary Release (`DEAP02-spec-core`).

Every Work Package must define:
1. Target repository and workspace path.
2. Formal mechanical entrance gates.
3. Concrete, step-by-step deliverables.
4. Formal mechanical exit gates with reproducible CLI verification commands (`exit code == 0`) and zero regression tolerance.

---

## Acceptance Criteria

### Blueprint Purity & Verification
- [ ] Every file in `docs/architecture/blueprints/` is verified to have exactly zero occurrences of:
  - Munition/combat/vehicle terms: `ESAD`, `squib`, `warhead`, `fuze`, `arrestor`, `standoff`, `wing`, `aileron`, `rudder`.
  - Regulatory agencies: `FAA`, `EASA`, `FDA`, `NHTSA`.
  - Unicode em dashes: `\u2014`.
  - Unresolved markers: `TODO`, `TBD`, `FIXME`, `draft`, `pending`.
- [ ] All 14 blueprints have valid YAML frontmatter with `status: APPROVED / PRODUCTION-GRADE`.

### Master Execution Plan Integrity
- [ ] `docs/architecture/MASTER_EXECUTION_PLAN.md` is authored, committed, and defines the complete cross-repository lifecycle from `deap-compiler-spec` dogfooding to `DEAP02-spec-core` binary compilation.
- [ ] All entrance and exit gates specify exact shell commands with objective pass criteria.

### Baseline Validation
- [ ] `python3 scripts/verify_downstream_baseline.py` exits with code 0 across all checks in `DEAP01-spec-core`.
- [ ] Git working tree clean and synchronized with origin branch.

</USER_REQUEST>


## 2026-10-06T13:28:43Z

Use a full multi-agent team to implement a flawless fix for Issue #422, verify lossless SysML v2 AST round-trip compilation, and distribute the verified fixes across all downstream repositories and leaf nodes.

Working directory: `/Users/perkunas/jail/DEAP01-spec-core`
Integrity mode: development

Reference: https://github.com/gintatkinson/DEAP01-spec-core/issues/422

## Requirements

### R1. Lossless SysML v2 AST Parsing and Serialization
Ensure `skills/spec-orchestrator/scripts/sysmlv2_ast.py` losslessly parses and serializes SysML v2 models without altering semantics or syntax:
- Retain typed part usages (`part x : Type;` and `part x : Type { ... }`) as typed instances (`is_def=False`, `type_name`) rather than degrading them into empty `part def x { }` blocks.
- Preserve the `inout` port direction keyword in `to_sysml()` serialization so bidirectional ports do not become directionless.
- Parse `action def` constructs without defaulting performer/allocation attributes to the parent container, eliminating circular self-performing calls (`perform <ParentPart>;`).

### R2. Resilient Dependency Handling for Tooling and Validators
Guard top-level `import yaml` in `skills/spec-orchestrator/parity_auditor/src/parity_auditor/validators/uml.py` and any related validator modules with fallback handling (`try...except ImportError: yaml = None`) so that baseline verification tools execute cleanly in minimal Python environments.

### R3. Semantic AST Round-Trip Parity Verification Suite
Deliver an automated regression test suite (`tests/test_sysml_compiler_parity.py`) verifying:
- Lossless round-trip parsing and serialization of typed part usages.
- Bidirectional port direction (`inout`) preservation.
- Clean `action def` serialization without recursive self-invocation.
- Full compatibility with existing SysML model compilation pipelines.

### R4. Downstream Distribution and Baseline Conformance Verification
Distribute the updated AST compiler, parity auditor validators, and test tooling to all active downstream repositories and leaf nodes located in `/Users/perkunas/jail/` (including `uav-009`, `uav-011`, `uav-012`, `DEAP-netcom`, `3dgs-040`, `deap-compiler-spec`, `DEAP02-spec-core`):
- Run `python3 scripts/verify_downstream_baseline.py --no-domain` across `DEAP01-spec-core` and every target downstream project.
- Verify zero regressions across all repository quality gates.

### R5. Governance Compliance and Issue Tracker Resolution
- Strictly adhere to the Commit Message Non-Closure Invariant (no auto-closing keywords like `fixes #`; use neutral citations `(refs #422)`).
- Strictly adhere to the Zero Em Dash Invariant (ASCII `--` or `-` exclusively).
- Update GitHub Issue #422 with empirical verification evidence and transition label `status:fixed-resolved`.

## Acceptance Criteria

### AST Round-Trip Parity
- [ ] `part fcc_board : AutopilotController;` parses with `is_def=False` and `type_name="AutopilotController"`, and re-serializes identically to `part fcc_board : AutopilotController;`.
- [ ] `inout port rs485_esad : RS485Port;` retains the `inout` keyword upon `to_sysml()` round-trip serialization.
- [ ] `action def ExecuteFlightGuidance;` parses with empty performer and serializes cleanly without injecting `perform <ParentPart>;`.
- [ ] `tests/test_sysml_compiler_parity.py` passes with 100% assertions green.

### Baseline Conformance
- [ ] `python3 scripts/verify_downstream_baseline.py --no-domain` in `DEAP01-spec-core` exits with code 0 without requiring external PyYAML installation.
- [ ] `python3 scripts/verify_downstream_baseline.py --no-domain` passes with exit code 0 across all updated downstream projects in `/Users/perkunas/jail/`.

### Distribution & Tracker Integrity
- [ ] All target downstream projects in `/Users/perkunas/jail/` have the updated tooling synchronized and git trees clean.
- [ ] All commit messages use neutral citations (`(refs #422)` or `(#422)`) with zero em dashes.
- [ ] Issue #422 has a verification comment and label `status:fixed-resolved`.

---
PROCEED
