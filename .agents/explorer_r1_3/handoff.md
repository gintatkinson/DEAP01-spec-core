# Handoff Report: Work Package 3 (R3 Robust Model & Schema Copying) & Baseline Verification

**Author**: Explorer 3 (`explorer_r1_3`)  
**Target Work Package**: WP3 (R3: Robust Model & Schema Copying) and Downstream Baseline Verification  
**Repository**: `DEAP01-spec-core` (Classification: `UPSTREAM_SPEC_CORE_COMPILER`)  

---

## 1. Observation

### 1.1 `scripts/install_pipeline.sh` (Lines 283–289)
In `scripts/install_pipeline.sh`:
```bash
283:   if [ ! -e "$TARGET_DIR/schema" ]; then
284:     if [ -d "$INSTALLER_ROOT/schema" ]; then
285:       cp -RP "$INSTALLER_ROOT/schema" "$TARGET_DIR/"
286:     else
287:       mkdir -p "$TARGET_DIR/schema"
288:     fi
289:   fi
```
- Line 283 guards the copy behind `[ ! -e "$TARGET_DIR/schema" ]`.
- If `$TARGET_DIR/schema` exists as a directory (even if completely empty or only containing `.gitkeep`), the condition evaluates to `false`, completely skipping lines 284–288.
- If `$TARGET_DIR/schema` does NOT exist, line 285 executes `cp -RP "$INSTALLER_ROOT/schema" "$TARGET_DIR/"`. However, if line 283 were simply replaced with `if [ -d "$INSTALLER_ROOT/schema" ]; then` without modifying the destination path in `cp`, `cp -RP "$INSTALLER_ROOT/schema" "$TARGET_DIR/"` on a target directory where `$TARGET_DIR/schema` already exists results in copying the `schema` folder *into* `$TARGET_DIR/schema`, producing `$TARGET_DIR/schema/schema/`.

### 1.2 Empirical Failure Reproduction (Old Logic vs. New Logic)
An isolated test in a temporary sandbox executing the current logic (lines 283–289) vs. proposed logic yielded the following empirical results across 5 standard scenarios:

| Scenario | Pre-existing `$TARGET_DIR/schema` | `$INSTALLER_ROOT/schema` Content | Old Logic Result (`TARGET_DIR/schema`) | Proposed Logic Result (`TARGET_DIR/schema`) |
|---|---|---|---|---|
| **1. Empty Target Schema** | Empty `schema/` | `DEAP_MODEL.sysml` | `[]` (FAIL - model skipped) | `['DEAP_MODEL.sysml']` (PASS) |
| **2. Target with `.gitkeep`** | `schema/.gitkeep` | `DEAP_MODEL.sysml`, `.gitkeep` | `['.gitkeep']` (FAIL - model skipped) | `['.gitkeep', 'DEAP_MODEL.sysml']` (PASS) |
| **3. Target without `schema`** | No `schema/` dir | `DEAP_MODEL.sysml` | `['DEAP_MODEL.sysml']` (PASS) | `['DEAP_MODEL.sysml']` (PASS) |
| **4. Nested Schema Subdir** | `schema/.gitkeep` | `sub/nested.sysml` | `['.gitkeep']` (FAIL - model skipped) | `['.gitkeep', 'sub/nested.sysml']` (PASS) |
| **5. Pre-existing User Files** | `schema/custom.json`| `DEAP_MODEL.sysml` | `['custom.json']` (FAIL - model skipped) | `['DEAP_MODEL.sysml', 'custom.json']` (PASS) |

In 4 out of 5 scenarios, the existing installer logic completely discarded domain models and schemas from the installer root.

### 1.3 `scripts/verify_downstream_baseline.py` Verification Results
Executing `python3 scripts/verify_downstream_baseline.py --no-domain` on `DEAP01-spec-core`:
- Result: **Exit Code 0** (All Checks 10 through 30 verified successfully).
- Specifically:
  - **Check 10**: `.gitignore` verified.
  - **Check 11**: Zero `.DS_Store` files.
  - **Check 12**: Upstream master core repository detected; duplicate blueprint check skipped.
  - **Check 13**: KaTeX / LaTeX mathematical syntax valid across all markdown files.
  - **Check 14**: `README.md`, agent entrypoints, and `rules/sysml-ssot-completeness.md` verified.
  - **Check 15**: `scripts/reconcile_backlog.py` verified executable.
  - **Check 16**: Upstream distribution template clean landing zones verified (all landing zones, including `schema/`, contain only `.gitkeep` / `README.md`).
  - **Checks 17–19**: Safety landing zone clean, architecture blueprints clean, domain-agnostic AST cleanliness passed.
  - **Checks 20–30**: WBS, AST parity, prose invariants, factual grounding, ICD completeness, etc. verified.

