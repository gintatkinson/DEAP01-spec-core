doc /* Structural Architecture for Subsystem 2 Universal Schema Ingestion Engine */
package UniversalSchemaIngestion {
    import DEAP_Compiler_System::*;

    doc /* Primary execution engine for multi-format schema ingestion and normalization */
    part def UniversalIngestionEngine {
        port in schema_in : RawSchemaStreamPort;
        port in rules_in : CompilerRulePort;
        port out token_out : TokenStreamPort;
        port out diag_out : DiagnosticPort;

        part classifier : FormatClassifier;
        part mdLexer : CommonMarkTableLexer;
        part protoParser : ProtobufParser;
        part openapiParser : OpenApiParser;
    }

    doc /* Autonomous format classifier identifying SysML, CommonMark, Protobuf, and OpenAPI schemas */
    part def FormatClassifier {
        port in raw_in : RawSchemaStreamPort;
        port out token_out : TokenStreamPort;
        attribute auto_detect : Boolean;
    }

    doc /* Deterministic pushdown automaton table lexer for CommonMark table ingestion */
    part def CommonMarkTableLexer {
        port in raw_in : RawSchemaStreamPort;
        port out token_out : TokenStreamPort;
        attribute table_delimiter : String;
    }

    doc /* Binary and textual Protocol Buffer schema parser */
    part def ProtobufParser {
        port in raw_in : RawSchemaStreamPort;
        port out token_out : TokenStreamPort;
        attribute syntax_version : Integer;
    }

    doc /* OpenAPI / JSON Schema definition parser and structural object lowerer */
    part def OpenApiParser {
        port in raw_in : RawSchemaStreamPort;
        port out token_out : TokenStreamPort;
        attribute spec_version : String;
    }
}
