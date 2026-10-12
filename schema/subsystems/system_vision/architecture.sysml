doc /* Structural Architecture for Subsystem 1 System Vision */
package SystemVision {
    import DEAP_Compiler_System::*;

    doc /* Primary execution engine for System Vision, Bootstrapping & Foundational Invariants */
    part def SystemVisionEngine {
        port out rules_out : CompilerRulePort;

        part bootstrapper : InvariantBootstrapEngine;
        part uuidGen : NamespaceUuidGenerator;
        part vocabValidator : DomainVocabularyValidator;
    }

    doc /* Invariant bootstrapper establishing foundational compiler constraints and namespace anchors */
    part def InvariantBootstrapEngine {
        port out rules_out : CompilerRulePort;
        attribute compiler_namespace_uuid : String;
    }

    doc /* Deterministic RFC 4122 UUIDv5 topological namespace hashing engine */
    part def NamespaceUuidGenerator {
        port in path_in : ASTGraphPort;
        port out uuid_out : CompilerRulePort;
        attribute uuid_version : Integer;
    }

    doc /* Domain vocabulary validator enforcing zero hardcoded domain semantics */
    part def DomainVocabularyValidator {
        port in token_in : TokenStreamPort;
        port out diag_out : DiagnosticPort;
        attribute max_violations : Integer;
    }
}
