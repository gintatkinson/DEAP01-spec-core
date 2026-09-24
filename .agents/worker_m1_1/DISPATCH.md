## 2026-09-21T16:38:15Z

Execute `view_file` on `/Users/perkunas/jail/DEAP01-spec-core/skills/feature-driven-implementation/SKILL.md` as your very first step before executing any file edits or commands, and strictly follow its formatting templates and instruction guidelines.

Repository Classification: UPSTREAM_SPEC_CORE_COMPILER
Working directory: /Users/perkunas/jail/DEAP01-spec-core/.agents/worker_m1_1

Primary Commercial Toolchain Integration Context: All subagent dispatches for control law synthesis, safety statechart modeling, and DO-178C C/SPARK Ada code generation MUST cite MATLAB / Simulink / Stateflow / Embedded Coder as the primary commercial toolchain integration context.

Exclusive Write Ownership: You exclusively own `/Users/perkunas/jail/DEAP01-spec-core/README.md`. You MUST NOT write to any other repository source or script file.

Inputs to read:
- `/Users/perkunas/jail/DEAP01-spec-core/.agents/ORIGINAL_REQUEST.md` (specifically section ## 2026-09-21T16:32:10Z § R1)
- `/Users/perkunas/jail/DEAP01-spec-core/.agents/explorer_r4_1/handoff.md`
- `/Users/perkunas/jail/DEAP01-spec-core/.agents/explorer_r4_1/report.md`
- `/Users/perkunas/jail/DEAP01-spec-core/.agents/orchestrator_4/PROJECT.md`

Objective: Implement Milestone 1 (R1) in `/Users/perkunas/jail/DEAP01-spec-core/README.md`:
1. Purge Broken Manual Snippets: Delete Section 5.4's 90-line manual cp loops and inline Python monkeypatching scripts (lines 273–362).
2. Compiler-Centric Focus: Section 5 must clearly document:
   - How to run and verify the compiler itself:
     `python3 scripts/compile_sysml.py --compile`
     `python3 -m pytest tests/`
     `python3 scripts/verify_downstream_baseline.py --no-domain`
   - Clean maintainer commands to propagate compiler tooling into a domain distribution template repository:
     Local propagation: `bash scripts/install_pipeline.sh "<path-to-domain-template>"`
     Remote bootstrap: `git clone https://github.com/gintatkinson/DEAP01-spec-core.git /tmp/deap_compiler && bash /tmp/deap_compiler/scripts/install_pipeline.sh . && rm -rf /tmp/deap_compiler`
     In-place update: `bash scripts/install_pipeline.sh .`
3. Remove Conflated Domain Content:
   - Remove hardcoded customer onboarding commands cloning domain repositories (such as `git clone "https://github.com/gintatkinson/DEAP-uas-infrastructure-safety.git" ...` in Section 5.3).
   - Clarify the architectural boundary: end-user customer projects (`uav-*`) clone from domain distribution templates (`DEAP-*`), not from the upstream compiler.
4. Syntax and Code Fence Hygiene:
   - Ensure all code fences contain pure, valid shell syntax with zero unescaped parentheses in comments and zero unquoted angle-bracket placeholders (always quote `"<path-to-domain-template>"`).
5. Run verification commands:
   - `python3 scripts/verify_downstream_baseline.py --no-domain`
   - `python3 -m pytest tests/`
   - Ensure all checks pass cleanly.

MANDATORY INTEGRITY WARNING:
DO NOT CHEAT. All implementations must be genuine. DO NOT hardcode test results, create dummy/facade implementations, or circumvent the intended task. A teamwork_preview_auditor will independently verify your work. Integrity violations WILL be detected and your work WILL be rejected.

Write your changes report to `/Users/perkunas/jail/DEAP01-spec-core/.agents/worker_m1_1/changes.md` and handoff report to `/Users/perkunas/jail/DEAP01-spec-core/.agents/worker_m1_1/handoff.md`.

PROCEED
