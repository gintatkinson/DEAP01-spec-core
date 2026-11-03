| Attribute | Specification Detail |
| :--- | :--- |
| **Issue ID** | #766 |
| **Title** | Use Case 37: Dogfood SysML v2 Blueprint Ingestion and Twelve Gate Closed Loop Conformance |
| **Type** | use-case |
| **Primary Actor** | Operator |
| **Scope** | FourLevelAstPartitionerAndHoister |
| **Level** | User Goal |
| **Generation Mode** | subagent |
| **Specification Source** | [schema/model.sysml](../../schema/model.sysml) |

# Use Case 37: Dogfood SysML v2 Blueprint Ingestion and Twelve Gate Closed Loop Conformance

## UML Diagrams

```mermaid
graph TD
    subgraph "FourLevelAstPartitionerAndHoister System Boundary"
        UC_Main(["Dogfood SysML v2 Blueprint Ingestion and Twelve Gate Closed Loop Conformance"])
        UC_Step01(["Run verify-baseline check-blueprint on Blueprint 07 Specification"])
        UC_Step02(["Extract and Roundtrip Compile BP-07 SysML v2 Package and Digest"])
    end
    Operator(("Operator")) --- UC_Main
    UC_Main -. "<<include>>" .-> UC_Step01
    UC_Main -. "<<include>>" .-> UC_Step02
```

## 1. Actors
- `Operator`

## 2. Preconditions
- The engineering blueprint at `./docs/architecture/blueprints/blueprint-07-self-hosting-modular-codegen-compiler.md` (and `./docs/designs/blueprint-07-self-hosting-modular-codegen-compiler.md`) specifying **Contract A** (full OMG SysML v2.0 + KerML 1.0 textual grammar coverage with full Definition and Usage symmetry, zero 16-construct artificial caps) and **Contract B** (`syn` + `proc-macro2` + `quote` + `prettyplease` deterministic fixed-point Rust codegen `SHA-256(Stage 1) == SHA-256(Stage 2)`, `logos` + `rowan` + `salsa` + `ariadne`, `ra_ap_syntax`, zero hand-rolled scanners).

## 3. Trigger
- `./target/release/verify-baseline check-blueprint ./docs/architecture/blueprints/blueprint-07-self-hosting-modular-codegen-compiler.md` is executed.

## 4. Main Success Scenario
1. `Operator` executes `./target/release/verify-baseline check-blueprint ./docs/architecture/blueprints/blueprint-07-self-hosting-modular-codegen-compiler.md` on the upstream blueprint specification.
2. `verify-baseline` evaluates all 12 mechanical blueprint verification gates (8-part structural headers, metadata table, 5-column STPA/FMECA table, KaTeX math blocks, Mermaid diagrams, and 6 Given-When-Then acceptance criteria) via `pulldown-cmark` AST walking with zero hand-rolled scanners.
3. `ingest-sysml` deterministically extracts the SysML v2 package `BP_07_Self_Hosting_4_Level_Modular_Multi_File_Code_Generator_Blueprint` (`REQ-0607`, `C_MOD_01..05`, `FourLevelAstPartitionerAndHoister`, `ModularRustMultiFileEmitter`, `ModularFlutterMultiFileEmitter`, `SelfHostingModularCodegenCompiler`, `TopologicalLayerDagVerifier`) across the full OMG SysML v2.0 + KerML 1.0 grammar (`deap_core::sysml_cst`).
4. `compile-sysml` compiles the extracted `.sysml` model, verifies bijective AST roundtrip equivalence and `< 380` exec SLOC submodule modularity (`./crates/compile-sysml/src/modular_codegen/` with `-450+` SLOC retired in `./crates/compile-sysml/src/driver.rs` and `-650+` SLOC retired in `./crates/compile-sysml/src/flutter_codegen.rs`), and persists `./.pipeline/upstream/schema/model.sysml` and `./.pipeline/upstream/schema/model-digest.json`.
5. All 12 blueprint verification gates pass with exit code 0 and zero Check 34 / Check 36 / Check 37 baseline violations, recording closed-loop conformance for `Operator`.

## 5. Alternate and Exception Flows
- 3a. Any of the 12 mechanical blueprint verification gates fails on `blueprint-07-self-hosting-modular-codegen-compiler.md`:
  - `verify-baseline check-blueprint` emits a structured gate failure diagnostic, leaves `.sysml` and `.digest.json` artifacts untouched, and exits fail-closed with code 1.
- 4a. Bijective SysML v2 AST roundtrip or constraint compilation fails during dogfood ingestion (`Diagnostic: E0704`):
  - `compile-sysml` aborts compilation via `Result::Err`, preserves prior verified schema state, and terminates fail-closed with exit code 1.
- 5a. Invariant violation while enforcing `Invariant_REQ_0607_REQ_0607` in `FourLevelAstPartitionerAndHoister`:
  - `FourLevelAstPartitionerAndHoister` aborts execution, preserves prior verified state, and notifies `Operator`.

## 6. Postconditions
- **Success/Fail-Closed Guarantee (3-Layer DoD)**:
  - **Layer 1 (Domain State & Data Model)**: `./.pipeline/upstream/schema/model.sysml` and `./.pipeline/upstream/schema/model-digest.json` persist the verified `BP_07` SysML v2 package AST and SHA-256 digest.
  - **Layer 2 (Logic & State Management)**: All 12 blueprint verification gates and bijective SysML v2 round-trip compilation pass inside `< 380` exec SLOC submodules with legacy hand-rolled scanners and string-concatenation emitters retired.
  - **Layer 3 (Presentation & Actuator Interface Binding)**: `./target/release/verify-baseline check-blueprint` exits with code 0 and confirms closed-loop GitHub SSOT parity.

## 8. Realization Matrix
| Specification Item | Type | Link / Reference |
| :--- | :--- | :--- |
| **#800** | User Story | [User Story 50: emit modular rust tree](../user-stories/US-50-emit_modular_rust_tree.md) (persists modular Rust tree from compiled `BP_07` schema) |
| **#802** | User Story | [User Story 52: verify fixed point digest](../user-stories/US-52-verify_fixed_point_digest.md) (verifies deterministic fixed-point SHA-256 digest parity) |
| **#829** | Feature | [Feature 31: [BP 07 Self Hosting 4 Level Modular Multi File Code Generator Blueprint] FourLevelAstPartitionerAndHoister](../features/FEAT-31-FourLevelAstPartitionerAndHoister.md) (enforces `Invariant_REQ_0607_REQ_0607` across `BP_07` AST) |
| **#832** | Feature | [Feature 34: [BP 07 Self Hosting 4 Level Modular Multi File Code Generator Blueprint] SelfHostingModularCodegenCompiler](../features/FEAT-34-SelfHostingModularCodegenCompiler.md) (compiles dogfood `BP_07` package AST and enforces `C_MOD_01..05`) |

## Source References
Use case operational flows and lifecycle activity references:
- System Architecture Model: `schema/model.sysml`
- Normative Systems Engineering Standard: ISO/IEC/IEEE 29148:2018 §6.4.2 Concept of Operations

Operational use case realization and traceability links derive from `schema/model.sysml` pursuant to ISO/IEC/IEEE 29148.
