#![deny(clippy::too_many_lines, clippy::too_many_arguments, clippy::undocumented_unsafe_blocks)]
//! Unit tests for `SchemaDigest`, file manifests, NIST FIPS 180-4 SHA-256 vectors, and bounded stack traversal.
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C verification objectives.
//! ## 2. Invariants & Mathematical Properties
//! Verifies digest counts, relative manifest paths, PID-scoped atomic writes, and 4,000-level deep tree traversal.
//! ## 3. Preconditions
//! Host temporary directory is writable for file manifest and atomic persistence tests.
//! ## 4. Postconditions
//! Cleans up all temporary test directories and asserts exact SHA-256 and AST count invariants.
//! ## 5. Algorithmic Complexity Bounds
//! Time Complexity: O(N) over synthetic AST fixtures. Space Complexity: O(N) heap work-stack.
//! ## 6. Failure Modes & Fault Tolerance
//! - Interface / State / Action / Resource: Zero `.unwrap()` or `.expect()` calls across all test functions.
//! ## 7. Realization & Traceability
//! Realises: [REQ-0603, REQ-SEMANTIC-DIGEST-TEST-COMPUTE-DIGEST, REQ-SEMANTIC-DIGEST-TEST-MULTI-FILE-MANIFEST]

use super::digest::*;
use super::digest_counts::{collect_schema_nodes, compute_node_counts};
use deap_core::sysml_ast::{AttributeDef, PackageDef, PartDef, PortDef, RequirementDef};
use std::fs;
use std::path::Path;

#[test]
fn test_compute_digest_counts_and_hash() -> Result<(), Box<dyn std::error::Error>> {
    let pkg = PackageDef {
        name: "DigestTest".to_string(),
        attribute_defs: vec![AttributeDef {
            name: "param1".to_string(),
            type_name: "Real".to_string(),
            default_value: Some("1.0".to_string()),
            doc: None,
        }],
        part_defs: vec![PartDef {
            name: "ComponentA".to_string(),
            ports: vec![PortDef {
                name: "p1".to_string(),
                direction: "out".to_string(),
                type_name: "Data".to_string(),
                ..Default::default()
            }],
            ..Default::default()
        }],
        ..Default::default()
    };

    let sysml_source = "package DigestTest {\n    attribute param1 : Real = 1.0;\n}\n";
    let digest = SchemaDigest::compute(&pkg, sysml_source);

    assert_eq!(digest.total_lines, 3);
    assert!(!digest.sha256.is_empty());
    assert_eq!(*digest.node_counts.get("packages").ok_or("missing packages")?, 1);
    assert_eq!(*digest.node_counts.get("part_defs").ok_or("missing part_defs")?, 1);
    assert_eq!(*digest.node_counts.get("port_defs").ok_or("missing port_defs")?, 1);
    assert_eq!(*digest.node_counts.get("attribute_defs").ok_or("missing attribute_defs")?, 1);
    assert!(digest.schema_nodes.contains(&"DigestTest".to_string()));
    assert!(digest.schema_nodes.contains(&"ComponentA".to_string()));
    assert!(digest.schema_nodes.contains(&"p1".to_string()));
    Ok(())
}

fn assert_manifest_entry(
    entry: &FileDigestEntry,
    expected_path: &str,
    content: &str,
    expected_entities: usize,
) {
    assert_eq!(entry.path, expected_path);
    assert_eq!(entry.sha256, compute_sha256(content.as_bytes()));
    assert_eq!(entry.bytes, content.len() as u64);
    assert_eq!(entry.lines, content.lines().count());
    assert_eq!(entry.entities, expected_entities);
}

#[test]
fn test_schema_digest_multi_file_manifest() -> Result<(), Box<dyn std::error::Error>> {
    let nanos = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)?.as_nanos();
    let temp_dir = std::env::temp_dir().join(format!("compile_sysml_manifest_test_{}_{nanos}", std::process::id()));
    fs::create_dir_all(&temp_dir)?;

    let file1 = temp_dir.join("subsystem_a.sysml");
    let file2 = temp_dir.join("subsystem_b.sysml");
    let content1 = "package SubsystemA {\n    part def ComponentA;\n    attribute mass : Real = 10.0;\n}\n";
    let content2 = "package SubsystemB {\n    part def ComponentB;\n}\n";

    fs::write(&file1, content1)?;
    fs::write(&file2, content2)?;

    let manifest = compute_file_manifest(&[file1, file2]);
    assert_eq!(manifest.len(), 2);
    assert_manifest_entry(manifest.first().ok_or("missing 0")?, "subsystem_a.sysml", content1, 2);
    assert_manifest_entry(manifest.get(1).ok_or("missing 1")?, "subsystem_b.sysml", content2, 1);

    let digest = SchemaDigest::default().with_file_manifest(manifest.clone());
    assert_eq!(digest.file_manifest, manifest);
    let json = serde_json::to_string_pretty(&digest)?;
    let deserialized: SchemaDigest = serde_json::from_str(&json)?;
    assert_eq!(deserialized.file_manifest, manifest);

    fs::remove_dir_all(&temp_dir)?;
    Ok(())
}

