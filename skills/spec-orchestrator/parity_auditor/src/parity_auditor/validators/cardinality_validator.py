"""
Validator that enforces 1:1 mapping between schema containers/cases and spec files,
SysML package structure & subsystem allocation (Check 18), and Feature operation &
typed parameter coverage for Embedded Coder and DO-178C synthesis (Check 19).
"""

import os
import re
import sys
from typing import List, Dict, Any, Optional, Set, Tuple

import yaml

from .base import IValidator
from ..core.findings import Finding
from ..core.workspace import WorkspaceRepository

# Import SysML v2 AST classes safely
try:
    from sysmlv2_ast import (
        SysMLPackage, SysMLParser, SysMLCapabilityDef, ActionDef,
        SysMLOperationDef, SysMLConstraintDef, PartDef, AttributeDef
    )
except ImportError:
    try:
        from skills.spec_orchestrator.scripts.sysmlv2_ast import (
            SysMLPackage, SysMLParser, SysMLCapabilityDef, ActionDef,
            SysMLOperationDef, SysMLConstraintDef, PartDef, AttributeDef
        )
    except ImportError:
        _script_dir = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", "..", "..", "scripts"))
        if _script_dir not in sys.path:
            sys.path.insert(0, _script_dir)
        try:
            from sysmlv2_ast import (
                SysMLPackage, SysMLParser, SysMLCapabilityDef, ActionDef,
                SysMLOperationDef, SysMLConstraintDef, PartDef, AttributeDef
            )
        except ImportError:
            SysMLPackage = None
            SysMLParser = None
            SysMLCapabilityDef = None
            ActionDef = None
            SysMLOperationDef = None
            SysMLConstraintDef = None
            PartDef = None
            AttributeDef = None


def _extract_frontmatter(content: str):
    m = re.match(r"^---\s*\n(.*?)\n---\s*\n", content, re.DOTALL)
    if not m:
        return None
    try:
        return yaml.safe_load(m.group(1).replace('\x01', ''))
    except Exception:
        return None


def _find_sysml_files(repo: WorkspaceRepository, schemas_dir: Optional[str] = None) -> List[str]:
    sysml_files = []
    if schemas_dir and os.path.exists(schemas_dir):
        if os.path.isfile(schemas_dir) and schemas_dir.endswith(".sysml"):
            sysml_files.append(schemas_dir)
        elif os.path.isdir(schemas_dir):
            for f in sorted(os.listdir(schemas_dir)):
                if f.endswith(".sysml") and not f.startswith("."):
                    sysml_files.append(os.path.join(schemas_dir, f))

    if not sysml_files:
        pipeline_sysml = os.path.join(repo.workspace_dir, ".pipeline", "schema.sysml")
        if os.path.exists(pipeline_sysml):
            sysml_files.append(pipeline_sysml)

    return sysml_files


def _collect_sysml_elements(pkg: Any) -> Tuple[List[Any], List[Any], List[Any], List[Any], List[Any], List[Any]]:
    """Recursively collect (packages, parts, capabilities, actions, operations, constraints) from a SysMLPackage."""
    if not pkg:
        return [], [], [], [], [], []

    packages = [pkg]
    parts = list(getattr(pkg, "part_defs", []) or [])
    capabilities = list(getattr(pkg, "capability_defs", []) or [])
    actions = list(getattr(pkg, "action_defs", []) or [])
    operations = list(getattr(pkg, "operation_defs", []) or [])
    constraints = list(getattr(pkg, "constraint_defs", []) or [])

    def _traverse_part(p: Any):
        for sub in (getattr(p, "parts", []) or []):
            parts.append(sub)
            _traverse_part(sub)
        for cap in (getattr(p, "capabilities", []) or []):
            capabilities.append(cap)
        for act in (getattr(p, "actions", []) or []):
            actions.append(act)
        for op in (getattr(p, "operations", []) or []):
            operations.append(op)
        for c in (getattr(p, "constraints", []) or []):
            constraints.append(c)

    for p in (getattr(pkg, "part_defs", []) or []):
        _traverse_part(p)

    for sub_pkg in (getattr(pkg, "sub_packages", []) or []):
        sub_pks, sub_pts, sub_caps, sub_acts, sub_ops, sub_cons = _collect_sysml_elements(sub_pkg)
        packages.extend(sub_pks)
        parts.extend(sub_pts)
        capabilities.extend(sub_caps)
        actions.extend(sub_acts)
        operations.extend(sub_ops)
        constraints.extend(sub_cons)

    return packages, parts, capabilities, actions, operations, constraints


