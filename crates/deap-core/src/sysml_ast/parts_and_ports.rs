//! Structural part, port, runtime type reflection, and schema-extension catalog definitions for SysML v2 AST models.
//!
//! Safety & Traceability: Realises: [REQ-SYSML-AST-PARTS-AND-PORTS]
//! Contract: Defines canonical structural block (`PartDef`), interface port (`PortDef`), part member (`OperationDef`, `CapabilityDef`, `ItemDef`), runtime reflection (`FieldDescriptor`, `TypeDescriptor`), and closed schema format (`SupportedSchemaExtension`, `SupportedExtensionsCatalog`) types.
//! Bounds & Faults: All structural lookups and descriptor projections execute deterministically without call-stack recursion, panics, or unchecked indexing.

use super::identifiers::{
    default_direction_inout, default_port_category, default_true, extract_si_unit_and_bounds,
};
use super::{
    ActionDef, AttributeDef, CalcDef, ConnectionDef, ConstraintDef, FlowDef, HazardDef,
    InteractionDef, RequirementDef, RiskDef, StateDef, TestCaseDef, UseCaseDef,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Represents a port definition or usage interface boundary point in SysML v2.
///
/// Safety & Traceability: Realises: [REQ-DEAP-CORE-SYSML-AST-PORTDEF]
/// Contract: Captures port identifier, typed interface reference, directionality (`in`, `out`, `inout`), conjugation flag, protocol metadata, deterministic `BTreeMap` electrical attributes, and nested item flows.
/// Bounds & Faults: Bounded by declared attribute and flow count per port; defaults direction to `"inout"` and category to `"logical"` when omitted.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct PortDef {
    pub name: String,
    pub type_name: String,
    #[serde(default = "default_direction_inout")] pub direction: String,
    #[serde(default)] pub is_conjugated: bool,
    pub doc: Option<String>,
    #[serde(default = "default_port_category")] pub port_category: String,
    #[serde(default)] pub protocol_family: String,
    #[serde(default)] pub electrical_attributes: BTreeMap<String, String>,
    #[serde(default)] pub item_flows: Vec<FlowDef>,
}

/// Represents a typed operation signature declared on a SysML v2 structural part.
///
/// Safety & Traceability: Realises: [REQ-DEAP-CORE-SYSML-AST-OPERATIONDEF]
/// Contract: Preserves operation identifier, parameter direction, primary parameter type, optional return type, and ordered `AttributeDef` parameter list.
/// Bounds & Faults: Linear memory in parameter count; defaults direction to `"inout"` on deserialization.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct OperationDef {
    pub name: String,
    #[serde(default = "default_direction_inout")] pub direction: String,
    #[serde(default)] pub param_type: String,
    pub return_type: Option<String>,
    pub doc: Option<String>,
    #[serde(default)] pub parameters: Vec<AttributeDef>,
}

/// Represents a subsystem capability specification attached to a SysML v2 package or part.
///
/// Safety & Traceability: Realises: [REQ-DEAP-CORE-SYSML-AST-CAPABILITYDEF]
/// Contract: Associates a capability identifier with its optional prose description, owning subsystem name, package reference, and documentation block.
/// Bounds & Faults: O(1) fixed-field value container with infallible default construction.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct CapabilityDef {
    pub name: String,
    pub description: Option<String>,
    pub subsystem: Option<String>,
    pub package_ref: Option<String>,
    pub doc: Option<String>,
}

/// Represents a typed data payload or item definition flowing across SysML v2 ports and connections.
///
/// Safety & Traceability: Realises: [REQ-DEAP-CORE-SYSML-AST-ITEMDEF]
/// Contract: Encapsulates item type name, optional documentation comment, and ordered payload `AttributeDef` fields.
/// Bounds & Faults: Space bounded linearly by `attributes.len()` with deterministic field ordering.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct ItemDef {
    pub name: String,
    pub doc: Option<String>,
    #[serde(default)] pub attributes: Vec<AttributeDef>,
}

