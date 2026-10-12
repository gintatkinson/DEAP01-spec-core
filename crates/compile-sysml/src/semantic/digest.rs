//! Schema digest computation and JSON persistence matching .pipeline/schema-digest.json.

use deap_core::sysml_ast::*;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashSet};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// Individual file entry recorded in the schema digest file manifest.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct FileDigestEntry {
    pub path: String,
    pub sha256: String,
    pub bytes: u64,
    pub lines: usize,
    pub entities: usize,
}

/// Operational activity descriptor in schema digest.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct OperationalActivity {
    pub id: String,
    pub name: String,
    pub doc: String,
    pub performer: String,
    pub inputs: Vec<String>,
    pub outputs: Vec<String>,
}

/// Operational exchange descriptor in schema digest.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct OperationalExchange {
    pub id: String,
    pub name: String,
    pub source_port: String,
    pub target_port: String,
    pub source_part: String,
    pub target_part: String,
    pub protocol: String,
}

/// Operational scenario descriptor in schema digest.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct OperationalScenario {
    pub id: String,
    pub name: String,
    pub actor: String,
    pub objective: String,
    pub steps: Vec<String>,
}

/// Operational node descriptor in schema digest.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct OperationalNode {
    pub name: String,
    pub node_type: String,
    pub doc: String,
    pub allocated_activities: Vec<String>,
}

/// Authoritative schema digest model written to .pipeline/schema-digest.json.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct SchemaDigest {
    pub sha256: String,
    pub total_lines: usize,
    pub node_counts: BTreeMap<String, usize>,
    pub schema_nodes: Vec<String>,
    #[serde(default)]
    pub operational_activities: Vec<OperationalActivity>,
    #[serde(default)]
    pub operational_exchanges: Vec<OperationalExchange>,
    #[serde(default)]
    pub operational_scenarios: Vec<OperationalScenario>,
    #[serde(default)]
    pub operational_nodes: Vec<OperationalNode>,
    #[serde(default)]
    pub file_manifest: Vec<FileDigestEntry>,
}

impl SchemaDigest {
    /// Compute schema digest from a PackageDef and its compiled SysML textual bytes.
    pub fn compute(pkg: &PackageDef, sysml_text: &str) -> Self {
        let sha256 = compute_sha256(sysml_text.as_bytes());
        let total_lines = sysml_text.lines().count();
        let node_counts = compute_node_counts(pkg);
        let schema_nodes = collect_schema_nodes(pkg);

        let operational_activities = extract_operational_activities(pkg);
        let operational_exchanges = extract_operational_exchanges(pkg);
        let operational_scenarios = extract_operational_scenarios(pkg);
        let operational_nodes = extract_operational_nodes(pkg);

        Self {
            sha256,
            total_lines,
            node_counts,
            schema_nodes,
            operational_activities,
            operational_exchanges,
            operational_scenarios,
            operational_nodes,
            file_manifest: Vec::new(),
        }
    }

    /// Builder method to attach a file manifest to the schema digest.
    pub fn with_file_manifest(mut self, manifest: Vec<FileDigestEntry>) -> Self {
        self.file_manifest = manifest;
        self
    }

    /// Atomically write digest JSON to disk using a temporary sibling file and rename.
    pub fn write_to_file(&self, path: &Path) -> Result<(), io::Error> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let json_text = serde_json::to_string_pretty(self)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        let tmp_path = path.with_extension("tmp");
        fs::write(&tmp_path, json_text)?;
        fs::rename(&tmp_path, path)?;

        Ok(())
    }
}

/// Compute schema digest from a PackageDef and its compiled SysML textual bytes.
pub fn generate_digest(pkg: &PackageDef, sysml_text: &str) -> SchemaDigest {
    SchemaDigest::compute(pkg, sysml_text)
}

