#![deny(
    clippy::too_many_lines,
    clippy::too_many_arguments,
    clippy::undocumented_unsafe_blocks
)]
//! Logical Interface Control Document (ICD) Compiler and CLI Artifact Emission Dispatcher.
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety-critical
//! software engineering standards. Synthesizes Level 1C `ICD_01_SYSTEM_INTERFACE_MATRIX.md` and
//! `ICD_02_MASTER_SIGNAL_DICTIONARY.md` specifications from SysML v2 `PortDef` and `ConnectionDef`
//! AST nodes and dispatches `--emit-icd` / `--emit-wbs` CLI flags.
//!
//! ## 2. Invariants & Mathematical Properties
//! - Fail-Closed Interface Verification: Undeclared connection endpoints or zero-interface models
//!   fail closed with `Diagnostic: E0200`.
//! - Total Ordering: `SignalDictionaryRow` entries are sorted lexicographically and deduplicated.
//!
//! ## 3. Preconditions
//! - `pkg` is a parsed `PackageDef` containing declared `PartDef`, `PortDef`, or `ConnectionDef` nodes.
//!
//! ## 4. Postconditions
//! - Emits validated `IcdDocumentPair` artifacts and persists files atomically via `write_atomic`.
//!
//! ## 5. Algorithmic Complexity Bounds
//! - Time Complexity: $O(V^2 + E \log E)$ where $V$ is part count and $E$ is port/connection count.
//! - Space Complexity: $O(V^2 + E)$ for interconnect matrices and signal rows.
//!
//! ## 6. Failure Modes & Fault Tolerance
//! - Returns `Err(String)` containing `Diagnostic: E0200` or `Diagnostic: E0300` on invalid topology or CLI arguments.
//!
//! ## 7. Realization & Traceability
//! Realises: [REQ-0604]

use super::*;

type ConnectionRegisterRow = (String, String, String, String, String);
type CellLinkMap = BTreeMap<(String, String), BTreeSet<String>>;

/// Canonical row in `ICD_02_MASTER_SIGNAL_DICTIONARY.md`.
///
/// Safety & Traceability: Realises: [REQ-0604]
/// Contract: Totally ordered lexicographically by `(signal_name, payload_type, direction, source_endpoint, target_endpoint, package_name)`.
/// Bounds & Faults: Immutable value semantics under `Clone`, `Eq`, and `Ord`.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct SignalDictionaryRow {
    pub signal_name: String,
    pub payload_type: String,
    pub direction: String,
    pub source_endpoint: String,
    pub target_endpoint: String,
    pub package_name: String,
}

/// Compiled Level 1C Logical Interface Control Document pair (`ICD_01` and `ICD_02`).
///
/// Safety & Traceability: Realises: [REQ-0604]
/// Contract: Holds validated `interface_matrix_md`, `signal_dictionary_md`, and sorted `rows`.
/// Bounds & Faults: Pure value container with bounded string buffers.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct IcdDocumentPair {
    pub interface_matrix_md: String,
    pub signal_dictionary_md: String,
    pub rows: Vec<SignalDictionaryRow>,
}

/// Resolves `(component_name, endpoint_label)` from optional part and port strings on a `ConnectionDef`.
/// Realises: [REQ-0604]
pub(super) fn resolve_connection_endpoint(
    part_opt: Option<&str>,
    port_str: &str,
) -> (String, String) {
    let clean_port = port_str.trim();
    if let Some(part_name) = part_opt.map(str::trim).filter(|s| !s.is_empty()) {
        if clean_port.is_empty() {
            (part_name.to_string(), part_name.to_string())
        } else if clean_port.contains('.') {
            (part_name.to_string(), clean_port.to_string())
        } else {
            (
                part_name.to_string(),
                format!("{}.{}", part_name, clean_port),
            )
        }
    } else if let Some((comp, port)) = clean_port.split_once('.') {
        let c = comp.trim();
        let p = port.trim();
        (c.to_string(), format!("{}.{}", c, p))
    } else {
        (clean_port.to_string(), clean_port.to_string())
    }
}