/// Represents a structural `part def` block (`is_def == true`) or `part` usage (`is_def == false`) in SysML v2.
///
/// Safety & Traceability: Realises: [REQ-DEAP-CORE-SYSML-AST-PARTDEF]
/// Contract: Aggregates structural attributes, ports, nested sub-parts, behavioral actions/states/use-cases/interactions, parametric constraints/calculations, requirements, and STPA hazard/risk annotations for a single block.
/// Bounds & Faults: Tree containment is finite and acyclic; `Default` initializes `is_def` to `true` and all member collections to empty vectors.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PartDef {
    pub name: String,
    pub doc: Option<String>,
    #[serde(default = "default_true")] pub is_def: bool,
    pub type_name: Option<String>,
    #[serde(default)] pub attributes: Vec<AttributeDef>,
    #[serde(default)] pub ports: Vec<PortDef>,
    #[serde(default)] pub actions: Vec<ActionDef>,
    #[serde(default)] pub constraints: Vec<ConstraintDef>,
    #[serde(default)] pub requirements: Vec<RequirementDef>,
    #[serde(default)] pub satisfied_requirements: Vec<String>,
    #[serde(default)] pub parts: Vec<PartDef>,
    #[serde(default)] pub operations: Vec<OperationDef>,
    #[serde(default)] pub capabilities: Vec<CapabilityDef>,
    #[serde(default)] pub interactions: Vec<InteractionDef>,
    #[serde(default)] pub test_cases: Vec<TestCaseDef>,
    #[serde(default)] pub states: Vec<StateDef>,
    #[serde(default)] pub use_cases: Vec<UseCaseDef>,
    #[serde(default)] pub item_defs: Vec<ItemDef>,
    #[serde(default)] pub hazards: Vec<HazardDef>,
    #[serde(default)] pub risks: Vec<RiskDef>,
    #[serde(default)] pub connections: Vec<ConnectionDef>,
    #[serde(default)] pub calcs: Vec<CalcDef>,
    #[serde(default)] pub triggers_hazards: Vec<String>,
    #[serde(default)] pub safety_realises: Vec<String>,
    #[serde(default)] pub layer: Option<String>,
}

impl Default for PartDef {
    /// Constructs a default `PartDef` with `is_def` initialized to `true` and all member vectors empty.
    /// Realises: [REQ-SYSML-AST-SSOT-PARITY/PART-DEF-DEFAULT]
    fn default() -> Self {
        Self {
            name: String::new(), doc: None, is_def: true, type_name: None,
            attributes: Vec::new(), ports: Vec::new(), actions: Vec::new(),
            constraints: Vec::new(), requirements: Vec::new(), satisfied_requirements: Vec::new(),
            parts: Vec::new(), operations: Vec::new(), capabilities: Vec::new(),
            interactions: Vec::new(), test_cases: Vec::new(), states: Vec::new(),
            use_cases: Vec::new(), item_defs: Vec::new(), hazards: Vec::new(),
            risks: Vec::new(), connections: Vec::new(), calcs: Vec::new(),
            triggers_hazards: Vec::new(), safety_realises: Vec::new(), layer: None,
        }
    }
}

impl PartDef {
    /// Searches `self` and all nested sub-parts in pre-order depth-first order for a mutable `PartDef` matching `name`.
    ///
    /// Safety & Traceability: Realises: [REQ-SYSML-AST-SSOT-PARITY/PART-DEF-MUT]
    /// Contract: Returns `Some(&mut PartDef)` for the first pre-order node where `part.name == name`, or `None` without mutating the tree when absent.
    /// Bounds & Faults: Uses an explicit iterative worklist stack in O(V_part) time and O(V_part) heap stack space with zero call-stack recursion.
    pub fn find_part_mut(&mut self, name: &str) -> Option<&mut PartDef> {
        let mut stack = vec![self];
        while let Some(part) = stack.pop() {
            if part.name == name {
                return Some(part);
            }
            for sub in part.parts.iter_mut().rev() {
                stack.push(sub);
            }
        }
        None
    }
}

/// Runtime metadata descriptor for a single `AttributeDef` field declared on a SysML v2 `PartDef`.
///
/// Safety & Traceability: Realises: [REQ-DEAP-CORE-SYSML-AST-FIELDDESCRIPTOR]
/// Contract: Captures field identifier, SysML/KerML type name, optional extracted SI unit symbol, and optional numeric `[min, max]` range bounds.
/// Bounds & Faults: Fixed-size metadata value struct; unannotated fields set `si_unit` and `range_bounds` to `None`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct FieldDescriptor {
    pub name: String,
    pub type_name: String,
    pub si_unit: Option<String>,
    pub range_bounds: Option<(f64, f64)>,
}

/// Runtime metadata reflection descriptor projected from a SysML v2 `PartDef` component.
///
/// Safety & Traceability: Realises: [REQ-DEAP-CORE-SYSML-AST-TYPEDESCRIPTOR]
/// Contract: Binds `type_name`, enclosing `package_name`, optional architectural `@Layer`, ordered `FieldDescriptor` list, and `satisfied_requirements`.
/// Bounds & Faults: Space and cloning cost scale linearly in `fields.len()` and `satisfied_requirements.len()`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct TypeDescriptor {
    pub type_name: String,
    pub package_name: String,
    pub layer: Option<String>,
    pub fields: Vec<FieldDescriptor>,
    pub satisfied_requirements: Vec<String>,
}

