# Master Handover Checkpoint & Formal Acceptance Instructions

**Generated at:** 2026-09-03T12:38:55+03:00  
**Target Audience:** Incoming Autonomous Engineering Agent / Master Coordinator  
**Classification:** `SAFETY-CRITICAL FORENSIC AUDIT, RECOVERY PLAN & FORMAL ACCEPTANCE INSTRUCTIONS`

---

## 1. Executive Summary of System Failures & Contamination

The previous agent made severe architectural and judgment errors across both the upstream compiler (`DEAP01-spec-core`) and downstream domain repositories (`DEAP01-uas-infrastructure-safety`):

1. **Host Filesystem / Local Ingestion Contamination**:
   - The agent ingested unmanaged, unverified local files from local workstation directories (`DEAP_SYSML_V2_SAFETY_MODEL_SPECIFICATION.sysml`, `DEAP_FLIGHT_SYSTEMS_SAFETY_CONCEPT_PAPER.txt`, `UDS-STP-FMECA/`, `UAV Safety Project WBS.docx`) directly into `DEAP01-uas-infrastructure-safety` without establishing cryptographic provenance, verifying whether they were stale/abandoned drafts, or conducting an adversarial audit.
2. **Synthetic Template Pollution in ConOps**:
   - The agent copied generic upstream template units (`skills/spec-conops-engineering/resources/units/`) into `DEAP01-uas-infrastructure-safety/docs/conops/units/`.
   - The agent then ran `assemble_conops.py` with a synthetic `domain_config.json` containing severe dimensional scaling mismatches, producing an assembled `CONOPS.md` (1,581 lines) and `MISSION_INTENT.md` (430 lines) full of blatant physical absurdities (e.g. $20,000\text{ km}$ C2 range for a $20\text{ kg}$ drone, $1.0\text{ min}$ endurance).
   - The agent mistakenly defended checking in both the 12/10 raw template units AND the monolithic assembled Markdown files, creating a dual-source-of-truth nightmare.
3. **Quality Gate False-Positive Blindness ("Failed-Pass")**:
   - Quality Gate 26 (`conops_completeness_validator.py`) passed with exit code 0 because it only checks for surface regex patterns and table structures, completely failing to detect obvious physical, mathematical, and dimensional contradictions.

---

## 2. Inventory of Repositories & Current Git States

