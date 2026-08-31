# Claude Code Project Guidelines

## Primary Commercial Toolchain Integration Context
This project explicitly declares MATLAB / Simulink / Stateflow / Embedded Coder as the Primary Tier-1 Commercial Toolchain Integration Context (Model-Based Design, Control Law Synthesis, DO-178C C/SPARK Ada code generation).

## Workflow & Quality Gates
- Follow all pipeline rules in `rules/` and skills in `skills/` and `.agents/skills/`.
- Strict Planning Gate: Do not execute unauthorized modifications without an approved implementation plan.
- Execute baseline verification: `pytest tests/test_baseline.py` and `python3 scripts/verify_downstream_baseline.py --no-domain`.
