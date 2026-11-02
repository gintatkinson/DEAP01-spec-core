| Attribute | Specification Detail |
| :--- | :--- |
| **Issue ID** | #654 |
| **Title** | Use Case 31: Dogfood SysML v2 Markdown Ingestion and Blueprint Gate Conformance |
| **Type** | use-case |
| **Primary Actor** | Operator_Or_CI_Pipeline |
| **Scope** | ClosedLoopDefectPublisherGate |
| **Level** | User Goal |
| **Generation Mode** | subagent |
| **Specification Source** | [schema/model.sysml](../../schema/model.sysml) |

# Use Case 31: Dogfood SysML v2 Markdown Ingestion and Blueprint Gate Conformance

## UML Diagrams

```mermaid
graph TD
    subgraph "ClosedLoopDefectPublisherGate System Boundary"
        UC_Main(["Dogfood SysML v2 Markdown Ingestion and Blueprint Gate Co..."])
        UC_Step01(["./target/release/verify-baseline check-blueprint docs/des..."])
        UC_Step02(["All 12 blueprint verification gates pass, DeterministicSt..."])
    end
    Operator_Or_CI_Pipeline(("Operator_Or_CI_Pipeline")) --- UC_Main
    UC_Main -. "<<include>>" .-> UC_Step01
    UC_Main -. "<<include>>" .-> UC_Step02
```

## 1. Actors
- `Operator_Or_CI_Pipeline`

## 2. Preconditions
- This blueprint specification at `./.pipeline/upstream/docs/designs/blueprint-06-deterministic-static-defect-dossier-auditor.md` (and `schema/model.sysml`) is present.
- **Contract A** (100% normative OMG SysML v2.0 + KerML 1.0 textual grammar coverage with full Definition and Usage symmetry and zero 16-construct artificial caps) and **Contract B** (`ra_ap_syntax` typed AST + Stage-2 `./lints/deap-dylint-mir/` `rustc_middle::mir` (`TyCtxt`) 5-pillar defect dossier auditor + `deap_core::dart_cst` and `deap_core::sysml_cst` (`logos` + `rowan` + `salsa` + `ariadne`) + `pulldown-cmark` + complete deletion of `./crates/deap-core/src/source_ast.rs` (`source_ast.rs`) with zero hand-rolled scanners) are active across `< 380`-SLOC submodules.

## 3. Trigger
- `./target/release/verify-baseline check-blueprint ./docs/designs/blueprint-06-deterministic-static-defect-dossier-auditor.md --dry-run` is executed.

## 4. Main Success Scenario
1. `Operator_Or_CI_Pipeline` executes `./target/release/verify-baseline check-blueprint ./docs/designs/blueprint-06-deterministic-static-defect-dossier-auditor.md --dry-run` using `pulldown-cmark` to parse the 8-part blueprint AST.
2. `ClosedLoopDefectPublisherGate` validates all 12 mechanical blueprint verification gates for **REQ-0606**, including KaTeX invariants (`Invariant_REQ_0606_SC_01`, `Invariant_REQ_0606_SC_02`), Mermaid state and sequence diagrams, and 5-column STPA/FMECA tables.
3. `Operator_Or_CI_Pipeline` invokes `./target/release/ingest-sysml` using `deap_core::sysml_cst` (`logos` + `rowan` + `salsa`) with full OMG SysML v2.0 + KerML 1.0 Definition and Usage symmetry (zero 16-construct caps) to lower `DeterministicStaticDefectAuditor`, `ClosedLoopDefectPublisherGate`, `DeterministicDossierSynthesizer`, `FivePillarStaticAstAnalyzer`, and `MermaidUmlDiagramGenerator` into canonical SysML v2.
4. `ingest-sysml` emits `./docs/designs/blueprint-06-deterministic-static-defect-dossier-auditor.sysml` and `.digest.json` and verifies bijective roundtrip parity (`roundtrip_ast_mismatch_count == 0`).
5. `ClosedLoopDefectPublisherGate` confirms 100% schema conformance with `schema/model.sysml` and exits with code 0.

## 5. Alternate and Exception Flows
- 3a. Blueprint Markdown fails any of the 12 mechanical verification gates in `check-blueprint`:
  - `verify-baseline` returns `Result::Err`, emits an `ariadne` / `miette` structural diagnostic, and terminates fail-closed with exit code 1.
- 4a. Bijective SysML v2 roundtrip mismatch or unresolved port/part reference during `ingest-sysml`:
  - `ingest-sysml` aborts emission via `Result<T, E>`, preserves prior verified `.sysml` and `.digest.json` state, and exits fail-closed with code 1.
- 5a. Invariant violation under `Invariant_REQ_0606_SC_01` or `Invariant_REQ_0606_SC_02`:
  - `ClosedLoopDefectPublisherGate` aborts execution without partial state mutation, preserves prior verified state, and returns fail-closed status code 1 to `Operator_Or_CI_Pipeline`.

## 6. Postconditions
- **3-Layer DoD Realization**: Layer 1 (Domain State & Data Model: lossless `deap_core::sysml_cst` green/red CST and `.digest.json` metadata), Layer 2 (Logic & State Management: 12-gate blueprint verification and bijective SysML v2 lowering inside `< 380`-SLOC submodules with `source_ast.rs` and all legacy hand-rolled scanners deleted), and Layer 3 (Presentation & Actuator Interface Binding: `check-blueprint` and `ingest-sysml` CLI outputs and `ariadne` / `miette` diagnostics) are satisfied.
- All 12 blueprint verification gates pass, `DeterministicStaticDefectAuditor` and its component metamodel are lowered into `./docs/designs/blueprint-06-deterministic-static-defect-dossier-auditor.sysml` and `.digest.json` with bijective roundtrip parity, and the command exits with code 0.

## 8. Realization Matrix
| Specification Item | Type | Link / Reference |
| :--- | :--- | :--- |
| **#726** | User Story | [User Story 37: Verify Baseline Audit Code Path Crates](../user-stories/US-37-VerifyBaselineAuditCodePathCrates.md) |
| **#729** | User Story | [User Story 42: Execute12 Check Validate Defect Dossier](../user-stories/US-42-Execute12CheckValidateDefectDossier.md) |
| **#680** | Feature | [Feature 26: [BP 06 Deterministic Static 5 Pillar Code Auditor and Defect Dossier Synthesizer Blueprint] ClosedLoopDefectPublisherGate](../features/FEAT-26-ClosedLoopDefectPublisherGate.md) |
| **#682** | Feature | [Feature 28: [BP 06 Deterministic Static 5 Pillar Code Auditor and Defect Dossier Synthesizer Blueprint] DeterministicStaticDefectAuditor](../features/FEAT-28-DeterministicStaticDefectAuditor.md) |

## Source References
Use case operational flows and lifecycle activity references:
- System Architecture Model: `schema/model.sysml`
- Normative Systems Engineering Standard: ISO/IEC/IEEE 29148:2018 §6.4.2 Concept of Operations

Operational use case realization and traceability links derive from `schema/model.sysml` pursuant to ISO/IEC/IEEE 29148.
