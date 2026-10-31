//! Logical Interface Control Document (ICD) Compiler and CLI Artifact Emission Dispatcher.
//!
//! Safety & Traceability: Realises: [REQ-0604]
//! Contract: Synthesizes Level 1C `ICD_01_SYSTEM_INTERFACE_MATRIX.md` and `ICD_02_MASTER_SIGNAL_DICTIONARY.md` specifications from SysML v2 `PortDef` and `ConnectionDef` AST nodes and dispatches `--emit-icd` / `--emit-wbs` CLI flags.
//! Bounds & Faults: Fails closed with `Diagnostic: E0200` on undeclared connection endpoints or zero-interface models, enforces deterministic `BTreeMap`/`BTreeSet` ordering, and persists files via `write_atomic`.

use super::*;

/// Canonical row in `ICD_02_MASTER_SIGNAL_DICTIONARY.md`.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for struct `SignalDictionaryRow` in `crates/compile-sysml/src/sync/forward/icd_wbs_emitter.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Totally ordered lexicographically by `(signal_name, payload_type, direction, source_endpoint, target_endpoint, package_name)`.
///
/// ## 3. Preconditions
/// - Populated from verified `PortDef` and `ConnectionDef` AST nodes.
///
/// ## 4. Postconditions
/// - Represents a single deterministic row in `ICD_02_MASTER_SIGNAL_DICTIONARY.md`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(1) per row comparison.
/// - Space Complexity: O(1) bounded string fields.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Strongly typed struct fields.
/// - State: Immutable value semantics under `Clone` and `Eq`.
/// - Action: Deterministic ordering via `Ord`.
/// - Resource: Bounded heap strings.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-0604] | SysML v2: PartDef(SignalDictionaryRow)
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
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for struct `IcdDocumentPair` in `crates/compile-sysml/src/sync/forward/icd_wbs_emitter.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - `interface_matrix_md` contains the $N \times N$ component interconnect matrix with `---` diagonal cells; `signal_dictionary_md` contains the lexicographically sorted signal dictionary.
///
/// ## 3. Preconditions
/// - Constructed by `try_compile_logical_icd_artifacts`.
///
/// ## 4. Postconditions
/// - Holds validated Markdown payloads ready for atomic disk persistence.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(V^2 + E log E).
/// - Space Complexity: O(V^2 + E).
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validated prior to construction.
/// - State: Pure value container.
/// - Action: Deterministic serialization.
/// - Resource: Bounded string buffers.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-0604] | SysML v2: PartDef(IcdDocumentPair)
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct IcdDocumentPair {
    pub interface_matrix_md: String,
    pub signal_dictionary_md: String,
    pub rows: Vec<SignalDictionaryRow>,
}

/// Resolves `(component_name, endpoint_label)` from optional part and port strings on a `ConnectionDef`.
/// Normalizes both `(Some("Comp"), "port")` and `(None, "Comp.port")` to `("Comp", "Comp.port")` without panicking on empty inputs.
///
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

/// Compiles `ICD_01_SYSTEM_INTERFACE_MATRIX.md` and `ICD_02_MASTER_SIGNAL_DICTIONARY.md` from `pkg`, failing closed with `Diagnostic: E0200` on undeclared endpoints or empty interface models.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `try_compile_logical_icd_artifacts` in `crates/compile-sysml/src/sync/forward/icd_wbs_emitter.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Diagonal cells `(i, i)` in the $N \times N$ interconnect matrix are always `---`.
/// - `rows` in `ICD_02_MASTER_SIGNAL_DICTIONARY.md` are sorted in strict lexicographical order.
/// - Undeclared connection endpoints fail closed with `Diagnostic: E0200`.
///
/// ## 3. Preconditions
/// - `pkg: &PackageDef` is a parsed SysML AST root package.
///
/// ## 4. Postconditions
/// - Returns `Ok(IcdDocumentPair)` on valid interface topology or `Err(String)` containing `Diagnostic: E0200`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(V^2 + E log E) where V is `PartDef` count and E is port/connection count.
/// - Space Complexity: O(V^2 + E).
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Rejects undeclared endpoints and zero-interface models fail-closed (`Diagnostic: E0200`).
/// - State: Pure function with zero filesystem mutation.
/// - Action: Deterministic BTree traversal.
/// - Resource: Bounded memory proportional to AST interface size.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-0604] | SysML v2: ActionDef(try_compile_logical_icd_artifacts)
pub fn try_compile_logical_icd_artifacts(pkg: &PackageDef) -> Result<IcdDocumentPair, String> {
    let all_packages = pkg.get_all_packages();
    let mut declared_parts: BTreeSet<String> = BTreeSet::new();
    let mut rows: Vec<SignalDictionaryRow> = Vec::new();
    let mut all_connections: Vec<(String, String, String, String, String)> = Vec::new();
    let mut cell_links: BTreeMap<(String, String), BTreeSet<String>> = BTreeMap::new();

    for p in &all_packages {
        for part in &p.part_defs {
            declared_parts.insert(part.name.clone());
            for sub in &part.parts {
                declared_parts.insert(sub.name.clone());
                if let Some(ref t) = sub.type_name {
                    declared_parts.insert(t.clone());
                }
            }
            for port in &part.ports {
                let sig_name = if !port.type_name.trim().is_empty() {
                    port.type_name.trim().to_string()
                } else {
                    port.name.trim().to_string()
                };
                let dir = if port.direction.trim().is_empty() {
                    "inout".to_string()
                } else {
                    port.direction.trim().to_string()
                };
                rows.push(SignalDictionaryRow {
                    signal_name: sig_name.clone(),
                    payload_type: sig_name,
                    direction: dir,
                    source_endpoint: format!("{}.{}", part.name, port.name),
                    target_endpoint: "-".to_string(),
                    package_name: p.name.clone(),
                });
            }
        }
        for port in &p.port_defs {
            let sig_name = if !port.type_name.trim().is_empty() {
                port.type_name.trim().to_string()
            } else {
                port.name.trim().to_string()
            };
            rows.push(SignalDictionaryRow {
                signal_name: sig_name.clone(),
                payload_type: sig_name,
                direction: if port.direction.trim().is_empty() {
                    "inout".to_string()
                } else {
                    port.direction.trim().to_string()
                },
                source_endpoint: format!("{}.{}", p.name, port.name),
                target_endpoint: "-".to_string(),
                package_name: p.name.clone(),
            });
        }
    }

    for p in &all_packages {
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
                .entry((src_comp.clone(), dst_comp.clone()))
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
    let mut icd01 = String::new();
    icd01.push_str("# ICD_01: System Interface Matrix\n\n");
    icd01.push_str("## 1. N x N Subsystem & Component Interconnect Matrix\n\n");
    if part_list.is_empty() {
        icd01.push_str("| Component | System |\n| :--- | :--- |\n| **System** | --- |\n\n");
    } else {
        icd01.push_str("| Component |");
        for col in &part_list {
            icd01.push_str(&format!(" {} |", col));
        }
        icd01.push('\n');
        icd01.push_str("| :--- |");
        for _ in &part_list {
            icd01.push_str(" :--- |");
        }
        icd01.push('\n');
        for row_part in &part_list {
            icd01.push_str(&format!("| **{}** |", row_part));
            for col_part in &part_list {
                if row_part == col_part {
                    icd01.push_str(" --- |");
                } else if let Some(sigs) = cell_links.get(&(row_part.clone(), col_part.clone())) {
                    let joined: Vec<String> = sigs.iter().cloned().collect();
                    icd01.push_str(&format!(" {} |", joined.join(", ")));
                } else {
                    icd01.push_str(" - |");
                }
            }
            icd01.push('\n');
        }
        icd01.push('\n');
    }

    icd01.push_str("## 2. Point-to-Point Logical Connection Register\n\n");
    icd01.push_str(
        "| Connection ID | Source Endpoint | Target Endpoint | Signal / Item Flow | Package |\n",
    );
    icd01.push_str("| :--- | :--- | :--- | :--- | :--- |\n");
    if all_connections.is_empty() {
        for r in &rows {
            icd01.push_str(&format!(
                "| `PORT-{}` | `{}` | `{}` | `{}` | `{}` |\n",
                sanitize_id(&r.signal_name),
                r.source_endpoint,
                r.target_endpoint,
                r.signal_name,
                r.package_name
            ));
        }
    } else {
        for (cid, src, dst, sig, pkg_name) in &all_connections {
            icd01.push_str(&format!(
                "| `{}` | `{}` | `{}` | `{}` | `{}` |\n",
                cid, src, dst, sig, pkg_name
            ));
        }
    }

    let mut icd02 = String::new();
    icd02.push_str("# ICD_02: Master Signal Dictionary\n\n");
    icd02.push_str("## 1. Canonical Signal & Port Definitions\n\n");
    icd02.push_str(
        "| Signal Name | Payload Type | Direction | Source Endpoint | Target Endpoint | Package |\n",
    );
    icd02.push_str("| :--- | :--- | :--- | :--- | :--- | :--- |\n");
    for r in &rows {
        icd02.push_str(&format!(
            "| `{}` | `{}` | `{}` | `{}` | `{}` | `{}` |\n",
            r.signal_name,
            r.payload_type,
            r.direction,
            r.source_endpoint,
            r.target_endpoint,
            r.package_name
        ));
    }

    Ok(IcdDocumentPair {
        interface_matrix_md: icd01,
        signal_dictionary_md: icd02,
        rows,
    })
}

/// Compiles `ICD_01_SYSTEM_INTERFACE_MATRIX.md` and `ICD_02_MASTER_SIGNAL_DICTIONARY.md` as a `(String, String)` tuple.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `compile_logical_icd_artifacts` in `crates/compile-sysml/src/sync/forward/icd_wbs_emitter.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Delegates to `try_compile_logical_icd_artifacts` and returns deterministic Markdown strings.
///
/// ## 3. Preconditions
/// - `pkg: &PackageDef` is a valid AST root package.
///
/// ## 4. Postconditions
/// - Returns `(interface_matrix_md, signal_dictionary_md)`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(V^2 + E log E).
/// - Space Complexity: O(V^2 + E).
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Embeds `Diagnostic: E0200` error text if `pkg` violates interface preconditions.
/// - State: Pure function.
/// - Action: Deterministic execution.
/// - Resource: Bounded string buffers.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-0604] | SysML v2: ActionDef(compile_logical_icd_artifacts)
pub fn compile_logical_icd_artifacts(pkg: &PackageDef) -> (String, String) {
    match try_compile_logical_icd_artifacts(pkg) {
        Ok(pair) => (pair.interface_matrix_md, pair.signal_dictionary_md),
        Err(diag) => (diag.clone(), diag),
    }
}

/// Compiles and atomically persists `ICD_01_SYSTEM_INTERFACE_MATRIX.md` and `ICD_02_MASTER_SIGNAL_DICTIONARY.md` into `interfaces_dir`.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `emit_logical_icd_artifacts` in `crates/compile-sysml/src/sync/forward/icd_wbs_emitter.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Uses `write_atomic` to guarantee crash-safe persistence without partial writes.
///
/// ## 3. Preconditions
/// - `pkg: &PackageDef` satisfies `try_compile_logical_icd_artifacts` and `interfaces_dir: &Path` is writable.
///
/// ## 4. Postconditions
/// - Returns `Ok((icd01_path, icd02_path))` after both files exist on disk.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(V^2 + E log E).
/// - Space Complexity: O(V^2 + E).
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Propagates `Diagnostic: E0200` or filesystem I/O errors via `Result::Err`.
/// - State: Atomic file replacement via `write_atomic`.
/// - Action: Zero silent error discards.
/// - Resource: Bounded file buffers.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-0604] | SysML v2: ActionDef(emit_logical_icd_artifacts)
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
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `execute_emit_cli_args` in `crates/compile-sysml/src/sync/forward/icd_wbs_emitter.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Returns `Ok(None)` when neither `--emit-icd` nor `--emit-wbs` is present so standard `clap` CLI execution proceeds unaltered.
/// - Returns `Ok(Some(0))` when all requested ICD/WBS artifacts are emitted cleanly, or `Err` on failure.
///
/// ## 3. Preconditions
/// - `args: &[String]` contains process command-line arguments.
///
/// ## 4. Postconditions
/// - Emits requested ICD and/or WBS artifacts when `--emit-icd` or `--emit-wbs` is present.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N log N) over parsed SysML model.
/// - Space Complexity: O(N).
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates argument pairs and schema file readability fail-closed.
/// - State: Atomic file persistence via `emit_logical_icd_artifacts` and `emit_wbs_artifacts`.
/// - Action: Deterministic CLI flag parsing.
/// - Resource: Bounded memory and file handles.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-0604] | SysML v2: ActionDef(execute_emit_cli_args)
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
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `run_cli_with_emit_flags` in `crates/compile-sysml/src/sync/forward/icd_wbs_emitter.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves $v(G) = 1$ in `main.rs` while supporting `--emit-icd` and `--emit-wbs` CLI flags.
///
/// ## 3. Preconditions
/// - `default_runner: F` is the standard `execute_cli(Cli::parse())` closure.
///
/// ## 4. Postconditions
/// - Returns process exit code `0` on success or `1` on error.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N log N).
/// - Space Complexity: O(N).
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Logs errors to `stderr` and returns exit code `1`.
/// - State: Delegates cleanly to `default_runner` when emit flags are absent.
/// - Action: Single-entry/single-exit status code resolution.
/// - Resource: Bounded process argument collection.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-0604] | SysML v2: ActionDef(run_cli_with_emit_flags)
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
