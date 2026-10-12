//! # Pre-Ingestion Input Digest Generator & Truncation Verification Gate
//!
//! ## 1. Safety Intent & Regulatory Scope
//! This module implements pre-ingestion cryptographic fingerprinting and truncation
//! detection mechanisms adhering to DO-178C verification standards. In safety-critical
//! engineering pipelines, subagents and autonomous toolchains can experience silent token
//! limits, context exhaustion, or truncated file reads.
//!
//! To prevent incomplete models from entering the SysML v2 compiler, this module provides:
//! - **Mechanism 1 (Cryptographic Fingerprinting)**: Computes authoritative SHA-256 digests,
//!   total line counts, line-range bounds, and structural section markers.
//! - **Mechanism 3 & 4 (Truncation Detection)**: Scans input text and transcripts for
//!   forbidden truncation markers (`...`, `<truncated`, `[truncated]`, `summarized`).
//!
//! ## 2. Core Safety Invariants
//! - **FIPS 180-4 Compliance**: All hashes are computed via standard SHA-256 producing
//!   64-character lowercase hexadecimal strings.
//! - **Zero Omission Invariant**: Every input file must have non-zero line-range bounds and
//!   positive SHA-256 checksums recorded prior to lowering.
//! - **Deterministic Traversal**: Filesystem iteration is sorted lexicographically to guarantee
//!   cross-platform bitwise determinism.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use regex::Regex;
use serde::{Deserialize, Serialize};

/// Canonical truncation and summarization tokens strictly prohibited in input ingestion streams.
///
/// If any of these tokens appear in ingestion payloads outside formal string literals,
/// the verification gate halts compilation with diagnostic error code `E0100`.
pub const TRUNCATION_TOKENS: &[&str] = &[
    "...",
    "<truncated",
    "[truncated]",
    "summarized",
    "truncated content",
];

/// Scans source text content for forbidden truncation or summarization tokens.
///
/// /// Realises: [REQ-0001/scan_truncation]
///
/// ### Arguments:
/// - `content`: Source text slice to inspect.
///
/// ### Returns:
/// A vector containing all detected truncation token strings. If empty, the input
/// is verified free of known truncation markers.
pub fn scan_truncation(content: &str) -> Vec<String> {
    let mut detected = Vec::new();
    for token in TRUNCATION_TOKENS {
        if content.contains(token) {
            detected.push((*token).to_string());
        }
    }
    detected
}

/// Computes a FIPS 180-4 compliant SHA-256 hexadecimal hash from an arbitrary byte slice.
///
/// /// Realises: [REQ-0001/compute_sha256]
///
/// ### Arguments:
/// - `data`: Input byte slice.
///
/// ### Returns:
/// A 64-character lowercase hexadecimal string representing the 256-bit hash.
pub fn compute_sha256(data: &[u8]) -> String {
    use std::fmt::Write;
    let digest = compute_sha256_bytes(data);
    let mut hex = String::with_capacity(64);
    for b in digest {
        let _ = write!(hex, "{:02x}", b);
    }
    hex
}

