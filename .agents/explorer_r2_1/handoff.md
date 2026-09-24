# Handoff Report: Forensic Investigation of Copy & File Operation Robustness under Adverse Permissions in `scripts/install_pipeline.sh`

**Verdict:** `RECOMMEND_CHANGES` (Full Robustness Hardening)  
**Role:** Explorer R2_1 (Investigation & Synthesis)  
**Target:** `scripts/install_pipeline.sh` (and `scripts/scaffold_downstream_agents.py`)  
**Working Directory:** `/Users/perkunas/jail/DEAP01-spec-core/.agents/explorer_r2_1`  

---

## 1. Observation

### 1.1 Context & Investigation Target
In Iteration 1, Challenger 1 issued `REQUEST_CHANGES` on lines 283–286 in `scripts/install_pipeline.sh`:
```bash
283:   mkdir -p "$TARGET_DIR/schema"
284:   if [ -d "$INSTALLER_ROOT/schema" ]; then
285:     cp -RP "$INSTALLER_ROOT/schema/." "$TARGET_DIR/schema/"
286:   fi
```
Challenger 1 demonstrated that if `$TARGET_DIR/schema/.gitkeep` (or any matching schema file) or the `$TARGET_DIR/schema` directory itself has read-only mode (`0444` or `0555`), `cp -RP` fails with:
```
cp: .../schema/./.gitkeep: Permission denied
```
Because line 2 of `scripts/install_pipeline.sh` declares `set -e`, this error halts execution immediately, leaving the target workspace in a corrupted, half-installed state.

### 1.2 Comprehensive Audit of All Copy and File Operations in `scripts/install_pipeline.sh`
A systematic line-by-line inspection of `scripts/install_pipeline.sh` revealed that read-only destination failure modes are **not isolated to `schema/`**. The following operations throughout the script were evaluated:

1. **Target Workspace Root Preparation (Line 160):**
   ```bash
   mkdir -p "$TARGET_DIR"
   TARGET_DIR="$(cd -P "$TARGET_DIR" 2>/dev/null && pwd -P || echo "$TARGET_DIR")"
   ```
   If the target directory itself has read-only permissions (`0555`), subsequent `cp`, `mkdir`, and `rm -rf` operations fail immediately.

2. **Removal of Pipeline Directories in Non-Installer Target (Line 268):**
   ```bash
   rm -rf "$TARGET_DIR/skills" "$TARGET_DIR/rules" "$TARGET_DIR/.pipeline" "$TARGET_DIR/.agents" "$TARGET_DIR/scripts"
   ```
   Under POSIX filesystem semantics, `rm -rf` on a directory containing a subdirectory with `0555` (read-only directory) fails with:
   ```
   rm: .../skills/sub/foo.md: Permission denied
   rm: .../skills/sub: Directory not empty
   ```
   Halting `install_pipeline.sh` under `set -e`.

3. **Copy of Platform Directories (Lines 269–282):**
   ```bash
   cp -RP "$INSTALLER_ROOT/skills" "$TARGET_DIR/"
   cp -RP "$INSTALLER_ROOT/rules" "$TARGET_DIR/"
   cp -RP "$INSTALLER_ROOT/.pipeline" "$TARGET_DIR/"
   ...
   cp -RP "$INSTALLER_ROOT/.agents" "$TARGET_DIR/"
   cp -RP "$INSTALLER_ROOT/scripts" "$TARGET_DIR/"
   ```
   All use `cp -RP` without the force flag (`-f`).

4. **Schema Copying (Lines 283–286):**
   ```bash
   mkdir -p "$TARGET_DIR/schema"
   if [ -d "$INSTALLER_ROOT/schema" ]; then
     cp -RP "$INSTALLER_ROOT/schema/." "$TARGET_DIR/schema/"
   fi
   ```
   `schema/` is merged rather than purged with `rm -rf` to preserve customer models. Without `chmod -R u+w` and `-f`, read-only files (`0444`) or directories (`0555`) crash `cp`.

5. **Python Requirements & Configuration (Lines 287–288):**
   ```bash
   cp -P "$INSTALLER_ROOT/requirements.txt" "$TARGET_DIR/" 2>/dev/null || true
   cp -P "$INSTALLER_ROOT/pyproject.toml" "$TARGET_DIR/" 2>/dev/null || true
   ```
   Guarded by `2>/dev/null || true`, but will silently fail to update existing read-only copies without `-f` and write permission.