/// Collects declared `PartDef` names and port-derived `SignalDictionaryRow` entries across `all_packages`.
/// Realises: [REQ-0604]
fn collect_declared_parts_and_port_rows(
    all_packages: &[&PackageDef],
) -> (BTreeSet<String>, Vec<SignalDictionaryRow>) {
    let mut declared_parts: BTreeSet<String> = BTreeSet::new();
    let mut rows: Vec<SignalDictionaryRow> = Vec::new();

    for p in all_packages {
        for part in &p.part_defs {
            declared_parts.insert(part.name.clone());
            for sub in &part.parts {
                declared_parts.insert(sub.name.clone());
                if let Some(ref t) = sub.type_name {
                    declared_parts.insert(t.clone());
                }
            }
            for port in &part.ports {
                let sig_name = if port.type_name.trim().is_empty() {
                    port.name.trim().to_string()
                } else {
                    port.type_name.trim().to_string()
                };
                let dir = if port.direction.trim().is_empty() { "inout" } else { port.direction.trim() };
                rows.push(SignalDictionaryRow {
                    signal_name: sig_name.clone(),
                    payload_type: sig_name,
                    direction: dir.to_string(),
                    source_endpoint: format!("{}.{}", part.name, port.name),
                    target_endpoint: "-".to_string(),
                    package_name: p.name.clone(),
                });
            }
        }
        for port in &p.port_defs {
            let sig_name = if port.type_name.trim().is_empty() {
                port.name.trim().to_string()
            } else {
                port.type_name.trim().to_string()
            };
            let dir = if port.direction.trim().is_empty() { "inout" } else { port.direction.trim() };
            rows.push(SignalDictionaryRow {
                signal_name: sig_name.clone(),
                payload_type: sig_name,
                direction: dir.to_string(),
                source_endpoint: format!("{}.{}", p.name, port.name),
                target_endpoint: "-".to_string(),
                package_name: p.name.clone(),
            });
        }
    }
    (declared_parts, rows)
}

/// Validates and collects all package and part `ConnectionDef` edges, failing closed with `Diagnostic: E0200` on undeclared endpoints.
/// Realises: [REQ-0604]
fn collect_verified_connections(
    all_packages: &[&PackageDef],
    declared_parts: &BTreeSet<String>,
    rows: &mut Vec<SignalDictionaryRow>,
) -> Result<(Vec<ConnectionRegisterRow>, CellLinkMap), String> {
    let mut all_connections: Vec<ConnectionRegisterRow> = Vec::new();
    let mut cell_links: CellLinkMap = BTreeMap::new();

    for p in all_packages {
        let mut pkg_conns: Vec<&ConnectionDef> = p.connection_defs.iter().collect();
        for part in &p.part_defs {
            pkg_conns.extend(part.connections.iter());
        }
        for conn in pkg_conns {
            let (src_comp, src_ep) =
                resolve_connection_endpoint(conn.source_part.as_deref(), &conn.source_port);
            let (dst_comp, dst_ep) =
                resolve_connection_endpoint(conn.target_part.as_deref(), &conn.target_port);
            if src_comp.is_empty()
                || dst_comp.is_empty()
                || !declared_parts.contains(&src_comp)
                || !declared_parts.contains(&dst_comp)
            {
                return Err(format!(
                    "Diagnostic: E0200: undeclared interface endpoint '{}' -> '{}'",
                    src_ep, dst_ep
                ));
            }
            let sig = conn
                .item_flow_ref
                .as_deref()
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| {
                    if !conn.name.trim().is_empty() {
                        conn.name.trim()
                    } else {
                        "LogicalFlow"
                    }
                })
                .to_string();
            let conn_id = if conn.name.trim().is_empty() {
                format!("{}_to_{}", sanitize_id(&src_comp), sanitize_id(&dst_comp))
            } else {
                conn.name.trim().to_string()
            };
            cell_links
                .entry((src_comp, dst_comp))
                .or_default()
                .insert(format!("`{}`", sig));
            all_connections.push((
                conn_id,
                src_ep.clone(),
                dst_ep.clone(),
                sig.clone(),
                p.name.clone(),
            ));
            rows.push(SignalDictionaryRow {
                signal_name: sig.clone(),
                payload_type: sig,
                direction: if conn.is_flow {
                    "flow".to_string()
                } else {
                    "inout".to_string()
                },
                source_endpoint: src_ep,
                target_endpoint: dst_ep,
                package_name: p.name.clone(),
            });
        }
    }
    Ok((all_connections, cell_links))
}

/// Compiles `ICD_01_SYSTEM_INTERFACE_MATRIX.md` and `ICD_02_MASTER_SIGNAL_DICTIONARY.md` from `pkg`, failing closed with `Diagnostic: E0200` on undeclared endpoints or empty interface models.
/// Realises: [REQ-0604]
pub fn try_compile_logical_icd_artifacts(pkg: &PackageDef) -> Result<IcdDocumentPair, String> {
    let all_packages = pkg.get_all_packages();
    let (declared_parts, mut rows) = collect_declared_parts_and_port_rows(&all_packages);
    let (mut all_connections, cell_links) =
        collect_verified_connections(&all_packages, &declared_parts, &mut rows)?;

    if rows.is_empty() && all_connections.is_empty() {
        return Err(
            "Diagnostic: E0200: zero logical interface artifacts (ports, connections, or item flows) found in PackageDef"
                .to_string(),
        );
    }

    rows.sort();
    rows.dedup();
    all_connections.sort();

    let part_list: Vec<String> = declared_parts.into_iter().collect();
    let icd01 =
        render_icd01_matrix_and_register(&part_list, &cell_links, &all_connections, &rows);
    let icd02 = render_icd02_signal_dictionary(&rows);

    Ok(IcdDocumentPair {
        interface_matrix_md: icd01,
        signal_dictionary_md: icd02,
        rows,
    })
}

