| Attribute | Specification Detail |
| :--- | :--- |
| **Issue ID** | #647 |
| **Title** | Use Case 24: Bit for Bit Idempotency and Fail Closed Check 34 Per Function Normalized Body Uniqueness |
| **Type** | use-case |
| **Primary Actor** | Compiler_CLI_CI_Pipeline |
| **Scope** | Check36AstCompletenessVerifier |
| **Level** | User Goal |
| **Generation Mode** | subagent |
| **Specification Source** | [schema/model.sysml](../../schema/model.sysml) |

# Use Case 24: Bit for Bit Idempotency and Fail Closed Check 34 Per Function Normalized Body Uniqueness

## UML Diagrams

```mermaid
graph TD
    subgraph "Check36AstCompletenessVerifier System Boundary"
        UC_Main(["Bit for Bit Idempotency and Fail Closed Check 34 Uniqueness"])
        UC_Step01(["Execute compile-sysml --inject-safety-docstrings via ra_ap_text_edit"])
        UC_Step02(["Verify zero diff bytes on second pass and Check 34 uniqueness"])
    end
    Compiler_CLI_CI_Pipeline(("Compiler_CLI_CI_Pipeline")) --- UC_Main
    UC_Main -. "<<include>>" .-> UC_Step01
    UC_Main -. "<<include>>" .-> UC_Step02
```

## 1. Actors
- `Compiler_CLI_CI_Pipeline`

## 2. Preconditions
- Any Rust crate or Dart source directory enriched by `compile-sysml --inject-safety-docstrings <DIR>` bound to a full OMG SysML v2.0 + KerML 1.0 model (zero 16-construct caps), as well as a negative fixture containing duplicate normalized docstring bodies or missing tags.

## 3. Trigger
- `compile-sysml --inject-safety-docstrings <DIR>` is executed a second time and followed by `./target/release/verify-baseline . --no-domain`.

## 4. Main Success Scenario
1. `Compiler_CLI_CI_Pipeline` executes `RunCompileSysmlInjectSafetyDocstrings` (`compile-sysml --inject-safety-docstrings <DIR>`) a first time, where `SevenSectionDocstringGenerator` synthesizes `<= 4`-line Tier-B/C `[REQ-*]`/`[FLS: fls_*]` docstrings on internal declarations and explicit `[DOC-TIER: A]` 7-section docstrings on `pub` crate entrypoints and `unsafe`/`extern` blocks bound to `schema_context_in` via `ra_ap_syntax` (`ast::HasDocComments`, `ast::HasAttrs`), `ra_ap_text_edit`, `ra_ap_line_index`, `syn`, `quote`, and `prettyplease` with zero hand-rolled scanners.
2. `Compiler_CLI_CI_Pipeline` executes `RunCompileSysmlInjectSafetyDocstrings` a second time on the same target directory `<DIR>`.
3. `SevenSectionDocstringGenerator` and `UnifiedSafetyDocstringSynthesizer` verify existing Tier-B/C (`<= 4`-line `[REQ-*]`/`[FLS: fls_*]`) and `[DOC-TIER: A]` docblocks via `ra_ap_syntax` (`ast::HasDocComments`, `ast::HasAttrs`) and produce 0 modified bytes (`docstring_idempotency_diff_bytes == 0`).
4. `Compiler_CLI_CI_Pipeline` invokes `RunVerifyBaselineNoDomain` (`./target/release/verify-baseline . --no-domain`) on `Check36AstCompletenessVerifier`, which parses all docblocks via `pulldown-cmark`, `ra_ap_syntax` (`ast::HasDocComments`, `ast::HasAttrs`), and `deap_core::dart_cst`.
5. `Check36AstCompletenessVerifier` confirms 100% Check 36 compliance (`<= 4`-line Tier-B/C `[REQ-*]`/`[FLS: fls_*]` docstrings and `[DOC-TIER: A]` 7-section completeness `## 1. Safety Intent & Regulatory Scope` through `## 7. Realization & Traceability`) and Check 34 `seen_docblocks` per-function normalized body uniqueness, emitting `true` on `docstring_verdict_out` with exit code 0.