6. **Gitignore Update / Creation (Lines 289–295):**
   ```bash
   if [ -f "$TARGET_DIR/.gitignore" ]; then
     cat "$INSTALLER_ROOT/.gitignore" >> "$TARGET_DIR/.gitignore"
     # Deduplicate lines in .gitignore
     sort -u "$TARGET_DIR/.gitignore" -o "$TARGET_DIR/.gitignore"
   elif [ -f "$INSTALLER_ROOT/.gitignore" ]; then
     cp "$INSTALLER_ROOT/.gitignore" "$TARGET_DIR/"
   fi
   ```
   If `$TARGET_DIR/.gitignore` has permission `0444`, `cat >> "$TARGET_DIR/.gitignore"` fails with `Permission denied` and crashes under `set -e`.

7. **Test Fixtures (Lines 296–297):**
   ```bash
   mkdir -p "$TARGET_DIR/tests"
   cp -RP "$INSTALLER_ROOT/tests/fixtures" "$TARGET_DIR/tests/" 2>/dev/null || true
   ```
   If existing fixture files are `0444`, `cp -RP` silently fails, causing lines 1148–1155 (safety fixtures verification) to fail.

8. **Governance & Architecture Documentation (Lines 310–322):**
   ```bash
   if [ "$TARGET_DIR" != "$INSTALLER_ROOT" ]; then
     if [ -f "$INSTALLER_ROOT/docs/conops/README.md" ]; then
       cp -P "$INSTALLER_ROOT/docs/conops/README.md" "$TARGET_DIR/docs/conops/"
     fi
     if [ -f "$INSTALLER_ROOT/docs/safety/README.md" ]; then
       cp -P "$INSTALLER_ROOT/docs/safety/README.md" "$TARGET_DIR/docs/safety/"
     fi
     if [ -f "$INSTALLER_ROOT/docs/OPERATOR_PROMPT_CATALOG.md" ]; then
       cp -P "$INSTALLER_ROOT/docs/OPERATOR_PROMPT_CATALOG.md" "$TARGET_DIR/docs/"
     fi
     if [ -f "$INSTALLER_ROOT/docs/JIRA_INTEGRATION_GUIDE.md" ]; then
       cp -P "$INSTALLER_ROOT/docs/JIRA_INTEGRATION_GUIDE.md" "$TARGET_DIR/docs/"
     fi
   fi
   ```
   **CRITICAL:** These four `cp -P` invocations do NOT have `|| true` and lack `-f`. If any of these files exists in `$TARGET_DIR/docs/` with mode `0444`, `cp -P` fails with `Permission denied` and crashes under `set -e`.

9. **GitLab CI Configuration (Lines 328–336):**
   ```bash
   if [ "$PROVIDER" = "gitlab" ] || [ -n "$GITLAB_GROUP" ] || [ "$GITLAB_URL" != "https://gitlab.com" ]; then
     if [ -f "$INSTALLER_ROOT/.pipeline/templates/.gitlab-ci.yml" ]; then
       cp -P "$INSTALLER_ROOT/.pipeline/templates/.gitlab-ci.yml" "$TARGET_DIR/.gitlab-ci.yml"
     elif [ -f "$INSTALLER_ROOT/.pipeline/.gitlab-ci.yml" ]; then
       cp -P "$INSTALLER_ROOT/.pipeline/.gitlab-ci.yml" "$TARGET_DIR/.gitlab-ci.yml"
     elif [ -f "$TARGET_DIR/.pipeline/templates/.gitlab-ci.yml" ]; then
       cp -P "$TARGET_DIR/.pipeline/templates/.gitlab-ci.yml" "$TARGET_DIR/.gitlab-ci.yml"
     elif [ -f "$TARGET_DIR/.pipeline/.gitlab-ci.yml" ]; then
       cp -P "$TARGET_DIR/.pipeline/.gitlab-ci.yml" "$TARGET_DIR/.gitlab-ci.yml"
     fi
   ```
   **CRITICAL:** Uses `cp -P` without `-f`. If `$TARGET_DIR/.gitlab-ci.yml` exists with mode `0444`, `cp -P` fails with `Permission denied` and crashes under `set -e`.

10. **Environment Variable Template (Line 429):**
    ```bash
    cat << 'EOF' > "$TARGET_DIR/.env.template"
    ```
    If `$TARGET_DIR/.env.template` exists with mode `0444`, shell redirection `> "$TARGET_DIR/.env.template"` fails with `Permission denied` under `set -e`.