/// Count structural AST definitions within a PackageDef.
pub fn count_structural_elements(pkg: &PackageDef) -> usize {
    let direct = pkg.part_defs.len()
        + pkg.attribute_defs.len()
        + pkg.port_defs.len()
        + pkg.action_defs.len()
        + pkg.operation_defs.len()
        + pkg.capability_defs.len()
        + pkg.interaction_defs.len()
        + pkg.constraint_defs.len()
        + pkg.test_case_defs.len()
        + pkg.requirement_defs.len()
        + pkg.connection_defs.len()
        + pkg.state_defs.len()
        + pkg.use_case_defs.len()
        + pkg.item_defs.len()
        + pkg.hazard_defs.len()
        + pkg.risk_defs.len();

    let nested: usize = pkg.packages.iter().map(count_structural_elements).sum();
    direct + nested
}

/// Compute a file manifest for a slice of SysML files.
///
/// For each file:
/// - `path`: relative path as string.
/// - `sha256`: SHA-256 hex string using `compute_sha256`.
/// - `bytes`: file size in bytes (`std::fs::metadata(p)?.len()`).
/// - `lines`: line count of file.
/// - `entities`: count of AST definitions parsed from that file (using `parse_sysml(&content)`).
pub fn compute_file_manifest(files: &[PathBuf]) -> Vec<FileDigestEntry> {
    let mut manifest = Vec::new();
    let cwd = std::env::current_dir().ok();

    for p in files {
        let rel_p = p.strip_prefix(".").unwrap_or(p);
        let rel_p = if let Some(ref cwd_path) = cwd {
            rel_p.strip_prefix(cwd_path).unwrap_or(rel_p)
        } else {
            rel_p
        };
        let path = rel_p.to_string_lossy().to_string();

        let bytes = fs::metadata(p).map(|m| m.len()).unwrap_or(0);

        let content = fs::read(p).unwrap_or_default();
        let sha256 = compute_sha256(&content);
        let content_str = String::from_utf8_lossy(&content);
        let lines = if content.is_empty() {
            0
        } else {
            content_str.lines().count()
        };

        let entities = crate::parse_sysml(&content_str)
            .map(|pkg| count_structural_elements(&pkg))
            .unwrap_or(0);

        manifest.push(FileDigestEntry {
            path,
            sha256,
            bytes,
            lines,
            entities,
        });
    }

    manifest
}

/// Atomically write digest JSON to disk using a temporary sibling file and rename.
pub fn write_digest_atomic(path: &Path, digest: &SchemaDigest) -> Result<(), io::Error> {
    digest.write_to_file(path)
}

/// Atomically write arbitrary text to disk using a temporary sibling file and rename.
pub fn write_atomic(path: &Path, content: &str) -> Result<(), io::Error> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let tmp_path = path.with_extension("tmp");
    fs::write(&tmp_path, content)?;
    fs::rename(&tmp_path, path)?;
    Ok(())
}


