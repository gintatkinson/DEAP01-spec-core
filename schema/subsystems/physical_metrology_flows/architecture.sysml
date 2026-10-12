doc /* Structural Architecture for Subsystem 5 7D Physical Metrology Flow Networks */
package PhysicalMetrologyFlows {
    import DEAP_Compiler_System::*;

    doc /* Primary execution engine for physical unit metrology and flow conservation verification */
    part def MetrologyFlowEngine {
        port in ast_in : ASTStreamPort;
        port out metrology_out : MetrologyFlowPort;

        part unitChecker : DimensionalUnitChecker;
        part flowSolver : FlowConservationSolver;
        part networkRouter : ConservationNetworkRouter;
    }

    doc /* Dimensional analysis and SI 7D base unit coherence checker */
    part def DimensionalUnitChecker {
        port in ast_in : ASTStreamPort;
        port out metrology_out : MetrologyFlowPort;
        attribute strict_si_mode : Boolean;
    }

    doc /* Kirchhoff flow conservation and mass/energy conservation solver */
    part def FlowConservationSolver {
        port in ast_in : ASTStreamPort;
        port out metrology_out : MetrologyFlowPort;
        attribute tolerance_epsilon : Real;
    }

    doc /* Topological flow network router validating directed port connections and flow rates */
    part def ConservationNetworkRouter {
        port in ast_in : ASTStreamPort;
        port out metrology_out : MetrologyFlowPort;
        attribute max_hops : Integer;
    }
}
