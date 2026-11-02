| Attribute | Specification Detail |
| :--- | :--- |
| **Issue ID** | #637 |
| **Title** | Use Case 14: Lossless to sysml Serialization and Fail Closed Bijective Roundtrip Gate |
| **Type** | use-case |
| **Primary Actor** | Compiler_CLI_Caller |
| **Scope** | BijectiveSysmlSerializer |
| **Level** | User Goal |
| **Generation Mode** | subagent |
| **Specification Source** | [schema/model.sysml](../../schema/model.sysml) |

# Use Case 14: Lossless to sysml Serialization and Fail Closed Bijective Roundtrip Gate

## UML Diagrams

```mermaid
graph TD
    subgraph "BijectiveSysmlSerializer System Boundary"
        UC_Main(["Lossless to sysml Serialization and Fail Closed Bijective..."])
        UC_Step01(["verify_bijective_ast_roundtrip(&pkg) executes during comp..."])
        UC_Step02(["pkg.to_sysml(0) emits valid un-double-quoted SysML v2 tex..."])
    end
    Compiler_CLI_Caller(("Compiler_CLI_Caller")) --- UC_Main
    UC_Main -. "<<include>>" .-> UC_Step01
    UC_Main -. "<<include>>" .-> UC_Step02
```

## 1. Actors
- `Compiler_CLI_Caller`

## 2. Preconditions
- A populated PackageDef AST containing StateDef transitions (source, trigger, guard, effect, target, doc), PortDef / HazardDef / RiskDef attributes, and UseCaseDef / ConnectionDef / RequirementDef / ActionDef blocks (#493, #497, #505), or a mutated AST with a simulated field divergence.

## 3. Trigger
- verify_bijective_ast_roundtrip(&pkg) executes during compile-sysml.

## 4. Main Success Scenario
1. `Compiler_CLI_Caller` invokes `verify_bijective_ast_roundtrip` on `CompilerRoundtripVerificationGate` with a populated `PackageDef` AST (`ast_orig`).
2. `CompilerRoundtripVerificationGate` transitions to `CanonicalSerialization` and invokes `ast_orig_to_sysml` on `BijectiveSysmlSerializer` to emit canonical, un-double-quoted SysML v2 text via `SerializedSysmlOutputPort` with `dropped_ast_field_count == 0`.
3. `CompilerRoundtripVerificationGate` transitions to `RoundtripReparsing` and routes the serialized text through `BP_03_SerializerToRoundtripGateConnection` (`RawSysmlSourceInputPort`) into `NativeSigilAndLifetimeLexer` (`scan_all`) and `FailClosedRecursiveDescentParser` (`parse_package_or_file`) using `logos` and `rowan`.
4. `CompilerRoundtripVerificationGate` transitions to `AstStructuralEqualityCheck` and executes `AssertAstOrigEqualsAstReparsed` to compare `ast_orig` against `ast_reparsed` across all `StateDef`, `PortDef`, `HazardDef`, `RiskDef`, `UseCaseDef`, `ConnectionDef`, `RequirementDef`, and `ActionDef` nodes.
5. `CompilerRoundtripVerificationGate` confirms `roundtrip_ast_mismatch_count == 0`, transitions to `VerifiedBijectiveAst`, and returns `Result::Ok` to `Compiler_CLI_Caller`.

## 5. Alternate and Exception Flows
- 3a. Serialized SysML v2 text fails re-lexing or re-parsing during `RoundtripReparsing`:
  - `CompilerRoundtripVerificationGate` transitions to `RejectedE0400` on `reparseSyntaxFailure`, emits a diagnostic `E0400` via `ariadne` / `miette` on `BijectiveDiffDiagnosticPort`, and exits fail-closed with code 1.
- 4a. Structural AST divergence detected between `ast_orig` and `ast_reparsed` (`roundtrip_ast_mismatch_count > 0`):
  - `CompilerRoundtripVerificationGate` transitions to `RejectedE0400` on `structuralAstDiffDetected`, emits a field-level AST diff diagnostic (`E0400`) on `BijectiveDiffDiagnosticPort`, and exits fail-closed with code 1.
- 5a. Invariant violation under `Invariant_REQ_0603_Formal_Bijectivity_Complexity_Invariants` or `Invariant_REQ_0603_SC_04`:
  - `BijectiveSysmlSerializer` aborts serialization without partial file writes, preserves prior verified state, and notifies `Compiler_CLI_Caller` with fail-closed exit status 1.

## 6. Postconditions
- pkg.to_sysml(0) emits valid un-double-quoted SysML v2 text, SysmlParser::parse_source(&pkg.to_sysml(0), &pkg.name) returns a PackageDef structurally equal to pkg (roundtrip_ast_mismatch_count == 0), and any.

## 8. Realization Matrix
| Specification Item | Type | Link / Reference |
| :--- | :--- | :--- |
| **#582** | User Story | [User Story 15: verify bijective ast roundtrip](../user-stories/US-15-verify_bijective_ast_roundtrip.md) |
| **#667** | Feature | [Feature 11: [BP 03 SysML v2 AST Bijective Roundtrip and Token Stream Parser Gate Blueprint] BijectiveSysmlSerializer](../features/FEAT-11-BijectiveSysmlSerializer.md) |

## Source References
Use case operational flows and lifecycle activity references:
- System Architecture Model: `schema/model.sysml`
- Normative Systems Engineering Standard: ISO/IEC/IEEE 29148:2018 §6.4.2 Concept of Operations

Operational use case realization and traceability links derive from `schema/model.sysml` pursuant to ISO/IEC/IEEE 29148.
