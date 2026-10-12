doc /* Behavioral models and execution flows for FormalSafetyTraceability */
package FormalSafetyTraceability {
    import DEAP_Compiler_System::*;

    doc /* 7-tier bipartite DAG reachability sweep state machine */
    state def TraceabilitySweepFSM {
        entry;
        state GraphIngest;
        state TierPartitioning;
        state ForwardReachabilitySweep;
        state BackwardReachabilitySweep;
        state TraceabilityVerified;

        transition InitialToIngest
            first entry
            then GraphIngest;

        transition IngestToPartition
            first GraphIngest
            then TierPartitioning;

        transition PartitionToForward
            first TierPartitioning
            then ForwardReachabilitySweep;

        transition ForwardToBackward
            first ForwardReachabilitySweep
            then BackwardReachabilitySweep;

        transition BackwardToVerified
            first BackwardReachabilitySweep
            then TraceabilityVerified;
    }

    doc /* Action pipeline for constructing bipartite traceability graphs and verifying safety matrices */
    action def VerifySafetyTraceabilityAction {
        in metrology_in : MetrologyFlowPort;
        in state_in : StateSolverPort;
        out safety_out : SafetyPort;
    }

    doc /* Interaction lifeline modeling state solver to safety assurance verification */
    interaction def SafetyTraceabilityInteraction {
        lifeline solver : StateMachineSolverEngine;
        lifeline safety : SafetyAssuranceEngine;

        message SubmitStateModel from solver to safety;
        message EmitSafetyProof from safety to solver;
    }

    doc /* Comprehensive use case for safety assurance and formal traceability verification */
    use case def UC_07_Safety_Assurance_and_Traceability_Workflow {
        doc /*
        Actors: CIContinuousIntegrationRunner, HumanEngineer
        Subject: SafetyAssuranceEngine
        Preconditions: Metrology flows and state models are verified.
        Trigger: Safety verification pass initiated by compiler pipeline.
        Main Success Scenario:
        1. SafetyAssuranceEngine ingests metrology and state solver outputs.
        2. BipartiteGraphBuilder builds the 7-tier traceability directed acyclic graph.
        3. DisjointTierValidator verifies that edges connect strictly adjacent or permitted tiers.
        4. BidirectionalSweepEngine performs forward reachability sweeps from requirements to code.
        5. BidirectionalSweepEngine performs backward reachability sweeps from code to hazards.
        6. SafetyAssuranceEngine verifies 100 percent closure of STPA, FMECA, and safety invariants.
        Alternate and Exception Flows:
        5a. Traceability gap detected: Bipartite graph builder flags unlinked requirement and emits E0400.
        5b. Cross-tier violation detected: Tier validator rejects illegal edge crossing multiple tiers.
        5c. Unmitigated hazard identified: Safety engine emits critical hazard warning and binds failsafe constraint.
        5d. Cyclic traceability edge detected: Graph builder halts traversal, breaks cycle, and reports E0402.
        5e. Orphan design element found: Backward sweep detects element without upstream parent requirement.
        5f. Missing FMECA failure mode: Component failure mode matrix reports unanalyzed failure mode.
        5g. UCA guide word omitted: Generator synthesizes default guide word and logs coverage advisory.
        5h. Safety constraint violation in code mapping: Engine reports verification mismatch.
        5i. RPN calculation overflow: Criticality calculator saturates at maximum RPN 1000.
        5j. Incomplete regulatory objective coverage: Audit engine lists missing regulatory clause citations.
        Postconditions: 7-tier traceability DAG is complete, bipartite, and closed with zero dangling elements.
        */
    }
}
