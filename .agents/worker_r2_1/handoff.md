# Handoff Report: Robustness Hardening for Adverse Permissions in `scripts/install_pipeline.sh` and `scripts/scaffold_downstream_agents.py`

**Type:** Hard Handoff (Task Complete)  
**Agent:** Worker R2_1 (implementer, qa, specialist)  
**Target Files:**
- `/Users/perkunas/jail/DEAP01-spec-core/scripts/install_pipeline.sh`
- `/Users/perkunas/jail/DEAP01-spec-core/scripts/scaffold_downstream_agents.py`
**Working Directory:** `/Users/perkunas/jail/DEAP01-spec-core/.agents/worker_r2_1`  

---

## 1. Observation

1. **Vulnerability Identified in Iteration 1 & Audited in Iteration 2:**
   Challenger 1 and Explorer R2_1 identified that when a matching target file (such as `.gitkeep`, `.gitignore`, `OPERATOR_PROMPT_CATALOG.md`, `.gitlab-ci.yml`, `.env.template`, `AGENTS.md`, `README.md`) or directory has read-only mode (`0444` or `0555`), `scripts/install_pipeline.sh` crashes under `set -e` with:
   ```
   cp: ...: Permission denied
   ```
   or:
   ```
   PermissionError: [Errno 13] Permission denied: '.../AGENTS.md'
   ```
   or shell redirection error:
   ```
   line XXX: ...: Permission denied
   ```

2. **Implemented Modifications in `scripts/scaffold_downstream_agents.py`:**
   In `scaffold_downstream_agents()` lines 95–110, defensive write-permission restoration was implemented before opening files:
   ```python
   target_dot_agents_dir = os.path.join(target_dir, ".agents")
   if os.path.exists(target_dot_agents_dir):
       try:
           os.chmod(target_dot_agents_dir, 0o755)
       except OSError:
           pass
   os.makedirs(target_dot_agents_dir, exist_ok=True)

   dot_agents_path = os.path.join(target_dot_agents_dir, "AGENTS.md")
   root_agents_path = os.path.join(target_dir, "AGENTS.md")
   claude_path = os.path.join(target_dir, "CLAUDE.md")
   readme_path = os.path.join(target_dir, "README.md")

   for target_path in [dot_agents_path, root_agents_path, claude_path, readme_path]:
       if os.path.exists(target_path):
           try:
               os.chmod(target_path, 0o644)
           except OSError:
               pass
   ```