impl TypeDescriptor {
    /// Synthesizes a runtime `TypeDescriptor` from a `PartDef` AST node and its enclosing `package_name`.
    ///
    /// Safety & Traceability: Realises: [REQ-SYSML-AST-TYPE-DESCRIPTOR-FROM-PART]
    /// Contract: Preserves exact declaration order of `part.attributes`, extracting SI units and numeric bounds from `default_value` or falling back to `doc`.
    /// Bounds & Faults: Executes in O(A * L) time over `A` attributes of text length `L`; never mutates `part` or panics on unannotated attributes.
    pub fn from_part_def(package_name: &str, part: &PartDef) -> Self {
        let fields = part
            .attributes
            .iter()
            .map(|attr| {
                let (mut si_unit, mut range_bounds) = attr
                    .default_value
                    .as_deref()
                    .map_or((None, None), extract_si_unit_and_bounds);
                if si_unit.is_none() && range_bounds.is_none() {
                    if let Some(ref doc_str) = attr.doc {
                        let (doc_unit, doc_bounds) = extract_si_unit_and_bounds(doc_str);
                        si_unit = doc_unit;
                        range_bounds = doc_bounds;
                    }
                }
                FieldDescriptor {
                    name: attr.name.clone(),
                    type_name: attr.type_name.clone(),
                    si_unit,
                    range_bounds,
                }
            })
            .collect();
        Self {
            type_name: part.name.clone(),
            package_name: package_name.to_string(),
            layer: part.layer.clone(),
            fields,
            satisfied_requirements: part.satisfied_requirements.clone(),
        }
    }
}

/// Closed enumeration of every schema file extension recognized by multi-format AST discovery.
///
/// Safety & Traceability: Realises: [REQ-SYSML-AST-SUPPORTED-SCHEMA-EXTENSION], [deap_core::sysml_ast::SupportedSchemaExtension]
/// Contract: Enumerates all 10 supported model formats (`.sysml`, `.yaml`, `.yml`, `.json`, `.proto`, `.idl`, `.arxml`, `.md`, `.markdown`, `.yang`) in canonical discovery order.
/// Bounds & Faults: Copyable fieldless enum with exhaustive compile-time suffix projection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SupportedSchemaExtension {
    Sysml,
    Yaml,
    Yml,
    Json,
    Proto,
    Idl,
    Arxml,
    Md,
    Markdown,
    Yang,
}

impl SupportedSchemaExtension {
    /// Canonical declaration-ordered table of all 10 supported schema file extensions.
    /// Realises: [REQ-SYSML-AST-MOD-SUPPORTEDSCHEMAEXTENSION-ALL]
    pub const ALL: &'static [Self] = &[
        Self::Sysml, Self::Yaml, Self::Yml, Self::Json, Self::Proto,
        Self::Idl, Self::Arxml, Self::Md, Self::Markdown, Self::Yang,
    ];

    /// Projects a schema extension variant onto its dot-prefixed lowercase file suffix.
    ///
    /// Safety & Traceability: Realises: [REQ-SYSML-AST-SSOT-PARITY/SUPPORTED-SCHEMA-EXTENSION-AS-STR]
    /// Contract: Bijectively maps each of the 10 `SupportedSchemaExtension` variants to its canonical lowercase `".<ext>"` literal.
    /// Bounds & Faults: O(1) `const` match with zero heap allocation and compile-time exhaustiveness.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Sysml => ".sysml",
            Self::Yaml => ".yaml",
            Self::Yml => ".yml",
            Self::Json => ".json",
            Self::Proto => ".proto",
            Self::Idl => ".idl",
            Self::Arxml => ".arxml",
            Self::Md => ".md",
            Self::Markdown => ".markdown",
            Self::Yang => ".yang",
        }
    }

    /// Reference adapter yielding the dot-prefixed suffix for a borrowed `SupportedSchemaExtension`.
    ///
    /// Safety & Traceability: Realises: [REQ-SYSML-AST-SSOT-PARITY/SUPPORTED-SCHEMA-EXTENSION-AS-STR-REF]
    /// Contract: Pure forwarding projection satisfying `ext.as_str_ref() == ext.as_str()` for slice iterator mapping.
    /// Bounds & Faults: O(1) time and space; total and side-effect free.
    pub fn as_str_ref(&self) -> &'static str {
        self.as_str()
    }
}

/// Zero-sized catalog providing deterministic iteration and membership queries over supported schema suffixes.
///
/// Safety & Traceability: Realises: [REQ-SYSML-AST-SUPPORTED-EXTENSIONS-CATALOG], [deap_core::sysml_ast::SupportedExtensionsCatalog]
/// Contract: Acts as the stateless admission filter backed directly by `SupportedSchemaExtension::ALL`.
/// Bounds & Faults: Zero-sized value requiring zero heap memory.
#[derive(Debug, Clone, Copy, Default)]
pub struct SupportedExtensionsCatalog;

