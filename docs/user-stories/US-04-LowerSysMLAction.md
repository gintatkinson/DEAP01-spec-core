| Attribute | Specification Detail |
| :--- | :--- |
| **Issue ID** | #92 |
| **Title** | User Story 04: Lower Sys MLAction |
| **Type** | user-story |
| **Parent Epic** | [EPIC-06-SysmlGrammarLowering](../epics/EPIC-06-SysmlGrammarLowering.md) |
| **SysML Interaction** | `GrammarLoweringInteraction` |
| **SysML Test Case** | `Test_LowerSysMLAction` |
| **Generation Mode** | subagent |
| **Specification Source** | [schema/subsystems/sysml_grammar_lowering/behavior.sysml](../../schema/subsystems/sysml_grammar_lowering/behavior.sysml) |

# User Story 04: Lower Sys MLAction

## UML Sequence Diagram

```mermaid
sequenceDiagram
    autonumber
    actor HumanEngineer
    participant NodeArenaASTGraphEngine
    participant GrammarLoweringEngine
    HumanEngineer->>NodeArenaASTGraphEngine: LowerSysMLAction()
    NodeArenaASTGraphEngine->>GrammarLoweringEngine: LowerSysMLAction()
    GrammarLoweringEngine-->>NodeArenaASTGraphEngine: EmitStream
    NodeArenaASTGraphEngine-->>HumanEngineer: StatusReport
```

## Acceptance Criteria (BDD)
- [ ] AC-US-04-01: Given operational environment initialized, When LowerSysMLAction executes, Then GrammarLoweringEngine processes the AST within defined latency envelope.
- [ ] AC-US-04-02: Given formal constraints, When semantic validator executes, Then asserts invariant satisfaction.
- [ ] AC-US-04-03: Given exceptional conditions, When error recovery activates, Then graceful recovery occurs without data loss.

## Required Features
- [ ] [Feature 01: [ConOps] Human Engineer Interface](../features/FEAT-01-HumanEngineer.md)
- [ ] [Feature 15: [Core Metamodel Node Arena] NodeArenaASTGraphEngine](../features/FEAT-15-NodeArenaASTGraphEngine.md)
- [ ] [Feature 19: [Sysml Grammar Lowering] GrammarLoweringEngine](../features/FEAT-19-GrammarLoweringEngine.md)

## Source References
Operational concept definitions and system sequence interaction models:
- Subsystem Behavior Specification: `schema/subsystems/sysml_grammar_lowering/behavior.sysml`
- ConOps Operational Activities: `schema/conops/activities.sysml`
- System Architecture Model: `schema/model.sysml`
- Normative Systems Engineering Standard: ISO/IEC/IEEE 29148:2018 §6.4.2 Concept of Operations

Authoritative user story flows and interaction lifelines derive from `schema/subsystems/sysml_grammar_lowering/behavior.sysml` pursuant to ISO/IEC/IEEE 29148.
