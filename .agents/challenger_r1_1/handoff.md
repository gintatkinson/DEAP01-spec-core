# Handoff Report: Empirical Stress-Test of Schema Copy Logic in `scripts/install_pipeline.sh`

**Verdict:** `REQUEST_CHANGES`  
**Overall Risk Assessment:** `MEDIUM`  
**Role:** Challenger 1 (Empirical Challenger / Adversarial Critic)  
**Target:** `scripts/install_pipeline.sh` (lines 283–286)

---

## 1. Observation

### Code Under Review
In `/Users/perkunas/jail/DEAP01-spec-core/scripts/install_pipeline.sh`, lines 283–286:
```bash
283:   mkdir -p "$TARGET_DIR/schema"
284:   if [ -d "$INSTALLER_ROOT/schema" ]; then
285:     cp -RP "$INSTALLER_ROOT/schema/." "$TARGET_DIR/schema/"
286:   fi
```
Line 2 of `scripts/install_pipeline.sh`:
```bash
2: set -e
```

### Direct Empirical Findings & Raw Tool Outputs

1. **Edge Case 1: Pre-existing `schema/` directory containing `.gitkeep`**
   - **Setup:** Target project initialized with `schema/.gitkeep`. Source repository (domain template) contains `schema/.gitkeep`, `schema/DEAP_MODEL.sysml`, and `schema/telemetry.proto`.
   - **Command:** `bash scripts/install_pipeline.sh $TARGET_DIR`
   - **Result:** Exit code `0`. Target `schema/` contains `['.gitkeep', 'telemetry.proto', 'DEAP_MODEL.sysml']`.
   - **Nested check:** `os.path.exists("$TARGET_DIR/schema/schema")` returned `False`.

2. **Edge Case 2: Pre-existing `schema/` directory containing custom schema files**
   - **Setup:** Target project contains pre-existing `schema/my_custom_payload.sysml`. Source repository contains `schema/DEAP_MODEL.sysml` and `schema/domain_types.sysml`.
   - **Command:** `bash scripts/install_pipeline.sh $TARGET_DIR`
   - **Result:** Exit code `0`.
   - **Preservation check:** `my_custom_payload.sysml` remained intact and uncorrupted in `$TARGET_DIR/schema/`.
   - **Copy check:** Both `DEAP_MODEL.sysml` and `domain_types.sysml` copied into `$TARGET_DIR/schema/`.
   - **Nested check:** `os.path.exists("$TARGET_DIR/schema/schema")` returned `False`.

3. **Edge Case 3: Nested subdirectories in `schema/`**
   - **Setup:** Target contains `schema/custom/nested/custom.proto` and `schema/telemetry/v1/custom_extra.proto`. Source contains `schema/telemetry/v1/messages.proto` and `schema/root_model.sysml`.
   - **Command:** `bash scripts/install_pipeline.sh $TARGET_DIR`
   - **Result:** Exit code `0`.
   - **Merge check:** Target `schema/telemetry/v1/` contains both `messages.proto` and `custom_extra.proto`.
   - **Preservation check:** `schema/custom/nested/custom.proto` preserved.
   - **Nested check:** No duplicate path segments created (`$TARGET_DIR/schema/schema` is `False`).

