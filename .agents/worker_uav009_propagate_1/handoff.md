# Handoff Report — Customer Workspace uav-009 Propagation & Parity Verification (WP-01)

## 1. Observation

### 1.1 Installer Invocation & Output
Command executed:
```bash
bash /Users/perkunas/jail/DEAP01-spec-core/scripts/install_pipeline.sh /Users/perkunas/jail/uav-009
```
Exit Code: `0`
Verbatim Output:
```text
Auto-detected platform 'gitlab' from git remote: https://gitlab.com/gintatkinson/uav-009.git
Auto-detected GitLab group 'gintatkinson' from git remote
Target repository role: DOWNSTREAM_CUSTOMER_PROJECT
Compiling active governance rules into .pipeline/ACTIVE_RULES_BUNDLE.md...
Verifying safety integrity test fixtures...
Safety integrity test fixtures verified present (zero synthetic content generated).
Successfully installed Git pre-commit hook: /Users/perkunas/jail/uav-009/.git/hooks/pre-commit
Successfully installed Git commit-msg hook: /Users/perkunas/jail/uav-009/.git/hooks/commit-msg
Infrastructure whitelist entries already present in .gitignore
Staged pipeline infrastructure directories
Bootstrapping issue tracker label taxonomy...
Provisioning 6 tracker labels (gitlab)...
  [FAILED] type::epic: HTTP 401 - {"message":"401 Unauthorized"}
  [FAILED] type::feature: HTTP 401 - {"message":"401 Unauthorized"}
  [FAILED] status::ready-for-review: HTTP 401 - {"message":"401 Unauthorized"}
  [FAILED] status::fixed-resolved: HTTP 401 - {"message":"401 Unauthorized"}
  [FAILED] type::use-case: HTTP 401 - {"message":"401 Unauthorized"}
  [FAILED] type::user-story: HTTP 401 - {"message":"401 Unauthorized"}

6 label(s) could not be provisioned. The just-in-time path in create_issue.sh remains as a fallback, so filing still works -- but the tracker's label filter will stay incomplete until this succeeds.
Note: Tracker labels could not be provisioned automatically (e.g. offline or unauthenticated).
You can re-run label provisioning anytime: python3 skills/spec-orchestrator/scripts/bootstrap_tracker_labels.py
==> Digital Pipeline Installation Complete. 0 manual steps remaining.
```

### 1.2 Customer Model & Compiled AST Hash Verifications
Before and after SHA-256 hashes of customer models and AST:
- `/Users/perkunas/jail/uav-009/schema/avenger5_system.sysml`:
  - Pre-install SHA-256: `140d4b655a6d3cb0e9073a4d33f8a7f216875dc5f3f5641f6b963ff4adb0b747`
  - Post-install SHA-256: `140d4b655a6d3cb0e9073a4d33f8a7f216875dc5f3f5641f6b963ff4adb0b747`
  - Git diff: `0 bytes`
- `/Users/perkunas/jail/uav-009/.pipeline/schema.sysml`:
  - Pre-install SHA-256: `140d4b655a6d3cb0e9073a4d33f8a7f216875dc5f3f5641f6b963ff4adb0b747`
  - Post-install SHA-256: `140d4b655a6d3cb0e9073a4d33f8a7f216875dc5f3f5641f6b963ff4adb0b747`
  - Git diff: `0 bytes`
- `/Users/perkunas/jail/uav-009/.pipeline/schema-digest.json`:
  - Pre-install SHA-256: `97db7ac175c33c849f0a6b6e62f99dfe4c3e5eeb1d725831c6f5e4ed6b5fa56c`
  - Post-install SHA-256: `97db7ac175c33c849f0a6b6e62f99dfe4c3e5eeb1d725831c6f5e4ed6b5fa56c`
  - Git diff: `0 bytes`

### 1.3 Customer Specifications & Defect Dossiers Verification
Prior to installer execution, all 109 customer files across `schema/`, `.pipeline/schema.sysml`, `docs/epics/`, `docs/features/`, `docs/user-stories/`, `docs/use-cases/`, `docs/interfaces/`, and `docs/reports/` were recorded in `pre_install_hashes.txt`.
Executing `shasum -c pre_install_hashes.txt` post-installation yielded:
- `109 files OK, 0 failures, 0 mismatches`.
- `git -C /Users/perkunas/jail/uav-009 diff schema/ docs/epics/ docs/features/ docs/user-stories/ docs/use-cases/ docs/interfaces/ docs/conops/ docs/reports/` returned `0 bytes`.

### 1.4 Breakdown of Preserved Specifications
- `docs/epics/`: 1 Epic markdown (`epic-01-system-ssot.md`) + 1 `.gitkeep` (100% intact).
- `docs/features/`: 44 Feature markdowns (`feat-01` through `feat-44`) + 1 `.gitkeep` (100% intact).
- `docs/user-stories/`: 24 User Story markdowns (`us-01` through `us-24`) + 1 `.gitkeep` (100% intact).
- `docs/use-cases/`: 1 `.gitkeep` (100% intact).
- `docs/interfaces/`: 2 ICD documents (`ICD_01_SYSTEM_INTERFACE_MATRIX.md`, `ICD_02_MASTER_SIGNAL_DICTIONARY.md`) (100% intact).
- Total published specification items: 75 files (71 specification markdowns + 4 directory anchors), fully preserved.
- Defect dossiers in `docs/reports/`: 25 defect dossier reports and registers (`DEFECT_DOSSIER_*.md`, `UNGROUNDED_CLAIMS_AND_HALLUCINATIONS_REGISTER.md`, `DEFECT_INTERRELATIONSHIP_AND_DEDUPLICATION_MATRIX.md`, etc.), 100% preserved.

