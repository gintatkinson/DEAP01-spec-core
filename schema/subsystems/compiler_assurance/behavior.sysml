doc /* Behavioral models and execution flows for CompilerAssurance */
package CompilerAssurance {
    import DEAP_Compiler_System::*;

    doc /* Compiler performance assurance and CLI orchestration state machine */
    state def CompilerAssuranceFSM {
        entry;
        state CliDispatch;
        state PerformanceProfiling;
        state RegressionExecution;
        state MetricsAggregation;
        state AssuranceEmission;

        transition InitialToDispatch
            first entry
            then CliDispatch;

        transition DispatchToProfiling
            first CliDispatch
            then PerformanceProfiling;

        transition ProfilingToRegression
            first PerformanceProfiling
            then RegressionExecution;

        transition RegressionToAggregation
            first RegressionExecution
            then MetricsAggregation;

        transition AggregationToEmission
            first MetricsAggregation
            then AssuranceEmission;
    }

    doc /* Action pipeline for executing headless CLI commands and compiling performance assurance metrics */
    action def VerifyCompilerAssuranceAction {
        in cli_in : CLIPort;
        out metrics_out : PerformanceMetricsPort;
    }

    doc /* Interaction lifeline modeling CLI dispatcher to compiler assurance engine */
    interaction def CompilerAssuranceInteraction {
        lifeline cli : CliCommandDispatcher;
        lifeline assurance : CompilerAssuranceEngine;

        message DispatchCommand from cli to assurance;
        message ReturnMetrics from assurance to cli;
    }

    doc /* Comprehensive use case for compiler performance profiling and regression assurance */
    use case def UC_12_Compiler_Performance_Assurance_Workflow {
        doc /*
        Actors: HumanEngineer, ContinuousIntegrationOrchestrator
        Subject: CompilerAssuranceEngine
        Preconditions: Compiler CLI binary is built and test suites/schemas are present in workspace.
        Trigger: Headless CLI command invocation or automated CI regression test trigger.
        Main Success Scenario:
        1. HumanEngineer or CI runner invokes compiler CLI entry point via cli_in port.
        2. CliCommandDispatcher parses CLI arguments and flags into structured execution request.
        3. PerformanceProfiler starts execution timers and tracks memory heap allocation rates.
        4. RegressionInoculationRunner executes compilation passes across authoritative schemas.
        5. PerformanceProfiler aggregates throughput, CPU time, and peak memory figures.
        6. CompilerAssuranceEngine emits formatted performance report and verification status via metrics_out.
        Alternate and Exception Flows:
        5a. Invalid CLI flag specified: Dispatcher prints usage help and terminates with exit code 2.
        5b. Schema path does not exist: Dispatcher emits E0300 invalid path diagnostic.
        5c. Performance threshold violated: Profiler records latency regression warning in report.
        5d. Non-zero exit code in compilation pass: Engine halts regression sequence and returns failure status.
        5e. Environment variable overrides configured: Dispatcher incorporates flags into configuration profile.
        5f. Unicode em dash in CLI parameter: Dispatcher sanitizes string to ASCII double hyphen.
        5g. Memory budget exceeded: Profiler issues memory pressure alert and triggers arena compaction.
        5h. JSON output format requested: Dispatcher serializes metrics as structured JSON telemetry payload.
        5i. Dry-run mode specified: Dispatcher validates command parameters without invoking full build.
        5j. Benchmark baseline missing: Engine establishes current run as initial performance reference point.
        Postconditions: Headless CLI command terminates cleanly and assurance metrics are verified.
        */
    }
}
