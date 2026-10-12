doc /* Abstract Model-Based Systems Engineering (MBSE) Compiler System Architecture */
package DEAP_Compiler_System {
    import ConOps::*;
    import SystemVision::*;
    import UniversalSchemaIngestion::*;
    import CoreMetamodelNodeArena::*;
    import SysmlGrammarLowering::*;
    import PhysicalMetrologyFlows::*;
    import SpatioTemporalStateSolvers::*;
    import FormalSafetyTraceability::*;
    import InterconnectContracts::*;
    import DownstreamAgileProjections::*;
    import MultiTargetCodegen::*;
    import DiagnosticErrorCatalog::*;
    import CompilerAssurance::*;

    doc /* Common Port Definitions */
    port def CLIPort;
    port def RawSchemaStreamPort;
    port def CodeArtifactPort;
    port def DiagnosticReportPort;
    port def PerformanceMetricsPort;
    port def CompilerRulePort;
    port def TokenStreamPort;
    port def ASTGraphPort;
    port def ASTStreamPort;
    port def MetrologyFlowPort;
    port def StateSolverPort;
    port def SafetyPort;
    port def ICDInterconnectPort;
    port def SpecificationPort;
    port def DiagnosticPort;

    doc /* Level 1 System of Interest Execution Lifecycle */
    state def SystemExecutionLifecycle {
        entry;
        state SystemColdStart;
        state SubsystemDiscovery;
        state PipelineExecution;
        state HealthDiagnosticsEvaluation;
        state SystemReady;
        state DegradedFailure;

        transition InitialToColdStart
            first entry
            then SystemColdStart;

        transition ColdStartToDiscovery
            first SystemColdStart
            then SubsystemDiscovery;

        transition DiscoveryToPipeline
            first SubsystemDiscovery
            then PipelineExecution;

        transition PipelineToDiagnostics
            first PipelineExecution
            then HealthDiagnosticsEvaluation;

        transition DiagnosticsToReady
            first HealthDiagnosticsEvaluation
            then SystemReady;

        transition DiagnosticsToDegraded
            first HealthDiagnosticsEvaluation
            then DegradedFailure;
    }

    doc /* Level 1 System of Interest (SoI) Architecture Definition */
    part def DEAPCompilerSystem {
        port in cli_in : CLIPort;
        port in schema_in : RawSchemaStreamPort;
        port out code_out : CodeArtifactPort;
        port out diag_out : DiagnosticReportPort;
        port out metrics_out : PerformanceMetricsPort;

        part systemVision : SystemVisionEngine;
        part universalIngestion : UniversalIngestionEngine;
        part nodeArenaASTGraph : NodeArenaASTGraphEngine;
        part grammarLowering : GrammarLoweringEngine;
        part metrologyFlow : MetrologyFlowEngine;
        part stateMachineSolver : StateMachineSolverEngine;
        part safetyAssurance : SafetyAssuranceEngine;
        part icdEngine : ICDEngine;
        part agileProjection : AgileProjectionEngine;
        part codeGen : CodeGenEngine;
        part diagnosticCatalog : DiagnosticErrorCatalogEngine;
        part compilerAssurance : CompilerAssuranceEngine;

        connect cli_in to compilerAssurance.cli_in;
        connect compilerAssurance.metrics_out to metrics_out;
        connect schema_in to universalIngestion.schema_in;
        connect systemVision.rules_out to universalIngestion.rules_in;
        connect universalIngestion.token_out to nodeArenaASTGraph.token_in;
        connect nodeArenaASTGraph.ast_graph_out to grammarLowering.ast_in;
        connect grammarLowering.ast_out to metrologyFlow.ast_in;
        connect grammarLowering.ast_out to stateMachineSolver.ast_in;
        connect metrologyFlow.metrology_out to safetyAssurance.metrology_in;
        connect stateMachineSolver.state_out to safetyAssurance.state_in;
        connect safetyAssurance.safety_out to icdEngine.safety_in;
        connect icdEngine.icd_out to agileProjection.icd_in;
        connect agileProjection.specs_out to codeGen.specs_in;
        connect codeGen.code_out to code_out;
        connect universalIngestion.diag_out to diagnosticCatalog.diag_in;
        connect diagnosticCatalog.diag_out to diag_out;
    }
}