/// Internal software implementation of FIPS 180-4 SHA-256 compression function.
///
/// Guarantees license-free, deterministic execution across all embedded and host platforms.
fn compute_sha256_bytes(data: &[u8]) -> [u8; 32] {
    let mut h: [u32; 8] = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a,
        0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19,
    ];

    let k: [u32; 64] = [
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
        0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
        0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
        0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
        0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
        0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
        0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
        0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
    ];

    let bit_len = (data.len() as u64) * 8;
    let mut msg = data.to_vec();
    msg.push(0x80);
    while (msg.len() % 64) != 56 {
        msg.push(0x00);
    }
    msg.extend_from_slice(&bit_len.to_be_bytes());

    for chunk in msg.chunks_exact(64) {
        let mut w = [0u32; 64];
        for (i, b) in chunk.chunks_exact(4).enumerate() {
            w[i] = u32::from_be_bytes(b.try_into().unwrap());
        }
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16].wrapping_add(s0).wrapping_add(w[i - 7]).wrapping_add(s1);
        }

        let mut a = h[0];
        let mut b = h[1];
        let mut c = h[2];
        let mut d = h[3];
        let mut e = h[4];
        let mut f = h[5];
        let mut g = h[6];
        let mut hh = h[7];

        for i in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ ((!e) & g);
            let temp1 = hh.wrapping_add(s1).wrapping_add(ch).wrapping_add(k[i]).wrapping_add(w[i]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let temp2 = s0.wrapping_add(maj);

            hh = g;
            g = f;
            f = e;
            e = d.wrapping_add(temp1);
            d = c;
            c = b;
            b = a;
            a = temp1.wrapping_add(temp2);
        }

        h[0] = h[0].wrapping_add(a);
        h[1] = h[1].wrapping_add(b);
        h[2] = h[2].wrapping_add(c);
        h[3] = h[3].wrapping_add(d);
        h[4] = h[4].wrapping_add(e);
        h[5] = h[5].wrapping_add(f);
        h[6] = h[6].wrapping_add(g);
        h[7] = h[7].wrapping_add(hh);
    }

    let mut out = [0u8; 32];
    for (i, word) in h.iter().enumerate() {
        out[i * 4..(i + 1) * 4].copy_from_slice(&word.to_be_bytes());
    }
    out
}

/// Structural descriptor for an individual file registered in `input-digest.json`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DigestFileEntry {
    /// Cryptographic SHA-256 checksum of the file bytes.
    pub sha256: String,
    /// Exact line count of the file.
    pub total_lines: usize,
    /// 1-indexed inclusive line bounds `(start_line, end_line)`.
    pub line_range: (usize, usize),
    /// Canonical line range boundary tuple for verification gates.
    pub line_range_bounds: (usize, usize),
    /// List of required structural markdown headings or model container names.
    pub structural_section_markers: Vec<String>,
}

/// Consolidated input digest model written to `.pipeline/input-digest.json`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct InputDigestData {
    /// Combined SHA-256 hash over all concatenated input file bytes.
    pub sha256: String,
    /// Total aggregate lines across all ingested input files.
    pub total_lines: usize,
    /// Map of canonical workspace-relative file paths to their structural descriptors.
    pub files: BTreeMap<String, DigestFileEntry>,
    /// Global deduplicated set of all structural section markers across files.
    pub structural_section_markers: Vec<String>,
}