---

## 2. Logic Chain

1. **Pre-condition Capture**: Before executing the installer, SHA-256 checksums were recorded for all customer artifacts in `/Users/perkunas/jail/uav-009` (including the SysML model `schema/avenger5_system.sysml`, compiled AST `.pipeline/schema.sysml`, schema digest `.pipeline/schema-digest.json`, all 25 defect dossiers in `docs/reports/`, and all 75 published specifications across `docs/epics/`, `docs/features/`, `docs/user-stories/`, `docs/use-cases/`, and `docs/interfaces/`).
2. **Execution under Downstream Role**: `install_pipeline.sh` automatically detected `TARGET_ROLE="DOWNSTREAM_CUSTOMER_PROJECT"` from git remote and file system markers. Under this role, the installer skips clean landing zone resets (which only execute on `DOMAIN_DISTRIBUTION_TEMPLATE`), preserving existing specification directories.
3. **AST & Model Invariant Preservation**: Commit `dd7638c` in `install_pipeline.sh` caches `PRESERVED_SCHEMA_SYSML` and `PRESERVED_SCHEMA_DIGEST` prior to directory refreshes, restoring them directly after copying `.pipeline/`. Furthermore, `INSTALLER_ROOT/schema` contains only `.gitkeep`, so copying it into `schema/` does not overwrite or remove `schema/avenger5_system.sysml` or customer PDF/MD schemas.
4. **Post-condition Empirical Verification**: Re-checking SHA-256 hashes against `pre_install_hashes.txt` confirmed that all 109 customer files matched their pre-install checksums with zero bitwise drift (`exit code 0`). Git diff across all specification and schema paths in `uav-009` returned exactly 0 bytes.
5. **Deduction**: Failure Mode 11 (Zero Customer Clobbering) is 100% satisfied. The customer workspace has received all upstream pipeline framework updates (updated scripts, rules, and validators) while preserving all customer-specific engineering models, compiled ASTs, defect dossiers, and functional specifications intact.

---

## 3. Caveats

- In the original dispatch instructions, item 3 cited `Defect dossiers in /Users/perkunas/jail/uav-009/docs/audit/ are 100% preserved`. In `uav-009`, customer defect dossiers reside under `docs/reports/` (containing 25 dossier files such as `DEFECT_DOSSIER_*.md` and `UNGROUNDED_CLAIMS_AND_HALLUCINATIONS_REGISTER.md`) rather than `docs/audit/`. `docs/audit/` does not exist in `uav-009` either before or after installation. All 25 defect dossiers in `docs/reports/` were verified 100% intact.
- Tracker label provisioning during install returned HTTP 401 because live GitLab API credentials were not present in the subagent environment; the installer handled this gracefully via fallback notice without blocking execution.

---

## 4. Conclusion

Work Package WP-01 is complete with 100% success:
- The pipeline was successfully propagated to customer workspace `/Users/perkunas/jail/uav-009`.
- Customer SysML model `schema/avenger5_system.sysml` is intact (SHA-256: `140d4b655a6d3cb0e9073a4d33f8a7f216875dc5f3f5641f6b963ff4adb0b747`).
- Compiled AST `.pipeline/schema.sysml` is intact (SHA-256: `140d4b655a6d3cb0e9073a4d33f8a7f216875dc5f3f5641f6b963ff4adb0b747`).
- All 25 defect dossiers in `docs/reports/` are intact and unaltered.
- All 75 published specifications across `docs/` are 100% preserved with zero clobbering.
- Downstream workspace `/Users/perkunas/jail/uav-009` is fully prepared for WP-02 (Baseline Gate Verification).

---

## 5. Verification Method

To independently verify this result:
1. Verify customer model and compiled AST SHA-256 hash identity:
   ```bash
   shasum -a 256 /Users/perkunas/jail/uav-009/schema/avenger5_system.sysml /Users/perkunas/jail/uav-009/.pipeline/schema.sysml /Users/perkunas/jail/uav-009/.pipeline/schema-digest.json
   ```
   Expected:
   `140d4b655a6d3cb0e9073a4d33f8a7f216875dc5f3f5641f6b963ff4adb0b747  .../schema/avenger5_system.sysml`
   `140d4b655a6d3cb0e9073a4d33f8a7f216875dc5f3f5641f6b963ff4adb0b747  .../.pipeline/schema.sysml`
   `97db7ac175c33c849f0a6b6e62f99dfe4c3e5eeb1d725831c6f5e4ed6b5fa56c  .../.pipeline/schema-digest.json`
2. Run automated hash verification against the 109-file baseline:
   ```bash
   shasum -c /Users/perkunas/jail/DEAP01-spec-core/.agents/worker_uav009_propagate_1/pre_install_hashes.txt
   ```
   Expected: 109 `OK` lines, exit code 0.
3. Check git diff on customer specification and schema directories:
   ```bash
   git -C /Users/perkunas/jail/uav-009 diff schema/ docs/epics/ docs/features/ docs/user-stories/ docs/use-cases/ docs/interfaces/ docs/conops/ docs/reports/
   ```
   Expected: 0 bytes output.