3. **Implemented Modifications in `scripts/install_pipeline.sh`:**
   - **Target workspace preparation (line 161):**
     ```bash
     mkdir -p "$TARGET_DIR"
     chmod u+w "$TARGET_DIR" 2>/dev/null || true
     TARGET_DIR="$(cd -P "$TARGET_DIR" 2>/dev/null && pwd -P || echo "$TARGET_DIR")"
     ```
   - **Pipeline directories cleanup and force copying (lines 269–302):**
     ```bash
     if [ "$TARGET_DIR" != "$INSTALLER_ROOT" ]; then
       chmod -R u+w "$TARGET_DIR/skills" "$TARGET_DIR/rules" "$TARGET_DIR/.pipeline" "$TARGET_DIR/.agents" "$TARGET_DIR/scripts" 2>/dev/null || true
       rm -rf "$TARGET_DIR/skills" "$TARGET_DIR/rules" "$TARGET_DIR/.pipeline" "$TARGET_DIR/.agents" "$TARGET_DIR/scripts"
       cp -RPf "$INSTALLER_ROOT/skills" "$TARGET_DIR/"
       cp -RPf "$INSTALLER_ROOT/rules" "$TARGET_DIR/"
       cp -RPf "$INSTALLER_ROOT/.pipeline" "$TARGET_DIR/"
       rm -rf "$TARGET_DIR/.pipeline/upstream"
       rm -rf "$TARGET_DIR/.pipeline/diagnostics"

       if [ -n "$PRESERVED_METADATA" ]; then
         chmod u+w "$TARGET_DIR/.pipeline/project_metadata.json" 2>/dev/null || true
         echo "$PRESERVED_METADATA" > "$TARGET_DIR/.pipeline/project_metadata.json"
       fi
       if [ -n "$PRESERVED_PROFILE_CONFIG" ]; then
         chmod u+w "$TARGET_DIR/.pipeline/profile_config.json" 2>/dev/null || true
         echo "$PRESERVED_PROFILE_CONFIG" > "$TARGET_DIR/.pipeline/profile_config.json"
       fi
       cp -RPf "$INSTALLER_ROOT/.agents" "$TARGET_DIR/"
       cp -RPf "$INSTALLER_ROOT/scripts" "$TARGET_DIR/"
       mkdir -p "$TARGET_DIR/schema"
       chmod -R u+w "$TARGET_DIR/schema" 2>/dev/null || true
       if [ -d "$INSTALLER_ROOT/schema" ]; then
         cp -RPf "$INSTALLER_ROOT/schema/." "$TARGET_DIR/schema/"
       fi
       chmod u+w "$TARGET_DIR/requirements.txt" "$TARGET_DIR/pyproject.toml" 2>/dev/null || true
       cp -Pf "$INSTALLER_ROOT/requirements.txt" "$TARGET_DIR/" 2>/dev/null || true
       cp -Pf "$INSTALLER_ROOT/pyproject.toml" "$TARGET_DIR/" 2>/dev/null || true
       chmod u+w "$TARGET_DIR/.gitignore" 2>/dev/null || true
       if [ -f "$TARGET_DIR/.gitignore" ]; then
         cat "$INSTALLER_ROOT/.gitignore" >> "$TARGET_DIR/.gitignore"
         # Deduplicate lines in .gitignore
         sort -u "$TARGET_DIR/.gitignore" -o "$TARGET_DIR/.gitignore"
       elif [ -f "$INSTALLER_ROOT/.gitignore" ]; then
         cp -Pf "$INSTALLER_ROOT/.gitignore" "$TARGET_DIR/"
       fi
       mkdir -p "$TARGET_DIR/tests"
       chmod -R u+w "$TARGET_DIR/tests" 2>/dev/null || true
       cp -RPf "$INSTALLER_ROOT/tests/fixtures" "$TARGET_DIR/tests/" 2>/dev/null || true
     else
       # Running in-place in an initialized downstream project
       chmod -R u+w "$TARGET_DIR/.pipeline" 2>/dev/null || true
       rm -rf "$TARGET_DIR/.pipeline/upstream"
     fi
     ```
   - **Documentation and GitLab CI copy hardening (lines 317–346):**
     Added `chmod -R u+w "$TARGET_DIR/docs"`, `chmod -R u+w "$TARGET_DIR/.pipeline"`, `chmod u+w "$TARGET_DIR/.gitlab-ci.yml"`, `chmod u+w "$rules_file"`, and changed `cp -P` to `cp -Pf`.
   - **Environment template, Agent Governance, and README redirection (lines 441, 484, 635):**
     Added `chmod u+w "$TARGET_DIR/.env.template"`, `chmod u+w "$TARGET_DIR/AGENTS.md" "$TARGET_DIR/.agents/AGENTS.md" "$TARGET_DIR/CLAUDE.md" "$TARGET_DIR/README.md"`, and `chmod u+w "$TARGET_DIR/README.md"`.

4. **Empirical Verification Results:**
   - Syntax validation: `bash -n scripts/install_pipeline.sh` passed cleanly with exit code 0.
   - Python compilation: `python3 -m py_compile scripts/scaffold_downstream_agents.py` passed cleanly with exit code 0.
   - Baseline suite: `python3 scripts/verify_downstream_baseline.py --no-domain` passed all checks cleanly with exit code 0.
   - 9-Scenario Adverse Permissions Test Harness (executed in isolated temporary directories):
     ```
     PASS: 1. Target dir 0555
     PASS: 2. Subdir in skills 0555
     PASS: 3. schema/.gitkeep 0444
     PASS: 4. .gitignore 0444
     PASS: 5. docs/OPERATOR_PROMPT_CATALOG.md 0444
     PASS: 6. .gitlab-ci.yml 0444 (--provider gitlab)
     PASS: 7. .env.template 0444
     PASS: 8. AGENTS.md 0444
     PASS: 9. README.md 0444
     ```
     All 9 scenarios exited with code 0.

---

## 2. Logic Chain

1. **Root Cause Analysis (Observation 1.1):**
   - The installer was operating under `set -e` without write permission guarantees prior to file modifications, deletions, and copies.
   - Files checked out from Git or modified by IDEs/build processes can easily become read-only (`0444` / `0555`).
   - Under POSIX semantics, `cp` without `-f` fails when overwriting a read-only file; `rm -rf` fails when removing an entry inside a read-only directory; and shell redirection `>`/`>>` fails when writing to a read-only file.

2. **Resolution Strategy (Observations 1.2 & 1.3):**
   - By ensuring `chmod u+w` (or `chmod -R u+w`) precedes all write/delete operations and using the force flag `-f` on `cp` (`cp -RPf` and `cp -Pf`), the executing user is guaranteed permission to write, truncate, or unlink existing destination files.
   - In Python, `os.chmod(target_path, 0o644)` guarantees write access before opening files in write mode (`"w"`).
   - `chmod` commands are silenced with `2>/dev/null || true` to prevent false failures if non-standard filesystem attributes or ownership restrictions exist.

