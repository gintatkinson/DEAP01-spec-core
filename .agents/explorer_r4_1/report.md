# Investigation & Survey Report: Upstream Compiler README.md Hygiene & Propagation

**Explorer Subagent ID:** `explorer_r4_1`  
**Repository Classification:** `UPSTREAM_SPEC_CORE_COMPILER`  
**Target File:** `/Users/perkunas/jail/DEAP01-spec-core/README.md`  
**Date:** 2026-09-21  

---

## Executive Summary

This report documents the architectural and textual survey of `README.md` in `DEAP01-spec-core`. The investigation identified:
1. Exact line bounds (lines 273–362, 90 lines total) of the fragile Section 5.4 containing an ~80-line inline Python monkeypatching script, manual `cp`/`rm`/`mkdir` loops, and brittle JSON edits.
2. Inappropriate customer-facing and domain-specific content in the upstream compiler README, including Section 5.3 which hardcodes customer onboarding commands cloning `DEAP-uas-infrastructure-safety`.
3. Clear requirements for restructuring Section 5 around upstream compiler execution (`python3 scripts/compile_sysml.py`, `pytest`, `verify_downstream_baseline.py --no-domain`) and clean maintainer propagation commands (`bash scripts/install_pipeline.sh "<path-to-domain-template>"` and remote bootstrap via `/tmp/deap_compiler`).
4. Shell code fence syntax rules ensuring zero unquoted angle-bracket placeholders and zero unescaped parentheses in shell comments.

---

## 1. Survey of Section 5.4: Manual Snippets & Inline Monkeypatching Scripts

### 1.1 Line Boundaries and Structure
- **Section Heading:** Line 273 (`### 5.4 Direct Copy / Manual Setup`)
- **Prose Intro:** Lines 274–276
- **Code Block Start:** Line 277 (```` ```bash ````)
- **Code Block End:** Line 361 (```` ``` ````)
- **Section End:** Line 362 (preceding `### 5.5 Setup for Google Antigravity / Gemini CLI`)
- **Total Span:** 90 lines (lines 273–362)

### 1.2 Breakdown of Content to Purge
1. **Manual File and Directory Copy Loops (Lines 278–308, 31 lines):**
   ```bash
   git clone "https://github.com/gintatkinson/DEAP01-spec-core.git" ./.tmp-pipeline
   rm -rf ./skills ./rules ./.pipeline ./.agents ./scripts ./tests
   cp -RP ./.tmp-pipeline/skills ./
   cp -RP ./.tmp-pipeline/rules ./
   if [ ! -d ./schema ]; then
     if [ -d ./.tmp-pipeline/schema ]; then
       cp -RP ./.tmp-pipeline/schema ./
     else
       mkdir -p ./schema
     fi
   fi
   cp -RP ./.tmp-pipeline/.pipeline ./
   rm -rf ./.pipeline/upstream
   cp -RP ./.tmp-pipeline/.agents ./
   cp -RP ./.tmp-pipeline/scripts ./
   cp -RP ./.tmp-pipeline/tests ./
   if [ ! -f ./README.md ]; then
     if [ -f ./.tmp-pipeline/README.md ]; then
       cp ./.tmp-pipeline/README.md ./
     fi
   fi
   # For GitLab Customer Projects:
   if [ -f ./.tmp-pipeline/.pipeline/templates/.gitlab-ci.yml ]; then
     cp ./.tmp-pipeline/.pipeline/templates/.gitlab-ci.yml ./.gitlab-ci.yml
   fi
   if [ -f ./.gitignore ]; then
     cat ./.tmp-pipeline/.gitignore >> ./.gitignore
   else
     cp ./.tmp-pipeline/.gitignore ./
   fi
   ```
2. **Inline Python Monkeypatching Script 1: AGENTS.md Rewriting (Lines 310–339, 30 lines):**
   ```bash
   mkdir -p ./.agents
   python3 -c "
   import os
   src = './.tmp-pipeline/AGENTS.md' if os.path.exists('./.tmp-pipeline/AGENTS.md') else 'AGENTS.md'
   with open(src, 'r', encoding='utf-8') as f:
       content = f.read()

   upstream_h = '''## Repository Role & Scope Classification
   - **Repository Classification:** \`UPSTREAM_SPEC_CORE_COMPILER\` (Digital Engineering Agent Platform Core Specification Compiler)
   ...'''

   downstream_h = '''## Repository Role & Scope Classification
   - **Repository Classification:** \`DOWNSTREAM_CUSTOMER_PROJECT\` (Domain-Specific Safety-Critical Engineering Project)
   ...'''

   if upstream_h in content:
       transformed = content.replace(upstream_h, downstream_h)
   else:
       import re
       transformed = re.sub(
           r'## Repository Role & Scope Classification\n- \*\*Repository Classification:\*\* `UPSTREAM_SPEC_CORE_COMPILER`[^\n]*\n- \*\*Sentinel Indicator:\*\* [^\n]*\n- \*\*Domain Template & Customer Data Boundary:\*\* [^\n]*',
           downstream_h,
           content
       )

   for dest in ['./.agents/AGENTS.md', './AGENTS.md']:
       with open(dest, 'w', encoding='utf-8') as f:
           f.write(transformed)
   "
   ```
