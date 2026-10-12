doc /* Behavioral models and execution flows for PhysicalMetrologyFlows */
package PhysicalMetrologyFlows {
    import DEAP_Compiler_System::*;

    doc /* Flow conservation convergence state machine */
    state def FlowConservationFSM {
        entry;
        state NetworkIngest;
        state UnitDimensionalAnalysis;
        state KirchhoffBalanceCheck;
        state FlowRouting;
        state Converged;

        transition InitialToIngest
            first entry
            then NetworkIngest;

        transition IngestToUnits
            first NetworkIngest
            then UnitDimensionalAnalysis;

        transition UnitsToBalance
            first UnitDimensionalAnalysis
            then KirchhoffBalanceCheck;

        transition BalanceToRouting
            first KirchhoffBalanceCheck
            then FlowRouting;

        transition RoutingToConverged
            first FlowRouting
            then Converged;
    }

    doc /* Action pipeline for dimensional analysis and flow conservation solving */
    action def SolveFlowsAction {
        in stream_in : ASTStreamPort;
        out metrology_out : MetrologyFlowPort;
    }

    doc /* Interaction lifeline modeling grammar lowering to physical metrology verification */
    interaction def MetrologyFlowInteraction {
        lifeline grammar : GrammarLoweringEngine;
        lifeline metrology : MetrologyFlowEngine;

        message SubmitAST from grammar to metrology;
        message EmitMetrology from metrology to grammar;
    }

    doc /* Comprehensive use case for 7D physical metrology and flow conservation verification */
    use case def UC_05_Physical_Metrology_and_Flow_Conservation_Workflow {
        doc /*
        Actors: CIContinuousIntegrationRunner, HumanEngineer
        Subject: MetrologyFlowEngine
        Preconditions: AST stream containing port definitions and flow definitions is available.
        Trigger: Metrology flow verification command dispatched by compiler pipeline.
        Main Success Scenario:
        1. GrammarLoweringEngine passes AST stream to MetrologyFlowEngine.
        2. DimensionalUnitChecker verifies SI unit coherence across feature types (length, mass, time, current, temp, amount, luminous).
        3. FlowConservationSolver inspects connection networks and verifies conservation of mass, energy, and data rates.
        4. ConservationNetworkRouter checks that total inward flow equals total outward flow across intermediate nodes.
        5. MetrologyFlowEngine calculates margin bounds and validates nominal operational tolerances.
        6. MetrologyFlowEngine outputs verified MetrologyFlowPort payload to downstream safety engine.
        Alternate and Exception Flows:
        5a. Dimensional incompatibility detected: Unit checker emits diagnostic E0201 for dimension mismatch.
        5b. Flow conservation violation: Kirchhoff balance solver reports residual flow leak diagnostic.
        5c. Unrecognized engineering unit: Metrology engine emits unrecognized unit warning and defaults to scalar.
        5d. Negative flow rate specified on unidirectional port: Router flags illegal direction and reports E0202.
        5e. Floating-point tolerance divergence: Solver adjusts iterative step size and re-evaluates balance.
        5f. Missing unit on numeric parameter: Unit checker assigns default SI base unit and logs advisory.
        5g. Over-constrained flow loop: Solver detects algebraic loop and applies loop-breaking heuristic.
        5h. Flow capacity overflow: Router flags exceedance of maximum rated port capacity.
        5i. Zero-flow deadlock condition: Engine identifies dormant branch and warns operator.
        5j. Non-linear flow curve singularity: Solver activates boundary clipping and logs numerical warning.
        Postconditions: All physical flows, dimensional units, and conservation laws are formally verified.
        */
    }
}
