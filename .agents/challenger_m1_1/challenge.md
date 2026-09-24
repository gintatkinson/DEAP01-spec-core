# Empirical Challenge Report: DEAP01-spec-core/README.md

## Challenge Summary

**Overall risk assessment**: LOW  
**Verdict**: **APPROVE** (with 1 Low-Severity Advisory Observation)

This adversarial audit evaluated `README.md` in `DEAP01-spec-core` against four correctness pillars:
1. Syntactic validity of every `bash`/`sh` code fence via `bash -n`.
2. Absence of residual inline Python scripts (`python3 -c`) or monkeypatching strings.
3. Absence of residual domain repository clone commands (`git clone ... DEAP-uas-infrastructure-safety`).
4. Absence of unescaped parentheses in comments inside bash code blocks and absence of unquoted angle-bracket (`<...>`) placeholders.

---

## Empirical Verification Findings

### 1. Bash Code Fence Syntax (`bash -n`)
- **Total bash code fences identified**: 17
- **Fences tested via `bash -n`**: 17 / 17 passed (100% pass rate, exit code 0)

| Block | Lines | Command / Summary | `bash -n` Result |
|---|---|---|---|
| Block 1 | 195–200 | Python 3.12 installation (Homebrew / macOS) | PASS (exit 0) |
| Block 2 | 202–207 | Python 3.12 installation (Ubuntu / Debian) | PASS (exit 0) |
| Block 3 | 215–217 | `python3 scripts/compile_sysml.py --compile` | PASS (exit 0) |
| Block 4 | 221–223 | `python3 -m pytest tests/` | PASS (exit 0) |
| Block 5 | 227–229 | `python3 scripts/verify_downstream_baseline.py --no-domain` | PASS (exit 0) |
| Block 6 | 239–242 | `bash scripts/install_pipeline.sh "<path-to-domain-template>"` | PASS (exit 0) |
| Block 7 | 246–249 | `git clone https://github.com/gintatkinson/DEAP01-spec-core.git /tmp/deap_compiler && bash /tmp/deap_compiler/scripts/install_pipeline.sh . && rm -rf /tmp/deap_compiler` | PASS (exit 0) |
| Block 8 | 253–256 | `bash scripts/install_pipeline.sh .` | PASS (exit 0) |
| Block 9 | 324–327 | `echo "Read all SKILL.md..." >> CLAUDE.md` | PASS (exit 0) |
| Block 10 | 337–340 | Pytest & baseline verification commands | PASS (exit 0) |
| Block 11 | 378–390 | Backlog reconciliation commands (GitHub, GitLab, Offline) | PASS (exit 0) |
| Block 12 | 428–437 | Forward AST, Reverse AST, and Parity Lock commands | PASS (exit 0) |
| Block 13 | 465–467 | `python3 scripts/generate_wbs_suite.py [--workspace PATH] [--output-dir PATH]` | PASS (exit 0) |
| Block 14 | 950–952 | `./scripts/reconcile_backlog.py --provider gitlab` | PASS (exit 0) |
| Block 15 | 955–957 | `./scripts/reconcile_backlog.py --provider gitlab --gitlab-url ...` | PASS (exit 0) |
| Block 16 | 960–962 | `./scripts/reconcile_backlog.py --provider github` | PASS (exit 0) |
| Block 17 | 965–974 | Reverse-sync, offline reconciliation, and 23-gate parity lock | PASS (exit 0) |

---

### 2. Residual Inline Python Scripts & Monkeypatching
- **Search Pattern `python3\s+-c.*`**: 0 occurrences found across all 1092 lines of `README.md`.
- **Search Pattern `monkeypatch` (case-insensitive)**: 0 occurrences found.
- **Section 5.4 Audit**: Verified that Section 5.4 no longer contains the previous 80-line inline Python monkeypatching snippet or manual `cp` loops. It now contains a clean architectural boundary diagram and instructions directing users to canonical domain distribution templates.

---