/// Compute SHA-256 hash formatted as lowercase hexadecimal (FIPS 180-4).
pub fn compute_sha256(bytes: &[u8]) -> String {
    const K: [u32; 64] = [
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
        0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
        0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
        0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
        0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
        0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
        0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
        0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
    ];

    let mut h0: u32 = 0x6a09e667;
    let mut h1: u32 = 0xbb67ae85;
    let mut h2: u32 = 0x3c6ef372;
    let mut h3: u32 = 0xa54ff53a;
    let mut h4: u32 = 0x510e527f;
    let mut h5: u32 = 0x9b05688c;
    let mut h6: u32 = 0x1f83d9ab;
    let mut h7: u32 = 0x5be0cd19;

    let bit_len = (bytes.len() as u64).wrapping_mul(8);
    let mut msg = bytes.to_vec();
    msg.push(0x80);
    while (msg.len() % 64) != 56 {
        msg.push(0x00);
    }
    msg.extend_from_slice(&bit_len.to_be_bytes());

    for chunk in msg.chunks_exact(64) {
        let mut w = [0u32; 64];
        for (i, part) in chunk.chunks_exact(4).enumerate() {
            w[i] = u32::from_be_bytes([part[0], part[1], part[2], part[3]]);
        }
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16].wrapping_add(s0).wrapping_add(w[i - 7]).wrapping_add(s1);
        }

        let mut a = h0;
        let mut b = h1;
        let mut c = h2;
        let mut d = h3;
        let mut e = h4;
        let mut f = h5;
        let mut g = h6;
        let mut h = h7;

        for i in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ ((!e) & g);
            let temp1 = h
                .wrapping_add(s1)
                .wrapping_add(ch)
                .wrapping_add(K[i])
                .wrapping_add(w[i]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let temp2 = s0.wrapping_add(maj);

            h = g;
            g = f;
            f = e;
            e = d.wrapping_add(temp1);
            d = c;
            c = b;
            b = a;
            a = temp1.wrapping_add(temp2);
        }

        h0 = h0.wrapping_add(a);
        h1 = h1.wrapping_add(b);
        h2 = h2.wrapping_add(c);
        h3 = h3.wrapping_add(d);
        h4 = h4.wrapping_add(e);
        h5 = h5.wrapping_add(f);
        h6 = h6.wrapping_add(g);
        h7 = h7.wrapping_add(h);
    }

    format!(
        "{:08x}{:08x}{:08x}{:08x}{:08x}{:08x}{:08x}{:08x}",
        h0, h1, h2, h3, h4, h5, h6, h7
    )
}

fn compute_node_counts(pkg: &PackageDef) -> BTreeMap<String, usize> {
    let mut counts = BTreeMap::new();
    counts.insert("packages".to_string(), 1);
    counts.insert("part_defs".to_string(), 0);
    counts.insert("attribute_defs".to_string(), 0);
    counts.insert("port_defs".to_string(), 0);
    counts.insert("action_defs".to_string(), 0);
    counts.insert("capability_defs".to_string(), 0);
    counts.insert("operation_defs".to_string(), 0);
    counts.insert("interaction_defs".to_string(), 0);
    counts.insert("constraint_defs".to_string(), 0);
    counts.insert("test_case_defs".to_string(), 0);
    counts.insert("requirement_defs".to_string(), 0);
    counts.insert("state_defs".to_string(), 0);
    counts.insert("use_case_defs".to_string(), 0);
    counts.insert("item_defs".to_string(), 0);
    counts.insert("hazard_defs".to_string(), 0);
    counts.insert("risk_defs".to_string(), 0);
    counts.insert("connection_defs".to_string(), 0);
    counts.insert("containers".to_string(), 0);
    counts.insert("lists".to_string(), 0);
    counts.insert("leaves".to_string(), 0);
    counts.insert("typedefs".to_string(), 0);
    counts.insert("identities".to_string(), 0);
    counts.insert("groupings".to_string(), 0);

    accumulate_package_counts(pkg, &mut counts);

    // Derived counts
    let containers = *counts.get("part_defs").unwrap_or(&0);
    let lists = *counts.get("action_defs").unwrap_or(&0);
    let leaves = *counts.get("attribute_defs").unwrap_or(&0);

    counts.insert("containers".to_string(), containers);
    counts.insert("lists".to_string(), lists);
    counts.insert("leaves".to_string(), leaves);

    counts
}

