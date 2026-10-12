doc /* Structural Architecture for Subsystem 6 Spatio Temporal State Solvers */
package SpatioTemporalStateSolvers {
    import DEAP_Compiler_System::*;

    doc /* Primary execution engine for discrete state machines and temporal transition reachability */
    part def StateMachineSolverEngine {
        port in ast_in : ASTStreamPort;
        port out state_out : StateSolverPort;

        part flattener : StateHierarchyFlattener;
        part solver : DiscreteTransitionSolver;
        part evaluator : ReachabilityMatrixEvaluator;
    }

    doc /* Hierarchical composite state flattener resolving orthostates and entry/exit actions */
    part def StateHierarchyFlattener {
        port in ast_in : ASTStreamPort;
        port out state_out : StateSolverPort;
        attribute max_nesting_depth : Integer;
    }

    doc /* Discrete transition solver evaluating guards, events, and deterministic transition firing */
    part def DiscreteTransitionSolver {
        port in ast_in : ASTStreamPort;
        port out state_out : StateSolverPort;
        attribute guard_evaluation_mode : String;
    }

    doc /* Reachability graph evaluator proving deadlock-free execution and state reachability */
    part def ReachabilityMatrixEvaluator {
        port in ast_in : ASTStreamPort;
        port out state_out : StateSolverPort;
        attribute deadlock_detection : Boolean;
    }
}
