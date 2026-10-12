doc /* Structural Architecture for Subsystem 12 Compiler Performance CLI Assurance */
package CompilerAssurance {
    import DEAP_Compiler_System::*;

    doc /* Primary execution engine for compiler performance profiling, CLI orchestration, and regression assurance */
    part def CompilerAssuranceEngine {
        port in cli_in : CLIPort;
        port out metrics_out : PerformanceMetricsPort;

        part cliDispatcher : CliCommandDispatcher;
        part perfProfiler : PerformanceProfiler;
        part regressionRunner : RegressionInoculationRunner;
    }

    doc /* Headless command line interface parser and subcommand execution dispatcher */
    part def CliCommandDispatcher {
        port in cli_in : CLIPort;
        port out metrics_out : PerformanceMetricsPort;
    }

    doc /* High-resolution performance profiler capturing heap allocations, AST throughput, and wall-clock latencies */
    part def PerformanceProfiler {
        port in cli_in : CLIPort;
        port out metrics_out : PerformanceMetricsPort;
    }

    doc /* Regression inoculation test runner verifying compilation benchmarks and semantic invariant preservation */
    part def RegressionInoculationRunner {
        port in cli_in : CLIPort;
        port out metrics_out : PerformanceMetricsPort;
    }
}