fn accumulate_package_counts(pkg: &PackageDef, counts: &mut BTreeMap<String, usize>) {
    *counts.get_mut("attribute_defs").unwrap() += pkg.attribute_defs.len();
    *counts.get_mut("port_defs").unwrap() += pkg.port_defs.len();
    *counts.get_mut("action_defs").unwrap() += pkg.action_defs.len();
    *counts.get_mut("capability_defs").unwrap() += pkg.capability_defs.len();
    *counts.get_mut("operation_defs").unwrap() += pkg.operation_defs.len();
    *counts.get_mut("interaction_defs").unwrap() += pkg.interaction_defs.len();
    *counts.get_mut("constraint_defs").unwrap() += pkg.constraint_defs.len();
    *counts.get_mut("test_case_defs").unwrap() += pkg.test_case_defs.len();
    *counts.get_mut("requirement_defs").unwrap() += pkg.requirement_defs.len();
    *counts.get_mut("state_defs").unwrap() += pkg.state_defs.len();
    *counts.get_mut("use_case_defs").unwrap() += pkg.use_case_defs.len();
    *counts.get_mut("item_defs").unwrap() += pkg.item_defs.len();
    *counts.get_mut("hazard_defs").unwrap() += pkg.hazard_defs.len();
    *counts.get_mut("risk_defs").unwrap() += pkg.risk_defs.len();
    *counts.get_mut("connection_defs").unwrap() += pkg.connection_defs.len();
    *counts.get_mut("part_defs").unwrap() += pkg.part_defs.len();

    for part in &pkg.part_defs {
        accumulate_part_counts(part, counts);
    }

    for subpkg in &pkg.packages {
        *counts.get_mut("packages").unwrap() += 1;
        accumulate_package_counts(subpkg, counts);
    }
}

fn accumulate_part_counts(part: &PartDef, counts: &mut BTreeMap<String, usize>) {
    *counts.get_mut("attribute_defs").unwrap() += part.attributes.len();
    *counts.get_mut("port_defs").unwrap() += part.ports.len();
    *counts.get_mut("action_defs").unwrap() += part.actions.len();
    *counts.get_mut("capability_defs").unwrap() += part.capabilities.len();
    *counts.get_mut("operation_defs").unwrap() += part.operations.len();
    *counts.get_mut("interaction_defs").unwrap() += part.interactions.len();
    *counts.get_mut("constraint_defs").unwrap() += part.constraints.len();
    *counts.get_mut("test_case_defs").unwrap() += part.test_cases.len();
    *counts.get_mut("requirement_defs").unwrap() += part.requirements.len();
    *counts.get_mut("state_defs").unwrap() += part.states.len();
    *counts.get_mut("use_case_defs").unwrap() += part.use_cases.len();
    *counts.get_mut("item_defs").unwrap() += part.item_defs.len();
    *counts.get_mut("hazard_defs").unwrap() += part.hazards.len();
    *counts.get_mut("risk_defs").unwrap() += part.risks.len();
    *counts.get_mut("connection_defs").unwrap() += part.connections.len();
    *counts.get_mut("part_defs").unwrap() += part.parts.len();

    for sub in &part.parts {
        accumulate_part_counts(sub, counts);
    }
}

fn collect_schema_nodes(pkg: &PackageDef) -> Vec<String> {
    let mut nodes = HashSet::new();
    nodes.insert(pkg.name.clone());

    for a in &pkg.attribute_defs {
        nodes.insert(a.name.clone());
    }
    for p in &pkg.port_defs {
        nodes.insert(p.name.clone());
    }
    for act in &pkg.action_defs {
        nodes.insert(act.name.clone());
    }
    for c in &pkg.constraint_defs {
        nodes.insert(c.name.clone());
    }
    for r in &pkg.requirement_defs {
        nodes.insert(r.name.clone());
    }
    for s in &pkg.state_defs {
        nodes.insert(s.name.clone());
    }
    for conn in &pkg.connection_defs {
        nodes.insert(conn.name.clone());
    }
    for uc in &pkg.use_case_defs {
        nodes.insert(uc.name.clone());
    }
    for item in &pkg.item_defs {
        nodes.insert(item.name.clone());
    }
    for hz in &pkg.hazard_defs {
        nodes.insert(hz.name.clone());
    }
    for rk in &pkg.risk_defs {
        nodes.insert(rk.name.clone());
    }

    for part in &pkg.part_defs {
        collect_part_nodes(part, &mut nodes);
    }

    for sub in &pkg.packages {
        let sub_nodes = collect_schema_nodes(sub);
        nodes.extend(sub_nodes);
    }

    let mut result: Vec<String> = nodes.into_iter().collect();
    result.sort();
    result
}

