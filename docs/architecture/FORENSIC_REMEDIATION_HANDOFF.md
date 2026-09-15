| Attribute | Value |
| :--- | :--- |
| **Document Identifier** | `DEAP-HANDOFF-FORENSIC-REMEDIATION-001` |
| **Title** | Forensic Remediation & Systems Engineering Standards Conformance Handoff |
| **Version** | 1.0.0 |
| **Date** | 2026-09-14 |
| **Status** | Approved / Baseline |

# Forensic Remediation & Systems Engineering Standards Conformance Handoff

**Document Identifier:** `DEAP-HANDOFF-FORENSIC-REMEDIATION-001`  
**Generated at:** 2026-09-14T15:18:45+03:00  
**Target Repository:** `gintatkinson/DEAP01-spec-core` (Upstream Core Compiler)  
**Base Commit:** `5afda13` on `origin/main` (Clean working tree)  
**Target Audience:** Incoming Autonomous Engineering Agent / Master Coordinator  

---

## 1. Executive Summary

This handoff document provides the authoritative engineering truth and execution baseline to remediate the upstream specification compiler (`DEAP01-spec-core`) and synchronize downstream domain distribution repositories (`DEAP-avionic-flight-safety`, `DEAP-uas-infrastructure-safety`) and customer projects (`uav-009`).

The ConOps assembly compiler (`scripts/assemble_conops.py`) and Gate 26 validator (`conops_completeness_validator.py`) suffered from 6 structural root causes that resulted in:
1. Premature truncation of ConOps at Section 4.5.
2. Silent omission of Section 4.7 (Super-System OV-2) and Section 4.8 (Subsystems SV-1 / SV-4).
3. False-positive discarding of legitimate OEM subsystem hardware specifications.
4. Hallucination of synthetic, fictional subsystem archetypes (`_get_archetype_subsystems`) out of thin air.
5. Ingestion blindness to non-SysML schemas (`.yaml`, `.json`, `.proto`, `.idl`, `.arxml`) and markdown subsystem tables.
6. Gate 26 fail-open verification emitting false-positive `100% PASS (0/0)` when 0 parts were ingested.

All 6 root causes have been formally audited against the 5-pillar adversarial framework, mechanically verified with `scripts/file_defect.py --validate-only`, and lodged on GitHub.

---

## 2. Master Defect Dossiers Lodged on GitHub

