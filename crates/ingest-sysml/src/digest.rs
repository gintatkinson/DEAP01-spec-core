//! # Pre-Ingestion Input Digest Generator & Truncation Verification Gate
//!
//! Pre-ingestion cryptographic fingerprinting (FIPS 180-4 SHA-256) and truncation
//! detection mechanisms adhering to DO-178C verification standards.
//!
//! Realises: [REQ-0001]

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Canonical truncation and summarization tokens strictly prohibited in input ingestion streams. Realises: [REQ-0001/TRUNCATION_TOKENS]
pub const TRUNCATION_TOKENS: &[&str] = &[
    "... [truncated]",
    "<truncated",
    "[truncated]",
    "summarized",
    "truncated content",
];

/// Scans source text content for forbidden truncation or summarization tokens. Realises: [REQ-0001/scan_truncation]
pub fn scan_truncation(content: &str) -> Vec<String> {
    let mut detected = Vec::new();
    for token in TRUNCATION_TOKENS {
        if content.contains(token) {
            detected.push((*token).to_string());
        }
    }
    detected
}

/// Computes a FIPS 180-4 compliant SHA-256 hexadecimal hash from an arbitrary byte slice. Realises: [REQ-0001/compute_sha256]
pub fn compute_sha256(data: &[u8]) -> String {
    deap_core::digest::compute_sha256(data)
}

/// Internal software implementation of FIPS 180-4 SHA-256 compression function. Realises: [REQ-INGEST-SYSML-DIGEST-COMPUTE-SHA256-BYTES]
pub fn compute_sha256_bytes(data: &[u8]) -> [u8; 32] {
    deap_core::digest::compute_sha256_bytes(data)
}

/// Structural descriptor for an individual file registered in `input-digest.json`. Realises: [REQ-INGEST-SYSML-DIGEST-DIGESTFILEENTRY]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DigestFileEntry {
    pub sha256: String,
    pub total_lines: usize,
    pub line_range: (usize, usize),
    pub line_range_bounds: (usize, usize),
    pub structural_section_markers: Vec<String>,
}

/// Consolidated input digest model written to `.pipeline/input-digest.json`. Realises: [REQ-INGEST-SYSML-DIGEST-INPUTDIGESTDATA]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct InputDigestData {
    pub sha256: String,
    pub total_lines: usize,
    pub files: BTreeMap<String, DigestFileEntry>,
    pub structural_section_markers: Vec<String>,
}

/// Parses an input specification file extracting line count, sha256, and structural section markers. Realises: [REQ-0001/parse_input_file]
pub fn parse_input_file(file_path: &Path) -> Result<(DigestFileEntry, Vec<u8>), String> {
    let content_bytes = fs::read(file_path)
        .map_err(|e| format!("Failed to read file '{}': {}", file_path.display(), e))?;
    let sha256 = compute_sha256(&content_bytes);
    let text = String::from_utf8_lossy(&content_bytes);
    let lines: Vec<&str> = text.lines().collect();
    let total_lines = lines.len();
    let mut structural_section_markers = Vec::new();
    const YANG_KEYWORDS: &[&str] = &["container", "list", "typedef", "identity", "grouping", "module"];

    for line in &lines {
        let trimmed = line.trim_start();
        let hash_count = trimmed.chars().take_while(|&c| c == '#').count();
        if (1..=6).contains(&hash_count) && trimmed[hash_count..].starts_with(char::is_whitespace) {
            let marker = trimmed.trim().to_string();
            if !structural_section_markers.contains(&marker) {
                structural_section_markers.push(marker);
            }
        }

        for &kw in YANG_KEYWORDS {
            if let Some(rest) = trimmed.strip_prefix(kw) {
                if rest.starts_with(char::is_whitespace) {
                    let id_rest = rest.trim_start();
                    let id_len = id_rest
                        .chars()
                        .take_while(|c| c.is_ascii_alphanumeric() || *c == '_' || *c == '-')
                        .count();
                    if id_len > 0 {
                        let marker = format!("{} {}", kw, &id_rest[..id_len]);
                        if !structural_section_markers.contains(&marker) {
                            structural_section_markers.push(marker);
                        }
                        break;
                    }
                }
            }
        }
    }

    let bounds = if total_lines > 0 { (1, total_lines) } else { (0, 0) };
    Ok((
        DigestFileEntry {
            sha256,
            total_lines,
            line_range: bounds,
            line_range_bounds: bounds,
            structural_section_markers,
        },
        content_bytes,
    ))
}

