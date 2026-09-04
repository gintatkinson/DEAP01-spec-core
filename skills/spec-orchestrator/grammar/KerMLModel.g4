grammar KerMLModel;

// ==========================================
// Parser Rules
// ==========================================

root
    : element* EOF
    ;

element
    : metadataDef
    | packageDecl
    | partDecl
    | attributeDecl
    ;

metadataDef
    : 'metadata' 'def' ID '{' metadataFieldDef* '}'
    ;

metadataFieldDef
    : 'attribute' ID (':' qualifiedName)? ';'
    ;

metadataAnnotation
    : '@' qualifiedName ('{' (binding ';')* '}' | '(' (binding (',' binding)*)? ')')
    ;

binding
    : ID '=' expression
    ;

packageDecl
    : metadataAnnotation* 'package' qualifiedName '{' element* '}'
    ;

partDecl
    : metadataAnnotation* 'part' 'def'? ID '{' element* '}'
    ;

attributeDecl
    : metadataAnnotation* 'attribute' ID (':' qualifiedName)? ('=' expression)? ('[' unitRef=qualifiedName ']')? ';'
    ;

expression
    : literal
    | collectionLiteral
    | qualifiedName
    ;

collectionLiteral
    : '(' (expression (',' expression)*)? ')'
    | '[' (expression (',' expression)*)? ']'
    ;

literal
    : STRING            # StringLiteral
    | INT               # IntLiteral
    | FLOAT             # FloatLiteral
    | BOOLEAN           # BoolLiteral
    ;

qualifiedName
    : ID ('::' ID)*
    ;

// ==========================================
// Lexer Rules
// ==========================================

BOOLEAN
    : 'true' | 'false'
    ;

ID
    : [a-zA-Z_][a-zA-Z0-9_]*
    ;

INT
    : [0-9]+
    ;

FLOAT
    : [0-9]+ '.' [0-9]+ ([eE][+-]?[0-9]+)?
    ;

STRING
    : '"' (~["\\] | '\\' .)* '"'
    ;

WS
    : [ \t\r\n]+ -> skip
    ;

LINE_COMMENT
    : '//' ~[\r\n]* -> skip
    ;

BLOCK_COMMENT
    : '/*' .*? '*/' -> skip
    ;
