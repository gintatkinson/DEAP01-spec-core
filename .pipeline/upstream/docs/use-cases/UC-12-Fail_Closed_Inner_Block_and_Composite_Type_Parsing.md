| Attribute | Specification Detail |
| :--- | :--- |
| **Issue ID** | #635 |
| **Title** | Use Case 12: Fail Closed Inner Block and Composite Type Parsing |
| **Type** | use-case |
| **Primary Actor** | Compiler_CLI_Caller |
| **Scope** | BijectiveSysmlSerializer |
| **Level** | User Goal |
| **Generation Mode** | subagent |
| **Specification Source** | [schema/model.sysml](../../schema/model.sysml) |

# Use Case 12: Fail Closed Inner Block and Composite Type Parsing

## UML Diagrams

```mermaid
graph TD
    subgraph "BijectiveSysmlSerializer System Boundary"
        UC_Main(["Fail Closed Inner Block and Composite Type Parsing"])
        UC_Step01(["SysmlParser parses the declaration block."])
        UC_Step02(["Valid composite types are preserved verbatim via parse_ty..."])
    end
    Compiler_CLI_Caller(("Compiler_CLI_Caller")) --- UC_Main
    UC_Main -. "<<include>>" .-> UC_Step01
    UC_Main -. "<<include>>" .-> UC_Step02
```

## 1. Actors
- `Compiler_CLI_Caller`

## 2. Preconditions
- Any of the 12 SysML v2 inner declaration blocks (port, action, requirement, state, connection, use case, item, hazard, risk, test case, capability, interaction) containing either composite parameter types (Result<PackageDef, ParseError>, &'a [Token]) or an invalid inner statement (#504).

## 3. Trigger
- SysmlParser parses the declaration block.

## 4. Main Success Scenario
1. `Compiler_CLI_Caller` invokes `parse_package_or_file` on `FailClosedRecursiveDescentParser`, streaming span-exact tokens from `NativeSigilAndLifetimeLexer` across `TokenStreamChannelPort`.
2. `FailClosedRecursiveDescentParser` enters `RecursiveDescentParsing` and dispatches inner declaration block parsers across all 12 SysML v2 block kinds (`port`, `action`, `requirement`, `state`, `connection`, `use case`, `item`, `hazard`, `risk`, `test case`, `capability`, `interaction`) using `logos` and `rowan`.
3. `FailClosedRecursiveDescentParser` invokes `ParseTypeExprAndParseTransition` (`parse_type_expr`) to parse composite generic, reference, lifetime, and slice parameter types verbatim into typed AST nodes.
4. `FailClosedRecursiveDescentParser` enforces `silent_skip_branch_count == 0` so zero `skip_until_semi_or_brace` silent-erasure branches execute during inner block parsing.
5. For valid composite types, `FailClosedRecursiveDescentParser` constructs the canonical `PackageDef` AST and transitions to `CanonicalSerialization`; for any unrecognized inner statement, it transitions to `RejectedE0200` and emits `Err(ParseError)` with diagnostic code `E0200` via `BijectiveDiffDiagnosticPort`.

## 5. Alternate and Exception Flows
- 3a. Unrecognized inner block token or malformed statement encountered during `parse_package_or_file`:
  - `FailClosedRecursiveDescentParser` transitions to `RejectedE0200` on `unexpectedInnerBlockToken`, emits an `ariadne` / `miette` `E0200` diagnostic on `BijectiveDiffDiagnosticPort`, and exits fail-closed with code 1.
- 4a. Unbalanced angle brackets or invalid lifetime sigil in `ParseTypeExprAndParseTransition`:
  - `FailClosedRecursiveDescentParser` returns `Result::Err(ParseError)` without silent skip recovery, preserves prior verified state, and terminates fail-closed with exit code 1.
- 5a. Invariant violation while enforcing `Invariant_REQ_0603_SC_04` (`silent_skip_branch_count > 0`):
  - `BijectiveSysmlSerializer` and `SysmlBijectiveRoundtripCompilerGate` abort compilation, emit a structured diagnostic to `Compiler_CLI_Caller`, and exit with non-zero status code 1.

## 6. Postconditions
- Valid composite types are preserved verbatim via parse_type_expr() and any unrecognized statement fails closed and is rejected with Err(ParseError) reporting diagnostic code E0200 (silent_skip_branch_count == 0).

## 8. Realization Matrix
| Specification Item | Type | Link / Reference |
| :--- | :--- | :--- |
| **#579** | User Story | [User Story 13: parse package or file](../user-stories/US-13-parse_package_or_file.md) |
| **#667** | Feature | [Feature 11: [BP 03 SysML v2 AST Bijective Roundtrip and Token Stream Parser Gate Blueprint] BijectiveSysmlSerializer](../features/FEAT-11-BijectiveSysmlSerializer.md) |

## Source References
Use case operational flows and lifecycle activity references:
- System Architecture Model: `schema/model.sysml`
- Normative Systems Engineering Standard: ISO/IEC/IEEE 29148:2018 §6.4.2 Concept of Operations

Operational use case realization and traceability links derive from `schema/model.sysml` pursuant to ISO/IEC/IEEE 29148.