11. **Agent Governance Scaffolding (Lines 468–469):**
    ```bash
    mkdir -p "$TARGET_DIR/.agents"
    python3 "$INSTALLER_ROOT/scripts/scaffold_downstream_agents.py" "$INSTALLER_ROOT" "$TARGET_DIR"
    ```
    In `scripts/scaffold_downstream_agents.py` lines 101–105:
    ```python
    with open(dot_agents_path, "w", encoding="utf-8") as f:
        f.write(transformed)
    with open(root_agents_path, "w", encoding="utf-8") as f:
        f.write(transformed)
    ```
    While `.agents/AGENTS.md` was cleaned up by line 268, `$TARGET_DIR/AGENTS.md` (root) is not. If `$TARGET_DIR/AGENTS.md` is `0444`, Python throws `PermissionError: [Errno 13] Permission denied: '$TARGET_DIR/AGENTS.md'` and crashes under `set -e`.

12. **Root README Scaffolding (Line 619):**
    ```bash
    cat << EOF > "$TARGET_DIR/README.md"
    ```
    If `$TARGET_DIR/README.md` is `0444`, shell redirection `> "$TARGET_DIR/README.md"` fails with `Permission denied` under `set -e`.

---

### 1.3 Empirical Verification of Unpatched Failures
An isolated test harness was executed against the unpatched `scripts/install_pipeline.sh` across all 9 identified failure scenarios:

| Test Scenario | Condition | Result with Unpatched Code | Verbatim Stderr |
|---|---|---|---|
| **Scenario 1** | Target root `$TARGET_DIR` is mode `0555` | **CRASH (rc=1)** | `cp: .../skills: Permission denied` |
| **Scenario 2** | Pre-existing `$TARGET_DIR/skills/sub` is `0555` | **CRASH (rc=1)** | `rm: .../skills/sub/foo.md: Permission denied` |
| **Scenario 3** | Target has `$TARGET_DIR/schema/.gitkeep` with `0444` | **CRASH (rc=1)** | `cp: .../schema/./.gitkeep: Permission denied` |
| **Scenario 4** | Target has `$TARGET_DIR/.gitignore` with `0444` | **CRASH (rc=1)** | `line 290: .../.gitignore: Permission denied` |
| **Scenario 5** | Target has `docs/OPERATOR_PROMPT_CATALOG.md` with `0444` | **CRASH (rc=1)** | `cp: .../docs/OPERATOR_PROMPT_CATALOG.md: Permission denied` |
| **Scenario 6** | Target has `.gitlab-ci.yml` with `0444` (`--provider gitlab`) | **CRASH (rc=1)** | `cp: .../.gitlab-ci.yml: Permission denied` |
| **Scenario 7** | Target has `.env.template` with `0444` | **CRASH (rc=1)** | `line 429: .../.env.template: Permission denied` |
| **Scenario 8** | Target has root `AGENTS.md` with `0444` | **CRASH (rc=1)** | `PermissionError: [Errno 13] Permission denied: '.../AGENTS.md'` |
| **Scenario 9** | Target has `README.md` with `0444` | **CRASH (rc=1)** | `line 619: .../README.md: Permission denied` |

### 1.4 Empirical Verification of Proposed Robust Fixes
When the proposed robustness enhancements were applied, all 9 scenarios passed with exit code `0`:
```
Test [1. Target dir 0555]: PASS
Test [2. Subdir in skills 0555]: PASS
Test [3. schema/.gitkeep 0444]: PASS
Test [4. .gitignore 0444]: PASS
Test [5. docs/OPERATOR_PROMPT_CATALOG.md 0444]: PASS
Test [6. .gitlab-ci.yml 0444 (--provider gitlab)]: PASS
Test [7. .env.template 0444]: PASS
Test [8. AGENTS.md 0444]: PASS
Test [9. README.md 0444]: PASS
```

---

## 2. Logic Chain

1. **POSIX File Deletion and Directory Permissions (Scenario 2):**
   - Observation 1.2.2 shows that `rm -rf` cannot delete files within a directory that lacks write permissions (`0555`), because `unlink()` requires write permission on the containing directory entry.
   - Preceding `rm -rf` with `chmod -R u+w "$TARGET_DIR/skills" ... 2>/dev/null || true` ensures the executing user has write access to all subdirectories before removal, preventing deletion failures.

