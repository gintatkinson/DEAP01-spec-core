doc /* Structural Architecture for Subsystem 4 Complete SysMLv2 KerML Grammar */
package SysmlGrammarLowering {
    import DEAP_Compiler_System::*;

    doc /* Primary execution engine for OMG SysML v2 / KerML grammar parsing and semantic lowering */
    part def GrammarLoweringEngine {
        port in ast_in : ASTGraphPort;
        port out ast_out : ASTStreamPort;

        part tokenStream : KerMLTokenStream;
        part grammarParser : SysMLv2GrammarParser;
        part loweringEngine : ASTLoweringEngine;
    }

    doc /* KerML root token stream provider and lexical keyword tokenizer */
    part def KerMLTokenStream {
        port in ast_in : ASTGraphPort;
        port out ast_out : ASTStreamPort;
        attribute buffer_size : Integer;
    }

    doc /* Recursive descent SysML v2 parser constructing typed AST classifier elements */
    part def SysMLv2GrammarParser {
        port in ast_in : ASTGraphPort;
        port out ast_out : ASTStreamPort;
        attribute recursion_limit : Integer;
    }

    doc /* Semantic lowering pipeline resolving feature usages, port conjugation, and type inheritance */
    part def ASTLoweringEngine {
        port in ast_in : ASTGraphPort;
        port out ast_out : ASTStreamPort;
        attribute pass_count : Integer;
    }
}
