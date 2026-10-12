doc /* Behavioral models and execution flows for SpatioTemporalStateSolvers */
package SpatioTemporalStateSolvers {
    import DEAP_Compiler_System::*;

    doc /* State machine solver execution lifecycle statechart */
    state def StateSolverExecutionStatechart {
        entry;
        state IngestingStates;
        state FlatteningHierarchy;
        state SolvingTransitions;
        state EvaluatingReachability;
        state Solved;

        transition InitialToIngest
            first entry
            then IngestingStates;

        transition IngestToFlatten
            first IngestingStates
            then FlatteningHierarchy;

        transition FlattenToSolve
            first FlatteningHierarchy
            then SolvingTransitions;

        transition SolveToEvaluate
            first SolvingTransitions
            then EvaluatingReachability;

        transition EvaluateToSolved
            first EvaluatingReachability
            then Solved;
    }

    doc /* Action pipeline for resolving state machines and reachability matrices */
    action def SolveStateDynamicsAction {
        in stream_in : ASTStreamPort;
        out state_out : StateSolverPort;
    }

    doc /* Interaction lifeline modeling grammar lowering to state machine solver execution */
    interaction def StateSolverInteraction {
        lifeline grammar : GrammarLoweringEngine;
        lifeline solver : StateMachineSolverEngine;

        message SubmitModel from grammar to solver;
        message StateOutput from solver to grammar;
    }

    doc /* Comprehensive use case for state machine analysis and reachability proving */
    use case def UC_06_State_Machine_Solver_Workflow {
        doc /*
        Actors: CIContinuousIntegrationRunner, HumanEngineer
        Subject: StateMachineSolverEngine
        Preconditions: AST stream containing state definitions and transitions is available.
        Trigger: State machine verification trigger dispatched by compilation pipeline.
        Main Success Scenario:
        1. GrammarLoweringEngine streams state definitions to StateMachineSolverEngine.
        2. StateHierarchyFlattener transforms composite states into equivalent flat automata.
        3. DiscreteTransitionSolver verifies determinism across transition guards and events.
        4. ReachabilityMatrixEvaluator computes transitive closure of reachability graph.
        5. ReachabilityMatrixEvaluator confirms zero unreachable states and zero deadlocks.
        6. StateMachineSolverEngine emits verified StateSolverPort to downstream safety assurance.
        Alternate and Exception Flows:
        5a. Deadlock state identified: Reachability evaluator flags terminal sink without exit and reports E0203.
        5b. Unreachable state detected: Solver logs warning for orphan state unreachable from initial state.
        5c. Non-deterministic transition guards: Solver flags overlapping boolean guards on same trigger.
        5d. Cyclic state transition without termination: Livelock detection heuristic logs warning note.
        5e. Deep statechart nesting exceeds bound: Flattener halts and reports excessive recursion depth.
        5f. Missing initial transition in region: Solver inserts default entry transition to first state.
        5g. Undeclared event trigger referenced: Solver logs unresolved signal diagnostic.
        5h. Conflicting concurrent orthostate transitions: Flattener applies priority ordering.
        5i. State timeout value missing physical unit: Engine binds default millisecond timing unit.
        5j. State machine graph size exceeds solver capacity: Evaluator switches to bounded depth search.
        Postconditions: State machine dynamics, transition determinism, and reachability are formally verified.
        */
    }
}