3. **Manual Workspace Cleanups & Mkdirs (Lines 340–346, 7 lines):**
   ```bash
   rm -rf ./.tmp-pipeline
   find . -name ".DS_Store" -delete 2>/dev/null || true
   mkdir -p ./docs/conops ./docs/safety ./docs/management ./docs/architecture/blueprints ./docs/epics ./docs/features ./docs/user-stories ./docs/use-cases ./.pipeline/contracts ./.pipeline/domain_specs ./.pipeline/profiles

   # Verify pipeline directories, agent configuration, and skills
   test -d ./.pipeline && test -d ./skills && test -d ./.agents/skills && echo "Pipeline directories (.pipeline, skills, .agents/skills) verified successfully."
   ```
4. **Inline Python Monkeypatching Script 2: GitLab Rules JSON (Lines 347–358, 12 lines):**
   ```bash
   # Configure for GitLab if applicable
   python3 -c "
   import json, os
   p = '.pipeline/codebase_rules.json' if os.path.exists('.pipeline') else 'codebase_rules.json'
   try:
     with open(p, 'r') as f: d = json.load(f)
   except Exception: d = {}
   d.setdefault('tracker_rules', {})['provider'] = 'gitlab'
   d['tracker_rules']['labels'] = {'epic':'type::epic','feature':'type::feature','user_story':'type::user-story','use_case':'type::use-case','ready_for_review':'status::ready-for-review','resolved':'status::fixed-resolved'}
   with open(p, 'w') as f: json.dump(d, f, indent=2)
   "
   ```
5. **Turnkey Setup Scripts Called Post-Copy (Lines 359–360, 2 lines):**
   ```bash
   python3 scripts/setup_git_hooks.py --install
   python3 skills/spec-orchestrator/scripts/bootstrap_tracker_labels.py
   ```

### 1.3 Rationale for Purging Section 5.4
- **Brittle and Stale:** Hardcoded Python replacement strings fall out of synchronization whenever `AGENTS.md` headers or pipeline schemas evolve.
- **Redundant:** Every single operation performed in this snippet is executed deterministically and safely by `scripts/install_pipeline.sh`.
- **Architectural Contamination:** Direct copy instructs users to bypass the tested installer, leaving workspaces in unverified, semi-configured states.

---

## 2. Survey of Conflated Onboarding & Domain Repos in README.md

| Section & Lines | Conflated Content | Reason for Remediation | Proposed Remediation |
|---|---|---|---|
| **Section 1 (lines 17–18)** | Mentions onboarding `uav-*` workspaces via "...`scripts/install_pipeline.sh` or Direct Copy." | Obsolete reference to purged Section 5.4 Direct Copy method. | Remove "or Direct Copy"; state that customer workspaces derive from Tier 1 domain templates. |
| **Section 5.2 (lines 217–220)** | `bash ../DEAP01-spec-core/scripts/install_pipeline.sh` under sibling propagation. | Assumes execution from within the target domain checkout rather than from the compiler repository root. | Document execution from compiler root: `bash scripts/install_pipeline.sh "<path-to-domain-template>"`. |
| **Section 5.2 (lines 227–229)** | `git clone "https://github.com/gintatkinson/DEAP01-spec-core.git" ./.tmp-pipeline && bash ./.tmp-pipeline/scripts/install_pipeline.sh . && rm -rf ./.tmp-pipeline` | Clones into working directory (`./.tmp-pipeline`) rather than `/tmp/deap_compiler`. | Update to: `git clone https://github.com/gintatkinson/DEAP01-spec-core.git /tmp/deap_compiler && bash /tmp/deap_compiler/scripts/install_pipeline.sh . && rm -rf /tmp/deap_compiler`. |
| **Section 5.3 (lines 241–272)** | Complete section titled `### 5.3 Tier 2: Customer Project Onboarding Guide (Domain Template -> Customer Workspace)` with hardcoded `DOMAIN_REMOTE_URL="https://github.com/gintatkinson/DEAP-uas-infrastructure-safety.git"`. | Conflates customer project onboarding with compiler usage. Hardcodes a specific domain repository (`DEAP-uas-infrastructure-safety`) in upstream compiler docs, violating the Pure Schema-Driven Compiler Invariant. | Replace Section 5.3 with an architectural boundary statement clarifying that customer workspaces onboard from domain distribution templates (`DEAP-*`), referencing Section 4. |
| **Section 5.5 (lines 373–376)** | States `.pipeline/upstream/` indicates "Template Distribution Mode". | In this repo, `.pipeline/upstream/` denotes `UPSTREAM_SPEC_CORE_COMPILER`. | Update description to accurately identify `UPSTREAM_SPEC_CORE_COMPILER`. |
| **Section 5.6 (lines 383–402)** | Embeds full sample `DOWNSTREAM_CUSTOMER_PROJECT` `AGENTS.md` block. | Inappropriate in upstream compiler documentation; belongs in downstream installer generators and tests. | Remove or replace with reference to `AGENTS.md` being configured by `install_pipeline.sh`. |
| **Section 5.9 (lines 414–422)** | Labels baseline verification as "Downstream Baseline Verification Gate". | In `DEAP01-spec-core`, this validates the upstream compiler itself. | Re-title to "Compiler Verification & Baseline Gate". |

