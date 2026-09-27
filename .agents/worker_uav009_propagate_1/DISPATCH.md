## 2026-09-27T07:07:52Z

Execute view_file on /Users/perkunas/jail/DEAP01-spec-core/skills/spec-orchestrator/SKILL.md as your very first step before executing any file edits or commands, and strictly follow its formatting templates and instruction guidelines.

Repository Classification: UPSTREAM_SPEC_CORE_COMPILER
Working directory: /Users/perkunas/jail/DEAP01-spec-core/.agents/worker_uav009_propagate_1
Parent Orchestrator ID: d0acf8cb-0b5f-428f-bbe8-5c9e60d7dedc
Target Workspace: /Users/perkunas/jail/uav-009
Primary Commercial Toolchain Integration Context: MATLAB / Simulink / Stateflow / Embedded Coder

Read the authoritative user request from /Users/perkunas/jail/DEAP01-spec-core/.agents/ORIGINAL_REQUEST.md.

Task Objective (WP-01):
Execute pipeline propagation from upstream DEAP01-spec-core to customer workspace /Users/perkunas/jail/uav-009 and verify that customer assets are 100% preserved with zero clobbering.

Instructions:
1. Read /Users/perkunas/jail/DEAP01-spec-core/.agents/ORIGINAL_REQUEST.md.
2. Execute the pipeline installer:
   bash /Users/perkunas/jail/DEAP01-spec-core/scripts/install_pipeline.sh /Users/perkunas/jail/uav-009
3. In strict adherence to Failure Mode 11 (Zero Customer Clobbering), empirically verify in /Users/perkunas/jail/uav-009:
   - Customer model /Users/perkunas/jail/uav-009/schema/avenger5_system.sysml exists and is intact (check git diff or hash).
   - Compiled AST /Users/perkunas/jail/uav-009/.pipeline/schema.sysml exists and is intact.
   - Defect dossiers in /Users/perkunas/jail/uav-009/docs/audit/ are 100% preserved.
   - All 75 published specifications in /Users/perkunas/jail/uav-009/docs/ (docs/epics/, docs/features/, docs/user-stories/, docs/use-cases/) are 100% preserved with zero clobbering.
4. Record your findings, command outputs, and file check verifications in /Users/perkunas/jail/DEAP01-spec-core/.agents/worker_uav009_propagate_1/handoff.md following the Handoff Protocol (Observation, Logic Chain, Caveats, Conclusion, Verification Method).
5. Send a message to Parent Orchestrator (Recipient: d0acf8cb-0b5f-428f-bbe8-5c9e60d7dedc) when done.

DO NOT CHEAT. All implementations must be genuine. DO NOT hardcode test results, create dummy/facade implementations, or circumvent the intended task. A teamwork_preview_auditor will independently verify your work. Integrity violations WILL be detected and your work WILL be rejected.

PROCEED
