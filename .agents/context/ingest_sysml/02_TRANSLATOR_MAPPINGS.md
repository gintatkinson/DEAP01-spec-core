# Markdown Translator Mappings & Metamodel Rules

## 1. Identifier Sanitization Rules

Any arbitrary text string is converted into a valid SysML v2 identifier following these deterministic steps:
1. Strip bracketed or parenthesized content if that leaves a non-empty name (e.g. `Mass (kg)` -> `Mass`, `[REQ-0001] Title` -> `Title`).
2. Remove Markdown backticks, asterisks, formatting marks (`*`, `` ` ``, `_`, `#`).
3. Replace all remaining non-alphanumeric characters with underscores (`_`).
4. Collapse consecutive underscores into a single `_` and trim leading/trailing underscores.
5. If the resulting string starts with a digit, prefix with `_` (e.g. `1. Normative Statement` -> `_1_Normative_Statement`).
6. If the identifier matches a SysML v2 reserved keyword, append `_item` (e.g. `Subsystem` -> `Subsystem_item`, `port` -> `port_item`).

Reserved Keywords:
`package`, `part`, `def`, `port`, `attribute`, `action`, `flow`, `item`, `connect`, `connection`, `interface`, `doc`, `assert`, `constraint`, `state`, `requirement`, `use`, `case`, `test`, `in`, `out`, `inout`, `import`, `public`, `private`, `alias`, `perform`, `subsystem`, `first`, `then`, `step`, `entry`, `exit`, `do`, `transition`, `assume`, `require`, `verify`, `satisfy`.

---

## 2. Table Classification & Archetype Mapping

Tables are classified by matching normalized column names:

| Table Archetype | Column Header Triggers | SysML v2 Target AST Element |
|---|---|---|
| **BOM / Components** | Contains component name (`component`, `part`, `subsystem`, `item`, `module`, `assembly`) AND property columns (`part_number`, `mass`, `power`, `qty`, `cost`) | `PartDef` with contained `AttributeDef` elements |
| **Ports / Interfaces** | Contains `port`, `signal`, `interface`, `flow` OR `direction` + `type`/`rate`/`protocol` | `PortDef` (in, out, inout), `FlowDef`, `ConnectionDef` |
| **Constraints / Limits** | Contains `min`, `max`, `lower_bound`, `upper_bound`, `range`, `limit`, `tolerance` | `ConstraintDef` (`assert constraint`) + `AttributeDef` |
| **Properties / Key-Value** | 2-3 columns with first column matching `property`, `parameter`, `attribute`, `key`, `field`, `name` | `AttributeDef` on parent container |
| **Generic** | Default structured table | Fallback to `AttributeDef` properties |

---

## 3. Data Type Normalization

| Raw Cell Format | Inferred SysML Type | Serialized SysML Value |
|---|---|---|
| Decimal integer (`42`, `-7`, `0`) | `Integer` | `42`, `-7`, `0` |
| Hexadecimal (`0x1A`, `0xFF`) | `Integer` | Parsed integer literal `26`, `255` |
| Floating point (`1800.0`, `1e-3`) | `Real` | `1800.0`, `0.001` |
| Boolean (`true`, `false`) | `Boolean` | `true`, `false` |
| Quoted or unquoted text string | `String` | `"cleaned text"` |

---

## 4. Provenance & Docstring Synthesis

Docstrings capture metadata, units, and OEM citations in a standardized format:
- Formula: `[Description] [unit: <unit>] [Source: <provenance_citation>]`
- Provenance columns: `provenance_citation`, `provenance`, `source_reference`, `oem_source_reference`, `oem_document_citation`, `citation`, `reference`, `source`, `clause`, `specification_reference`.
