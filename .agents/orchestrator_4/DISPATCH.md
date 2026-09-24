## 2026-09-21T16:32:10Z

You are the Project Orchestrator for DEAP01-spec-core.

Identity: orchestrator
Working directory: /Users/perkunas/jail/DEAP01-spec-core/.agents/orchestrator_4
Parent Sentinel: 53a83729-d570-4f8b-8add-4510519cda78
Repository Classification: UPSTREAM_SPEC_CORE_COMPILER
Active Workspace: /Users/perkunas/jail/DEAP01-spec-core

Your task is to orchestrate the implementation of the user request recorded under the latest timestamp header in /Users/perkunas/jail/DEAP01-spec-core/.agents/ORIGINAL_REQUEST.md:
"Overhaul the installation instructions and README templates across all three repository tiers to eliminate broken, contradictory, and circular onboarding instructions:
1. Upstream Spec Core Compiler (DEAP01-spec-core)
2. Domain Distribution Templates (DEAP-*, e.g. DEAP-uas-infrastructure-safety)
3. Customer Application Workspaces (uav-*, e.g. uav-011)"

Requirements to orchestrate:
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

### Acceptance Criteria
- [ ] DEAP01-spec-core/README.md contains zero inline multi-line Python scripts and zero hardcoded domain repository clone commands in its installation sections.
- [ ] Scaffolding in scripts/install_pipeline.sh detects repository role dynamically and generates distinct, accurate READMEs for Domain Templates vs Customer Workspaces with zero circular clone commands.
- [ ] python3 scripts/verify_downstream_baseline.py --no-domain passes cleanly in DEAP01-spec-core.
- [ ] All code fences contain pure, executable shell syntax with zero unescaped parentheses in comments and zero unquoted angle-bracket placeholders.

Follow all repository rules in /Users/perkunas/jail/DEAP01-spec-core/AGENTS.md and .agents/AGENTS.md:
1. Strict Planning Gate: Update /Users/perkunas/jail/DEAP01-spec-core/implementation_plan.md covering all requirements (R1, R2, R3) and verification steps. User prompt explicitly contains "PROCEED", fully authorizing continuous execution through documented work packages.
2. Maintain your own BRIEFING.md and progress.md in your working directory (/Users/perkunas/jail/DEAP01-spec-core/.agents/orchestrator_4/).
3. Decompose the work packages and dispatch context-isolated subagents for exploration, implementation, review, and verification. Do not write target source code or functional specifications directly.
4. When all requirements and verification steps are complete, report victory back to the parent sentinel.

PROCEED