### 3. Residual Domain Clone Commands
- **Search Pattern `git\s+clone.*DEAP-uas-infrastructure-safety.*`**: 0 occurrences found across all 1092 lines.
- **All `git clone` commands audited in `README.md`**:
  1. Line 248 (in executable bash block 7):
     ```bash
     git clone https://github.com/gintatkinson/DEAP01-spec-core.git /tmp/deap_compiler && bash /tmp/deap_compiler/scripts/install_pipeline.sh . && rm -rf /tmp/deap_compiler
     ```
     *Target*: Upstream compiler itself (`DEAP01-spec-core`), strictly adhering to R1 remote bootstrap specifications.
  2. Line 282 (in non-executable ASCII art box diagram):
     ```text
     │  git clone "<domain-url>" ./.tmp-pipeline && ...
     ```
     *Target*: Quoted generic placeholder within an architecture diagram illustrating Tier 2 customer onboarding. Zero executable bash code fences clone domain templates.

---

### 4. Syntax Traps: Parentheses in Comments & Unquoted Angle Brackets
- **Comments with parentheses**: Scanned all `#` comment lines across all 17 bash blocks.
  - **Count**: 0 comments with parentheses found.
- **Angle-bracket (`<...>`) placeholders in bash blocks**: Scanned all non-comment lines across all 17 bash blocks.
  - **Count**: 2 matches:
    - Line 241: `bash scripts/install_pipeline.sh "<path-to-domain-template>"` (properly double-quoted).
    - Line 326: `... >> CLAUDE.md` (standard shell redirection, not a placeholder).
  - **Unquoted `<...>` placeholders**: 0 found.

---

## Challenges & Advisory Observations

### [Low] Advisory Observation: CLI Grammar Synopsis Notation in Executable Bash Block 13

- **Assumption challenged**: That all content inside ```` ```bash ```` fences is directly executable without editing.
- **Observed snippet** (lines 465–467):
  ```bash
  python3 scripts/generate_wbs_suite.py [--workspace PATH] [--output-dir PATH]
  ```
- **Attack scenario**:
  - In `bash`: `bash -n` parses `[--workspace PATH]` as valid pathname expansion pattern (glob syntax). However, executing it directly causes Python `argparse` to fail with:
    `generate_wbs_suite.py: error: unrecognized arguments: [--workspace PATH] [--output-dir PATH]`.
  - In `zsh` (macOS default interactive shell): Running the line directly fails immediately with:
    `zsh:1: bad pattern: [--workspace`.
- **Blast radius**: If an operator attempts to copy-paste line 466 directly into their terminal without removing the synopsis brackets, the command fails.
- **Mitigation**: Change line 466 to standard default execution, or use comments/text fence:
  ```bash
  # Generate WBS suite (or specify --workspace and --output-dir)
  python3 scripts/generate_wbs_suite.py
  ```
- **Impact on Verdict**: LOW. This does not violate any R1 requirement and passes `bash -n`. Recommended as a polish item.

---

## Stress Test Results

1. **`bash -n` on all 17 bash blocks** → Exit 0 → PASS (all 17 blocks exit 0).
2. **Regex scan for `python3 -c`** → 0 matches → PASS.
3. **Regex scan for `monkeypatch`** → 0 matches → PASS.
4. **Regex scan for `git clone ... DEAP-uas-infrastructure-safety`** → 0 matches → PASS.
5. **AST baseline conformance check** (`python3 scripts/verify_downstream_baseline.py --no-domain`) → 30/30 checks pass → PASS.
6. **Pytest test suite** (`pytest tests/`) → 23/23 tests pass → PASS.

---

## Unchallenged Areas

- Non-bash code fences (`text`, `mermaid`, `json`): Validated structurally via `verify_downstream_baseline.py` Check 13 (KaTeX) and Check 14 (Mermaid integrity), but not executed as shell scripts since they are declared as text/data.
- Downstream repositories (`DEAP-uas-infrastructure-safety`, `uav-011`): Out of scope for this task (focused strictly on `DEAP01-spec-core/README.md`).

---

## Verdict

**APPROVE**  
`DEAP01-spec-core/README.md` complies fully with all requirements of R1 and meets all syntax hygiene gates.