4. **Edge Case 4: Read-only files and special attributes**
   - **Sub-case 4A (Target has read-only file with DIFFERENT name):**
     - Target has `schema/readonly_custom.sysml` with permission mode `0444`. Source has `schema/DEAP_MODEL.sysml`.
     - Result: Exit code `0`. Both files present in `$TARGET_DIR/schema/`.
   - **Sub-case 4B (Target has read-only file with SAME name as incoming source file):**
     - Target has `schema/DEAP_MODEL.sysml` with mode `0444`. Source has updated `schema/DEAP_MODEL.sysml`.
     - Result: **CRASH** (Exit code `1`).
     - Verbatim stderr:
       ```
       cp: /private/var/folders/1g/l0zx9f054xn2vkc4944jzzpr0000gp/T/test4b_dst_h8r9edai/schema/./DEAP_MODEL.sysml: Permission denied
       ```
   - **Sub-case 4B-Real (Direct real installer run on upstream repository's own `.gitkeep`):**
     - Target repository has `schema/.gitkeep` with mode `0444` (standard read-only checkout).
     - Command: `bash /Users/perkunas/jail/DEAP01-spec-core/scripts/install_pipeline.sh $TARGET_DIR`
     - Result: **CRASH** (Exit code `1`).
     - Verbatim stderr:
       ```
       cp: /private/var/folders/1g/l0zx9f054xn2vkc4944jzzpr0000gp/T/real_test_ro_gitkeep_ztfixf3i/schema/./.gitkeep: Permission denied
       ```
   - **Sub-case 4C (Target `schema/` directory itself has read-only permissions `0555`):**
     - Target has `chmod 555 $TARGET_DIR/schema`.
     - Result: **CRASH** (Exit code `1`).
     - Verbatim stderr:
       ```
       cp: /private/var/folders/1g/l0zx9f054xn2vkc4944jzzpr0000gp/T/test4c_dst_j2gh96cn/schema/./DEAP_MODEL.sysml: Permission denied
       ```

5. **Edge Case 5: Idempotency & Repeated Executions**
   - Executing `install_pipeline.sh` 3 consecutive times on the same target directory succeeds cleanly on every run (`exit code 0`), with zero schema nesting on runs 1, 2, or 3.

6. **Mitigation Validation:**
   - Replacing line 285 with:
     ```bash
     chmod -R u+w "$TARGET_DIR/schema" 2>/dev/null || true
     cp -RPf "$INSTALLER_ROOT/schema/." "$TARGET_DIR/schema/"
     ```
   - Result: Both Sub-case 4B and Sub-case 4C pass with exit code `0`, successfully unlinking/overwriting read-only existing files while preserving distinct customer files.

---

## 2. Logic Chain

1. **Observation 1 & Observation 2 demonstrate that the trailing `/.` syntax in `cp -RP "$INSTALLER_ROOT/schema/." "$TARGET_DIR/schema/"` correctly resolves R3**:
   - Because `schema/.` copies directory contents rather than the directory container itself, it copies domain schemas into `$TARGET_DIR/schema/` without creating `$TARGET_DIR/schema/schema/`, even when `$TARGET_DIR/schema/` already exists with `.gitkeep` or custom schemas.
2. **Observation 2 and 3 show that custom schemas and nested directory hierarchies are safely preserved**:
   - Because `$TARGET_DIR/schema` is not purged with `rm -rf` (unlike `skills/`, `rules/`, etc.), customer schemas are not destroyed during installation or updates.
3. **Observation 4B and 4B-Real reveal a critical failure mode under adverse file permission boundary conditions**:
   - POSIX/BSD `cp` without the `-f` flag defaults to opening existing destination files with `O_WRONLY | O_TRUNC`. When a destination file with the same name (such as `.gitkeep` or a previously installed domain model) has read-only mode (`0444`), `open()` fails with `EACCES` (`Permission denied`).
   - Because line 2 specifies `set -e`, this error halts `install_pipeline.sh` immediately, leaving the target repository in a partially installed state.
   - This occurs in real-world scenarios such as CI/CD environments where checkout trees are made read-only, shared repositories (`core.sharedRepository`), or restrictive umasks.
4. **Observation 4C reveals that if the target `schema/` directory itself is non-writable (`0555`), `cp` cannot write into it**:
   - `mkdir -p "$TARGET_DIR/schema"` succeeds silently on an existing directory even if read-only, but the subsequent `cp` command immediately errors.
5. **Observation 6 demonstrates that adding `-f` to `cp -RP` (and ensuring write permission on `$TARGET_DIR/schema`) completely closes this failure mode**:
   - `cp -f` unlinks destination files that cannot be opened for writing before copying, allowing clean replacement of read-only files owned by the user.

---

## 3. Caveats

- Tests were conducted with destination files owned by the active user executing `install_pipeline.sh`. If destination files or directories are owned by a different user (`root` or another UID) and lack write permissions, user-level `chmod` and `cp -f` cannot override OS access controls without `sudo`.
- Extended access control lists (NFSv4/macOS ACLs) and immutable file flags (`chflags uchg`) were observed to block writes regardless of standard POSIX permissions unless cleared by the user.
- No other caveats.

---

## 4. Conclusion & Verdict

**Verdict:** `REQUEST_CHANGES`

While the syntax change from `cp -RP "$INSTALLER_ROOT/schema" "$TARGET_DIR/"` to `cp -RP "$INSTALLER_ROOT/schema/." "$TARGET_DIR/schema/"` successfully eliminates directory duplication (`$TARGET_DIR/schema/schema/`) and cleanly handles existing `.gitkeep` and custom files, it is vulnerable to failure under adverse permission conditions.

Specifically:
- Line 285 lacks the force flag (`-f`). If `$TARGET_DIR/schema/.gitkeep` or any domain model file exists with read-only permissions (`0444`), `install_pipeline.sh` crashes with `Permission denied`.
- Furthermore, ensuring write permissions on `$TARGET_DIR/schema` before copying guards against read-only directory states.

### Required Changes in `scripts/install_pipeline.sh`

Lines 283–286 should be updated to:
```bash
  mkdir -p "$TARGET_DIR/schema"
  chmod -R u+w "$TARGET_DIR/schema" 2>/dev/null || true
  if [ -d "$INSTALLER_ROOT/schema" ]; then
    cp -RPf "$INSTALLER_ROOT/schema/." "$TARGET_DIR/schema/"
  fi
```

---

## 5. Verification Method

### Test Reproduction Command

Run this self-contained Python stress harness from any temporary location:

```bash
python3 -c "
import tempfile, subprocess, os

with tempfile.TemporaryDirectory(prefix='repro_ro_') as dst:
    subprocess.run(['git', 'init', '-q'], cwd=dst, check=True)
    os.makedirs(os.path.join(dst, 'schema'), exist_ok=True)
    gitkeep_file = os.path.join(dst, 'schema', '.gitkeep')
    with open(gitkeep_file, 'w') as f:
        f.write('gitkeep\n')
    os.chmod(gitkeep_file, 0o444)
        
    script = '/Users/perkunas/jail/DEAP01-spec-core/scripts/install_pipeline.sh'
    res = subprocess.run(['bash', script, dst], capture_output=True, text=True)
    print('Return code:', res.returncode)
    print('Stderr:', res.stderr.strip())
    assert res.returncode == 0, 'Reproduction confirmed: install_pipeline.sh failed on read-only .gitkeep'
"
```

### Invalidation Conditions
This challenge is invalidated if:
1. `scripts/install_pipeline.sh` is updated with `cp -RPf` (and directory write-permission assurance), and the reproduction command above returns exit code `0`.
2. All 5 edge cases pass without errors, without directory nesting, and with zero file loss.

---

## Stress Test Results Summary

| Test Scenario | Input State | Expected Behavior | Actual Behavior | Status |
|---|---|---|---|---|
| **Scenario 1** | Target has `schema/.gitkeep` | Schemas copied to `schema/`, no `schema/schema/` | Clean copy, zero nesting | **PASS** |
| **Scenario 2** | Target has custom `schema/*.sysml` | Custom schemas preserved, domain schemas added | Custom preserved, domain added | **PASS** |
| **Scenario 3** | Target & Source have nested subdirs | Subdirs merged cleanly without double-nesting | Merged cleanly, zero nesting | **PASS** |
| **Scenario 4A** | Target has read-only file (different name) | Preserved without error | Preserved, exit code 0 | **PASS** |
| **Scenario 4B** | Target has read-only file (same name, e.g. `.gitkeep`) | Overwritten cleanly | **CRASH: Permission denied (exit code 1)** | **FAIL** |
| **Scenario 4C** | Target `schema/` directory is mode `0555` | Made writable and schemas copied | **CRASH: Permission denied (exit code 1)** | **FAIL** |
| **Scenario 5** | Multiple consecutive installer runs | Idempotent, zero nesting across runs | Idempotent across 3 runs, zero nesting | **PASS** |