---

## 2. Logic Chain

1. **Step 1: Identifying the Schema Loss Condition.**
   - By Observation 1.1, `[ ! -e "$TARGET_DIR/schema" ]` evaluates to `false` whenever `$TARGET_DIR/schema` exists.
   - In real-world customer workspace initialization, `git init` or repository templates create an initial `schema/` directory containing `.gitkeep`.
   - Therefore, running the installer into any pre-initialized customer directory causes all domain models (`schema/*.sysml`, `schema/*.json`, etc.) from the domain distribution repository to be skipped.

2. **Step 2: Preventing Directory Nesting (`schema/schema/`).**
   - In POSIX-compliant implementations of `cp` (BSD and GNU coreutils), executing `cp -RP "$INSTALLER_ROOT/schema" "$TARGET_DIR/"` when `$TARGET_DIR/schema` already exists places the source directory *inside* the target, creating `$TARGET_DIR/schema/schema/`.
   - Per POSIX.1-2017 specification for `cp`: if the source argument ends in `/.`, `cp` copies the *contents* of the directory into the destination directory.
   - Therefore, `cp -RP "$INSTALLER_ROOT/schema/." "$TARGET_DIR/schema/"` guarantees that files and subdirectories are copied directly into `$TARGET_DIR/schema/` without creating a redundant nested `schema/schema` directory.

3. **Step 3: Preserving Clean Landing Zones in Upstream (Check 16).**
   - By Observation 1.3, Check 16 (`check_upstream_template_clean_landing_zones` in `verify_downstream_baseline.py`) verifies that in repositories containing `.pipeline/upstream` (i.e. `DEAP01-spec-core`), `schema/` must contain only `.gitkeep` and `README.md`.
   - In downstream domain templates (`DEAP-*`) or customer repositories (`uav-*`), `.pipeline/upstream` is removed by line 272 of `install_pipeline.sh`:
     ```bash
     rm -rf "$TARGET_DIR/.pipeline/upstream"
     ```
   - Therefore, downstream repositories skip Check 16 and are permitted to hold concrete domain models in `schema/`.
   - The upstream repository `DEAP01-spec-core` maintains only `.gitkeep` in `schema/`.

4. **Step 4: Ensuring Seamless SysML Model Discovery.**
   - In `scripts/verify_downstream_baseline.py` lines 1348–1372, `_discover_sysml_model_text` searches directly in `os.path.join(repo_root, "schema")` for `*.sysml` files (non-recursive directory list).
   - If models were nested under `schema/schema/`, downstream checks (Checks 21–30) would fail to detect the authoritative SysML model text.
   - By copying to `$TARGET_DIR/schema/`, `DEAP_MODEL.sysml` resides at `$TARGET_DIR/schema/DEAP_MODEL.sysml`, enabling `_discover_sysml_model_text` to locate it immediately.

5. **Step 5: Code Block Syntax Integrity (Check 13 & Acceptance Criteria).**
   - Acceptance criteria require code blocks in updated markdown files to contain pure, valid shell syntax with zero unescaped parentheses in comments and zero unquoted angle-bracket placeholders.
   - In `scripts/install_pipeline.sh`, the downstream `README.md` generator must embed the resolved `$DOMAIN_REMOTE_URL` (or quoted placeholder `"$DOMAIN_REMOTE_URL"`), avoiding unquoted angle brackets (`<domain-repo-remote-url>`) which trigger redirection errors in shell environments and can confuse markdown/KaTeX validators.

---

## 3. Caveats

1. **In-Place Execution**:
   When `install_pipeline.sh` is executed in-place (`$TARGET_DIR == "$INSTALLER_ROOT"`), lines 267–301 are skipped by design. Line 306 ensures `mkdir -p "$TARGET_DIR/schema"` runs as an idempotent fallback.
2. **File Overwrites on Collision**:
   `cp -RP "$INSTALLER_ROOT/schema/." "$TARGET_DIR/schema/"` overwrites files in `$TARGET_DIR/schema` only if they share identical filenames with files in `$INSTALLER_ROOT/schema`. Custom files with distinct names (e.g. `schema/custom.json`) are preserved.
3. **Platform-Specific `cp` Compatibility**:
   The syntax `cp -RP "$SRC/." "$DEST/"` was tested and confirmed on macOS (BSD `cp`) and is POSIX standard (guaranteed on GNU Linux coreutils).

---

## 4. Conclusion

### 4.1 Required Code Changes in `scripts/install_pipeline.sh` (WP3: R3)
Replace lines 283–289 in `scripts/install_pipeline.sh`:

**Before:**
```bash
  if [ ! -e "$TARGET_DIR/schema" ]; then
    if [ -d "$INSTALLER_ROOT/schema" ]; then
      cp -RP "$INSTALLER_ROOT/schema" "$TARGET_DIR/"
    else
      mkdir -p "$TARGET_DIR/schema"
    fi
  fi
```

**After:**
```bash
  mkdir -p "$TARGET_DIR/schema"
  if [ -d "$INSTALLER_ROOT/schema" ]; then
    cp -RP "$INSTALLER_ROOT/schema/." "$TARGET_DIR/schema/"
  fi
```

### 4.2 Required Changes in `scripts/install_pipeline.sh` (WP2: R2)
In the downstream `README.md` scaffolding block (around line 596):
1. Compute `$DOMAIN_REMOTE_URL`:
   ```bash
   DOMAIN_REMOTE_URL="${REMOTE_URL:-https://github.com/your-org/your-domain-repo.git}"
   ```
2. Insert an onboarding section in the scaffolded `README.md`:
   ```markdown
   ## 2. Installation & Customer Onboarding

   To install this domain platform and its engineering pipeline into a customer application workspace:

   ```bash
   git clone "$DOMAIN_REMOTE_URL" ./.tmp-pipeline && bash ./.tmp-pipeline/scripts/install_pipeline.sh . && rm -rf ./.tmp-pipeline
   ```
   ```
   Ensure the onboarding command is strictly self-contained, operates within `.`, has zero sibling path dependencies (`../...`), and quotes `"$DOMAIN_REMOTE_URL"`.

### 4.3 Required Changes in `README.md` (WP1: R1)
- Update Sections 1.2 and 5 in `DEAP01-spec-core/README.md` to clearly define Tier 1 (`DEAP01-spec-core` -> `DEAP-*`) vs. Tier 2 (`DEAP-*` -> `uav-*`).
- Preserve Section 9 header `## 9. Multi-Pipeline Operator Prompt Catalog & Autonomous Execution Workflows` to ensure Check 14 in `scripts/verify_downstream_baseline.py` continues to pass.
- Ensure all markdown code fences have pure shell syntax with zero unescaped parentheses in comments and zero unquoted angle-bracket placeholders.

---

## 5. Verification Method

### 5.1 Automated Baseline Verification Command
Run the primary repository baseline verification gate:
```bash
python3 scripts/verify_downstream_baseline.py --no-domain
```
**Expected Output**:
- Exit code: 0
- Checks 10 through 30 verified with success messages.
- Clean restoration-point tagging.

### 5.2 Schema Copy Sandbox Test Command
Run an automated test script verifying schema copying in an external temporary sandbox:
```bash
python3 -c '
import tempfile, subprocess, os

with tempfile.TemporaryDirectory() as td:
    src = os.path.join(td, "src")
    tgt = os.path.join(td, "tgt")
    os.makedirs(os.path.join(src, "schema"))
    with open(os.path.join(src, "schema", "DEAP_MODEL.sysml"), "w") as f:
        f.write("package TestModel {}")
    with open(os.path.join(src, "schema", ".gitkeep"), "w") as f:
        f.write("")
    
    # Target has pre-existing schema with .gitkeep
    os.makedirs(os.path.join(tgt, "schema"))
    with open(os.path.join(tgt, "schema", ".gitkeep"), "w") as f:
        f.write("")

    cmd = """
    INSTALLER_ROOT="{src}"
    TARGET_DIR="{tgt}"
    mkdir -p "$TARGET_DIR/schema"
    if [ -d "$INSTALLER_ROOT/schema" ]; then
      cp -RP "$INSTALLER_ROOT/schema/." "$TARGET_DIR/schema/"
    fi
    """.format(src=src, tgt=tgt)

    res = subprocess.run(["bash", "-c", cmd], capture_output=True, text=True)
    assert res.returncode == 0, res.stderr
    contents = sorted(os.listdir(os.path.join(tgt, "schema")))
    assert contents == [".gitkeep", "DEAP_MODEL.sysml"], f"Unexpected contents: {contents}"
    print("Verification Passed: schema copied into existing schema directory successfully.")
'
```

### 5.3 Invalidation Conditions
The conclusion is invalidated if:
1. `cp -RP "$INSTALLER_ROOT/schema/." "$TARGET_DIR/schema/"` fails on any target operating system or creates nested directories.
2. `python3 scripts/verify_downstream_baseline.py --no-domain` fails with exit code != 0.
3. Upstream clean landing zone Check 16 fails due to uncommitted artifacts left in `DEAP01-spec-core/schema/`.
