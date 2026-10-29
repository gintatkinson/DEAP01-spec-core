//! Topology JSON ingestion, disk asset resolution, and SysML package slice discovery.
//! ## 1. Safety Intent & Regulatory Scope
//! Conforms to DO-178C Level A and DO-330 TQL-4 deterministic topology ingestion controls.
//! ## 2. Invariants & Mathematical Properties
//! - Deterministic `BTreeMap` property ordering and atomic `ErrorKind::NotFound` disk fallback.
//! ## 3. Preconditions
//! - Callers supply UTF-8 JSON payloads, workspace paths, or parsed SysML `PackageDef` trees.
//! ## 4. Postconditions
//! - Produces normalized `TopologyNode` collections and `(snake_case_stem, &PackageDef)` slices.
//! ## 5. Algorithmic Complexity Bounds
//! - Time Complexity: $O(N)$ linear in JSON entity count or SysML package hierarchy size.
//! - Space Complexity: $O(N)$ bounded heap allocation for normalized entities and slice tuples.
//! ## 6. Failure Modes & Fault Tolerance
//! - Propagates syntax and non-missing filesystem faults through `FlutterCodegenError`.
//! ## 7. Realization & Traceability
//! Realises: [REQ-0607], [REQ-FLUTTER-PARSE-TOPOLOGY-JSON]

use std::collections::BTreeMap;
use std::fs;
use std::io::ErrorKind;
use std::path::Path;

use deap_core::sysml_ast::{PackageDef, PartDef};

use super::contracts::{FlutterCodegenError, TopologyNode};
use super::naming::{format_display_name, to_pascal_case, to_snake_case_dart};

/// Trims whitespace and removes matching outer single or double quote delimiters.
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A lexical token sanitization for SysML and JSON string literals.
/// ## 2. Invariants & Mathematical Properties
/// - Strips outer delimiters exclusively when leading and trailing quote characters match.
/// ## 3. Preconditions
/// - `raw` is any valid UTF-8 string slice.
/// ## 4. Postconditions
/// - Returns a borrowed subslice free of surrounding whitespace and symmetric outer quotes.
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(N)$ bounded by leading and trailing whitespace scan over `raw`.
/// - Space Complexity: $O(1)$ zero-allocation subslice projection.
/// ## 6. Failure Modes & Fault Tolerance
/// - Total function returning `trimmed` verbatim when quotes are absent or asymmetric.
/// ## 7. Realization & Traceability
/// Realises: [REQ-0607], [REQ-FLUTTER-PARSE-TOPOLOGY-JSON]
pub(crate) fn strip_surrounding_quotes(raw: &str) -> &str {
    let trimmed = raw.trim();
    let double_stripped = trimmed.strip_prefix('"').and_then(|s| s.strip_suffix('"'));
    let single_stripped = trimmed.strip_prefix('\'').and_then(|s| s.strip_suffix('\''));
    double_stripped.or(single_stripped).unwrap_or(trimmed)
}

