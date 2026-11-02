| Attribute | Specification Detail |
| :--- | :--- |
| **Issue ID** | #762 |
| **Title** | Use Case 34: Metacircular Self Hosting Stage 1 to Stage 2 SHA 256 Fixed Point Identity |
| **Type** | use-case |
| **Primary Actor** | Operator |
| **Scope** | FourLevelAstPartitionerAndHoister |
| **Level** | User Goal |
| **Generation Mode** | subagent |
| **Specification Source** | [schema/model.sysml](../../schema/model.sysml) |

# Use Case 34: Metacircular Self Hosting Stage 1 to Stage 2 SHA 256 Fixed Point Identity

## UML Diagrams

```mermaid
graph TD
    subgraph "FourLevelAstPartitionerAndHoister System Boundary"
        UC_Main(["Metacircular Self Hosting Stage 1 to Stage 2 SHA 256 Fixe..."])
        UC_Step01(["The Stage-1-linked compile-sysml binary re-compiles .pipe..."])
        UC_Step02(["SHA-256(Stage 1) == SHA-256(Stage 2) holds across 100% of..."])
    end
    Operator(("Operator")) --- UC_Main
    UC_Main -. "<<include>>" .-> UC_Step01
    UC_Main -. "<<include>>" .-> UC_Step02
```

## 1. Actors
- `Operator`

## 2. Preconditions
- The authoritative self-hosting compiler schema .pipeline/upstream/schema/modular_codegen.sysml and the committed Stage-1 tree under crates/compile-sysml/src/modular_codegen/generated/ verified parse-clean by ra_ap_syntax and wired into compile-sysml.

## 3. Trigger
- The Stage-1-linked compile-sysml binary re-compiles .pipeline/upstream/schema/modular_codegen.sysml into an isolated Stage-2 temporary directory and compares GeneratedTreeManifest SHA-256 digests.

## 4. Main Success Scenario
1. `Operator` invokes the Stage-1-linked `compile-sysml` binary on `.pipeline/upstream/schema/modular_codegen.sysml` via `SelfHostingModularCodegenCompiler` (`package_ast_in`), with the committed Stage-1 tree under `crates/compile-sysml/src/modular_codegen/generated/` verified parse-clean by `ra_ap_syntax`.
2. `FourLevelAstPartitionerAndHoister` and `ModularRustMultiFileEmitter` lower `.pipeline/upstream/schema/modular_codegen.sysml` across Levels 1 through 4 via `partition_and_hoist`, `synthesize_module_tokens`, `build_syn_file`, and `format_pretty_syn_file`.
3. `ModularRustMultiFileEmitter` executes `emit_modular_rust_tree` into an isolated Stage-2 temporary directory (`std::env::temp_dir()`), producing a Stage-2 `GeneratedTreeManifest` on `rust_manifest_out` and transitioning to `VerifyingSelfHostFixedPoint`.
4. `ModularRustMultiFileEmitter` invokes `verify_fixed_point_digest(stage1, stage2)` inside `compile_sysml_codegen_ast_fixed_point_gate` (`< 380` exec SLOC) to compare relative file paths and SHA-256 digests between Stage 1 and Stage 2 under `Invariant_REQ_0607_SC_03`.
5. `SelfHostingModularCodegenCompiler` confirms `SHA-256(Stage 1) == SHA-256(Stage 2)` across 100% of relative paths and file contents with `stage_fixed_point_mismatch_files == 0` (`C_MOD_04`), transitions to `VerifiedFixedPoint`, and exits 0.

## 5. Alternate and Exception Flows
- 3a. Stage-1 or Stage-2 generated Rust unit fails `ra_ap_syntax` or `syn::parse2::<syn::File>` parse verification (`Diagnostic: E0704`):
  - `ModularRustMultiFileEmitter` transitions to `RejectedFailClosed`, emits diagnostic `E0704` on `diagnostic_out`, cleans up the temporary Stage-2 directory, and exits fail-closed with code 1.
- 4a. Stage-1 and Stage-2 `GeneratedTreeManifest` SHA-256 digests or file sets diverge (`stage_fixed_point_mismatch_files > 0`, `Diagnostic: E0705`):
  - `ModularRustMultiFileEmitter` transitions to `RejectedFailClosed`, emits diagnostic `E0705` enumerating every mismatched relative path, and terminates fail-closed with exit code 1.
- 5a. Invariant violation while enforcing `C_MOD_04` or `C_MOD_05` (`runtime_llm_subagent_count == 0`):
  - `FourLevelAstPartitionerAndHoister` aborts execution without mutating committed Stage-1 artifacts, preserves prior verified state, and notifies `Operator`.

## 6. Postconditions
- SHA-256(Stage 1) == SHA-256(Stage 2) holds across 100% of relative paths and file contents (**C-MOD-04**.

## 8. Realization Matrix
| Specification Item | Type | Link / Reference |
| :--- | :--- | :--- |
| **TBD** | User Story | [User Story 47: synthesize module tokens](../user-stories/US-47-synthesize_module_tokens.md) |
| **TBD** | Feature | [Feature 31: [BP 07 Self Hosting 4 Level Modular Multi File Code Generator Blueprint] FourLevelAstPartitionerAndHoister](../features/FEAT-31-FourLevelAstPartitionerAndHoister.md) |

## Source References
Use case operational flows and lifecycle activity references:
- System Architecture Model: `schema/model.sysml`
- Normative Systems Engineering Standard: ISO/IEC/IEEE 29148:2018 §6.4.2 Concept of Operations

Operational use case realization and traceability links derive from `schema/model.sysml` pursuant to ISO/IEC/IEEE 29148.