### Repository 1: `gintatkinson/DEAP01-spec-core` (Upstream Core Compiler)
- **Local Path:** `../DEAP01-spec-core`
- **Role:** Pure schema-driven abstract compiler and quality gate framework. MUST contain ZERO concrete specifications or domain files.
- **Git Commit:** [`796fcde`](https://github.com/gintatkinson/DEAP01-spec-core/commit/796fcde) (100% clean, synced with `origin/main`).
- **Test Baseline:** 588 / 588 unit tests passing (`python3 -m pytest tests/`).

### Repository 2: `gintatkinson/DEAP-uas-infrastructure-safety` (Contaminated Downstream Repo)
- **Local Path:** `../DEAP01-uas-infrastructure-safety`
- **Role:** Downstream UAS Infrastructure Safety workspace.
- **Git Commit:** [`2f3c6c4`](https://github.com/gintatkinson/DEAP-uas-infrastructure-safety/commit/2f3c6c4) (Pushed to `origin/main`).
- **Contaminated Files to Clean Up / Re-evaluate:**
  - `docs/conops/units/` (Synthetic template copies).
  - `docs/conops/CONOPS.md` (Contaminated 1,581-line assembled file with unit/scale errors).
  - `docs/conops/MISSION_INTENT.md` (Assembled file).
  - `schema/domain_config.json` (Synthetic JSON config with unit scaling mismatches).
  - `schema/UAS_INFRASTRUCTURE_SAFETY.sysml` (Ingested from unmanaged local file, needs formal provenance & AST verification).
  - `docs/research/fmeca/` (Ingested unmanaged files).

### Repository 3: `gintatkinson/DEAP-avionic-flight-safety` (Downstream Avionics Repo)
- **Local Path:** `../DEAP01-avionic-flight-safety`
- **Role:** Downstream Airborne Flight Safety / DO-178C DAL-A workspace.
- **Git Status:** Unpopulated clean landing zones.

---

## 3. Mandatory Acceptance Instructions & Evaluation Protocols

The incoming agent MUST strictly follow these **Four Acceptance Protocols** before declaring any task, fix, or compilation complete:

### Protocol 1: The Anti-Complacency / False-Positive Detection Gate
- **`exit code 0` is NEVER sufficient proof of success.**
- The agent is strictly forbidden from declaring a test passing just because a script finished without an exception.
- You MUST perform live artifact content inspection: read the generated output, check that actual compiled data exists, verify mathematical formulas, and check that no hallucinated or distorted numbers were injected.

### Protocol 2: The 4-Pillar Semantic Physical & Dimensional Check
Every number, formula, and engineering unit in the generated specifications MUST be physically and mathematically coherent:
1. **Dimensional Homogeneity**: Check that units match column headers (e.g. if the table header is `km`, a range of $20,000\text{ m}$ must be recorded as $20.0\text{ km}$, NOT $20,000.0\text{ km}$).
2. **Time Scaling**: Check that durations match column units (e.g. if the table header is `min`, an endurance of $1.0\text{ hr}$ must be recorded as $60.0\text{ min}$, NOT $1.0\text{ min}$).
3. **Mass & Energy Balance**: Subsystem mass fractions must sum to exactly $100.0\%$ MTOW. Power budgets must have positive margins ($\Delta P_{\mathrm{margin}} \ge 15\%$).
4. **Physical Reality Limits**: Never allow a 20 kg drone to claim global satellite range, supersonic velocity, or infinite battery life.

### Protocol 3: The Zero-Placeholder & Duplicate Header Invariant
- **Zero Placeholders**: Zero unrendered `{{...}}` or `[TBD]` placeholders in any document.
- **Strict Section Cardinality**: Exactly 12 sections for ConOps (`## 1.` through `## 12.`) and exactly 10 sections for Mission Intent (`## 1.` through `## 10.`).
- **Zero Duplicate Headers**: Every section header must be unique across the entire document.
- **Table of Contents Parity**: Every Level-2 header must have an exact 1:1 matching link in the Table of Contents.

### Protocol 4: The 10-Variant Acceptance Benchmark Test
To prove that a fix or compiler feature is production-grade, the agent must execute the **10-Variant Acceptance Benchmark**:
1. Provision 10 isolated scratch sandboxes (`../deap_sandboxes/run_01` through `run_10`).
2. Run the end-to-end compilation pipeline in each sandbox.
3. Validate all 10 sandboxes against Quality Gate 26, Gate 28, Gate 29, Gate 16, and Gate 24.
4. Verify that **ALL 10 runs achieve 100% PASS with ZERO warnings and ZERO semantic distortions**.

---

## 4. Step-by-Step Clean-Up & Recovery Action Plan

### Step 1: Purge ConOps Contamination in `DEAP01-uas-infrastructure-safety`
1. Remove the synthetic `docs/conops/units/` directory.
2. Remove the contaminated `docs/conops/CONOPS.md` and `docs/conops/MISSION_INTENT.md`.
3. Remove `schema/domain_config.json`.
4. Restore `docs/conops/` to a clean landing zone (`.gitkeep`) until proper, validated domain specifications are authored.

### Step 2: Establish Provenance of `schema/UAS_INFRASTRUCTURE_SAFETY.sysml`
1. Conduct an **Adversarial Code Audit** on `schema/UAS_INFRASTRUCTURE_SAFETY.sysml`.
2. Verify all 14 packages, 7 PartDefs, and 48 RequirementDefs against the authoritative SysML v2 OMG grammar and the user's flight safety concept paper.
3. Fix all parameter units and physical constants directly in the SysML v2 model (so the SysML AST is the Single Source of Truth).

### Step 3: Implement True Semantic False-Positive Detection in Quality Gates
1. Update `conops_completeness_validator.py` and `safety_trace_validator.py` in `DEAP01-spec-core` to check for **dimensional and physical plausibility** ($\mathbb{Z}^7$ SI units, sensible range bounds, mass-energy balance), rather than surface regex structure.
2. File an upstream defect on `gintatkinson/DEAP01-spec-core` documenting the semantic blindness of Gate 26.

### Step 4: Execute Proper Feature-by-Feature Specification Engineering
1. Use `skills/schema-specification-engineering` with context-isolated subagents to extract **Epics and Features** directly from `schema/UAS_INFRASTRUCTURE_SAFETY.sysml`.
2. Generate **ICD 01 & ICD 02** interface matrices via `skills/spec-icd-engineering`.
3. Author authentic BDD User Stories and UML Use Cases with full traceability tags (`/// Realises:` and `/// Safety-Realises:`).

---

## 5. Non-Negotiable Engineering Guardrails
- **Explicit Repository Paths Mandatory**: Always refer to files and symbols by their recognizable repository paths (e.g. `../DEAP01-uas-infrastructure-safety/schema/UAS_INFRASTRUCTURE_SAFETY.sysml`).
- **SysML v2 is the Sole SSOT**: Never generate specifications by string substitution on markdown templates. All data must derive from the SysML AST.
- **No Silent Assumptions**: If a requirement is ambiguous, verify and ask rather than guessing.