def _load_all_sysml_elements(repo: WorkspaceRepository, schemas_dir: Optional[str] = None) -> Tuple[List[str], List[Any], List[Any], List[Any], List[Any], List[Any], List[Any], List[Finding]]:
    sysml_files = _find_sysml_files(repo, schemas_dir)
    errors: List[Finding] = []
    all_pkgs: List[Any] = []
    all_parts: List[Any] = []
    all_caps: List[Any] = []
    all_actions: List[Any] = []
    all_ops: List[Any] = []
    all_constraints: List[Any] = []

    for sf in sysml_files:
        try:
            with open(sf, "r", encoding="utf-8") as file:
                content = file.read()
            if SysMLParser:
                pkg = SysMLParser.parse_text(content, default_name=os.path.splitext(os.path.basename(sf))[0])
            else:
                pkg = SysMLPackage(name=os.path.splitext(os.path.basename(sf))[0]) if SysMLPackage else None
            pks, pts, caps, acts, ops, cons = _collect_sysml_elements(pkg)
            all_pkgs.extend(pks)
            all_parts.extend(pts)
            all_caps.extend(caps)
            all_actions.extend(acts)
            all_ops.extend(ops)
            all_constraints.extend(cons)
        except Exception as exc:
            errors.append(Finding(
                "sysml-model-not-readable",
                f"Failed to read SysML model file '{os.path.basename(sf)}': {exc}",
                location="schemas"
            ))

    return sysml_files, all_pkgs, all_parts, all_caps, all_actions, all_ops, all_constraints, errors


