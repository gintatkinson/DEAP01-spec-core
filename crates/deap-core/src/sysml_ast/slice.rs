//! Token-budgeted subsystem AST projection / slicing.
//!
//! Safety & Traceability: Realises: [REQ-SYSML-AST-MODULE], [DO-330-TQL-4-SYSML-AST]
//! Contract: Implements `SysmlModel::slice_package_subsystem` for token-budgeted package and requirement AST slicing.
//! Bounds & Faults: O(N) projection over model elements; fails closed with `CoreError` on invalid package or token bounds.

use super::interfaces_and_items::InterfaceDef;
use super::package_model::{AstSlice, PackageDef, SysmlModel};
use super::parts_and_ports::{PartDef, PortDef};
use super::requirements::RequirementDef;
use super::safety_and_fmeca::HazardDef;
use crate::error::CoreError;

/// Aggregated references and intermediate collections extracted from parts during subsystem slicing.
/// Realises: [REQ-SYSML-AST-SLICE-PACKAGE-SUBSYSTEM]
#[derive(Default)]
struct PartExtractionContext {
    ports: Vec<PortDef>,
    interfaces: Vec<InterfaceDef>,
    referenced_req_ids: Vec<String>,
    referenced_hazard_ids: Vec<String>,
    discovered_hazards: Vec<HazardDef>,
    part_requirements: Vec<RequirementDef>,
}

/// Collects ports, interfaces, and requirement/hazard references from matched parts.
/// Realises: [REQ-SYSML-AST-SLICE-PACKAGE-SUBSYSTEM]
fn extract_part_context(matched_parts: &[PartDef], mut ports: Vec<PortDef>) -> PartExtractionContext {
    let mut interfaces: Vec<InterfaceDef> = Vec::new();
    let mut referenced_req_ids: Vec<String> = Vec::new();
    let mut referenced_hazard_ids: Vec<String> = Vec::new();
    let mut discovered_hazards: Vec<HazardDef> = Vec::new();
    let mut part_requirements: Vec<RequirementDef> = Vec::new();

    for part in matched_parts {
        for port in &part.ports {
            if !ports.iter().any(|existing| existing.name == port.name) {
                ports.push(port.clone());
            }
        }
        for conn in &part.connections {
            let iface = InterfaceDef {
                name: conn.name.clone(),
                source_port: conn.source_port.clone(),
                target_port: conn.target_port.clone(),
                doc: conn.doc.clone(),
            };
            if !interfaces.contains(&iface) {
                interfaces.push(iface);
            }
        }
        for req_id in &part.satisfied_requirements {
            if !referenced_req_ids.contains(req_id) {
                referenced_req_ids.push(req_id.clone());
            }
        }
        for safety_req_id in &part.safety_realises {
            if !referenced_req_ids.contains(safety_req_id) {
                referenced_req_ids.push(safety_req_id.clone());
            }
        }
        for haz_id in &part.triggers_hazards {
            if !referenced_hazard_ids.contains(haz_id) {
                referenced_hazard_ids.push(haz_id.clone());
            }
        }
        for haz in &part.hazards {
            if !discovered_hazards.iter().any(|h| h.name == haz.name) {
                discovered_hazards.push(haz.clone());
            }
        }
        for req in &part.requirements {
            if !part_requirements.iter().any(|r| r.name == req.name && r.req_id == req.req_id) {
                part_requirements.push(req.clone());
            }
        }
    }

    PartExtractionContext {
        ports,
        interfaces,
        referenced_req_ids,
        referenced_hazard_ids,
        discovered_hazards,
        part_requirements,
    }
}

