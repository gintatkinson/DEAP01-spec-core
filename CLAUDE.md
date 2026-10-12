# Claude Code Project Guidelines

## Workflow & Quality Gates
- Follow all pipeline rules by executing `view_file` on `.pipeline/ACTIVE_RULES_BUNDLE.md`, and skills in `skills/` and `.agents/skills/`.
- Strict Planning Gate: Do not execute unauthorized modifications without an approved implementation plan.
- Execute baseline verification: `./target/release/verify-baseline . --no-domain`.
- Compile SysML v2 models: `./target/release/compile-sysml --compile`.
- Reconcile backlog specifications: `./target/release/reconcile-backlog`.
- Ingest OEM schemas/documents: `./target/release/ingest-sysml --schema schema/ --format markdown --out schema/model.sysml`.
- Assemble ConOps & Mission Intent: `./target/release/assemble-conops --input-dir docs/conops/units/ --output-dir docs/conops/`.
- Follow the universal initialization sequence in Section 3.3 of README.md and execute view_file on .pipeline/ACTIVE_RULES_BUNDLE.md before starting any task.