#[test]
fn test_defect_03_compile_sysml_sha256_nist_vectors_and_deap_core_delegation() {
    assert_eq!(compute_sha256(b""), "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855");
    assert_eq!(compute_sha256(b"abc"), "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad");
    assert_eq!(
        compute_sha256(b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq"),
        "248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1"
    );
    assert_eq!(
        compute_sha256(b"abcdefghbcdefghicdefghijdefghijkefghijklfghijklmghijklmnhijklmnoijklmnopjklmnopqklmnopqrlmnopqrsmnopqrstnopqrstu"),
        "cf5b16a778af8380036ce59e7b0492370b249b11e8f07a51afac45037afee9d1"
    );

    let src = include_str!("digest.rs");
    let k0_token = concat!("0x428a", "2f98");
    assert!(!src.contains(k0_token), "digest.rs must delegate SHA-256 to deap_core::digest");
}

#[test]
fn test_arch_06_compute_file_manifest_deterministic_relative_paths() -> Result<(), Box<dyn std::error::Error>> {
    let pid = std::process::id();
    let dir_a = std::env::temp_dir().join(format!("deap_arch06_manifest_a_{pid}"));
    let dir_b = std::env::temp_dir().join(format!("deap_arch06_manifest_b_{pid}"));
    let _ = fs::remove_dir_all(&dir_a);
    let _ = fs::remove_dir_all(&dir_b);

    let (schema_a, schema_b) = (dir_a.join("schema"), dir_b.join("schema"));
    fs::create_dir_all(&schema_a)?;
    fs::create_dir_all(&schema_b)?;

    let content = "package DeterministicPkg {\n    part def UnitA;\n}\n";
    let (file_a, file_b) = (schema_a.join("unit_a.sysml"), schema_b.join("unit_a.sysml"));
    fs::write(&file_a, content)?;
    fs::write(&file_b, content)?;

    let manifest_a = compute_file_manifest(&[file_a]);
    let manifest_b = compute_file_manifest(&[file_b]);
    let _ = fs::remove_dir_all(&dir_a);
    let _ = fs::remove_dir_all(&dir_b);

    assert_eq!(manifest_a, manifest_b);
    let entry = manifest_a.first().ok_or("missing manifest entry")?;
    assert!(!Path::new(&entry.path).is_absolute());
    assert_eq!(entry.path, "schema/unit_a.sysml");
    Ok(())
}

#[test]
fn test_arch_04_write_atomic_pid_scoped_temp_and_cleanup() -> Result<(), Box<dyn std::error::Error>> {
    let pid = std::process::id();
    let tmp_dir = std::env::temp_dir().join(format!("deap_arch04_write_atomic_{pid}"));
    let _ = fs::remove_dir_all(&tmp_dir);
    fs::create_dir_all(&tmp_dir)?;

    let unrelated_tmp = tmp_dir.join("model.tmp");
    fs::write(&unrelated_tmp, "preserve-me")?;
    let target_file = tmp_dir.join("model.sysml");
    write_atomic(&target_file, "package AtomicPkg {}\n")?;
    assert_eq!(fs::read_to_string(&target_file)?, "package AtomicPkg {}\n");
    assert_eq!(fs::read_to_string(&unrelated_tmp)?, "preserve-me");

    let dir_target = tmp_dir.join("dir_target.sysml");
    fs::create_dir_all(dir_target.join("nested"))?;
    assert!(write_atomic(&dir_target, "should fail rename").is_err());
    assert!(!tmp_dir.join(format!(".dir_target.sysml.tmp.{pid}")).exists());

    let digest_unrelated_tmp = tmp_dir.join("schema-digest.tmp");
    fs::write(&digest_unrelated_tmp, "preserve-digest-tmp")?;
    let digest_target = tmp_dir.join("schema-digest.json");
    SchemaDigest::default().write_to_file(&digest_target)?;
    assert!(digest_target.is_file());
    assert_eq!(fs::read_to_string(&digest_unrelated_tmp)?, "preserve-digest-tmp");

    let _ = fs::remove_dir_all(&tmp_dir);
    Ok(())
}

fn build_multi_branch_digest_fixture() -> PackageDef {
    let leaf_part = PartDef {
        name: "PartA1a".to_string(),
        attributes: vec![AttributeDef { name: "leaf_attr".to_string(), type_name: "Real".to_string(), default_value: None, doc: None }],
        ..Default::default()
    };
    let sub_part_a1 = PartDef {
        name: "PartA1".to_string(),
        parts: vec![leaf_part],
        ports: vec![PortDef { name: "port_a1".to_string(), direction: "in".to_string(), type_name: "Signal".to_string(), ..Default::default() }],
        ..Default::default()
    };
    let root_part_a = PartDef {
        name: "PartA".to_string(),
        parts: vec![sub_part_a1, PartDef { name: "PartA2".to_string(), ..Default::default() }],
        ..Default::default()
    };
    let sub_pkg_a = PackageDef {
        name: "SubPkgA".to_string(),
        part_defs: vec![root_part_a],
        packages: vec![PackageDef {
            name: "SubPkgA1".to_string(),
            requirement_defs: vec![RequirementDef { name: "ReqDeep".to_string(), req_id: "REQ-DEEP-01".to_string(), ..Default::default() }],
            ..Default::default()
        }],
        ..Default::default()
    };
    let sub_pkg_b = PackageDef {
        name: "SubPkgB".to_string(),
        attribute_defs: vec![AttributeDef { name: "pkg_b_attr".to_string(), type_name: "Integer".to_string(), default_value: None, doc: None }],
        ..Default::default()
    };
    PackageDef { name: "RootPkg".to_string(), packages: vec![sub_pkg_a, sub_pkg_b], ..Default::default() }
}

fn verify_deep_digest_at_depth(depth: usize) -> Result<(), String> {
    let mut deep_part = PartDef {
        name: format!("DeepPart_{}", depth.saturating_sub(1)),
        attributes: vec![AttributeDef { name: "terminal_attr".to_string(), type_name: "Real".to_string(), default_value: None, doc: None }],
        ..Default::default()
    };
    for idx in (0..depth.saturating_sub(1)).rev() {
        deep_part = PartDef { name: format!("DeepPart_{idx}"), parts: vec![deep_part], ..Default::default() };
    }
    let mut deep_pkg = PackageDef { name: format!("DeepPkg_{}", depth.saturating_sub(1)), part_defs: vec![deep_part], ..Default::default() };
    for idx in (0..depth.saturating_sub(1)).rev() {
        deep_pkg = PackageDef { name: format!("DeepPkg_{idx}"), packages: vec![deep_pkg], ..Default::default() };
    }

    let (elem_count, pkg_count, part_count, node_len) =
        std::thread::scope(|s| -> Result<(usize, usize, usize, usize), String> {
            let handle = std::thread::Builder::new()
                .name(format!("bounded_digest_{depth}"))
                .stack_size(256 * 1024)
                .spawn_scoped(s, || {
                    let elems = count_structural_elements(&deep_pkg);
                    let counts = compute_node_counts(&deep_pkg);
                    let nodes = collect_schema_nodes(&deep_pkg);
                    (elems, counts.get("packages").copied().unwrap_or(0), counts.get("part_defs").copied().unwrap_or(0), nodes.len())
                })
                .map_err(|e| format!("failed to spawn scoped thread: {e}"))?;
            handle.join().map_err(|_| "bounded-stack digest thread panicked".to_string())
        })?;

    assert_eq!(elem_count, depth.saturating_add(1));
    assert_eq!(pkg_count, depth);
    assert_eq!(part_count, depth);
    assert_eq!(node_len, depth.saturating_mul(2).saturating_add(1));

    let (mut extracted_parts, mut pkg_drain) = (Vec::new(), vec![deep_pkg]);
    while let Some(mut p) = pkg_drain.pop() {
        extracted_parts.append(&mut p.part_defs);
        pkg_drain.append(&mut p.packages);
    }
    while let Some(mut pt) = extracted_parts.pop() {
        extracted_parts.append(&mut pt.parts);
    }
    Ok(())
}

#[test]
fn test_digest_iterative_stacks_parity_and_bounded_stack_deep_nesting() -> Result<(), String> {
    let root_pkg = build_multi_branch_digest_fixture();
    assert_eq!(count_structural_elements(&root_pkg), 8);
    let digest = SchemaDigest::compute(&root_pkg, "package RootPkg {}\n");
    assert_eq!(digest.node_counts.get("packages").copied(), Some(4));
    assert_eq!(digest.node_counts.get("part_defs").copied(), Some(4));
    assert_eq!(digest.node_counts.get("port_defs").copied(), Some(1));
    assert_eq!(digest.node_counts.get("attribute_defs").copied(), Some(2));
    assert_eq!(digest.node_counts.get("requirement_defs").copied(), Some(1));
    let expected: Vec<String> = [
        "PartA", "PartA1", "PartA1a", "PartA2", "ReqDeep", "RootPkg",
        "SubPkgA", "SubPkgA1", "SubPkgB", "leaf_attr", "pkg_b_attr", "port_a1",
    ]
    .iter()
    .map(|s| (*s).to_string())
    .collect();
    assert_eq!(digest.schema_nodes, expected);
    assert_eq!(digest.sha256, compute_sha256(b"package RootPkg {}\n"));

    for depth in [64usize, 65, 4_000] {
        verify_deep_digest_at_depth(depth)?;
    }
    Ok(())
}