2. **POSIX File Creation, Overwrite, and the `-f` Flag (Scenarios 1, 3, 5, 6):**
   - BSD and GNU `cp` default to opening an existing destination file with `open(path, O_WRONLY | O_TRUNC)`. When the file mode is `0444`, `open()` fails with `EACCES` (`Permission denied`).
   - When `-f` (force) is provided, `cp` will unlink (`unlink(path)`) any existing destination file that cannot be opened for writing, and then create a fresh writable file with the source contents.
   - For directories like `schema/` and `docs/`, combining `chmod -R u+w` on the target directory hierarchy with `cp -RPf` / `cp -Pf` guarantees that both existing files and containing folders permit unlinking and writing.

3. **Shell Redirection and Python File Writes (Scenarios 4, 7, 8, 9):**
   - Shell redirection (`>` and `>>`) also issues `open()` with `O_WRONLY`. If the target file already exists with mode `0444`, the shell raises `Permission denied`.
   - Applying `chmod u+w` to targeted files before shell redirection (`.gitignore`, `.env.template`, `README.md`) or Python file operations (`AGENTS.md`, `CLAUDE.md`, `codebase_rules.json`) guarantees unhindered file writes without needing to destroy custom downstream attributes.

4. **Preservation of Non-Conflicting Downstream Assets:**
   - Because `schema/` uses `cp -RPf "$INSTALLER_ROOT/schema/." "$TARGET_DIR/schema/"` (with trailing `/.`), existing customer schemas (e.g. `schema/custom_model.sysml`) and nested directories are preserved without duplication, while read-only incoming files (such as `.gitkeep`) are cleanly updated.

---

## 3. Caveats

- **File Ownership (UID Boundaries):** `chmod u+w` and `cp -f` operate within standard POSIX permissions for files owned by the user running the installer. If destination files are owned by `root` or another user, OS access controls will prevent modification unless run with elevated permissions (`sudo`).
- **Extended ACLs & Immutable Flags:** macOS immutable flags (`chflags uchg`) or immutable attributes (`chattr +i` on Linux) require explicit flag removal (`chflags nouchg`) before modification. Standard development workspaces do not set immutable flags.
- No other caveats.

---

## 4. Conclusion & Actionable Recommendation for Worker

The Worker should implement the following hardening edits to `scripts/install_pipeline.sh` and `scripts/scaffold_downstream_agents.py`:

### Edit 1: `scripts/install_pipeline.sh`
Apply write-permission assurances and force copy flags across all target operations:

```bash
# 1. Line 160: Ensure target directory itself is writable
mkdir -p "$TARGET_DIR"
chmod u+w "$TARGET_DIR" 2>/dev/null || true
TARGET_DIR="$(cd -P "$TARGET_DIR" 2>/dev/null && pwd -P || echo "$TARGET_DIR")"
```