/// Resolves requirement and hazard definitions matching the extracted part references.
/// Realises: [REQ-SYSML-AST-SLICE-PACKAGE-SUBSYSTEM]
fn resolve_slice_reqs_and_hazards(
    model_reqs: &[RequirementDef],
    mut ctx: PartExtractionContext,
) -> (Vec<PortDef>, Vec<InterfaceDef>, Vec<RequirementDef>, Vec<HazardDef>) {
    let mut requirements: Vec<RequirementDef> = Vec::new();
    for req in model_reqs.iter().chain(ctx.part_requirements.iter()) {
        let is_referenced = ctx.referenced_req_ids.is_empty()
            || ctx.referenced_req_ids.iter().any(|id| id == &req.name || id == &req.req_id);
        if is_referenced && !requirements.iter().any(|r| r.name == req.name && r.req_id == req.req_id) {
            requirements.push(req.clone());
            for haz_id in &req.triggers_hazards {
                if !ctx.referenced_hazard_ids.contains(haz_id) {
                    ctx.referenced_hazard_ids.push(haz_id.clone());
                }
            }
        }
    }

    let mut hazards: Vec<HazardDef> = Vec::new();
    for haz in &ctx.discovered_hazards {
        if (ctx.referenced_hazard_ids.is_empty() || ctx.referenced_hazard_ids.contains(&haz.name))
            && !hazards.iter().any(|h| h.name == haz.name)
        {
            hazards.push(haz.clone());
        }
    }
    for haz_id in &ctx.referenced_hazard_ids {
        if !hazards.iter().any(|h| &h.name == haz_id) {
            hazards.push(HazardDef {
                name: haz_id.clone(),
                doc: None,
                severity: 1,
                source_port: None,
                target_port: None,
                part_ref: None,
            });
        }
    }

    (ctx.ports, ctx.interfaces, requirements, hazards)
}

impl SysmlModel {
    /// Extracts a deterministic, token-budgeted `AstSlice` for `target_package` from this `SysmlModel`.
    ///
    /// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D.
    /// Realises: [REQ-SYSML-AST-SLICE-PACKAGE-SUBSYSTEM]
    pub fn slice_package_subsystem(
        &self,
        target_package: &str,
        max_tokens: usize,
    ) -> Result<AstSlice, CoreError> {
        let target = target_package.trim();
        if target.is_empty() {
            return Err(CoreError::validation("target_package must not be empty"));
        }
        if max_tokens == 0 {
            return Err(CoreError::validation("max_tokens must be greater than zero"));
        }

        let pkg_matches = self
            .package_name
            .as_deref()
            .is_some_and(|pkg_name| pkg_name == target || pkg_name.eq_ignore_ascii_case(target));

        let matched_parts: Vec<PartDef> = if pkg_matches {
            self.parts.clone()
        } else {
            self.parts
                .iter()
                .filter(|p| {
                    p.name == target
                        || p.name.starts_with(target)
                        || p.name.eq_ignore_ascii_case(target)
                })
                .cloned()
                .collect()
        };

        if !pkg_matches && matched_parts.is_empty() {
            return Err(CoreError::validation(format!(
                "No package or part matching '{}' found in SysmlModel",
                target
            )));
        }

        let mut initial_ports: Vec<PortDef> = Vec::new();
        if pkg_matches {
            for port in &self.ports {
                if !initial_ports.iter().any(|existing| existing.name == port.name) {
                    initial_ports.push(port.clone());
                }
            }
        }

        let ctx = extract_part_context(&matched_parts, initial_ports);
        let (ports, interfaces, requirements, hazards) =
            resolve_slice_reqs_and_hazards(&self.requirements, ctx);

        let slice_pkg = PackageDef {
            name: target.to_string(),
            doc: self.doc.clone(),
            part_defs: matched_parts.clone(),
            port_defs: ports.clone(),
            requirement_defs: requirements.clone(),
            hazard_defs: hazards.clone(),
            ..Default::default()
        };

        let mut slice = AstSlice {
            target_package: target.to_string(),
            packages: vec![slice_pkg],
            parts: matched_parts,
            ports,
            interfaces,
            requirements,
            hazards,
            estimated_tokens: 0,
        };

        let json_bytes = serde_json::to_vec(&slice).map_err(|e| {
            CoreError::validation(format!(
                "Failed to serialize AstSlice for token estimation: {}",
                e
            ))
        })?;
        let estimated_tokens = json_bytes.len().saturating_add(3) / 4;
        slice.estimated_tokens = estimated_tokens;

        if estimated_tokens > max_tokens {
            return Err(CoreError::validation(format!(
                "AstSlice for '{}' requires {} estimated tokens, exceeding max_tokens budget of {}",
                target, estimated_tokens, max_tokens
            )));
        }

        Ok(slice)
    }
}
