doc /* Behavioral models and execution flows for UniversalSchemaIngestion */
package UniversalSchemaIngestion {
    import DEAP_Compiler_System::*;

    doc /* Deterministic pushdown automaton for streaming CommonMark table lexing */
    state def CommonMarkTableLexerStatechart {
        entry;
        state ScanState;
        state HeaderState;
        state DelimiterState;
        state RowState;
        state CellState;
        state EscapeState;
        state HaltState;

        transition InitialToScan
            first entry
            then ScanState;

        transition ScanToHeader
            first ScanState
            accept PipeToken
            then HeaderState;

        transition HeaderToDelimiter
            first HeaderState
            accept NewlineToken
            then DelimiterState;

        transition DelimiterToRow
            first DelimiterState
            accept PipeToken
            then RowState;

        transition RowToCell
            first RowState
            accept CellTextToken
            then CellState;

        transition CellToEscape
            first CellState
            accept EscapeToken
            then EscapeState;

        transition EscapeToCell
            first EscapeState
            accept EscapedCharToken
            then CellState;

        transition CellToHalt
            first CellState
            accept EofToken
            then HaltState;
    }

    doc /* Autonomous format classifier action */
    action def ClassifyFormatAction {
        in input : RawSchemaStreamPort;
        out output : RawSchemaStreamPort;
    }

    doc /* Stream tokenization action converting schema streams into typed tokens */
    action def TokenizeStreamAction {
        in input : RawSchemaStreamPort;
        out output : TokenStreamPort;
    }

    doc /* Delimiter synchronization recovery action */
    action def RecoverSyncAction {
        in input : RawSchemaStreamPort;
        out output : TokenStreamPort;
    }

    doc /* End-to-end schema classification and tokenization action pipeline */
    action def IngestSchemaAction {
        in schema_file : RawSchemaStreamPort;
        out token_stream : TokenStreamPort;

        action classify_format : ClassifyFormatAction;
        action tokenize_stream : TokenizeStreamAction;
        action recover_synchronization : RecoverSyncAction;

        flow from schema_file to classify_format.input;
        flow from classify_format.output to tokenize_stream.input;
        flow from tokenize_stream.output to token_stream;
    }

    doc /* Interaction lifeline modeling client-to-engine schema ingestion */
    interaction def IngestSchemaInteraction {
        lifeline client : DEAPCompilerSystem;
        lifeline engine : UniversalIngestionEngine;

        message IngestRequest from client to engine;
        message TokenResponse from engine to client;
    }

    doc /* Comprehensive use case for universal schema ingestion */
    use case def UC_02_Universal_Schema_Ingestion_Workflow {
        doc /*
        Actors: HumanEngineer, CIContinuousIntegrationRunner
        Subject: UniversalIngestionEngine
        Preconditions: Raw schema files exist on filesystem with read permissions.
        Trigger: Ingestion command dispatched with path to target schema artifact.
        Main Success Scenario:
        1. Client submits raw schema file path through schema_in port.
        2. FormatClassifier inspects magic bytes and file extensions to identify source type.
        3. UniversalIngestionEngine selects corresponding parser engine (CommonMark, Protobuf, OpenAPI, SysML).
        4. Selected parser converts structural declarations, tables, and interfaces into typed token representations.
        5. Parser verifies token well-formedness and balances container delimiters.
        6. UniversalIngestionEngine streams normalized token stream through token_out port to node arena.
        Alternate and Exception Flows:
        5a. Unrecognized schema format: FormatClassifier fails identification and emits diagnostic E0101.
        5b. Unterminated table cell in CommonMark: Lexer transitions to EscapeState, reports warning, and closes cell safely.
        5c. Malformed Protobuf descriptor syntax: ProtobufParser recovers on message boundary and emits E0102.
        5d. Invalid OpenAPI YAML/JSON syntax: OpenApiParser emits line-col syntax diagnostic and halts processing.
        5e. Empty input file provided: Ingestion engine emits warning and produces empty token stream.
        5f. Premature EOF encountered: Parser closes open container scopes and logs diagnostic E0103.
        5g. Character encoding invalid (non-UTF8): Lexer rejects binary input with encoding diagnostic.
        5h. Extreme token stream depth (> 1024 nested scopes): Parser halts with recursion depth limit error.
        5i. Ingestion buffer overflow: Engine applies backpressure to raw schema input stream.
        5j. Circular schema file include: Ingestion preprocessor breaks circular loop and emits cycle diagnostic.
        Postconditions: Raw schema artifacts are converted into normalized, strongly-typed token streams.
        */
    }
}