```bash
# 2. Lines 268-286: Ensure cleaned directories and schema are writable, use cp -RPf
if [ "$TARGET_DIR" != "$INSTALLER_ROOT" ]; then
  chmod -R u+w "$TARGET_DIR/skills" "$TARGET_DIR/rules" "$TARGET_DIR/.pipeline" "$TARGET_DIR/.agents" "$TARGET_DIR/scripts" 2>/dev/null || true
  rm -rf "$TARGET_DIR/skills" "$TARGET_DIR/rules" "$TARGET_DIR/.pipeline" "$TARGET_DIR/.agents" "$TARGET_DIR/scripts"
  cp -RPf "$INSTALLER_ROOT/skills" "$TARGET_DIR/"
  cp -RPf "$INSTALLER_ROOT/rules" "$TARGET_DIR/"
  cp -RPf "$INSTALLER_ROOT/.pipeline" "$TARGET_DIR/"
  rm -rf "$TARGET_DIR/.pipeline/upstream"
  rm -rf "$TARGET_DIR/.pipeline/diagnostics"

  if [ -n "$PRESERVED_METADATA" ]; then
    echo "$PRESERVED_METADATA" > "$TARGET_DIR/.pipeline/project_metadata.json"
  fi
  if [ -n "$PRESERVED_PROFILE_CONFIG" ]; then
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

```bash
# 3. Lines 307-336: Protect docs and gitlab-ci copying
mkdir -p "$TARGET_DIR/docs" "$TARGET_DIR/docs/conops" "$TARGET_DIR/docs/conops/units/conops" "$TARGET_DIR/docs/conops/units/mission_intent" "$TARGET_DIR/docs/interfaces" "$TARGET_DIR/docs/safety" "$TARGET_DIR/docs/architecture/blueprints" "$TARGET_DIR/docs/epics" "$TARGET_DIR/docs/features" "$TARGET_DIR/docs/user-stories" "$TARGET_DIR/docs/use-cases" "$TARGET_DIR/docs/management"
chmod -R u+w "$TARGET_DIR/docs" 2>/dev/null || true
touch "$TARGET_DIR/docs/management/.gitkeep"
if [ "$TARGET_DIR" != "$INSTALLER_ROOT" ]; then
  if [ -f "$INSTALLER_ROOT/docs/conops/README.md" ]; then
    cp -Pf "$INSTALLER_ROOT/docs/conops/README.md" "$TARGET_DIR/docs/conops/"
  fi
  if [ -f "$INSTALLER_ROOT/docs/safety/README.md" ]; then
    cp -Pf "$INSTALLER_ROOT/docs/safety/README.md" "$TARGET_DIR/docs/safety/"
  fi
  if [ -f "$INSTALLER_ROOT/docs/OPERATOR_PROMPT_CATALOG.md" ]; then
    cp -Pf "$INSTALLER_ROOT/docs/OPERATOR_PROMPT_CATALOG.md" "$TARGET_DIR/docs/"
  fi
  if [ -f "$INSTALLER_ROOT/docs/JIRA_INTEGRATION_GUIDE.md" ]; then
    cp -Pf "$INSTALLER_ROOT/docs/JIRA_INTEGRATION_GUIDE.md" "$TARGET_DIR/docs/"
  fi
