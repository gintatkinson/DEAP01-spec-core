doc /* Behavioral models and execution flows for MultiTargetCodegen */
package MultiTargetCodegen {
    import DEAP_Compiler_System::*;
    import DownstreamAgileProjections::*;

    doc /* Multi-target code generation pipeline state machine */
    state def CodeGenFSM {
        entry;
        state SpecIngest;
        state TargetAstMapping;
        state TemplateRendering;
        state CodeFormatting;
        state ArtifactEmission;

        transition InitialToIngest
            first entry
            then SpecIngest;

        transition IngestToMapping
            first SpecIngest
            then TargetAstMapping;

        transition MappingToRendering
            first TargetAstMapping
            then TemplateRendering;

        transition RenderingToFormatting
            first TemplateRendering
            then CodeFormatting;

        transition FormattingToEmission
            first CodeFormatting
            then ArtifactEmission;
    }

    doc /* Action pipeline for compiling specification models into target language source files */
    action def SynthesizeCodeAction {
        in specs_in : SpecificationPort;
        out code_out : CodeArtifactPort;
    }

    doc /* Interaction lifeline modeling Agile projection engine to code generation engine */
    interaction def CodeGenInteraction {
        lifeline projection : AgileProjectionEngine;
        lifeline codegen : CodeGenEngine;

        message TransmitSpecs from projection to codegen;
        message EmitCode from codegen to projection;
    }

    doc /* Comprehensive use case for multi-target code generation and simulation runtime bindings */
    use case def UC_10_Multi_Target_CodeGen_Workflow {
        doc /*
        Actors: HumanEngineer, BuildAutomationBot
        Subject: CodeGenEngine
        Preconditions: Agile specifications and structural classifiers are loaded and verified.
        Trigger: Code generation command dispatched by user or automated build pipeline.
        Main Success Scenario:
        1. CodeGenEngine receives specifications from AgileProjectionEngine via specs_in port.
        2. TargetAstTranspiler maps abstract SysML classifiers into language-specific AST structures.
        3. TemplateRenderingEngine renders deterministic source code using target language templates.
        4. CodeArtifactFormatter runs language-specific formatters and linter checks.
        5. CodeGenEngine emits formatted code artifacts to designated workspace target directories.
        6. CompilerAssuranceEngine verifies artifact compilation and records performance metrics.
        Alternate and Exception Flows:
        5a. Target language template missing: Engine reports missing template diagnostic and aborts target.
        5b. Target syntax violation in rendered code: Transpiler flags invalid token and emits diagnostic.
        5c. Destination file path read-only: Engine logs permission error and halts execution.
        5d. Unsupported target architecture requested: Engine falls back to reference C++ or Rust implementation.
        5e. Transpilation timeout: Engine aborts code generation unit and logs timeout diagnostic.
        5f. Unicode em dash detected in target comments: Formatter replaces with ASCII double hyphen.
        5g. External linter tool unavailable: Formatter proceeds with built-in AST canonical printer.
        5h. Circular dependency in target modules: Transpiler topologically sorts dependencies or emits error.
        5i. Dry-run mode specified: Formatter emits artifact diff to stdout without touching disk.
        5j. Generation cache hit: Engine skips re-rendering unchanged modules to optimize throughput.
        Postconditions: Validated multi-target code artifacts are written to disk cleanly.
        */
    }
}
