| Attribute | Specification Detail |
| :--- | :--- |
| **Issue ID** | #764 |
| **Title** | Use Case 35: Modular Multi File Flutter_Contract Emission and Monolith Barrel Decomposition |
| **Type** | use-case |
| **Primary Actor** | Operator |
| **Scope** | FourLevelAstPartitionerAndHoister |
| **Level** | User Goal |
| **Generation Mode** | subagent |
| **Specification Source** | [schema/model.sysml](../../schema/model.sysml) |

# Use Case 35: Modular Multi File Flutter_Contract Emission and Monolith Barrel Decomposition

## UML Diagrams

```mermaid
graph TD
    subgraph "FourLevelAstPartitionerAndHoister System Boundary"
        UC_Main(["Modular Multi File Flutter_Contract Emission and Monolith Barrel Decomposition"])
        UC_Step01(["Execute compile-sysml --emit-flutter-contracts Across Lowered Module Graph"])
        UC_Step02(["Emit Per-Package .g.dart Units and Barrel Re-Export Facades Within Modularity Bounds"])
    end
    Operator(("Operator")) --- UC_Main
    UC_Main -. "<<include>>" .-> UC_Step01
    UC_Main -. "<<include>>" .-> UC_Step02
```

## 1. Actors
- `Operator`

## 2. Preconditions
- A multi-package SysML v2 schema parsed via `deap_core::sysml_cst` (`logos` + `rowan` + `salsa` + `ariadne`) under **Contract A** (100% normative OMG SysML v2.0 + KerML 1.0 textual grammar coverage with full Definition and Usage symmetry across `package`/`library package`/`standard library package`, `import`, `metadata def`/`@Metadata`/`#capability`/`#hazard`/`#risk`/`#rt_loop`/`#dal`/`#fls` and backward-compatible `hazard def`/`risk def`/`capability def`, `part def`/`part`, `item def`/`item`, `attribute def`/`attribute`, `enum def`/`enum`, `port def`/`port`, `connection def`/`connection`, `interface def`/`interface`, `flow def`/`flow`, `allocation def`/`allocation`, `occurrence def`/`occurrence`, `action def`/`action`, `state def`/`state`, `calc def`/`calc`, `constraint def`/`constraint`, `requirement def`/`requirement`, `concern def`/`concern`, `use case def`/`use case`, `analysis def`/`analysis`, `verification def`/`verification`, `test case def`/`test case`, `interaction def`/`interaction`, `view def`/`view`, `viewpoint def`/`viewpoint`, `rendering def`/`rendering`, `variation`/`variant`, and KerML specialization/multiplicity/expressions, with zero 16-construct artificial caps) compiled via `./target/release/compile-sysml --emit-flutter-contracts <APP_DIR> --schema <FIXTURE>` and the Rust workspace source file `./crates/compile-sysml/src/flutter_codegen.rs`.

## 3. Trigger
- `Modular_Flutter` contract generation executes and workspace modularity tests inspect both the emitted Dart tree (`deap_core::dart_cst`) and `./crates/compile-sysml/src/`.

## 4. Main Success Scenario
1. `Operator` invokes `./target/release/compile-sysml --emit-flutter-contracts <APP_DIR> --schema <FIXTURE>` on `SelfHostingModularCodegenCompiler`, which validates DAG acyclicity via `verify_dag_and_layers` and partitions the schema via `FourLevelAstPartitionerAndHoister` (`partition_and_hoist`).
2. `ModularFlutterMultiFileEmitter` receives the lowered module graph on `flutter_module_graph_in` (`emit_barrel_reexports = true`, `facade_max_lines = 80`) and invokes `emit_modular_flutter_tree(graph)`.
3. `ModularFlutterMultiFileEmitter` synthesizes per-package and per-classifier `.g.dart` units (`models/generated/<pkg>/<part>_view.g.dart`, `data/generated/ffi/*.g.dart`, `data/generated/dto/<pkg>/<dto>_dto.g.dart`, `data/seeds/generated/*.g.dart`, `test/generated/support/*.g.dart`) along with backward-compatible barrel re-export facades (`<= 80` LOC) verified via `deap_core::dart_cst` (`logos` + `rowan`).
4. Under **Contract B** (official Rust compiler and codegen stack `syn` + `proc-macro2` + `quote` + `prettyplease` + `ra_ap_syntax`, zero hand-rolled scanners) and the **3-Layer Definition of Done (DoD)**, workspace modularity refactoring decomposes monolithic templates in `./crates/compile-sysml/src/flutter_codegen.rs` (`-650+` SLOC deleted) and `./crates/compile-sysml/src/driver.rs` (`-450+` SLOC deleted) into bounded `< 380` exec SLOC submodules under `./crates/compile-sysml/src/modular_codegen/`.
5. `ModularFlutterMultiFileEmitter` emits the verified `GeneratedTreeManifest` on `flutter_manifest_out` and `flutter_tree_out`, satisfying `C_MOD_01` (`<= 400` LOC per file) and `C_MOD_02` (`<= 12` items per file) with exit code 0.