| Issue | Target File & Component | Pillar & Severity | Defect Summary & Root Cause |
| :--- | :--- | :--- | :--- |
| **[#296](https://github.com/gintatkinson/DEAP01-spec-core/issues/296)** | `scripts/assemble_conops.py:2650` | Resource Lifecycle<br>**Critical** | **Ingestion False-Positive Discard**: `is_component_icd_document()` rejects valid OEM subsystem specifications in `schema/` or `docs/` if they contain pinouts, tables, or the words *"interface control document"*. |
| **[#297](https://github.com/gintatkinson/DEAP01-spec-core/issues/297)** | `scripts/assemble_conops.py:1550` | Semantic Traceability<br>**Critical** | **Synthetic Subsystem Fabrication**: Fallback logic synthesizes ungrounded domain archetypes (`Clinical Supervisory Controller`, `Automatic Train Protection`, fake `PORT-...` names) when 0 parts are ingested, violating the *Pure Schema-Driven Compiler Invariant*. |
| **[#298](https://github.com/gintatkinson/DEAP01-spec-core/issues/298)** | `conops_completeness_validator.py:30` | Semantic Traceability<br>**Critical** | **Gate 26 Schema Offset**: Checks an obsolete 12-section layout (Pugh in Sec 4, SORA in Sec 6, Scenarios in Sec 10), diverging from IEEE 1362-1998 and ISO/IEC/IEEE 15288:2023. |
| **[#299](https://github.com/gintatkinson/DEAP01-spec-core/issues/299)** | `scripts/assemble_conops.py:2690` | Semantic Traceability<br>**Critical** | **Ingestion Blindness**: Auto-discovery ignores `.yaml`, `.json`, `.proto`, `.idl`, `.arxml`, and ignores `docs/architecture/` and `docs/research/`. Markdown parser fails to extract subsystem/port tables into `self.ast_parts`. |
| **[#300](https://github.com/gintatkinson/DEAP01-spec-core/issues/300)** | `README.md:100` & `install_pipeline.sh:150` | Semantic Traceability<br>**Important** | **Defective Prompts & Missing Scaffolding**: Prompts mandate authoring a monolithic `CONOPS.md` directly (omitting 24 modular units and `assemble_conops.py`), installer omits `docs/interfaces/` scaffolding, and `docs/OPERATOR_PROMPT_CATALOG.md` is missing upstream. |
| **[#301](https://github.com/gintatkinson/DEAP01-spec-core/issues/301)** | `conops_completeness_validator.py:380` | Test Integrity<br>**Critical** | **Gate 26 Fail-Open Blind Spot**: Wraps Section 4.8 PartDef coverage check in `if part_defs:`. When ingestion drops parts and `part_defs` is empty, it computes `0 / 0` and emits a false-positive `100% PASS`. |

---

## 3. Core Systems Engineering Architecture Invariants

The incoming agent must enforce these invariants unconditionally:

### 1. Pure Schema-Driven Invariant (Zero Synthetic Fabrication)
- All domain archetype synthesis (`_get_archetype_subsystems`) is strictly expunged.
- 100% of subsystems, ports, and allocations must derive from ingested OEM documents in `schema/`, `docs/architecture/`, and `docs/research/`.
- If 0 AST parts or schemas exist after workspace scanning, the compiler must **FAIL CLOSED** with exit code 1 (`RuntimeError`), forcing the operator to provide valid OEM documentation.

### 2. Section 4.7 Operational Context & Segment Boundaries (DoDAF OV-2 / IEEE 1362 §5.3)
- Derived dynamically from operational nodes, external actors, and boundary connections declared in the schema/model.
- Zero hardcoded drone topologies (`Ground Control Station`, `Launch & Recovery System`, `GSE`).

### 3. Section 4.8 Subsystems Architecture (DoDAF SV-1 / SV-4 / ISO 15288 §6.4.2)
For 100% of declared OEM AST parts:
- **4.8.x.1 Functional Allocation (SV-4)**: Explicit mapping of what operational capabilities (from Section 3) and operational activities (from Section 6 / OV-5b) are allocated to this subsystem.
- **4.8.x.2 Dedicated Interfaces (SV-1 / SV-2)**: Authentic ports, protocols, and interconnect bindings derived from OEM schemas.
- **4.8.x.3 Physical Resource Allocations**: Factually derived from OEM datasheets or established as a formal engineering budget allocation with defined margin ($\pm 15\%$), eliminating arbitrary divide-by-parts arithmetic.
- **4.8.x.4 Lifecycle Modes & Statecharts**: Operational behavior across $\Phi_{\mathrm{lifecycle}}$ modes.

### 4. Gate 26 Harmonization & Fail-Closed Gate
- Align `MANDATORY_SECTIONS` and tables to the canonical 12 sections (IEEE 1362-1998 / ISO 15288):
  * Sec 1: Scope, Identification & Normative Baseline
  * Sec 2: Current Situation & Deficiency Analysis
  * Sec 3: Proposed Capabilities & Trade-Offs (Pugh Decision Matrix)
  * Sec 4: User Classes, Stakeholders & Architecture (Sec 4.7 Super-System & Sec 4.8 Subsystems)
  * Sec 5: Operational State Space & Risk Assessment (SORA 4D Volume)
  * Sec 6: OMG UAF Operational Activity Taxonomy
  * Sec 7: Operational Information Exchange (Op-Tx) Matrix
  * Sec 8: Operational Environments & MIL-STD-810H
  * Sec 9: Multi-Threaded Operational Scenarios & Timelines
  * Sec 10: Maintenance & Sustainment Concepts (O/I/D Hierarchy)
  * Sec 11: Operational Impacts, Limitations & Trade Studies
  * Sec 12: Emergency Decision & Contingency Matrix
- Remove `if part_defs:` bypass; Section 4.8 with 100% AST part coverage is strictly mandatory.

---

## 4. Execution Roadmap

### Phase 1: Upstream Core Compiler Remediation (`DEAP01-spec-core`)
1. **WP-1.1 (Resolves #296, #299)**: Fix `is_component_icd_document()`, expand `auto_detect_workspace_parameters()` to scan `.yaml`, `.json`, `.proto`, `.idl`, `.arxml`, `.md` across `schema/`, `docs/architecture/`, and `docs/research/`, and implement markdown table parsing in `ingest_markdown_text()` to populate `self.ast_parts`.
2. **WP-1.2 (Resolves #297)**: Expunge `_get_archetype_subsystems()`, implement fail-closed abort if parts are missing, synthesize dynamic Section 4.7 (OV-2) and Section 4.8 (SV-1/SV-4) from model boundaries. Guaranteed injection in `04_USER_CLASSES_AND_STAKEHOLDERS.md`.
3. **WP-1.3 (Resolves #298, #301)**: Harmonize Gate 26 (`conops_completeness_validator.py` in `skills/` and `.agents/`) to the canonical 12 sections and eliminate the `if part_defs:` fail-open check.
4. **WP-1.4**: Harmonize `CONOPS_CANONICAL_TEMPLATE.md` in `skills/` and `.agents/` and update `tests/test_assemble_conops.py` and `tests/test_conops_and_mission_intent_validators.py`.
5. **WP-1.5 (Resolves #300)**: Update `scripts/install_pipeline.sh` (scaffold `docs/interfaces/` and modular units), author `docs/OPERATOR_PROMPT_CATALOG.md`, and update `README.md`.
6. **WP-1.6**: Run full test discovery (`python3 -m unittest discover tests`), commit, and push to `origin/main`.

### Phase 2: Downstream Domain Distribution Synchronization
1. Lodge tracking issues on `gintatkinson/DEAP-avionic-flight-safety` (Tier 1 Parent) and `gintatkinson/DEAP-uas-infrastructure-safety` (Tier 2 Child) linking to upstream issues #296–#301.
2. Propagate the updated tooling down the distribution chain and verify tests.

### Phase 3: Customer Project Re-Compilation (`uav-009`)
1. Re-compile `uav-009` with its authentic OEM documents.
2. Verify that Section 4.7 and Section 4.8 compile with 100% genuine OEM grounding, zero synthetic fallback, and pass Gate 26 with exit code 0.

---

## 5. Pre-Flight Verification Commands for Incoming Agent

```bash
# 1. Verify working tree is clean
git status

# 2. Verify git log is at base commit 5afda13
git log -n 1 --oneline

# 3. Read this handoff document in full
cat $APP_DATA_DIR/brain/$CONVERSATION_ID/FORENSIC_REMEDIATION_HANDOFF.md

# 4. View approved implementation plan
cat $APP_DATA_DIR/brain/$CONVERSATION_ID/implementation_plan.md
```