## 5. Alternate and Exception Flows
- 3a. Second pass of `RunCompileSysmlInjectSafetyDocstrings` mutates any byte (`docstring_idempotency_diff_bytes > 0`):
  - `UnifiedSafetyDocstringSynthesizer` returns `Result::Err` under `Invariant_REQ_0605_Under_REQ_0605_and_parametric_constraints_C_DOC_01_through_C_DOC_04_lexical_scanning_schema_context_resolution_and_forward_docstring_injection_satisfy_the_following_linear_complexity_and_idempotency_bounds`, emits an `ariadne` / `miette` idempotency diagnostic, and exits fail-closed with code 1.
- 4a. Duplicate normalized docstring body detected across distinct declarations in Check 34 `seen_docblocks`:
  - `Check36AstCompletenessVerifier` flags the copy-pasted boilerplate docblock with file and line coordinates via `ra_ap_line_index` and `miette`, sets `docstring_verdict_out` to `false`, and exits fail-closed with code 1.
- 5a. Missing `[REQ-*]`/`[FLS: fls_*]` tag on Tier-B/C item or missing section among sections 1 through 7 on `[DOC-TIER: A]` item:
  - `Check36AstCompletenessVerifier` rejects the declaration via `Result<T, E>`, preserves prior verified state, and terminates fail-closed with exit code 1.

## 6. Postconditions
- **Success Guarantee**: The second synthesis pass produces 0 modified bytes (`docstring_idempotency_diff_bytes == 0`), and every synthesized docblock passes Check 36 (`<= 4`-line Tier-B/C `[REQ-*]`/`[FLS: fls_*]` and `[DOC-TIER: A]` 7-section completeness) and Check 34 `seen_docblocks` uniqueness.
- **Failure Guarantee**: Any non-idempotent edit, missing tag/section, or duplicate normalized docstring body fails closed with exit code 1.
- **3-Layer Definition of Done (DoD)**:
  - **Layer 1 (Domain State & Data Model)**: Lossless `ra_ap_syntax` (`ast::HasDocComments`, `ast::HasAttrs`), `ra_ap_line_index`, `deap_core::dart_cst`, and `deap_core::sysml_cst` models covering 100% of the OMG SysML v2.0 + KerML 1.0 grammar (zero 16-construct caps).
  - **Layer 2 (Logic & State Management)**: Idempotent `ra_ap_text_edit` codemods and Check 34/Check 36 verification inside `< 380`-SLOC submodules (`./crates/compile-sysml/src/safety_docstrings/`, `./crates/deap-core/src/rust_cst/`, `./crates/deap-core/src/dart_cst/`) with explicit deletion of legacy hand-rolled line/char-by-char docblock injectors (`-800+` SLOC retired).
  - **Layer 3 (Presentation & Actuator Interface Binding)**: CLI bindings (`compile-sysml --inject-safety-docstrings`, `verify-baseline . --no-domain`) and `ariadne`/`miette` diagnostics.

## 8. Realization Matrix
| Specification Item | Type | Link / Reference |
| :--- | :--- | :--- |
| **#723** | User Story | [User Story 34: Run Verify Baseline No Domain](../user-stories/US-34-RunVerifyBaselineNoDomain.md) |
| **#678** | Feature | [Feature 21: [BP 05 Unified AST Safety Docstring Synthesizer and Check 36 Verification Blueprint] Check36AstCompletenessVerifier](../features/FEAT-21-Check36AstCompletenessVerifier.md) |

## Source References
Use case operational flows and lifecycle activity references:
- System Architecture Model: `schema/model.sysml`
- Normative Systems Engineering Standard: ISO/IEC/IEEE 29148:2018 §6.4.2 Concept of Operations

Operational use case realization and traceability links derive from `schema/model.sysml` pursuant to ISO/IEC/IEEE 29148.