## 5. Alternate and Exception Flows
- 3a. Emitted `.g.dart` unit or barrel facade violates `C_MOD_01` (`> 400` LOC) or `facade_max_lines > 80` (`Diagnostic: E0701`):
  - `ModularFlutterMultiFileEmitter` transitions to `RejectedFailClosed`, emits diagnostic `E0701` on `diagnostic_out`, and aborts transactionally with exit code 1.
- 4a. Directed cycle or upward layer violation detected in target UI/DTO graph (`Diagnostic: E0702` / `E0703`):
  - `TopologicalLayerDagVerifier` rejects the module graph before `emit_modular_flutter_tree`, leaves `<APP_DIR>` unmodified, and exits fail-closed with code 1.
- 5a. Invariant violation while enforcing `Invariant_REQ_0607_SC_01` in `FourLevelAstPartitionerAndHoister`:
  - `FourLevelAstPartitionerAndHoister` aborts `Modular Multi File Flutter_Contract Emission and Monolith Barrel Decomposition` execution, preserves prior verified state, and notifies `Operator`.

## 6. Postconditions
- **Success/Fail-Closed Guarantee (3-Layer DoD)**:
  - **Layer 1 (Domain State & Data Model)**: Per-package and per-classifier `.g.dart` units (`models/generated/<pkg>/<part>_view.g.dart`, `data/generated/ffi/*.g.dart`, `data/generated/dto/<pkg>/<dto>_dto.g.dart`, `data/seeds/generated/*.g.dart`, `test/generated/support/*.g.dart`) and `<= 80`-LOC barrel re-export facades satisfy `C_MOD_01` (`<= 400` LOC) and `C_MOD_02` (`<= 12` items).
  - **Layer 2 (Logic & State Management)**: `./crates/compile-sysml/src/flutter_codegen.rs` is reduced by `-650+` SLOC into `< 380` exec SLOC submodules under `./crates/compile-sysml/src/modular_codegen/` with zero hand-rolled scanners.
  - **Layer 3 (Presentation & Actuator Interface Binding)**: `GeneratedTreeManifest` SHA-256 records are emitted on `flutter_manifest_out` and `flutter_tree_out` with exit code 0.

## 8. Realization Matrix
| Specification Item | Type | Link / Reference |
| :--- | :--- | :--- |
| **#798** | User Story | [User Story 48: build syn file](../user-stories/US-48-build_syn_file.md) (validates typed Rust codegen AST units) |
| **#801** | User Story | [User Story 51: emit modular_flutter_tree](../user-stories/US-51-emit_modular_flutter_tree.md) (emits per-package `.g.dart` units and `<= 80`-LOC barrel facades) |
| **#829** | Feature | [Feature 31: [BP 07 Self Hosting 4 Level Modular Multi File Code Generator Blueprint] FourLevelAstPartitionerAndHoister](../features/FEAT-31-FourLevelAstPartitionerAndHoister.md) (partitions schema packages and classifiers for `Modular_Flutter` contract lowering) |
| **#830** | Feature | [Feature 32: [BP 07 Self Hosting 4 Level Modular Multi File Code Generator Blueprint] ModularFlutterMultiFileEmitter](../features/FEAT-32-ModularFlutterMultiFileEmitter.md) (synthesizes modular `.g.dart` tree and barrel re-exports) |

## Source References
Use case operational flows and lifecycle activity references:
- System Architecture Model: `schema/model.sysml`
- Normative Systems Engineering Standard: ISO/IEC/IEEE 29148:2018 §6.4.2 Concept of Operations

Operational use case realization and traceability links derive from `schema/model.sysml` pursuant to ISO/IEC/IEEE 29148.
