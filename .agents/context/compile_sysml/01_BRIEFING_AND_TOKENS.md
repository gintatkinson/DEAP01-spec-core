# SysML v2 Lexer & Token Specification

## 1. Overview & Architectural Principles

The `compile-sysml` compiler lexer provides high-performance, deterministic tokenization of SysML v2 and KerML (Kernel Modeling Language) textual notation.
The lexer operates on source text slices, tracking 1-indexed source spans (`line`, `col`, `byte_offset`) for precise diagnostic reporting and AST provenance.

Strict Compiler Invariants:
- Pure schema-driven compiler: zero hardcoded domain concepts.
- Zero Unicode em dashes (\u2014): use ASCII `--` or `-` exclusively.
- Zero-mocking live persistence mandate (.pipeline/constitution.md Section 1.9).

---

## 2. Token Categories & Syntax Rules

### 2.1 Keywords
The lexer recognizes canonical SysML v2 and KerML declaration keywords:
- Structural: `package`, `part`, `def`, `port`, `connection`, `flow`, `item`, `interface`, `subsystem`, `subject`
- Behavioral: `action`, `operation`, `state`, `transition`, `entry`, `do`, `exit`, `perform`, `step`, `trigger`
- Requirements & Constraints: `requirement`, `constraint`, `assert`, `assume`, `require`, `verify`, `satisfy`, `calc`
- Analysis & Verification: `test`, `case`, `hazard`, `risk`, `objective`, `capability`, `interaction`, `lifeline`, `message`
- Directional & Relational: `in`, `out`, `inout`, `from`, `to`, `by`, `include`, `extend`, `connect`
- Typed Attributes: `attribute`, `id`, `text`, `doc`

### 2.2 Operators
- Assignment & Definition: `=`, `:=`, `:>` (specialization / subtyping)
- Directional Arrows: `->` (transition / directed flow), `~>` (conjugate / streaming), `~` (conjugation prefix)
- Comparison: `==`, `!=`, `<=`, `>=`, `<`, `>`
- Arithmetic: `+`, `-`, `*`, `/`, `^`, `%`
- Boolean: `and`, `or`, `not`, `&&`, `||`, `!`

### 2.3 Punctuation & Delimiters
- Braces: `{`, `}`
- Parentheses: `(`, `)`
- Brackets: `[`, `]`
- Statement & List Separators: `;`, `,`, `:`, `::`, `.`

### 2.4 Literals & Identifiers
- Identifiers: `[a-zA-Z_][a-zA-Z0-9_-]*` or escaped names `'...'`
- String Literals: `"..."` supporting escape sequences (`\"`, `\\`, `\n`, `\t`)
- Numeric Literals: Integers (`42`, `-7`) and floating-point Reals (`12.0`, `1e-3`, `-0.005`)
- Boolean Literals: `true`, `false`

### 2.5 Doc Comments & Annotations
- SysML v2 Doc Comments: `doc /* ... */` and block comments `/* ... */`
- Single-line comments: `// ...` (treated as trivia or retained when attached to declarations)

---

## 3. Scanner Design & Error Recovery

The scanner employs a stateful byte cursor over string slices:
- Non-destructive lookahead for multi-character operators (`:>`, `:=`, `->`, `~>`).
- String and block comment delimiter tracking with unclosed-delimiter recovery.
- Full line and column accumulation preserving UTF-8 multi-byte integrity.
