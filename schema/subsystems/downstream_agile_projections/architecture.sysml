doc /* Structural Architecture for Subsystem 9 Downstream Specification Projections */
package DownstreamAgileProjections {
    import DEAP_Compiler_System::*;

    doc /* Primary execution engine for projecting SysML AST into Agile specification markdown hierarchies */
    part def AgileProjectionEngine {
        port in icd_in : ICDInterconnectPort;
        port out specs_out : SpecificationPort;

        part specProjector : MarkdownSpecProjector;
        part fmEncoder : FrontmatterIntegrityEncoder;
        part checklistReconciler : BacklogChecklistReconciler;
    }

    doc /* Markdown specification projector lowering packages to Epics, parts to Features, actions to Stories */
    part def MarkdownSpecProjector {
        port in icd_in : ICDInterconnectPort;
        port out specs_out : SpecificationPort;
    }

    doc /* Frontmatter encoder maintaining YAML metadata, UUIDv5 identifiers, and issue linkages */
    part def FrontmatterIntegrityEncoder {
        port in icd_in : ICDInterconnectPort;
        port out specs_out : SpecificationPort;
    }

    doc /* Backlog checklist reconciler updating checkmarks and cross-references against live tracker state */
    part def BacklogChecklistReconciler {
        port in icd_in : ICDInterconnectPort;
        port out specs_out : SpecificationPort;
    }
}
