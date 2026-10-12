doc /* Behavioral models and execution flows for SysmlGrammarLowering */
package SysmlGrammarLowering {
    import DEAP_Compiler_System::*;

    doc /* Grammar lowering compiler pass pipeline state machine */
    state def GrammarLoweringFSM {
        entry;
        state TokenIngest;
        state ClassifierResolution;
        state ConjugationResolution;
        state TypeChecking;
        state Complete;

        transition InitialToIngest
            first entry
            then TokenIngest;

        transition IngestToClassifier
            first TokenIngest
            then ClassifierResolution;

        transition ClassifierToConjugation
            first ClassifierResolution
            then ConjugationResolution;

        transition ConjugationToTypeCheck
            first ConjugationResolution
            then TypeChecking;

        transition TypeCheckToComplete
            first TypeChecking
            then Complete;
    }

    doc /* Action pipeline for lowering SysML v2 syntax into canonical KerML semantic representations */
    action def LowerSysMLAction {
        in graph_in : ASTGraphPort;
        out stream_out : ASTStreamPort;
    }

    doc /* Interaction lifeline modeling node arena to grammar lowering engine execution */
    interaction def GrammarLoweringInteraction {
        lifeline arena : NodeArenaASTGraphEngine;
        lifeline lowering : GrammarLoweringEngine;

        message RequestLowering from arena to lowering;
        message EmitStream from lowering to arena;
    }

    doc /* Comprehensive use case for SysML v2 grammar lowering and type validation */
    use case def UC_04_SysML_Grammar_Lowering_Workflow {
        doc /*
        Actors: CIContinuousIntegrationRunner, HumanEngineer
        Subject: GrammarLoweringEngine
        Preconditions: Node arena contains populated AST graph nodes.
        Trigger: Grammar lowering trigger dispatched after successful node allocation.
        Main Success Scenario:
        1. NodeArenaASTGraphEngine passes AST graph handle to GrammarLoweringEngine.
        2. KerMLTokenStream streams tokens into SysMLv2GrammarParser.
        3. SysMLv2GrammarParser parses package hierarchies, part defs, and port defs.
        4. ASTLoweringEngine resolves feature usage directionality and port conjugation (~).
        5. ASTLoweringEngine validates classifier inheritance and type compatibility.
        6. GrammarLoweringEngine emits lowered ASTStreamPort to downstream semantic engines.
        Alternate and Exception Flows:
        5a. Incompatible conjugate port pairing: Engine detects type mismatch and emits diagnostic E0200.
        5b. Undeclared classifier referenced in part usage: Parser logs missing symbol diagnostic and binds unresolved stub.
        5c. Grammar syntax error in expression: Parser emits precise source line/column error and recovers on semicolon.
        5d. Cyclic classifier inheritance detected: Lowering engine rejects cyclic hierarchy and reports E0201.
        5e. Port direction inversion violation: Compiler flags illegal directed connection and reports E0202.
        5f. Missing mandatory classifier attribute: Engine populates default value and logs informational note.
        5g. Deep recursion in nested grammar blocks: Parser enforces recursion bound and returns syntax error.
        5h. Ambiguous symbol resolution: Scoping rules apply lexical shadowing and report disambiguation note.
        5i. Unsupported KerML construct encountered: Engine safely skips unhandled construct and warns operator.
        5j. Multiplicity constraint violation: Multiplicity interval check fails and logs E0203.
        Postconditions: Strongly-typed, normalized SysML v2 / KerML AST stream is emitted.
        */
    }
}