3. **Validation Proof (Observation 1.4):**
   - All 9 adverse permissions edge cases now pass cleanly without errors.
   - Baseline verification confirms zero regressions across the codebase.

---

## 3. Caveats

- **File Ownership / Root Permissions:** Standard POSIX permission restoration (`chmod u+w`) applies to files owned by the executing user. If files in the destination workspace are owned by `root` or another user, elevated permissions (`sudo`) would be required.
- **No other caveats.**

---

## 4. Conclusion

The robustness hardening across `scripts/install_pipeline.sh` and `scripts/scaffold_downstream_agents.py` is fully implemented and verified. All copy, remove, and redirection operations now cleanly handle pre-existing read-only files and directories, completely resolving Challenger 1's defect report and meeting all criteria.

---

## 5. Verification Method

To independently reproduce and verify this work:

1. **Verify Shell & Python Syntax:**
   ```bash
   bash -n scripts/install_pipeline.sh
   python3 -m py_compile scripts/scaffold_downstream_agents.py
   ```

2. **Run Downstream Baseline Conformance Gate:**
   ```bash
   python3 scripts/verify_downstream_baseline.py --no-domain
   ```

3. **Run the 9-Scenario Empirical Adverse Permissions Test:**
   ```bash
   python3 -c "
   import tempfile, subprocess, os

   script = '$(pwd)/scripts/install_pipeline.sh'

   def run_test(name, setup_fn, provider=None):
       with tempfile.TemporaryDirectory(prefix='test_ro_') as d:
           subprocess.run(['git', 'init', '-q'], cwd=d, check=True)
           setup_fn(d)
           cmd = ['bash', script]
           if provider:
               cmd.extend(['--provider', provider])
           cmd.append(d)
           res = subprocess.run(cmd, capture_output=True, text=True)
           assert res.returncode == 0, f'Failed on {name}: {res.stderr}\nOutput: {res.stdout}'
           print(f'PASS: {name}')

   run_test('1. Target dir 0555', lambda d: os.chmod(d, 0o555))

   def s2(d):
       sub = os.path.join(d, 'skills', 'sub')
       os.makedirs(sub)
       with open(os.path.join(sub, 'foo.md'), 'w') as f: f.write('x')
       os.chmod(sub, 0o555)
   run_test('2. Subdir in skills 0555', s2)

   def s3(d):
       os.makedirs(os.path.join(d, 'schema'))
       gk = os.path.join(d, 'schema', '.gitkeep')
       with open(gk, 'w') as f: f.write('')
       os.chmod(gk, 0o444)
   run_test('3. schema/.gitkeep 0444', s3)

   def s4(d):
       gi = os.path.join(d, '.gitignore')
       with open(gi, 'w') as f: f.write('build/\n')
       os.chmod(gi, 0o444)
   run_test('4. .gitignore 0444', s4)

   def s5(d):
       doc_dir = os.path.join(d, 'docs')
       os.makedirs(doc_dir)
       f = os.path.join(doc_dir, 'OPERATOR_PROMPT_CATALOG.md')
       with open(f, 'w') as fp: fp.write('old')
       os.chmod(f, 0o444)
   run_test('5. docs/OPERATOR_PROMPT_CATALOG.md 0444', s5)

   def s6(d):
       f = os.path.join(d, '.gitlab-ci.yml')
       with open(f, 'w') as fp: fp.write('# old')
       os.chmod(f, 0o444)
   run_test('6. .gitlab-ci.yml 0444 (--provider gitlab)', s6, provider='gitlab')

   def s7(d):
       f = os.path.join(d, '.env.template')
       with open(f, 'w') as fp: fp.write('old')
       os.chmod(f, 0o444)
   run_test('7. .env.template 0444', s7)

   def s8(d):
       f = os.path.join(d, 'AGENTS.md')
       with open(f, 'w') as fp: fp.write('old')
       os.chmod(f, 0o444)
   run_test('8. AGENTS.md 0444', s8)

   def s9(d):
       f = os.path.join(d, 'README.md')
       with open(f, 'w') as fp: fp.write('Getting started with GitLab\n')
       os.chmod(f, 0o444)
   run_test('9. README.md 0444', s9)
   "
   ```

**Invalidation Conditions:**
- Fails if any test scenario in the test harness produces non-zero return code.
- Fails if `scripts/verify_downstream_baseline.py --no-domain` fails.