impl SupportedExtensionsCatalog {
    /// Checks whether a dot-prefixed, lowercased file suffix denotes a supported schema format.
    ///
    /// Safety & Traceability: Realises: [REQ-SYSML-AST-SSOT-PARITY/SUPPORTED-EXTENSIONS-CONTAINS]
    /// Contract: Returns `true` iff `*ext` exactly matches one of the 10 lowercase dot-prefixed suffixes in `SupportedSchemaExtension::ALL`.
    /// Bounds & Faults: O(1) bounded byte match; fails closed (`false`) on uppercase, unprefixed, empty, or unknown extensions.
    pub fn contains(&self, ext: &&str) -> bool {
        matches!(
            *ext,
            ".sysml" | ".yaml" | ".yml" | ".json" | ".proto" | ".idl" | ".arxml" | ".md" | ".markdown" | ".yang"
        )
    }

    /// Returns a non-allocating iterator over all 10 supported dot-prefixed schema file suffixes.
    ///
    /// Safety & Traceability: Realises: [REQ-SYSML-AST-SSOT-PARITY/SUPPORTED-EXTENSIONS-ITER]
    /// Contract: Yields all 10 suffixes in `SupportedSchemaExtension::ALL` order such that every yielded item satisfies `self.contains(&item)`.
    /// Bounds & Faults: O(1) iterator construction and O(10) traversal with zero heap allocation.
    pub fn iter(
        &self,
    ) -> std::iter::Map<
        std::slice::Iter<'static, SupportedSchemaExtension>,
        fn(&'static SupportedSchemaExtension) -> &'static str,
    > {
        let project: fn(&'static SupportedSchemaExtension) -> &'static str =
            SupportedSchemaExtension::as_str_ref;
        SupportedSchemaExtension::ALL.iter().map(project)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn part_def_find_part_mut_traverses_deep_hierarchy_iteratively() -> Result<(), Box<dyn std::error::Error>> {
        let mut current = PartDef { name: "LeafActuator_128".to_string(), ..Default::default() };
        for depth in (0..128_usize).rev() {
            let sibling = PartDef { name: format!("Sibling_{depth}"), ..Default::default() };
            current = PartDef { name: format!("Chain_{depth}"), parts: vec![sibling, current], ..Default::default() };
        }
        assert!(current.is_def && current.find_part_mut("NonExistentNode").is_none());
        current.find_part_mut("Sibling_64").ok_or("missing Sibling_64")?.layer = Some("Control".to_string());
        current.find_part_mut("LeafActuator_128").ok_or("missing LeafActuator_128")?.doc = Some("Deep leaf reached without call-stack recursion".to_string());
        assert_eq!(current.find_part_mut("Sibling_64").and_then(|p| p.layer.as_deref()), Some("Control"));
        assert_eq!(current.find_part_mut("LeafActuator_128").and_then(|p| p.doc.as_deref()), Some("Deep leaf reached without call-stack recursion"));
        Ok(())
    }

    #[test]
    fn type_descriptor_extracts_units_from_default_or_doc_fallback() -> Result<(), Box<dyn std::error::Error>> {
        let part = PartDef {
            name: "ThermalRegulator".to_string(),
            layer: Some("Plant".to_string()),
            satisfied_requirements: vec!["REQ-THERM-01".to_string()],
            attributes: vec![
                AttributeDef { name: "maxTemp".to_string(), type_name: "Real".to_string(), default_value: Some("85.0 [K] [0.0..120.0]".to_string()), doc: None },
                AttributeDef { name: "plainFlag".to_string(), type_name: "Boolean".to_string(), default_value: None, doc: None },
            ],
            ..Default::default()
        };
        let desc = TypeDescriptor::from_part_def("ThermalPkg", &part);
        assert_eq!((desc.type_name.as_str(), desc.package_name.as_str(), desc.layer.as_deref(), desc.fields.len()), ("ThermalRegulator", "ThermalPkg", Some("Plant"), 2));
        let second = desc.fields.get(1).ok_or("missing second field")?;
        assert!(second.name == "plainFlag" && second.si_unit.is_none() && second.range_bounds.is_none());
        Ok(())
    }

    #[test]
    fn supported_extensions_catalog_enforces_closed_lowercase_dot_set() {
        let catalog = SupportedExtensionsCatalog;
        let collected: Vec<&str> = catalog.iter().collect();
        assert_eq!(collected.len(), 10);
        for ext in &collected {
            assert!(catalog.contains(ext));
        }
        assert!(!catalog.contains(&".SYSML") && !catalog.contains(&"sysml") && !catalog.contains(&"") && !catalog.contains(&".toml"));
    }
}