fn collect_part_nodes(part: &PartDef, nodes: &mut HashSet<String>) {
    nodes.insert(part.name.clone());
    for a in &part.attributes {
        nodes.insert(a.name.clone());
    }
    for p in &part.ports {
        nodes.insert(p.name.clone());
    }
    for act in &part.actions {
        nodes.insert(act.name.clone());
    }
    for c in &part.constraints {
        nodes.insert(c.name.clone());
    }
    for r in &part.requirements {
        nodes.insert(r.name.clone());
    }
    for s in &part.states {
        nodes.insert(s.name.clone());
    }
    for conn in &part.connections {
        nodes.insert(conn.name.clone());
    }
    for item in &part.item_defs {
        nodes.insert(item.name.clone());
    }
    for sub in &part.parts {
        collect_part_nodes(sub, nodes);
    }
}

fn extract_operational_activities(pkg: &PackageDef) -> Vec<OperationalActivity> {
    let mut activities = Vec::new();
    let mut idx = 1;

    for act in &pkg.action_defs {
        activities.push(OperationalActivity {
            id: format!("OA-{:02}", idx),
            name: act.name.clone(),
            doc: act.doc.clone().unwrap_or_default(),
            performer: act.performer.clone().unwrap_or_default(),
            inputs: act.in_params.iter().map(|p| p.name.clone()).collect(),
            outputs: act.out_params.iter().map(|p| p.name.clone()).collect(),
        });
        idx += 1;
    }

    for part in &pkg.part_defs {
        for act in &part.actions {
            activities.push(OperationalActivity {
                id: format!("OA-{:02}", idx),
                name: act.name.clone(),
                doc: act.doc.clone().unwrap_or_default(),
                performer: act.performer.clone().unwrap_or_else(|| part.name.clone()),
                inputs: act.in_params.iter().map(|p| p.name.clone()).collect(),
                outputs: act.out_params.iter().map(|p| p.name.clone()).collect(),
            });
            idx += 1;
        }
    }

    activities
}

fn extract_operational_exchanges(pkg: &PackageDef) -> Vec<OperationalExchange> {
    let mut exchanges = Vec::new();
    let mut idx = 1;

    for conn in &pkg.connection_defs {
        exchanges.push(OperationalExchange {
            id: format!("OpTx-{:02}", idx),
            name: conn.name.clone(),
            source_port: conn.source_port.clone(),
            target_port: conn.target_port.clone(),
            source_part: conn.source_part.clone().unwrap_or_default(),
            target_part: conn.target_part.clone().unwrap_or_default(),
            protocol: conn.protocol.clone().unwrap_or_default(),
        });
        idx += 1;
    }

    for part in &pkg.part_defs {
        for conn in &part.connections {
            exchanges.push(OperationalExchange {
                id: format!("OpTx-{:02}", idx),
                name: conn.name.clone(),
                source_port: conn.source_port.clone(),
                target_port: conn.target_port.clone(),
                source_part: conn.source_part.clone().unwrap_or_else(|| part.name.clone()),
                target_part: conn.target_part.clone().unwrap_or_default(),
                protocol: conn.protocol.clone().unwrap_or_default(),
            });
            idx += 1;
        }
    }

    exchanges
}

fn extract_operational_scenarios(pkg: &PackageDef) -> Vec<OperationalScenario> {
    let mut scenarios = Vec::new();
    let mut idx = 1;

    for uc in &pkg.use_case_defs {
        scenarios.push(OperationalScenario {
            id: format!("SCN-{:02}", idx),
            name: uc.name.clone(),
            actor: uc.actor.clone().unwrap_or_default(),
            objective: uc.objective.clone().unwrap_or_default(),
            steps: uc.steps.clone(),
        });
        idx += 1;
    }

    scenarios
}

