# DISPATCH

## 2026-09-27T07:34:07Z

Execute view_file on /Users/perkunas/jail/DEAP01-spec-core/skills/spec-orchestrator/SKILL.md as your very first step before executing any file edits or commands, and strictly follow its formatting templates and instruction guidelines.

Repository Classification: UPSTREAM_SPEC_CORE_COMPILER
Working directory: /Users/perkunas/jail/DEAP01-spec-core/.agents/worker_uav011_propagate_1
Parent Orchestrator ID: d0acf8cb-0b5f-428f-bbe8-5c9e60d7dedc
Target Workspace: /Users/perkunas/jail/uav-011
Primary Commercial Toolchain Integration Context: MATLAB / Simulink / Stateflow / Embedded Coder

Read the authoritative user request from /Users/perkunas/jail/DEAP01-spec-core/.agents/ORIGINAL_REQUEST.md.

Task Objective (WP-04):
Execute pipeline propagation to application workspace /Users/perkunas/jail/uav-011 and verify clean landing zones.

Instructions:
1. Read /Users/perkunas/jail/DEAP01-spec-core/.agents/ORIGINAL_REQUEST.md.
2. Execute the installer:
   bash /Users/perkunas/jail/DEAP01-spec-core/scripts/install_pipeline.sh /Users/perkunas/jail/uav-011
3. Verify that landing zones maintain 100% clean .gitkeep state in /Users/perkunas/jail/uav-011:
   - schema/ contains only .gitkeep
   - docs/epics/ contains only .gitkeep
   - docs/features/ contains only .gitkeep
   - docs/user-stories/ contains only .gitkeep
   - docs/use-cases/ contains only .gitkeep
4. Record your findings, command outputs, and verification results in /Users/perkunas/jail/DEAP01-spec-core/.agents/worker_uav011_propagate_1/handoff.md following the Handoff Protocol.
5. Send message to Parent Orchestrator (d0acf8cb-0b5f-428f-bbe8-5c9e60d7dedc) when done.

DO NOT CHEAT. All implementations must be genuine. DO NOT hardcode test results, create dummy/facade implementations, or circumvent the intended task. A teamwork_preview_auditor will independently verify your work. Integrity violations WILL be detected and your work WILL be rejected.

PROCEED