---

## 3. Recommended Structure for Compiler Usage & Propagation in Section 5

Section 5 should be restructured into four focused, compiler-centric subsections:

### 3.1 Prerequisites & Environment Setup (Section 5.1)
- Python 3.12+ virtual environment setup:
  ```bash
  python3.12 -m venv .venv
  source .venv/bin/activate
  pip install -r requirements.txt
  ```

### 3.2 Running & Verifying the Compiler (Section 5.2)
- **1. Parse and Validate SysML v2 Schema:**
  ```bash
  python3 scripts/compile_sysml.py "schema/model.sysml"
  ```
- **2. Execute Pipeline 0 Compilation Gate:**
  ```bash
  python3 scripts/compile_sysml.py --compile
  ```
  Generates `.pipeline/schema.sysml` and `.pipeline/schema-digest.json`.
- **3. Execute Dynamic Cartesian STPA Transpilation:**
  ```bash
  python3 scripts/compile_sysml.py --stpa-transpile --schema "schema/model.sysml" --out-dir "docs/safety/"
  ```
- **4. Closed-Loop Reverse Synchronization (Specs -> SysML SSOT):**
  ```bash
  python3 scripts/compile_sysml.py --reverse-sync --docs docs/ --schema "schema/model.sysml" --out .pipeline/schema.sysml
  ```
- **5. Run Compiler Unit and Regression Test Suite:**
  ```bash
  python3 -m pytest tests/
  ```
- **6. Verify Upstream Compiler Conformance Baseline:**
  ```bash
  python3 scripts/verify_downstream_baseline.py --no-domain
  ```

### 3.3 Propagating Compiler Tooling to Domain Distribution Templates (Section 5.3)
- **Local Sibling Propagation (Executed from compiler root):**
  ```bash
  # Propagate compiler tooling into a local domain distribution template checkout
  bash scripts/install_pipeline.sh "<path-to-domain-template>"
  ```
- **Remote Bootstrap Propagation (Executed inside domain template without local compiler checkout):**
  ```bash
  # Bootstrap directly from upstream compiler repository into current directory
  git clone https://github.com/gintatkinson/DEAP01-spec-core.git /tmp/deap_compiler && bash /tmp/deap_compiler/scripts/install_pipeline.sh . && rm -rf /tmp/deap_compiler
  ```
- **In-Place Tooling & Governance Updates (Executed inside an initialized domain template):**
  ```bash
  # Re-apply latest pipeline tooling and rules in-place
  bash scripts/install_pipeline.sh .
  ```

### 3.4 Multi-Tier Architecture & Customer Onboarding Boundary (Section 5.4)
- Clear conceptual statement: Customer application workspaces (`uav-*`) do NOT onboard from `DEAP01-spec-core`. They onboard from canonical Domain Distribution Templates (`DEAP-*`).
- Provide cross-reference table of canonical domain distribution templates (as listed in Section 4).

---

## 4. Syntax & Shell Cleanliness Audit

### 4.1 Shell Code Fence Syntax Rules
- **Rule 1: All Angle-Bracket Placeholders Must Be Quoted:**
  - *Bad:* `bash scripts/install_pipeline.sh <path-to-domain-template>`
  - *Why it fails:* In interactive shells or scripts, `<` is parsed as input redirection. If `path-to-domain-template` does not exist on disk, bash exits with `bash: path-to-domain-template: No such file or directory`.
  - *Compliant:* `bash scripts/install_pipeline.sh "<path-to-domain-template>"` or `bash scripts/install_pipeline.sh "/path/to/domain-template"`.
- **Rule 2: Zero Unescaped Parentheses in Shell Comments:**
  - *Bad:* `# Run setup (optional step)`
  - *Why it fails:* In command-substitution subshells `$(...)` or string evals, unescaped parentheses inside comments trigger `syntax error near unexpected token ')'`.
  - *Compliant:* `# Run setup - optional step` or `# Run setup [optional step]`.
- **Rule 3: Explicit Code Fence Language Tags:**
  - Non-shell blocks (such as ASCII directory trees at lines 124–150 and 153–173) must use ```` ```text ```` or ```` ```plaintext ```` rather than bare ```` ``` ```` to prevent markdown linters from checking them as bash.

---

## 5. Verification Commands for Subsequent Worker Agents

Subsequent implementation workers (Milestone 1, 2, 3) must verify documentation updates using:
```bash
# 1. Verify test suite passes
python3 -m pytest tests/

# 2. Verify upstream compiler baseline passes all 30 checks
python3 scripts/verify_downstream_baseline.py --no-domain

# 3. Check for purged Section 5.4 remnants
git grep -n "5.4 Direct Copy" README.md || echo "Section 5.4 successfully purged"
git grep -n "upstream_h =" README.md || echo "Inline monkeypatch successfully purged"
```
