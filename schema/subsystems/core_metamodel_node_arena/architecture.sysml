doc /* Structural Architecture for Subsystem 3 Core Metamodel Node Arena */
package CoreMetamodelNodeArena {
    import DEAP_Compiler_System::*;

    doc /* Primary execution engine for contiguous memory arena allocation and AST graph indexing */
    part def NodeArenaASTGraphEngine {
        port in token_in : TokenStreamPort;
        port out ast_graph_out : ASTGraphPort;

        part allocator : ArenaAllocator;
        part indexer : NodeTableIndex;
        part interner : AstInterner;
    }

    doc /* Uniform-slot contiguous virtual memory chunk allocator for AST nodes */
    part def ArenaAllocator {
        port in token_in : TokenStreamPort;
        port out ast_graph_out : ASTGraphPort;
        attribute slot_size_bytes : Integer;
        attribute alignment_bytes : Integer;
    }

    doc /* Zero-indirection scalar NodeIndex lookup and topological table indexing */
    part def NodeTableIndex {
        port in token_in : TokenStreamPort;
        port out ast_graph_out : ASTGraphPort;
        attribute capacity : Integer;
    }

    doc /* String and structural symbol interner eliminating redundant AST allocations */
    part def AstInterner {
        port in token_in : TokenStreamPort;
        port out ast_graph_out : ASTGraphPort;
        attribute deduplication_enabled : Boolean;
    }
}
