doc /* Behavioral models and execution flows for DiagnosticErrorCatalog */
package DiagnosticErrorCatalog {
    import DEAP_Compiler_System::*;
    import UniversalSchemaIngestion::*;

    doc /* Diagnostic triage and reporting state machine */
    state def DiagnosticCatalogFSM {
        entry;
        state ErrorIngest;
        state CodeLookup;
        state LocationMapping;
        state ReportFormatting;
        state DiagnosticEmission;

        transition InitialToIngest
            first entry
            then ErrorIngest;

        transition IngestToLookup
            first ErrorIngest
            then CodeLookup;

        transition LookupToMapping
            first CodeLookup
            then LocationMapping;

        transition MappingToFormatting
            first LocationMapping
            then ReportFormatting;

        transition FormattingToEmission
            first ReportFormatting
            then DiagnosticEmission;
    }

    doc /* Action pipeline for receiving, categorizing, and formatting compiler diagnostics */
    action def TriageDiagnosticAction {
        in diag_in : DiagnosticPort;
        out diag_out : DiagnosticReportPort;
    }

    doc /* Interaction lifeline modeling schema ingestion engine to diagnostic catalog engine */
    interaction def DiagnosticCatalogInteraction {
        lifeline ingestion : UniversalIngestionEngine;
        lifeline catalog : DiagnosticErrorCatalogEngine;

        message EmitDiagnostic from ingestion to catalog;
        message ReportDiagnostic from catalog to ingestion;
    }

    doc /* Comprehensive use case for diagnostic catalog triage and reporting */
    use case def UC_11_Diagnostic_Catalog_Triage_Workflow {
        doc /*
        Actors: HumanEngineer, CompilerAssuranceEngine
        Subject: DiagnosticErrorCatalogEngine
        Preconditions: Compiler pipeline encounters warning, error, or fatal validation condition.
        Trigger: Diagnostic signal emitted on diag_in port during compilation run.
        Main Success Scenario:
        1. DiagnosticErrorCatalogEngine ingests raw diagnostic event via diag_in port.
        2. ErrorCodeRegistry resolves canonical error code (E-code), severity level, and guidance.
        3. SourceLocationMapper resolves byte offsets to exact 1-indexed line and column numbers.
        4. DiagnosticReportFormatter builds structured diagnostic record with code span highlights.
        5. Formatter emits rendered diagnostic to terminal stderr or JSON/SARIF diagnostic stream.
        6. CompilerAssuranceEngine records error tally in performance and compliance metrics.
        Alternate and Exception Flows:
        5a. Unknown error code encountered: Registry assigns fallback E0999 generic diagnostic code.
        5b. Source file location out of range: Mapper marks location as synthetic end-of-file.
        5c. Maximum error limit reached: Engine suppresses further errors and emits fatal E0199 limit notice.
        5d. Non-UTF8 source character in span: Mapper renders sanitization placeholder for byte offset.
        5e. Terminal lacks color support: Formatter strips ANSI escape sequences automatically.
        5f. Em dash detected in diagnostic message: Formatter replaces with ASCII double hyphen.
        5g. Cascading error suppression active: Engine suppresses follow-on errors on same AST node.
        5h. SARIF output mode requested: Formatter emits OASIS SARIF v2.1.0 compliant JSON schema payload.
        5i. Warning treated as error (-Werror): Registry elevates severity from Warning to Error.
        5j. Quiet mode active: Formatter filters non-fatal warnings and outputs only fatal errors.
        Postconditions: All compiler diagnostic records are classified, formatted, and delivered deterministically.
        */
    }
}
