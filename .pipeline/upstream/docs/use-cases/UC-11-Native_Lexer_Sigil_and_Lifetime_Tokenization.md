| Attribute | Specification Detail |
| :--- | :--- |
| **Issue ID** | #634 |
| **Title** | Use Case 11: Native Lexer Sigil and Lifetime Tokenization |
| **Type** | use-case |
| **Primary Actor** | Compiler_CLI_Caller |
| **Scope** | BijectiveSysmlSerializer |
| **Level** | User Goal |
| **Generation Mode** | subagent |
| **Specification Source** | [schema/model.sysml](../../schema/model.sysml) |

# Use Case 11: Native Lexer Sigil and Lifetime Tokenization

## UML Diagrams

```mermaid
graph TD
    subgraph "BijectiveSysmlSerializer System Boundary"
        UC_Main(["Native Lexer Sigil and Lifetime Tokenization"])
        UC_Step01(["logos_lexer::scan_all tokenizes the raw source buffer without..."])
        UC_Step02(["The lexer emits TokenKind::Ampersand, TokenKind::Question..."])
    end
    Compiler_CLI_Caller(("Compiler_CLI_Caller")) --- UC_Main
    UC_Main -. "<<include>>" .-> UC_Step01
    UC_Main -. "<<include>>" .-> UC_Step02
```

## 1. Actors
- `Compiler_CLI_Caller`

## 2. Preconditions
- A SysML v2 model containing reference type sigils (&), optionality sigils (?), lifetime annotations ('a), qualified paths (::), and logical && expressions (#495).

## 3. Trigger
- logos_lexer::scan_all tokenizes the raw source buffer without invoking normalize_type_sigils.

## 4. Main Success Scenario
1. `Compiler_CLI_Caller` feeds unmodified UTF-8 SysML v2 source text into `SysmlBijectiveRoundtripCompilerGate` via `RawSysmlSourceInputPort` without invoking pre-lexer string rewriting (`normalize_type_sigils`).
2. `NativeSigilAndLifetimeLexer` transitions `BP_03_SysML_v2_AST_Bijective_Roundtrip_and_Token_Stream_Parser_Gate_Blueprint_LifecycleState` to `LexicalScanning` and executes `scan_all` using `logos` and `rowan`.
3. `NativeSigilAndLifetimeLexer` tokenizes reference type sigils (`&`), optionality sigils (`?`), lifetime annotations (`'a`), qualified paths (`::`), and logical `&&` expressions directly from the UTF-8 byte buffer.
4. `NativeSigilAndLifetimeLexer` emits `TokenKind::Ampersand`, `TokenKind::Question`, `TokenKind::Lifetime`, `TokenKind::DoubleColon`, and `TokenKind::AmpAmp` onto `TokenStreamChannelPort` with exact 1:1 byte and column offsets (`span_offset_drift_bytes == 0`).
5. `SysmlBijectiveRoundtripCompilerGate` transitions to `RecursiveDescentParsing` on `zeroDriftTokenStream` and returns the span-exact token vector to `Compiler_CLI_Caller`.

## 5. Alternate and Exception Flows
- 3a. Invalid UTF-8 byte sequence or unrecognized lexical sigil on `RawSysmlSourceInputPort`:
  - `NativeSigilAndLifetimeLexer` transitions to `RejectedE0100` on `lexicalSpanError`, emits a diagnostic `E0100` via `ariadne` / `miette` on `BijectiveDiffDiagnosticPort`, and exits fail-closed with code 1.
- 4a. Non-zero byte span drift detected (`span_offset_drift_bytes != 0`) under `Invariant_REQ_0603_Formal_Bijectivity_Complexity_Invariants`:
  - `SysmlBijectiveRoundtripCompilerGate` returns `Result::Err`, emits a structured span-drift diagnostic on `BijectiveDiffDiagnosticPort`, and terminates fail-closed with exit code 1.
- 5a. Precondition or invariant violation under `Invariant_REQ_0603_SC_04`:
  - `BijectiveSysmlSerializer` aborts execution without partial state mutation, preserves prior verified state, and notifies `Compiler_CLI_Caller` with fail-closed status code 1.

## 6. Postconditions
- The lexer emits TokenKind::Ampersand, TokenKind::Question, TokenKind::Lifetime, TokenKind::DoubleColon, and TokenKind::AmpAmp with exact 1:1 byte and column offsets (span_offset_drift_bytes == 0).

## 8. Realization Matrix
| Specification Item | Type | Link / Reference |
| :--- | :--- | :--- |
| **#577** | User Story | [User Story 12: scan all](../user-stories/US-12-scan_all.md) |
| **#667** | Feature | [Feature 11: [BP 03 SysML v2 AST Bijective Roundtrip and Token Stream Parser Gate Blueprint] BijectiveSysmlSerializer](../features/FEAT-11-BijectiveSysmlSerializer.md) |

## Source References
Use case operational flows and lifecycle activity references:
- System Architecture Model: `schema/model.sysml`
- Normative Systems Engineering Standard: ISO/IEC/IEEE 29148:2018 §6.4.2 Concept of Operations

Operational use case realization and traceability links derive from `schema/model.sysml` pursuant to ISO/IEC/IEEE 29148.