/// Compiles `ICD_01_SYSTEM_INTERFACE_MATRIX.md` and `ICD_02_MASTER_SIGNAL_DICTIONARY.md` as a `(String, String)` tuple.
/// Realises: [REQ-0604]
pub fn compile_logical_icd_artifacts(pkg: &PackageDef) -> (String, String) {
    match try_compile_logical_icd_artifacts(pkg) {
        Ok(pair) => (pair.interface_matrix_md, pair.signal_dictionary_md),
        Err(diag) => (diag.clone(), diag),
    }
}

/// Compiles and atomically persists `ICD_01_SYSTEM_INTERFACE_MATRIX.md` and `ICD_02_MASTER_SIGNAL_DICTIONARY.md` into `interfaces_dir`.
/// Realises: [REQ-0604]
pub fn emit_logical_icd_artifacts(
    pkg: &PackageDef,
    interfaces_dir: &Path,
) -> Result<(PathBuf, PathBuf), String> {
    let pair = try_compile_logical_icd_artifacts(pkg)?;
    fs::create_dir_all(interfaces_dir).map_err(|e| {
        format!(
            "Failed to create interfaces directory '{}': {}",
            interfaces_dir.display(),
            e
        )
    })?;
    let icd01_path = interfaces_dir.join("ICD_01_SYSTEM_INTERFACE_MATRIX.md");
    let icd02_path = interfaces_dir.join("ICD_02_MASTER_SIGNAL_DICTIONARY.md");
    write_atomic(&icd01_path, &pair.interface_matrix_md).map_err(|e| e.to_string())?;
    write_atomic(&icd02_path, &pair.signal_dictionary_md).map_err(|e| e.to_string())?;
    Ok((icd01_path, icd02_path))
}

/// Evaluates CLI arguments for `--emit-icd <INTERFACES_DIR>` and `--emit-wbs <MANAGEMENT_DIR>` flags (`REQ-0604`).
/// Realises: [REQ-0604]
pub fn execute_emit_cli_args(args: &[String]) -> Result<Option<i32>, String> {
    let mut emit_icd_dir: Option<PathBuf> = None;
    let mut emit_wbs_dir: Option<PathBuf> = None;
    let mut schema_path: Option<PathBuf> = None;

    let mut idx = 1usize;
    while let Some(arg) = args.get(idx) {
        if arg == "--emit-icd" {
            let val = args.get(idx.saturating_add(1)).ok_or_else(|| {
                "Diagnostic: E0200: missing directory argument after --emit-icd".to_string()
            })?;
            emit_icd_dir = Some(PathBuf::from(val));
            idx = idx.saturating_add(2);
        } else if arg == "--emit-wbs" {
            let val = args.get(idx.saturating_add(1)).ok_or_else(|| {
                "Diagnostic: E0300: missing directory argument after --emit-wbs".to_string()
            })?;
            emit_wbs_dir = Some(PathBuf::from(val));
            idx = idx.saturating_add(2);
        } else if arg == "--schema" {
            let val = args
                .get(idx.saturating_add(1))
                .ok_or_else(|| "Missing path after --schema".to_string())?;
            schema_path = Some(PathBuf::from(val));
            idx = idx.saturating_add(2);
        } else if !arg.starts_with('-') && schema_path.is_none() {
            schema_path = Some(PathBuf::from(arg));
            idx = idx.saturating_add(1);
        } else {
            idx = idx.saturating_add(1);
        }
    }

    if emit_icd_dir.is_none() && emit_wbs_dir.is_none() {
        return Ok(None);
    }

    let resolved_schema = schema_path.unwrap_or_else(|| PathBuf::from(".pipeline/schema.sysml"));
    let content = fs::read_to_string(&resolved_schema).map_err(|e| {
        format!(
            "Failed to read SysML schema '{}': {}",
            resolved_schema.display(),
            e
        )
    })?;
    let pkg = crate::parse_sysml(&content).map_err(|e| e.to_string())?;

    if let Some(ref icd_dir) = emit_icd_dir {
        emit_logical_icd_artifacts(&pkg, icd_dir)?;
    }
    if let Some(ref wbs_dir) = emit_wbs_dir {
        emit_wbs_artifacts(&pkg, wbs_dir)?;
    }
    Ok(Some(0))
}

/// Runs CLI pre-dispatch for `--emit-icd` / `--emit-wbs` and delegates to `default_runner` when neither flag is present.
/// Realises: [REQ-0604]
pub fn run_cli_with_emit_flags<F: FnOnce() -> i32>(default_runner: F) -> i32 {
    let args: Vec<String> = std::env::args().collect();
    match execute_emit_cli_args(&args) {
        Ok(Some(code)) => code,
        Ok(None) => default_runner(),
        Err(err) => {
            eprintln!("{err}");
            1
        }
    }
}
