# Claude Code Project Guidelines

## Primary Commercial Toolchain Integration Context
This project explicitly declares MATLAB / Simulink / Stateflow / Embedded Coder as the Primary Tier-1 Commercial Toolchain Integration Context (Model-Based Design, Control Law Synthesis, DO-178C C/SPARK Ada code generation).

## Workflow & Quality Gates
- Follow all pipeline rules in `rules/` and skills in `skills/` and `.agents/skills/`.
- Strict Planning Gate: Do not execute unauthorized modifications without an approved implementation plan.
- Execute baseline verification: `cargo test --workspace` and `./target/release/verify-baseline . --no-domain` (or `cargo run --release -p verify-baseline -- . --no-domain`).
- Root `AGENTS.md` is the authoritative agent instruction entrypoint; follow § *Mandatory Native Rust TDD & Prohibition of Legacy Python/Pytest*. Legacy Python/shell tooling (`pytest`, `scripts/verify_downstream_baseline.py`, `reconcile_backlog.py`, `parity_auditor`) is retired per Check 32 and Check 33; use `./target/release/{verify-baseline, reconcile-backlog, compile-sysml, ingest-sysml, assemble-conops}`.