/// Generates an input digest for a single file or a directory of specification files. Realises: [REQ-0001/generate_input_digest]
pub fn generate_input_digest(input_path: &Path) -> Result<InputDigestData, String> {
    let mut input_files = Vec::new();
    if !input_path.exists() {
        return Err(format!("Input path does not exist: '{}'", input_path.display()));
    }

    if input_path.is_file() {
        input_files.push(input_path.to_path_buf());
    } else if input_path.is_dir() {
        for entry_res in walkdir::WalkDir::new(input_path).sort_by_file_name() {
            let e = entry_res.map_err(|err| {
                format!("Directory traversal error while computing input digest for '{}': {}", input_path.display(), err)
            })?;
            let p = e.path();
            if p.is_file() && !p.file_name().and_then(|s| s.to_str()).unwrap_or("").starts_with('.') {
                if let Some(ext) = p.extension().and_then(|s| s.to_str()) {
                    if ["md", "yang", "sysml", "proto", "json", "yaml", "yml", "txt"]
                        .contains(&ext.to_ascii_lowercase().as_str())
                    {
                        input_files.push(p.to_path_buf());
                    }
                }
            }
        }
    } else {
        return Err(format!("Input path is neither a regular file nor a directory: '{}'", input_path.display()));
    }

    if input_files.is_empty() {
        return Ok(InputDigestData {
            sha256: "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855".to_string(),
            total_lines: 0,
            files: BTreeMap::new(),
            structural_section_markers: Vec::new(),
        });
    }

    let (mut combined_bytes, mut total_lines, mut file_map, mut all_markers) =
        (Vec::new(), 0, BTreeMap::new(), Vec::new());

    for fpath in input_files {
        let (entry, bytes) = parse_input_file(&fpath)?;
        combined_bytes.extend_from_slice(&bytes);
        total_lines += entry.total_lines;
        for m in &entry.structural_section_markers {
            if !all_markers.contains(m) {
                all_markers.push(m.clone());
            }
        }
        file_map.insert(fpath.to_string_lossy().to_string(), entry);
    }

    Ok(InputDigestData {
        sha256: compute_sha256(&combined_bytes),
        total_lines,
        files: file_map,
        structural_section_markers: all_markers,
    })
}

