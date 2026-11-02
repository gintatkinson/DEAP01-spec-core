| Attribute | Specification Detail |
| :--- | :--- |
| **Issue ID** | #771 |
| **Title** | Use Case 43: Unified rustc Compatible Single Line JSON Diagnostics and Machine Applicable Auto Fix Across All Five CLIs |
| **Type** | use-case |
| **Primary Actor** | Operator |
| **Scope** | AirGappedCargoLockGovernor |
| **Level** | User Goal |
| **Generation Mode** | subagent |
| **Specification Source** | [schema/model.sysml](../../schema/model.sysml) |

# Use Case 43: Unified rustc Compatible Single Line JSON Diagnostics and Machine Applicable Auto Fix Across All Five CLIs

## UML Diagrams

```mermaid
graph TD
    subgraph "AirGappedCargoLockGovernor System Boundary"
        UC_Main(["Unified rustc Compatible Single Line JSON Diagnostics and..."])
        UC_Step01(["Standard output is parsed line-by-line as JSON and --appl..."])
        UC_Step02(["Every CLI emits strictly single-line NDJSON objects with ..."])
    end
    Operator(("Operator")) --- UC_Main
    UC_Main -. "<<include>>" .-> UC_Step01
    UC_Main -. "<<include>>" .-> UC_Step02
```

## 1. Actors
- `Operator`

## 2. Preconditions
- Validation failures triggered across compile-sysml, verify-baseline, ingest-sysml, assemble-conops, and reconcile-backlog with --message-format=json, including diagnostics carrying both \"MachineApplicable\" and \"MaybeIncorrect\" child replacement spans.

## 3. Trigger
- Standard output is parsed line-by-line as JSON and --apply-machine-fixes is subsequently invoked on the offending workspace fixture.

## 4. Main Success Scenario
1. `Operator` invokes `compile-sysml`, `verify-baseline`, `ingest-sysml`, `assemble-conops`, and `reconcile-backlog` (`cli_binary_count = 5`) with `--message-format=json` on workspace fixtures containing validation failures.
2. `UnifiedRustcJsonDiagnosticEmitter` receives structured findings on `finding_in` (`zero_ansi_in_json_mode = true`, `envelope_reason = "compiler-message"`) inside `< 380` exec SLOC submodules (`crates/deap-core/src/json_diag/`).
3. `UnifiedRustcJsonDiagnosticEmitter` invokes `format_ndjson_line` to serialize each finding on `ndjson_out` as a strictly single-line NDJSON object with `reason: "compiler-message"`, `message.code.code` prefixed with `"DEAP-E"`, exact UTF-8 `byte_start`/`byte_end` and 1-based line/column spans, and zero ANSI escape codes (`C_CST_05`).
4. `Operator` subsequently invokes `--apply-machine-fixes` on `UnifiedRustcJsonDiagnosticEmitter` (`machine_applicable_only = true`) via `machine_fix_inout`.
5. `UnifiedRustcJsonDiagnosticEmitter` filters exclusively for disjoint child suggestions tagged `"MachineApplicable"` (ignoring `"MaybeIncorrect"` spans), applies them in reverse byte order, verifies post-splice CST syntax via `ra_ap_syntax`, and commits the repaired files atomically with exit code 0.

## 5. Alternate and Exception Flows
- 3a. Any CLI emits multi-line unstructured text or ANSI escape sequences while `--message-format=json` is active under `Invariant_REQ_0608_SC_04`:
  - NDJSON schema verification fails closed with exit code 1 and flags the non-compliant CLI output stream.
- 4a. Two `"MachineApplicable"` child replacement spans overlap in byte range (`Diagnostic: DEAP-E0802`):
  - `UnifiedRustcJsonDiagnosticEmitter` aborts `--apply-machine-fixes` without mutating any file, emits single-line JSON diagnostic `DEAP-E0802`, and exits fail-closed with code 1.
- 5a. Applying a `"MachineApplicable"` replacement introduces a CST syntax error (`Diagnostic: DEAP-E0803`):
  - `UnifiedRustcJsonDiagnosticEmitter` rolls back the in-memory replacement buffer, preserves prior verified file contents, and exits fail-closed with code 1.

## 6. Postconditions
- Every CLI emits strictly single-line NDJSON objects with `reason: "compiler-message"`, `message.code.code` prefixed with `"DEAP-E"`, and exact UTF-8 `byte_start`/`byte_end` spans, and `--apply-machine-fixes` atomically applies only disjoint `"MachineApplicable"` suggestions.

## 8. Realization Matrix
| Specification Item | Type | Link / Reference |
| :--- | :--- | :--- |
| **TBD** | User Story | [User Story 58: analyze metrics](../user-stories/US-58-analyze_metrics.md) |
| **TBD** | Feature | [Feature 36: [BP 08 Lossless Rust CST Codemod Engine and Unified rustc Compatible JSON Diagnostics] AirGappedCargoLockGovernor](../features/FEAT-36-AirGappedCargoLockGovernor.md) |

## Source References
Use case operational flows and lifecycle activity references:
- System Architecture Model: `schema/model.sysml`
- Normative Systems Engineering Standard: ISO/IEC/IEEE 29148:2018 §6.4.2 Concept of Operations

Operational use case realization and traceability links derive from `schema/model.sysml` pursuant to ISO/IEC/IEEE 29148.
