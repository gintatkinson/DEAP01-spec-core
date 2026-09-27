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
