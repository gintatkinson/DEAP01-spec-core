# Handoff Report: Upstream Compiler README.md Survey & Architecture Hygiene

**Subagent ID:** `explorer_r4_1`  
**Handoff Type:** Hard (Task complete)  
**Target File:** `/Users/perkunas/jail/DEAP01-spec-core/README.md`  
**Working Directory:** `/Users/perkunas/jail/DEAP01-spec-core/.agents/explorer_r4_1`  
**Report Reference:** `/Users/perkunas/jail/DEAP01-spec-core/.agents/explorer_r4_1/report.md`  

---

## 1. Observation

Direct file readings, searches, and test executions confirmed the following facts:

1. **Section 5.4 Existence & Bounds:**
   - Lines 273–362 of `/Users/perkunas/jail/DEAP01-spec-core/README.md` contain `### 5.4 Direct Copy / Manual Setup`.
   - Lines 278–308 contain 31 lines of manual, unverified file/directory `cp` / `rm` / `mkdir` loops copying `.tmp-pipeline/skills`, `rules`, `.pipeline`, `.agents`, `scripts`, and `tests`.
   - Lines 310–339 contain a 30-line inline Python script using regex and string substitution (`upstream_h` vs `downstream_h`) to monkeypatch `AGENTS.md`.
   - Lines 347–358 contain a 12-line inline Python script modifying `.pipeline/codebase_rules.json` to hardcode GitLab tracker labels.
   - Total length of Section 5.4 is 90 lines.

2. **Conflated Customer Onboarding Content:**
   - Lines 241–271 of `README.md` contain `### 5.3 Tier 2: Customer Project Onboarding Guide (Domain Template -> Customer Workspace)`.
   - Line 253 hardcodes `DOMAIN_REMOTE_URL="https://github.com/gintatkinson/DEAP-uas-infrastructure-safety.git"`.
   - Line 262 hardcodes: `git clone "https://github.com/gintatkinson/DEAP-uas-infrastructure-safety.git" ./.tmp-pipeline && bash ./.tmp-pipeline/scripts/install_pipeline.sh . && rm -rf ./.tmp-pipeline`.
   - Line 17 refers to onboarding customer application workspaces (`uav-*`) via "scripts/install_pipeline.sh or Direct Copy."
   - Line 383–402 presents a `DOWNSTREAM_CUSTOMER_PROJECT` `AGENTS.md` file template inside the upstream compiler documentation.

3. **Missing Compiler-Centric Quickstart Commands:**
   - Section 5 currently has no documentation on executing the SysML compiler itself (`python3 scripts/compile_sysml.py`), testing the compiler (`pytest`), or running the compiler conformance check (`python3 scripts/verify_downstream_baseline.py --no-domain`).
   - Sibling maintainer propagation in Section 5.2 (lines 217–220) documents running `bash ../DEAP01-spec-core/scripts/install_pipeline.sh` from the target repo, rather than documenting execution from `DEAP01-spec-core` (`bash scripts/install_pipeline.sh "<path-to-domain-template>"`).
   - Remote bootstrap in Section 5.2 (line 228) creates a temporary folder `./.tmp-pipeline` inside the working directory rather than using `/tmp/deap_compiler`.

4. **Code Fence Syntax Hygiene:**
   - Bare code blocks at lines 124–150 and 153–173 omit language tags and contain `# ... (.gitkeep)` comments.
   - Any shell code block containing `<path-to-domain-template>` must be quoted (`"<path-to-domain-template>"`) to prevent bash input redirection errors (`<` interpreted as stdin file).

5. **Test and Conformance Baseline:**
   - `python3 -m pytest tests/` passes 23 of 23 tests in 6.72s.
   - `python3 scripts/verify_downstream_baseline.py --no-domain` passes all 30 checks cleanly.

---

## 2. Logic Chain

