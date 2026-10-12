doc /* Behavioral models and execution flows for DownstreamAgileProjections */
package DownstreamAgileProjections {
    import DEAP_Compiler_System::*;

    doc /* Agile specification projection pipeline state machine */
    state def AgileProjectionFSM {
        entry;
        state SpecIngest;
        state EpicProjection;
        state FeatureProjection;
        state StoryProjection;
        state ReconciliationComplete;

        transition InitialToIngest
            first entry
            then SpecIngest;

        transition IngestToEpic
            first SpecIngest
            then EpicProjection;

        transition EpicToFeature
            first EpicProjection
            then FeatureProjection;

        transition FeatureToStory
            first FeatureProjection
            then StoryProjection;

        transition StoryToReconcile
            first StoryProjection
            then ReconciliationComplete;
    }

    doc /* Action pipeline for lowering SysML AST into Agile Epics, Features, User Stories, and Use Cases */
    action def ProjectAgileSpecificationsAction {
        in icd_in : ICDInterconnectPort;
        out specs_out : SpecificationPort;
    }

    doc /* Interaction lifeline modeling ICD interconnect engine to agile projection engine */
    interaction def AgileProjectionInteraction {
        lifeline icd : ICDEngine;
        lifeline projection : AgileProjectionEngine;

        message TransmitContracts from icd to projection;
        message EmitSpecs from projection to icd;
    }

    doc /* Comprehensive use case for agile specification projection and backlog synchronization */
    use case def UC_09_Agile_Specification_Projection_Workflow {
        doc /*
        Actors: HumanEngineer, CIContinuousIntegrationRunner
        Subject: AgileProjectionEngine
        Preconditions: Authoritative SysML model and ICD interconnects are compiled and verified.
        Trigger: Forward specification synchronization command dispatched by compiler CLI.
        Main Success Scenario:
        1. AgileProjectionEngine ingests compiled AST and ICD contracts.
        2. MarkdownSpecProjector synthesizes Epics from top-level PackageDef nodes.
        3. MarkdownSpecProjector synthesizes Features from structural PartDef nodes with dynamic JSON Schemas.
        4. MarkdownSpecProjector synthesizes User Stories from InteractionDef and ActionDef nodes.
        5. MarkdownSpecProjector synthesizes Use Cases with AST-anchored Realization Matrices.
        6. FrontmatterIntegrityEncoder binds metadata and emits specification files to docs/ directories.
        Alternate and Exception Flows:
        5a. Output directory not writable: Engine reports filesystem permission error and halts safely.
        5b. Frontmatter metadata schema validation failure: Encoder repairs missing attributes with safe defaults.
        5c. Broken cross-reference link detected: Projector verifies target file exists before writing link.
        5d. Unicode em dash detected in text stream: Projector sanitizes character to ASCII double hyphen.
        5e. Mermaid diagram syntax error in generated block: Formatter validates fence syntax and escapes labels.
        5f. Orphan feature detected without parent epic: Projector assigns feature to system root epic.
        5g. Missing acceptance criteria in requirement: Projector extracts BDD scenario from requirement docstring.
        5h. Duplicate spec title detected: Metadata validator appends unique subsystem qualifier to title.
        5i. Dry-run mode enabled: Projector logs simulated file writes without altering disk.
        5j. Specification cardinality bounds exceeded: Projector emits advisory note for epic feature count.
        Postconditions: Full Agile specification hierarchy is projected to docs/ conforming to all quality gates.
        */
    }
}