/// Computes multi-file combined sha256 hash and total lines count. Realises: [REQ-0001/compute_multi_file_sha256]
pub fn compute_multi_file_sha256(paths: &[PathBuf]) -> Result<(String, usize), String> {
    let (mut combined_bytes, mut total_lines) = (Vec::new(), 0);
    for p in paths {
        let b = fs::read(p).map_err(|e| format!("Failed to read {}: {}", p.display(), e))?;
        total_lines += String::from_utf8_lossy(&b).lines().count();
        combined_bytes.extend_from_slice(&b);
    }
    Ok((compute_sha256(&combined_bytes), total_lines))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_sha256() {
        assert_eq!(compute_sha256(b""), "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855");
        assert_eq!(compute_sha256(b"hello world"), "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9");
    }

    #[test]
    fn test_defect_03_sha256_nist_vectors_and_zero_unwrap() {
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
        assert_eq!(compute_sha256_bytes(b"abc"), deap_core::digest::compute_sha256_bytes(b"abc"));

        let src = include_str!("digest.rs");
        let prod_src = src.split("#[cfg(test)]").next().unwrap_or(src);
        let unwrap_token = concat!(".un", "wrap(");
        let expect_token = concat!(".ex", "pect(");
        let k0_token = concat!("0x428a", "2f98");
        let h0_token = concat!("0x6a09", "e667");
        let flatten_token = concat!(".flat", "ten()");
        assert!(!src.contains(unwrap_token), "ingest-sysml/src/digest.rs must contain zero unwrap calls");
        assert!(!src.contains(expect_token), "ingest-sysml/src/digest.rs must contain zero expect calls");
        assert!(!prod_src.contains(k0_token) && !prod_src.contains(h0_token));
        assert!(!prod_src.contains(flatten_token));
    }

    #[test]
    fn test_scan_truncation() {
        let clean = "# Package_0\n## Classifier_Alpha\nPure clean specification content with normal ellipsis... and 1..10 range.";
        assert!(scan_truncation(clean).is_empty());
        let detected = scan_truncation("# Package_0\n... [truncated] truncated content");
        assert!(detected.contains(&"... [truncated]".to_string()) && detected.contains(&"truncated content".to_string()));
    }

    #[test]
    fn test_parse_input_file_req1() -> Result<(), String> {
        let tmp_dir = std::env::temp_dir().join(format!("deap_ingest_sysml_req1_test_{}", std::process::id()));
        if tmp_dir.exists() {
            fs::remove_dir_all(&tmp_dir).map_err(|e| e.to_string())?;
        }
        fs::create_dir_all(&tmp_dir).map_err(|e| e.to_string())?;

        let req_file = tmp_dir.join("REQ-0001.md");
        let content_1 = "# 1. Normative Statement\n## 1.1 Scope\nSystem shall verify digest.\n";
        fs::write(&req_file, content_1).map_err(|e| e.to_string())?;

        let second_file = tmp_dir.join("REQ-0002.md");
        let content_2 = "# 2. Secondary Requirement\nSystem shall validate multi-file hash.\n";
        fs::write(&second_file, content_2).map_err(|e| e.to_string())?;

        let (entry, raw_bytes) = parse_input_file(&req_file)?;
        assert_eq!(entry.total_lines, 3);
        assert_eq!(entry.sha256, compute_sha256(content_1.as_bytes()));
        assert_eq!(raw_bytes, content_1.as_bytes());
        assert!(entry.structural_section_markers.iter().any(|m| m.contains("1. Normative Statement")));

        let (combined_hash, combined_lines) = compute_multi_file_sha256(&[req_file, second_file])?;
        let mut expected_combined = Vec::from(content_1.as_bytes());
        expected_combined.extend_from_slice(content_2.as_bytes());
        assert_eq!(combined_hash, compute_sha256(&expected_combined));
        assert_eq!(combined_lines, 5);

        let missing_path = tmp_dir.join("DOES_NOT_EXIST.md");
        assert!(compute_multi_file_sha256(std::slice::from_ref(&missing_path)).is_err());
        let missing_digest_res = generate_input_digest(&missing_path);
        fs::remove_dir_all(&tmp_dir).map_err(|e| e.to_string())?;
        assert!(missing_digest_res.is_err());
        Ok(())
    }

    #[cfg(unix)]
    #[test]
    fn test_issue_494_generate_input_digest_fails_closed_on_unreadable_subdir() -> Result<(), Box<dyn std::error::Error>> {
        use std::os::unix::fs::PermissionsExt;

        let tmp_root = std::env::temp_dir().join(format!("deap_issue_494_walkdir_fail_closed_{}", std::process::id()));
        let _ = fs::remove_dir_all(&tmp_root);
        fs::create_dir_all(&tmp_root)?;
        fs::write(tmp_root.join("visible.md"), "# Section 1\nBody\n")?;

        let locked_dir = tmp_root.join("locked_subdir");
        fs::create_dir_all(&locked_dir)?;
        fs::write(locked_dir.join("hidden.md"), "# Section 2\nHidden\n")?;
        fs::set_permissions(&locked_dir, fs::Permissions::from_mode(0o000))?;

        let result = generate_input_digest(&tmp_root);
        let _ = fs::set_permissions(&locked_dir, fs::Permissions::from_mode(0o755));
        let _ = fs::remove_dir_all(&tmp_root);

        match result {
            Err(err_msg) => {
                assert!(err_msg.contains("Directory traversal error"));
                Ok(())
            }
            Ok(digest) => Err(format!("Expected Err, got sha256={}", digest.sha256).into()),
        }
    }
}