class SchemaCardinalityValidator(IValidator):
    def validate_package_structure_and_subsystem_allocation(self, repo: WorkspaceRepository, **kwargs) -> List[Finding]:
        """
        Check 18: Package Structure & Subsystem Allocation Audit.
        Verifies that every Epic is allocated to valid subsystem packages and references
        existing capability def blocks declared in the SysML AST.
        """
        rules = repo.get_codebase_rules()
        backlog_dirs = rules.backlog_directories
        schemas_dir_rel = getattr(backlog_dirs, "schemas", None)
        schemas_dir = os.path.join(repo.workspace_dir, schemas_dir_rel) if schemas_dir_rel else os.path.join(repo.workspace_dir, "schemas")
        epics_dir_rel = getattr(backlog_dirs, "epics", None)
        epics_dir = os.path.join(repo.workspace_dir, epics_dir_rel) if epics_dir_rel else os.path.join(repo.workspace_dir, "docs", "epics")

        sysml_files, all_pkgs, all_parts, all_caps, _, _, _, errors = _load_all_sysml_elements(repo, schemas_dir)
        if errors:
            return errors
        if not sysml_files:
            return []

        # Collect valid package names and capability names
        valid_packages: Set[str] = set()
        for p in all_pkgs:
            if hasattr(p, "name") and p.name:
                valid_packages.add(p.name)
        for c in all_caps:
            subsys = getattr(c, "subsystem", "") or getattr(c, "package_ref", "")
            if subsys:
                valid_packages.add(subsys)

        caps_by_package: Dict[str, List[Any]] = {}
        all_caps_map: Dict[str, Any] = {}
        for c in all_caps:
            c_name = getattr(c, "name", "")
            if c_name:
                all_caps_map[c_name] = c
                subsys = getattr(c, "subsystem", "") or getattr(c, "package_ref", "")
                if subsys:
                    caps_by_package.setdefault(subsys.lower(), []).append(c)

        if not epics_dir or not os.path.exists(epics_dir):
            epic_files = []
        else:
            epic_files = [f for f in sorted(os.listdir(epics_dir)) if f.endswith(".md")]

        if not epic_files:
            return []

        referenced_capabilities: Set[str] = set()

        for filename in epic_files:
            filepath = os.path.join(epics_dir, filename)
            try:
                with open(filepath, "r", encoding="utf-8") as f:
                    content = f.read()
            except Exception as exc:
                errors.append(Finding(
                    "epic-not-readable",
                    f"Failed to read Epic specification '{filename}': {exc}",
                    location="epics"
                ))
                continue

            fm = _extract_frontmatter(content) or {}
            
            # Determine allocated package/subsystem
            allocated_pkg = (
                fm.get("package")
                or fm.get("subsystem")
                or fm.get("schema_package")
            )

            if not allocated_pkg:
                # Check for package in title, headers, or Subsystem Capability Allocations table
                for pkg_cand in valid_packages:
                    if re.search(rf"\b{re.escape(pkg_cand)}\b", content, re.IGNORECASE):
                        allocated_pkg = pkg_cand
                        break

            if allocated_pkg:
                # Validate against valid_packages
                pkg_match = None
                for vp in valid_packages:
                    if vp.lower() == str(allocated_pkg).lower():
                        pkg_match = vp
                        break
                if not pkg_match:
                    errors.append(Finding(
                        "epic-subsystem-package-invalid",
                        f"Epic '{filename}': Allocated subsystem package '{allocated_pkg}' is not defined in SysML model. Valid packages: {sorted(list(valid_packages))}.",
                        location="epics"
                    ))
                else:
                    # Check capability allocation for this package
                    expected_caps = caps_by_package.get(pkg_match.lower(), [])
                    for exp_cap in expected_caps:
                        cap_name = getattr(exp_cap, "name", "")
                        if cap_name and not re.search(rf"\b{re.escape(cap_name)}\b", content):
                            errors.append(Finding(
                                "epic-capability-allocation-missing",
                                f"Epic '{filename}': Missing formal capability def '{cap_name}' declared in subsystem package '{pkg_match}'.",
                                location="epics"
                            ))
            else:
                if valid_packages:
                    errors.append(Finding(
                        "epic-package-allocation-missing",
                        f"Epic '{filename}': Missing subsystem package allocation. Every Epic must map to a valid SysML package boundary (e.g. {sorted(list(valid_packages))}).",
                        location="epics"
                    ))

            # Scan for capability references in Epic
            cap_refs = re.findall(r'(?:capability\s+def|capability\s*:?)\s+([A-Za-z0-9_]+)', content)
            # Also scan table rows: | CapName | Subsystem |
            table_caps = re.findall(r'\|\s*([A-Za-z0-9_]+)\s*\|\s*[A-Za-z0-9_]+\s*\|', content)
            all_found_refs = set(cap_refs + table_caps)

            for ref in all_found_refs:
                if ref in ("Capability", "Name", "capability_def", "SubsystemComponent"):
                    continue
                if all_caps_map and ref not in all_caps_map:
                    errors.append(Finding(
                        "epic-capability-unknown",
                        f"Epic '{filename}': References unknown capability def '{ref}' not found in SysML AST.",
                        location="epics"
                    ))
                else:
                    referenced_capabilities.add(ref)

            # Also add any AST capabilities present in content
            for c_name in all_caps_map:
                if re.search(rf"\b{re.escape(c_name)}\b", content):
                    referenced_capabilities.add(c_name)

        # Bidirectional check: ensure all SysML capability defs are allocated in at least one Epic
        for c_name, c_obj in all_caps_map.items():
            if c_name not in referenced_capabilities:
                subsys = getattr(c_obj, "subsystem", "") or getattr(c_obj, "package_ref", "") or "unspecified"
                errors.append(Finding(
                    "sysml-capability-unallocated",
                    f"SysML capability def '{c_name}' (subsystem '{subsys}') is not allocated to any Epic specification.",
                    location="epics"
                ))

        return errors

    def validate_feature_operation_and_constraint_coverage(self, repo: WorkspaceRepository, **kwargs) -> List[Finding]:
        """
        Check 19: Feature Operation & Schema Constraint Coverage.
        Enforces 100% typed interface parameter coverage for Embedded Coder and DO-178C code synthesis.
        Mandates that Feature specifications extract and bind formal action def operations with
        typed parameter signatures (in, out, parameter types) and constraints from the SysML AST.
        """
        rules = repo.get_codebase_rules()
        backlog_dirs = rules.backlog_directories
        schemas_dir_rel = getattr(backlog_dirs, "schemas", None)
        schemas_dir = os.path.join(repo.workspace_dir, schemas_dir_rel) if schemas_dir_rel else os.path.join(repo.workspace_dir, "schemas")
        features_dir_rel = getattr(backlog_dirs, "features", None)
        features_dir = os.path.join(repo.workspace_dir, features_dir_rel) if features_dir_rel else os.path.join(repo.workspace_dir, "docs", "features")

        sysml_files, all_pkgs, all_parts, _, all_actions, all_ops, all_constraints, errors = _load_all_sysml_elements(repo, schemas_dir)
        if errors:
            return errors
        if not sysml_files:
            return []

        if not features_dir or not os.path.exists(features_dir):
            feature_files = []
        else:
            feature_files = [f for f in sorted(os.listdir(features_dir)) if f.endswith(".md")]

        if not feature_files:
            return []

        # Map parts by name
        parts_map: Dict[str, Any] = {}
        for p in all_parts:
            p_name = getattr(p, "name", "")
            if p_name:
                parts_map[p_name] = p

        extracted_actions: Set[str] = set()

        for filename in feature_files:
            filepath = os.path.join(features_dir, filename)
            try:
                with open(filepath, "r", encoding="utf-8") as f:
                    content = f.read()
            except Exception as exc:
                errors.append(Finding(
                    "feature-not-readable",
                    f"Failed to read Feature specification '{filename}': {exc}",
                    location="features"
                ))
                continue

            fm = _extract_frontmatter(content) or {}
            
            # Determine which PartDef(s) this feature specifies
            matched_parts: List[Any] = []
            containers = fm.get("schema_containers", [])
            if isinstance(containers, list):
                for c in containers:
                    c_path = c.get("path", "") if isinstance(c, dict) else str(c)
                    leaf = c_path.split("/")[-1] if "/" in c_path else c_path
                    if ":" in leaf:
                        leaf = leaf.split(":", 1)[-1]
                    for p_name, p_obj in parts_map.items():
                        if p_name.lower() == leaf.lower():
                            matched_parts.append(p_obj)

            # Also match by class diagram classes or part names in text
            if not matched_parts:
                for p_name, p_obj in parts_map.items():
                    if re.search(rf"\b(?:class|part\s+def|part)\s+{re.escape(p_name)}\b", content) or re.search(rf"\b{re.escape(p_name)}\b", filename, re.IGNORECASE):
                        matched_parts.append(p_obj)

            for part in matched_parts:
                part_name = getattr(part, "name", "")
                part_actions = list(getattr(part, "actions", []) or [])
                part_ops = list(getattr(part, "operations", []) or [])
                part_constraints = list(getattr(part, "constraints", []) or [])

                # 1. Check Action & Operation coverage
                for act in (part_actions + part_ops):
                    act_name = getattr(act, "name", "")
                    if not act_name:
                        continue

                    # Check if action name exists in feature file
                    if not re.search(rf"\b{re.escape(act_name)}\b", content):
                        errors.append(Finding(
                            "feature-operation-missing",
                            f"Feature '{filename}': SysML action/operation '{act_name}' on part '{part_name}' is not extracted or bound in feature specification.",
                            location="features"
                        ))
                        continue

                    extracted_actions.add(act_name)

                    # 2. Check 100% Typed Parameter Coverage (Embedded Coder / DO-178C)
                    in_params = getattr(act, "in_params", []) or []
                    out_params = getattr(act, "out_params", []) or []
                    op_params = getattr(act, "parameters", []) or []
                    all_params = list(in_params) + list(out_params) + list(op_params)

                    for param in all_params:
                        p_name = getattr(param, "name", "")
                        p_type = getattr(param, "type_name", "")
                        if not p_name or not p_type:
                            continue

                        # Check if parameter name and its type are documented in feature content
                        param_typed_match = (
                            re.search(rf"\b{re.escape(p_name)}\s*:\s*{re.escape(p_type)}\b", content, re.IGNORECASE)
                            or re.search(rf"\b{re.escape(p_type)}\s+{re.escape(p_name)}\b", content, re.IGNORECASE)
                            or re.search(rf"\|\s*{re.escape(p_name)}\s*\|.*?\|\s*{re.escape(p_type)}\b", content, re.IGNORECASE)
                            or (re.search(rf"\b{re.escape(p_name)}\b", content, re.IGNORECASE) and re.search(rf"\b{re.escape(p_type)}\b", content))
                        )

                        if not param_typed_match:
                            errors.append(Finding(
                                "feature-operation-param-untyped",
                                f"Feature '{filename}': Parameter '{p_name}' of operation '{act_name}' is missing typed signature ({p_type}) required for Embedded Coder/DO-178C code synthesis.",
                                location="features"
                            ))

                # 3. Check Constraint Coverage
                for con in part_constraints:
                    c_name = getattr(con, "name", "")
                    c_expr = getattr(con, "expression", "")
                    if not c_name:
                        continue

                    norm_expr = re.sub(r'\s+', '', c_expr.strip().rstrip(';')) if c_expr else ""
                    norm_content = re.sub(r'\s+', '', content)

                    con_found = bool(
                        re.search(rf"\b{re.escape(c_name)}\b", content)
                        or (norm_expr and len(norm_expr) >= 4 and norm_expr in norm_content)
                    )

                    if not con_found:
                        errors.append(Finding(
                            "feature-constraint-coverage-missing",
                            f"Feature '{filename}': SysML constraint '{c_name}' on part '{part_name}' is not covered in feature specification.",
                            location="features"
                        ))

        # Bidirectional check: ensure all actions on parts in SysML AST are extracted in at least one feature
        for p in all_parts:
            p_name = getattr(p, "name", "")
            for act in (getattr(p, "actions", []) or []):
                act_name = getattr(act, "name", "")
                if act_name and act_name not in extracted_actions:
                    errors.append(Finding(
                        "sysml-action-unextracted",
                        f"SysML action def '{act_name}' on part '{p_name}' is not extracted into any feature specification.",
                        location="features"
                    ))

        return errors

    def validate(self, repo: WorkspaceRepository, **kwargs) -> List[str]:
        is_sysml = kwargs.get("is_sysml", False)
        rules = repo.get_codebase_rules()
        backlog_dirs = rules.backlog_directories
        
        schemas_dir_rel = getattr(backlog_dirs, "schemas", None)
        schemas_dir = os.path.join(repo.workspace_dir, schemas_dir_rel) if schemas_dir_rel else os.path.join(repo.workspace_dir, "schemas")
        features_dir_rel = getattr(backlog_dirs, "features", None)
        features_dir = os.path.join(repo.workspace_dir, features_dir_rel) if features_dir_rel else None

        errors = []

        if is_sysml:
            errors.extend(self.validate_package_structure_and_subsystem_allocation(repo, **kwargs))
            errors.extend(self.validate_feature_operation_and_constraint_coverage(repo, **kwargs))

        if not os.path.exists(schemas_dir):
            return errors
        schema_files = [f for f in os.listdir(schemas_dir) if not f.startswith('.')]
        if not schema_files:
            return errors
        use_cases_dir_rel = getattr(backlog_dirs, "use_cases", None)
        use_cases_dir = (
            os.path.join(repo.workspace_dir, use_cases_dir_rel)
            if use_cases_dir_rel
            else None
        )

        for dir_label, target_dir, file_type in [
            ("features", features_dir, "Feature"),
            ("use-cases", use_cases_dir, "Use Case"),
        ]:
            if not target_dir or not os.path.exists(target_dir):
                continue
            for filename in sorted(os.listdir(target_dir)):
                if not filename.endswith(".md"):
                    continue
                filepath = os.path.join(target_dir, filename)
                try:
                    with open(filepath, "r", encoding="utf-8") as f:
                        content = f.read()
                except Exception:
                    continue

                fm = _extract_frontmatter(content)
                if fm is None:
                    continue

                containers = fm.get("schema_containers", None)
                if containers is None:
                    errors.append(Finding(
                        "schema-container-declaration-missing",
                        f"{file_type} '{filename}': schema_containers is missing from frontmatter. "
                        f"Every {file_type.lower()} must declare exactly one schema container "
                        f"(e.g. path: 'module/container', node_type: container).",
                        location=dir_label,
                    ))
                    continue

                if not isinstance(containers, list):
                    errors.append(Finding(
                        "schema-container-field-must-be-a-list",
                        f"{file_type} '{filename}': schema_containers must be a list, "
                        f"got {type(containers).__name__}",
                        location=dir_label,
                    ))
                    continue

                n = len(containers)
                if n == 0:
                    errors.append(Finding(
                        "schema-container-declaration-empty",
                        f"{file_type} '{filename}': schema_containers is empty. "
                        f"Every {file_type.lower()} must declare at least one schema container "
                        f"(e.g. path: 'module/container', node_type: container).",
                        location=dir_label,
                    ))
                elif n > 1:
                    errors.append(Finding(
                        "schema-container-consolidation-forbidden",
                        f"{file_type} '{filename}': schema_containers declares {n} containers. "
                        f"Every {file_type.lower()} must declare exactly one. Split the "
                        f"consolidated containers into separate files, one per container.",
                        location=dir_label,
                    ))

        return errors