fi
mkdir -p "$TARGET_DIR/.pipeline/contracts" "$TARGET_DIR/.pipeline/domain_specs" "$TARGET_DIR/.pipeline/profiles"
chmod -R u+w "$TARGET_DIR/.pipeline" 2>/dev/null || true
chmod +x "$TARGET_DIR"/scripts/*.sh "$TARGET_DIR"/scripts/*.py 2>/dev/null || true

# Apply provider configurations if specified
if [ "$PROVIDER" = "gitlab" ] || [ -n "$GITLAB_GROUP" ] || [ "$GITLAB_URL" != "https://gitlab.com" ]; then
  chmod u+w "$TARGET_DIR/.gitlab-ci.yml" 2>/dev/null || true
  if [ -f "$INSTALLER_ROOT/.pipeline/templates/.gitlab-ci.yml" ]; then
    cp -Pf "$INSTALLER_ROOT/.pipeline/templates/.gitlab-ci.yml" "$TARGET_DIR/.gitlab-ci.yml"
  elif [ -f "$INSTALLER_ROOT/.pipeline/.gitlab-ci.yml" ]; then
    cp -Pf "$INSTALLER_ROOT/.pipeline/.gitlab-ci.yml" "$TARGET_DIR/.gitlab-ci.yml"
  elif [ -f "$TARGET_DIR/.pipeline/templates/.gitlab-ci.yml" ]; then
    cp -Pf "$TARGET_DIR/.pipeline/templates/.gitlab-ci.yml" "$TARGET_DIR/.gitlab-ci.yml"
  elif [ -f "$TARGET_DIR/.pipeline/.gitlab-ci.yml" ]; then
    cp -Pf "$TARGET_DIR/.pipeline/.gitlab-ci.yml" "$TARGET_DIR/.gitlab-ci.yml"
  fi
```

```bash
# 4. Lines 429, 468, 619: Protect template and README generation
chmod u+w "$TARGET_DIR/.env.template" 2>/dev/null || true
cat << 'EOF' > "$TARGET_DIR/.env.template"
...
mkdir -p "$TARGET_DIR/.agents"
chmod u+w "$TARGET_DIR/AGENTS.md" "$TARGET_DIR/.agents/AGENTS.md" "$TARGET_DIR/CLAUDE.md" "$TARGET_DIR/README.md" 2>/dev/null || true
python3 "$INSTALLER_ROOT/scripts/scaffold_downstream_agents.py" "$INSTALLER_ROOT" "$TARGET_DIR"
...
chmod u+w "$TARGET_DIR/README.md" 2>/dev/null || true
cat << EOF > "$TARGET_DIR/README.md"
```

### Edit 2: `scripts/scaffold_downstream_agents.py`
In `scaffold_downstream_agents()`:
Add defensive write-permission restoration before writing files:
```python
    for target_path in [dot_agents_path, root_agents_path, claude_path, readme_path]:
        if os.path.exists(target_path):
            try:
                os.chmod(target_path, 0o644)
            except OSError:
                pass
```

---

## 5. Verification Method

### 5.1 Project Baseline Verification Command
```bash
python3 scripts/verify_downstream_baseline.py --no-domain
```
**Expected Result:** Exit code `0` with all checks passing cleanly.

### 5.2 Empirical Regression Test Script (All 9 Adverse Permission Scenarios)
Run the following self-contained test in Python to verify that all 9 scenarios pass without error:

```python
import tempfile, subprocess, os

script = '/Users/perkunas/jail/DEAP01-spec-core/scripts/install_pipeline.sh'

def run_test(name, setup_fn, provider=None):
    with tempfile.TemporaryDirectory(prefix='test_ro_') as d:
        subprocess.run(['git', 'init', '-q'], cwd=d, check=True)
        setup_fn(d)
        cmd = ['bash', script]
        if provider:
            cmd.extend(['--provider', provider])
        cmd.append(d)
        res = subprocess.run(cmd, capture_output=True, text=True)
        assert res.returncode == 0, f"Failed on {name}: {res.stderr}"
        print(f"PASS: {name}")

# Test 1: Target root dir is 0555
run_test('1. Target dir 0555', lambda d: os.chmod(d, 0o555))

# Test 2: Subdir in skills is 0555
def s2(d):
    sub = os.path.join(d, 'skills', 'sub')
    os.makedirs(sub)
    with open(os.path.join(sub, 'foo.md'), 'w') as f: f.write('x')
    os.chmod(sub, 0o555)
run_test('2. Subdir in skills 0555', s2)

# Test 3: schema/.gitkeep is 0444
def s3(d):
    os.makedirs(os.path.join(d, 'schema'))
    gk = os.path.join(d, 'schema', '.gitkeep')
    with open(gk, 'w') as f: f.write('')
    os.chmod(gk, 0o444)
run_test('3. schema/.gitkeep 0444', s3)

# Test 4: .gitignore is 0444
def s4(d):
    gi = os.path.join(d, '.gitignore')
    with open(gi, 'w') as f: f.write('build/\n')
    os.chmod(gi, 0o444)
run_test('4. .gitignore 0444', s4)

# Test 5: docs/OPERATOR_PROMPT_CATALOG.md is 0444
def s5(d):
    doc_dir = os.path.join(d, 'docs')
    os.makedirs(doc_dir)
    f = os.path.join(doc_dir, 'OPERATOR_PROMPT_CATALOG.md')
    with open(f, 'w') as fp: fp.write('old')
    os.chmod(f, 0o444)
run_test('5. docs/OPERATOR_PROMPT_CATALOG.md 0444', s5)

# Test 6: .gitlab-ci.yml is 0444 (--provider gitlab)
def s6(d):
    f = os.path.join(d, '.gitlab-ci.yml')
    with open(f, 'w') as fp: fp.write('# old')
    os.chmod(f, 0o444)
run_test('6. .gitlab-ci.yml 0444 (--provider gitlab)', s6, provider='gitlab')

# Test 7: .env.template is 0444
def s7(d):
    f = os.path.join(d, '.env.template')
    with open(f, 'w') as fp: fp.write('old')
    os.chmod(f, 0o444)
run_test('7. .env.template 0444', s7)

# Test 8: AGENTS.md is 0444
def s8(d):
    f = os.path.join(d, 'AGENTS.md')
    with open(f, 'w') as fp: fp.write('old')
    os.chmod(f, 0o444)
run_test('8. AGENTS.md 0444', s8)

# Test 9: README.md is 0444
def s9(d):
    f = os.path.join(d, 'README.md')
    with open(f, 'w') as fp: fp.write('Getting started with GitLab\n')
    os.chmod(f, 0o444)
run_test('9. README.md 0444', s9)
```

### 5.3 Invalidation Conditions
This investigation report is invalidated if:
1. `scripts/install_pipeline.sh` fails on any of the 9 adverse permission test cases above when run by the owning user.
2. The changes in `scripts/install_pipeline.sh` or `scripts/scaffold_downstream_agents.py` break `python3 scripts/verify_downstream_baseline.py --no-domain`.
3. Downstream customer models or existing non-conflicting files in `schema/` are destroyed or duplicated into nested `schema/schema/`.