1. **From Observation 1:** Section 5.4 duplicates the functionality of `scripts/install_pipeline.sh` using brittle inline Python regex and raw `cp` commands. If `AGENTS.md` or `.pipeline/` structure changes, Section 5.4 becomes broken. Moreover, in an upstream compiler repository, direct manual copying violates standard repository boundaries. Therefore, Section 5.4 must be completely excised.
2. **From Observation 2:** `DEAP01-spec-core` is the upstream specification core compiler, not a customer workspace or domain template. Hardcoding customer onboarding commands that clone `DEAP-uas-infrastructure-safety` confuses end users and violates the Pure Schema-Driven Compiler Invariant. Therefore, Section 5.3 must be replaced with a clear architectural explanation that customer workspaces onboard from domain distribution templates (`DEAP-*`), not from `DEAP01-spec-core`.
3. **From Observation 3:** Maintainers and developers interacting with `DEAP01-spec-core` need clear, compiler-focused instructions: how to compile SysML schemas (`scripts/compile_sysml.py`), how to run test suites (`pytest`), how to run compiler baseline checks, and how to propagate compiler tooling into a domain distribution template (`bash scripts/install_pipeline.sh "<path-to-domain-template>"` or remote bootstrap). Therefore, Section 5 should be reorganized around these compiler capabilities.
4. **From Observation 4:** Shell parsers treat `<placeholder>` as input redirection unless quoted. Comments with unescaped parentheses in command-substitution or eval contexts cause subshell syntax errors. Therefore, all placeholders in bash fences must be quoted (`"<path-to-domain-template>"`) and comments must avoid parentheses.
5. **From Observation 5:** All existing regression tests and baseline gates are currently 100% green, providing an unambiguous baseline for subsequent refactoring.

---

## 3. Caveats

- **Scope Boundary:** This investigation was read-only per the explorer archetype and project rules. No source files or `README.md` were modified in this step.
- **Milestone Interdependency:** Milestone 1 modifies `DEAP01-spec-core/README.md`. Milestone 2 modifies `scripts/install_pipeline.sh` to dynamically detect repository roles (`DOMAIN_DISTRIBUTION_TEMPLATE` vs `DOWNSTREAM_CUSTOMER_PROJECT`) and scaffold appropriate README files for downstream tiers. The findings here inform Milestone 1 directly and provide context for Milestone 2.

---

## 4. Conclusion

1. **Section 5.4 (lines 273–362)** is completely mapped and ready for deletion by the Milestone 1 worker subagent.
2. **Section 5.3 (lines 241–272)** should be replaced with a clean conceptual boundary statement explaining that customer onboarding occurs exclusively from Tier 1 domain templates.
3. **Section 5.2** should document upstream compiler execution (`python3 scripts/compile_sysml.py`, `pytest`, `verify_downstream_baseline.py --no-domain`), followed by clean maintainer propagation commands:
   - Local: `bash scripts/install_pipeline.sh "<path-to-domain-template>"`
   - Remote: `git clone https://github.com/gintatkinson/DEAP01-spec-core.git /tmp/deap_compiler && bash /tmp/deap_compiler/scripts/install_pipeline.sh . && rm -rf /tmp/deap_compiler`
   - In-place: `bash scripts/install_pipeline.sh .`
4. All shell code fences must quote angle-bracket placeholders and avoid parentheses in shell comments.

---

## 5. Verification Method

To independently verify the findings in this report:

```bash
# 1. Verify lines 273-362 in README.md contain Section 5.4:
sed -n '273,362p' /Users/perkunas/jail/DEAP01-spec-core/README.md

# 2. Verify hardcoded domain URL in Section 5.3:
grep -n "DEAP-uas-infrastructure-safety" /Users/perkunas/jail/DEAP01-spec-core/README.md

# 3. Verify compiler test suite and baseline pass:
python3 -m pytest tests/
python3 scripts/verify_downstream_baseline.py --no-domain
```