fn extract_operational_nodes(pkg: &PackageDef) -> Vec<OperationalNode> {
    let mut nodes = Vec::new();

    for part in &pkg.part_defs {
        nodes.push(OperationalNode {
            name: part.name.clone(),
            node_type: "Part".to_string(),
            doc: part.doc.clone().unwrap_or_default(),
            allocated_activities: part.actions.iter().map(|a| a.name.clone()).collect(),
        });
    }

    nodes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_digest_counts_and_hash() {
        let pkg = PackageDef {
            name: "DigestTest".to_string(),
            attribute_defs: vec![
                AttributeDef {
                    name: "param1".to_string(),
                    type_name: "Real".to_string(),
                    default_value: Some("1.0".to_string()),
                    doc: None,
                },
            ],
            part_defs: vec![
                PartDef {
                    name: "ComponentA".to_string(),
                    ports: vec![
                        PortDef {
                            name: "p1".to_string(),
                            direction: "out".to_string(),
                            type_name: "Data".to_string(),
                            ..Default::default()
                        },
                    ],
                    ..Default::default()
                },
            ],
            ..Default::default()
        };

        let sysml_source = "package DigestTest {\n    attribute param1 : Real = 1.0;\n}\n";
        let digest = SchemaDigest::compute(&pkg, sysml_source);

        assert_eq!(digest.total_lines, 3);
        assert!(!digest.sha256.is_empty());
        assert_eq!(*digest.node_counts.get("packages").unwrap(), 1);
        assert_eq!(*digest.node_counts.get("part_defs").unwrap(), 1);
        assert_eq!(*digest.node_counts.get("port_defs").unwrap(), 1);
        assert_eq!(*digest.node_counts.get("attribute_defs").unwrap(), 1);
        assert!(digest.schema_nodes.contains(&"DigestTest".to_string()));
        assert!(digest.schema_nodes.contains(&"ComponentA".to_string()));
        assert!(digest.schema_nodes.contains(&"p1".to_string()));
    }

    #[test]
    fn test_schema_digest_multi_file_manifest() {
        let temp_dir = std::env::temp_dir().join(format!(
            "compile_sysml_manifest_test_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&temp_dir).unwrap();

        let file1 = temp_dir.join("subsystem_a.sysml");
        let file2 = temp_dir.join("subsystem_b.sysml");

        let content1 = "package SubsystemA {\n    part def ComponentA;\n    attribute mass : Real = 10.0;\n}\n";
        let content2 = "package SubsystemB {\n    part def ComponentB;\n}\n";

        fs::write(&file1, content1).unwrap();
        fs::write(&file2, content2).unwrap();

        let manifest = compute_file_manifest(&[file1.clone(), file2.clone()]);
        assert_eq!(manifest.len(), 2);

        // Entry 1 verification
        assert_eq!(manifest[0].path, file1.to_string_lossy().to_string());
        assert_eq!(manifest[0].sha256, compute_sha256(content1.as_bytes()));
        assert_eq!(manifest[0].bytes, content1.len() as u64);
        assert_eq!(manifest[0].lines, content1.lines().count());
        assert_eq!(manifest[0].entities, 2); // ComponentA and mass

        // Entry 2 verification
        assert_eq!(manifest[1].path, file2.to_string_lossy().to_string());
        assert_eq!(manifest[1].sha256, compute_sha256(content2.as_bytes()));
        assert_eq!(manifest[1].bytes, content2.len() as u64);
        assert_eq!(manifest[1].lines, content2.lines().count());
        assert_eq!(manifest[1].entities, 1); // ComponentB

        // Verify SchemaDigest integration and serialization round-trip
        let digest = SchemaDigest::default().with_file_manifest(manifest.clone());
        assert_eq!(digest.file_manifest, manifest);

        let json = serde_json::to_string_pretty(&digest).expect("Failed to serialize SchemaDigest");
        let deserialized: SchemaDigest =
            serde_json::from_str(&json).expect("Failed to deserialize SchemaDigest");
        assert_eq!(deserialized.file_manifest, manifest);

        let _ = fs::remove_dir_all(&temp_dir);
    }
}
