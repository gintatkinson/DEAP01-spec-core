doc /* Structural Architecture for Subsystem 11 Standardized Diagnostic Error Catalog */
package DiagnosticErrorCatalog {
    import DEAP_Compiler_System::*;

    doc /* Primary execution engine for standardized compiler diagnostic error cataloging and triage */
    part def DiagnosticErrorCatalogEngine {
        port in diag_in : DiagnosticPort;
        port out diag_out : DiagnosticReportPort;

        part registry : ErrorCodeRegistry;
        part locMapper : SourceLocationMapper;
        part reportFormatter : DiagnosticReportFormatter;
    }

    doc /* Error code registry maintaining unique E-codes, severity levels, and remediation templates */
    part def ErrorCodeRegistry {
        port in diag_in : DiagnosticPort;
        port out diag_out : DiagnosticReportPort;
    }

    doc /* Source location mapper resolving byte spans to exact 1-indexed line and column coordinates */
    part def SourceLocationMapper {
        port in diag_in : DiagnosticPort;
        port out diag_out : DiagnosticReportPort;
    }

    doc /* Diagnostic report formatter rendering ANSI terminal diagnostics, JSON payloads, and SARIF streams */
    part def DiagnosticReportFormatter {
        port in diag_in : DiagnosticPort;
        port out diag_out : DiagnosticReportPort;
    }
}
