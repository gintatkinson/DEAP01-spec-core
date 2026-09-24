# Handoff Report: Adversarial Code Audit on Downstream Onboarding & Rule-Ingestion Pipeline

## 1. Observation
- `scripts/install_pipeline.sh:360`: Copies the rules directory as raw unbundled files (`cp -RPf "$INSTALLER_ROOT/rules" "$TARGET_DIR/"`) without compiling or aggregating them into a single manifest (`.pipeline/ACTIVE_RULES_BUNDLE.md`).
- `scripts/install_pipeline.sh:809` and `scripts/install_pipeline.sh:890`: The scaffolded initialization sequence instructs AI agents:
  `3. **Load Governance Rules**: Ingest \`AGENTS.md\` and \`rules/\` to enforce project-scoped agentic rules, context-isolated subagent dispatch loops, and role boundary locks.`
- `scripts/install_pipeline.sh:1326`: Worker 2B prompt preamble selectively directs:
  `Adopt the feature-driven-implementation skill by reading \`.pipeline/constitution.md\`, \`rules/dual-track-mbd-verification.md\`, and \`docs/architecture/blueprints/SYSML_SSOT_BIDIRECTIONAL_SYNCHRONIZATION_ARCHITECTURE.md\`.`
- `rules/` directory: Contains 21 individual governance files (20 markdown files + 1 json file).
- `tests/test_readme_scaffolding.py`: Contains tests for role classifications and code fence hygiene, but lacks assertions checking for rule bundle generation or complete rule ingestion.
- Pre-submission dry-run execution:
  ```bash
  python3 scripts/file_defect.py --body-file /Users/perkunas/jail/DEAP01-spec-core/.agents/auditor_r1_6/defect_dossier.md --dry-run --title "Tooling Defect: Downstream Onboarding Rule-Shortcutting & Lack of Consolidated Rule Ingestion Bundle"
  ```
  Result: Exit code 0, output `[DRY RUN] Defect validation PASSED. Target payload: Provider: github, Repo: gintatkinson/DEAP01-spec-core, Label: bug`.
- Parity auditor offline Mermaid syntax check (Step D Check 7):
  ```bash
  python3 - "/Users/perkunas/jail/DEAP01-spec-core/.agents/auditor_r1_6/defect_dossier.md" <<'EOF'
  import sys, os
  sys.path.insert(0, "skills/spec-orchestrator/parity_auditor/src")
  from parity_auditor.validators.mermaid_syntax_validator import check_mermaid_text
  body = open(sys.argv[1], encoding="utf-8").read()
  errors = check_mermaid_text(body, source=os.path.basename(sys.argv[1]))
  assert not errors
  print("check 7 passed")
  EOF
  ```
  Result: Exit code 0, `check 7 passed`.

## 2. Logic Chain
1. In Step 3 of the downstream initialization sequence (`scripts/install_pipeline.sh:809, 890`), AI agents are instructed to ingest `rules/`.
2. Agent tooling (`view_file`) does not support reading directories; attempting `view_file` on `rules/` produces an error.
3. Agents fall back to `list_dir`, which reveals 21 separate rule files.
4. Sequentially reading 21 separate files requires 21 tool call round trips, exhausting token budgets and conversation turns.
5. Under token-conservation bias and anchoring heuristics, LLM agents sample only 1 or 2 files (e.g. `rules/role-boundary-lock.md`), or rely only on selectively cited rules (e.g. `rules/dual-track-mbd-verification.md` in prompt line 1326), assuming other rules are redundant.
6. Crucial domain and platform invariants (such as `rules/sysml-ssot-completeness.md`, `rules/tdd-mandate.md`, and `rules/user-authorization-lock.md`) are silently dropped from agent context.
7. Because `scripts/install_pipeline.sh` lacks an installation-time compilation pass that produces `.pipeline/ACTIVE_RULES_BUNDLE.md`, downstream customer projects have no atomic target for complete rule ingestion.
8. Therefore, the onboarding and rule-ingestion pipeline suffers from a confirmed Semantic Traceability defect with SEVERITY: Important.

## 3. Caveats
- No changes to implementation code were made during this turn, strictly adhering to the audit role boundary lock.
- Defect filing was verified via `--dry-run` and offline parity check 7; live submission to GitHub tracker will be executed by the designated worker subagent per `implementation_plan.md` Work Package 2.

## 4. Conclusion
The downstream onboarding and rule-ingestion pipeline exhibits a critical governance gap caused by open-ended folder directives, selective prompt citations, and the absence of an installation-time consolidated rule bundle. A complete, verified 7-section defect dossier has been generated and saved to:
`/Users/perkunas/jail/DEAP01-spec-core/.agents/auditor_r1_6/defect_dossier.md`

All 12 schema and syntax checks (including offline Mermaid validation and dry-run defect verification) passed with zero violations.

## 5. Verification Method
1. Verify defect dossier schema compliance:
   ```bash
   python3 scripts/file_defect.py --body-file /Users/perkunas/jail/DEAP01-spec-core/.agents/auditor_r1_6/defect_dossier.md --dry-run --title "Tooling Defect: Downstream Onboarding Rule-Shortcutting & Lack of Consolidated Rule Ingestion Bundle"
   ```
2. Verify offline Mermaid syntax:
   ```bash
   python3 - "/Users/perkunas/jail/DEAP01-spec-core/.agents/auditor_r1_6/defect_dossier.md" <<'EOF'
   import sys, os
   sys.path.insert(0, "skills/spec-orchestrator/parity_auditor/src")
   from parity_auditor.validators.mermaid_syntax_validator import check_mermaid_text
   body = open(sys.argv[1], encoding="utf-8").read()
   errors = check_mermaid_text(body, source=os.path.basename(sys.argv[1]))
   assert not errors, f"Errors: {errors}"
   print("check 7 passed")
   EOF
   ```
3. Inspect dossier file:
   `/Users/perkunas/jail/DEAP01-spec-core/.agents/auditor_r1_6/defect_dossier.md`