/// Normalizes a raw JSON object map into a typed `TopologyNode` when a non-empty `id` exists.
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A schema-independent topology attribute normalization rules.
/// ## 2. Invariants & Mathematical Properties
/// - Rejects anonymous records (`id.is_empty()`) and copies properties into a sorted `BTreeMap`.
/// ## 3. Preconditions
/// - `obj` references a decoded `serde_json::Map<String, serde_json::Value>` record.
/// ## 4. Postconditions
/// - Returns `Some(TopologyNode)` with canonical defaults or `None` when `id` is missing.
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(P \log P)$ where $P$ is the property count of `obj`.
/// - Space Complexity: $O(P)$ bounded allocation for cloned property entries.
/// ## 6. Failure Modes & Fault Tolerance
/// - Substitutes deterministic defaults for omitted optional type, label, and epoch fields.
/// ## 7. Realization & Traceability
/// Realises: [REQ-0607], [REQ-FLUTTER-PARSE-TOPOLOGY-JSON]
fn parse_topology_node_object(
    obj: &serde_json::Map<String, serde_json::Value>,
) -> Option<TopologyNode> {
    let id = obj.get("id").and_then(serde_json::Value::as_str).unwrap_or("").to_string();
    if id.is_empty() {
        return None;
    }
    let type_name = ["typeName", "type_name", "type"]
        .into_iter().find_map(|k| obj.get(k).and_then(serde_json::Value::as_str))
        .unwrap_or("Entity").to_string();
    let label =
        obj.get("label").and_then(serde_json::Value::as_str).unwrap_or(id.as_str()).to_string();
    let parent_id = ["parentId", "parent_id"]
        .into_iter().find_map(|k| obj.get(k).and_then(serde_json::Value::as_str))
        .map(str::to_string);
    let created_at = ["createdAt", "created_at"]
        .into_iter().find_map(|k| obj.get(k).and_then(serde_json::Value::as_i64))
        .unwrap_or(1_729_684_800_000);
    let updated_at = ["updatedAt", "updated_at"]
        .into_iter().find_map(|k| obj.get(k).and_then(serde_json::Value::as_i64))
        .unwrap_or(1_729_684_800_000);
    let properties: BTreeMap<String, serde_json::Value> = obj.get("properties")
        .and_then(serde_json::Value::as_object)
        .map(|m| m.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
        .unwrap_or_default();
    Some(TopologyNode { id, type_name, label, parent_id, properties, created_at, updated_at })
}

/// Parses a JSON array or `{"nodes": [...]}` envelope into normalized `TopologyNode` values.
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and DO-330 TQL-4 fail-closed JSON deserialization controls.
/// ## 2. Invariants & Mathematical Properties
/// - Preserves source array ordering while filtering non-object entries and empty identifiers.
/// ## 3. Preconditions
/// - `json_str` is a UTF-8 slice containing a candidate JSON topology document.
/// ## 4. Postconditions
/// - Returns `Ok(Vec<TopologyNode>)` on valid JSON or `Err(FlutterCodegenError::Json)` on syntax faults.
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(N \cdot P \log P)$ over $N$ JSON elements with $P$ properties each.
/// - Space Complexity: $O(N \cdot P)$ for parsed AST values and normalized output nodes.
/// ## 6. Failure Modes & Fault Tolerance
/// - Non-array and non-envelope JSON payloads degrade safely to an empty vector.
/// ## 7. Realization & Traceability
/// Realises: [REQ-0607], [REQ-FLUTTER-PARSE-TOPOLOGY-JSON]
pub fn parse_topology_json(json_str: &str) -> Result<Vec<TopologyNode>, FlutterCodegenError> {
    let value: serde_json::Value = serde_json::from_str(json_str)?;
    let node_array = if let Some(arr) = value.as_array() {
        Some(arr)
    } else {
        value.as_object().and_then(|o| o.get("nodes")).and_then(serde_json::Value::as_array)
    };
    let nodes = node_array
        .map(|arr| {
            arr.iter().filter_map(serde_json::Value::as_object)
                .filter_map(parse_topology_node_object).collect()
        })
        .unwrap_or_default();
    Ok(nodes)
}

/// Resolves topology nodes from an explicit JSON override, candidate asset files, or SysML parts.
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A TOCTOU-free filesystem ingestion and deterministic seed synthesis.
/// ## 2. Invariants & Mathematical Properties
/// - Probes candidate paths directly via `fs::read_to_string`, falling back to 1-based `PartDef` seeds.
/// ## 3. Preconditions
/// - `out_dir` and `base_dir` designate workspace directories; `parts` holds parsed `PartDef` items.
/// ## 4. Postconditions
/// - Returns `(raw_topology_json, resolved_nodes)` with empty raw text when synthesized from `parts`.
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(B + M)$ linear in JSON byte size $B$ or `PartDef` slice length $M$.
/// - Space Complexity: $O(B + M)$ bounded by raw payload buffer and resolved `TopologyNode` vector.
/// ## 6. Failure Modes & Fault Tolerance
/// - Missing files (`ErrorKind::NotFound`) continue probing; other I/O and JSON faults fail closed.
/// ## 7. Realization & Traceability
/// Realises: [REQ-0607], [REQ-FLUTTER-PARSE-TOPOLOGY-JSON]
pub fn resolve_topology_nodes(
    out_dir: &Path, base_dir: &Path, parts: &[PartDef], topology_json: Option<&str>,
) -> Result<(String, Vec<TopologyNode>), FlutterCodegenError> {
    if let Some(json_str) = topology_json {
        return parse_topology_json(json_str).map(|nodes| (json_str.to_string(), nodes));
    }
    for candidate in [
        base_dir.join("assets").join("topology_data.json"),
        out_dir.join("assets").join("topology_data.json"),
        out_dir.join("app_flutter").join("assets").join("topology_data.json"),
    ] {
        match fs::read_to_string(&candidate) {
            Ok(content) => {
                let parsed = parse_topology_json(&content);
                return parsed.map(|nodes| (content, nodes));
            }
            Err(err) if err.kind() == ErrorKind::NotFound => {}
            Err(err) => return Err(FlutterCodegenError::Io(err)),
        }
    }
    let synth = parts.iter().enumerate().map(|(idx, part)| TopologyNode {
        id: format!("{}_{}", to_pascal_case(&part.name), idx.saturating_add(1)),
        type_name: part.name.clone(),
        label: format_display_name(&part.name),
        parent_id: None,
        properties: BTreeMap::new(),
        created_at: 1_729_684_800_000,
        updated_at: 1_729_684_800_000,
    }).collect();
    Ok((String::new(), synth))
}

/// Traverses a SysML `PackageDef` tree iteratively in pre-order to collect non-empty Dart slices.
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A stack-bounded traversal rules without unbounded call-stack recursion.
/// ## 2. Invariants & Mathematical Properties
/// - Emits the initial package unconditionally and retains subsequent packages bearing definitions.
/// ## 3. Preconditions
/// - `root` references a valid parsed `PackageDef` hierarchy.
/// ## 4. Postconditions
/// - Returns pre-order `(snake_case_module_name, &PackageDef)` pairs across `root` and descendants.
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(K)$ visiting each of the $K$ nested `PackageDef` nodes once.
/// - Space Complexity: $O(K)$ explicit heap workstack and output slice descriptor vector.
/// ## 6. Failure Modes & Fault Tolerance
/// - Total panic-free iterative traversal over arbitrarily deep or empty package trees.
/// ## 7. Realization & Traceability
/// Realises: [REQ-0607], [REQ-FLUTTER-PARSE-TOPOLOGY-JSON]
pub fn collect_package_slices(root: &PackageDef) -> Vec<(String, &PackageDef)> {
    let mut stack = vec![root];
    let mut out = Vec::new();
    while let Some(pkg) = stack.pop() {
        let flags = [
            !pkg.part_defs.is_empty(), !pkg.port_defs.is_empty(),
            !pkg.item_defs.is_empty(), out.is_empty(),
        ];
        if flags.into_iter().any(std::convert::identity) {
            out.push((to_snake_case_dart(&pkg.name), pkg));
        }
        stack.extend(pkg.packages.iter().rev());
    }
    out
}