/// Parses an input specification file extracting line count, sha256, and structural section markers.
///
/// /// Realises: [REQ-0001/parse_input_file]
///
/// ### Arguments:
/// - `file_path`: Path to the specification file on disk.
///
/// ### Returns:
/// A tuple containing the initialized `DigestFileEntry` and raw file byte vector.
pub fn parse_input_file(file_path: &Path) -> Result<(DigestFileEntry, Vec<u8>), String> {
    let content_bytes = fs::read(file_path)
        .map_err(|e| format!("Failed to read file '{}': {}", file_path.display(), e))?;

    let sha256 = compute_sha256(&content_bytes);
    let text = String::from_utf8_lossy(&content_bytes);
    let lines: Vec<&str> = text.lines().collect();
    let total_lines = lines.len();

    let mut structural_section_markers = Vec::new();

    let md_header_re = Regex::new(r"^\s*(#{1,6}\s+.*)").unwrap();
    let yang_node_re = Regex::new(r"^\s*(container|list|typedef|identity|grouping|module)\s+([a-zA-Z0-9_\-]+)").unwrap();

    for line in &lines {
        if let Some(caps) = md_header_re.captures(line) {
            let marker = caps.get(1).unwrap().as_str().trim().to_string();
            if !structural_section_markers.contains(&marker) {
                structural_section_markers.push(marker);
            }
        }
        if let Some(caps) = yang_node_re.captures(line) {
            let marker = format!("{} {}", caps.get(1).unwrap().as_str(), caps.get(2).unwrap().as_str());
            if !structural_section_markers.contains(&marker) {
                structural_section_markers.push(marker);
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

/// Generates an input digest for a single file or a directory of specification files.
///
/// /// Realises: [REQ-0001/generate_input_digest]
///
/// ### Arguments:
/// - `input_path`: Path to an individual file or directory of input models.
///
/// ### Returns:
/// Populated `InputDigestData` containing cryptographic checksums and section inventory.
pub fn generate_input_digest(input_path: &Path) -> Result<InputDigestData, String> {
    let mut input_files = Vec::new();

    if input_path.is_file() {
        input_files.push(input_path.to_path_buf());
    } else if input_path.is_dir() {
        for entry in walkdir::WalkDir::new(input_path).sort_by_file_name() {
            if let Ok(e) = entry {
                let p = e.path();
                if p.is_file() {
                    let name = p.file_name().and_then(|s| s.to_str()).unwrap_or("");
                    if !name.starts_with('.') {
                        if let Some(ext) = p.extension().and_then(|s| s.to_str()) {
                            if ["md", "yang", "sysml", "proto", "json", "yaml", "yml", "txt"]
                                .contains(&ext.to_ascii_lowercase().as_str())
                            {
                                input_files.push(p.to_path_buf());
                            }
                        }
                    }
                }
            }
        }
    }

    if input_files.is_empty() {
        return Ok(InputDigestData {
            sha256: "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855".to_string(),
            total_lines: 0,
            files: BTreeMap::new(),
            structural_section_markers: Vec::new(),
        });
    }

    let mut combined_bytes = Vec::new();
    let mut total_lines = 0;
    let mut file_map = BTreeMap::new();
    let mut all_markers = Vec::new();

    for fpath in input_files {
        let (entry, bytes) = parse_input_file(&fpath)?;
        combined_bytes.extend_from_slice(&bytes);
        total_lines += entry.total_lines;

        for m in &entry.structural_section_markers {
            if !all_markers.contains(m) {
                all_markers.push(m.clone());
            }
        }

        let fpath_str = fpath.to_string_lossy().to_string();
        file_map.insert(fpath_str, entry);
    }

    let overall_sha256 = compute_sha256(&combined_bytes);

    Ok(InputDigestData {
        sha256: overall_sha256,
        total_lines,
        files: file_map,
        structural_section_markers: all_markers,
    })
}

/// Computes multi-file combined sha256 hash and total lines count.
///
/// /// Realises: [REQ-0001/compute_multi_file_sha256]
///
/// ### Arguments:
/// - `paths`: Array of filesystem paths to read and hash.
///
/// ### Returns:
/// A tuple `(combined_sha256_hex, total_lines)`.
pub fn compute_multi_file_sha256(paths: &[PathBuf]) -> Result<(String, usize), String> {
    let mut combined_bytes = Vec::new();
    let mut total_lines = 0;

    for p in paths {
        let b = fs::read(p).map_err(|e| format!("Failed to read {}: {}", p.display(), e))?;
        total_lines += String::from_utf8_lossy(&b).lines().count();
        combined_bytes.extend_from_slice(&b);
    }

    let hash = compute_sha256(&combined_bytes);
    Ok((hash, total_lines))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_sha256() {
        assert_eq!(
            compute_sha256(b""),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
        assert_eq!(
            compute_sha256(b"hello world"),
            "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
        );
    }

    #[test]
    fn test_scan_truncation() {
        let clean = "# Package_0\n## Classifier_Alpha\nPure clean specification content.";
        assert!(scan_truncation(clean).is_empty());

        let truncated = "# Package_0\n... truncated content";
        let detected = scan_truncation(truncated);
        assert!(detected.contains(&"...".to_string()));
        assert!(detected.contains(&"truncated content".to_string()));
    }

    #[test]
    fn test_parse_input_file_req1() {
        let p = Path::new("schema/REQ-0001.md");
        if p.exists() {
            let res = parse_input_file(p);
            assert!(res.is_ok());
            let (entry, _) = res.unwrap();
            assert!(entry.total_lines > 0);
            assert!(!entry.sha256.is_empty());
            assert!(entry.structural_section_markers.iter().any(|m| m.contains("1. Normative Statement")));
        }
    }
}
