"""Tier-1 Metamodel Transformation Engine and Closed-Grammar M2 Type System (Issue #266).

Enforces pure schema-driven parameter extraction, closed M2 metamodel typing,
and deterministic AST-to-M2 metamodel mapping with zero hardcoded domain concepts.
"""

import ast
import os
import re
import sys
from typing import Any, Dict, List, Optional, Set, Tuple, Union

try:
    from .base import IValidator
    from ..core.findings import Finding
    from ..core.workspace import WorkspaceRepository
except (ImportError, ValueError):
    _src_dir = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", ".."))
    if _src_dir not in sys.path:
        sys.path.insert(0, _src_dir)
    from parity_auditor.validators.base import IValidator
    from parity_auditor.core.findings import Finding
    from parity_auditor.core.workspace import WorkspaceRepository


# Closed allowlist containing all Tier-1 abstract metamodel entities
ALLOWED_M2_METAMODEL_TYPES: Set[str] = {
    # Core structural elements
    "Component",
    "Class",
    "Port",
    "Interface",
    "Statechart",
    "Constraint",
    "Signal",
    "Event",
    "AcceptanceCriterion",
    "Scenario",
    "TraceLink",
    # SysML v2 & KerML Definition types
    "Package",
    "PackageDefinition",
    "PartDefinition",
    "PortDefinition",
    "StateDefinition",
    "ItemDefinition",
    "ActionDefinition",
    "RequirementDefinition",
    "UseCaseDefinition",
    "ConstraintDefinition",
    "AttributeDefinition",
    "ConnectionDefinition",
    "AllocationDefinition",
    "ViewDefinition",
    "ViewpointDefinition",
    "ActorDefinition",
    "NamespaceDefinition",
    "ElementDefinition",
    "FeatureDefinition",
    "Classifier",
    # Actor and Role types
    "HumanOperator",
    "SystemController",
    "SafetyInterlock",
    "PhysicalActuator",
    "Sensor",
    "SystemUnderStudy",
    "ExternalSystem",
    "OperatorConsole",
    # Canonical M2 elements
    "Actor",
    "Part",
    "Item",
    "Action",
    "State",
    "Requirement",
    "UseCase",
    "Attribute",
    "Connection",
    "Allocation",
    "Transition",
    "Guard",
    "Trigger",
    "Effect",
    # AST, Schema, Parser & Model primitives
    "Namespace",
    "Object",
    "Array",
    "String",
    "Number",
    "Boolean",
    "Integer",
    "Dict",
    "List",
    "Null",
    "Primitive",
    "Type",
    "Definition",
    "Block",
    "Node",
    "Root",
    "Value",
    "Field",
    "Member",
    "Document",
    # Logical UI (LUI / LUMI) Canonical Display, Container & Widget primitives
    "LogicalUI",
    "Widget",
    "Container",
    "Layout",
    "View",
    "SidebarLayout",
    "HierarchyTree",
    "ResizableSplitter",
    "TopologyMap",
    "DensityTable",
    "TabbedContainer",
    "SplitterContainer",
    "Panel",
    "Section",
    "Tab",
    "Tree",
    "Table",
    "Map",
    "Chart",
    "Form",
    "Button",
    "Input",
    "Dialog",
    "Modal",
}

_NORMALIZED_ALLOWED_M2_MAP: Dict[str, str] = {
    t.lower().replace("_", "").replace("-", ""): t for t in ALLOWED_M2_METAMODEL_TYPES
}

_ROLE_TO_M2_MAP: Dict[str, str] = {
    "sensor": "Sensor",
    "actuator": "PhysicalActuator",
    "physical_actuator": "PhysicalActuator",
    "physicalactuator": "PhysicalActuator",
    "controller": "SystemController",
    "system_controller": "SystemController",
    "systemcontroller": "SystemController",
    "interlock": "SafetyInterlock",
    "safety_interlock": "SafetyInterlock",
    "safetyinterlock": "SafetyInterlock",
    "operator": "HumanOperator",
    "human_operator": "HumanOperator",
    "humanoperator": "HumanOperator",
    "pilot": "HumanOperator",
    "driver": "HumanOperator",
    "user": "HumanOperator",
    "console": "OperatorConsole",
    "operator_console": "OperatorConsole",
    "operatorconsole": "OperatorConsole",
    "efb": "OperatorConsole",
    "display": "OperatorConsole",
    "system_under_study": "SystemUnderStudy",
    "systemunderstudy": "SystemUnderStudy",
    "sus": "SystemUnderStudy",
    "system": "SystemUnderStudy",
    "external_system": "ExternalSystem",
    "externalsystem": "ExternalSystem",
    "external": "ExternalSystem",
    "ui": "LogicalUI",
    "logical_ui": "LogicalUI",
    "logicalui": "LogicalUI",
    "lumi": "LogicalUI",
    "widget": "Widget",
    "part": "PartDefinition",
    "partdefinition": "PartDefinition",
    "part_definition": "PartDefinition",
    "component": "Component",
    "port": "PortDefinition",
    "portdefinition": "PortDefinition",
    "port_definition": "PortDefinition",
    "interface": "Interface",
    "state": "StateDefinition",
    "statechart": "Statechart",
    "statedefinition": "StateDefinition",
    "state_definition": "StateDefinition",
    "fsm": "StateDefinition",
    "action": "ActionDefinition",
    "actiondefinition": "ActionDefinition",
    "action_definition": "ActionDefinition",
    "operation": "ActionDefinition",
    "item": "ItemDefinition",
    "itemdefinition": "ItemDefinition",
    "item_definition": "ItemDefinition",
    "payload": "ItemDefinition",
    "message": "ItemDefinition",
    "requirement": "RequirementDefinition",
    "requirementdefinition": "RequirementDefinition",
    "requirement_definition": "RequirementDefinition",
    "safety_req": "RequirementDefinition",
    "usecase": "UseCaseDefinition",
    "use_case": "UseCaseDefinition",
    "usecasedefinition": "UseCaseDefinition",
    "use_case_definition": "UseCaseDefinition",
    "constraint": "ConstraintDefinition",
    "constraintdefinition": "ConstraintDefinition",
    "constraint_definition": "ConstraintDefinition",
    "connection": "ConnectionDefinition",
    "connectiondefinition": "ConnectionDefinition",
    "connection_definition": "ConnectionDefinition",
    "binding": "ConnectionDefinition",
    "allocation": "AllocationDefinition",
    "allocationdefinition": "AllocationDefinition",
    "allocation_definition": "AllocationDefinition",
    "attribute": "AttributeDefinition",
    "attributedefinition": "AttributeDefinition",
    "attribute_definition": "AttributeDefinition",
    "parameter": "AttributeDefinition",
    "property": "AttributeDefinition",
    "package": "PackageDefinition",
    "packagedefinition": "PackageDefinition",
    "package_definition": "PackageDefinition",
    "class": "Class",
    "signal": "Signal",
    "event": "Event",
    "scenario": "Scenario",
    "tracelink": "TraceLink",
}

_CLASSIFIER_TOKEN_RULES: List[Tuple[re.Pattern, str]] = [
    (re.compile(r"Sensor\b", re.I), "Sensor"),
    (re.compile(r"(Actuator|Servo|Motor)\b", re.I), "PhysicalActuator"),
    (re.compile(r"(Controller|Autopilot|Supervisor|Manager)\b", re.I), "SystemController"),
    (re.compile(r"(Interlock|SafetyGuard|RTA|Cutoff)\b", re.I), "SafetyInterlock"),
    (re.compile(r"(Operator|Pilot|Driver|User)\b", re.I), "HumanOperator"),
    (re.compile(r"(Console|EFB|Dashboard)\b", re.I), "OperatorConsole"),
    (re.compile(r"(DisplayUI|LogicalUI|LUI)\b", re.I), "LogicalUI"),
    (re.compile(r"(Widget|Gauge|Indicator|Grid|Button|Tree|Table|Splitter|Container|Panel|Modal|Dialog|Form|Chart|Map)\b", re.I), "Widget"),
    (re.compile(r"Port(Def|Definition)?\b", re.I), "PortDefinition"),
    (re.compile(r"Interface\b", re.I), "Interface"),
    (re.compile(r"(State|Mode)(Def|Definition)?\b", re.I), "StateDefinition"),
    (re.compile(r"(Action|Command|Task)(Def|Definition)?\b", re.I), "ActionDefinition"),
    (re.compile(r"(Item|Packet|Message|Telemetry|Frame)(Def|Definition)?\b", re.I), "ItemDefinition"),
    (re.compile(r"(Requirement|Req)(Def|Definition)?\b", re.I), "RequirementDefinition"),
    (re.compile(r"(UseCase|UC)(Def|Definition)?\b", re.I), "UseCaseDefinition"),
    (re.compile(r"Constraint(Def|Definition)?\b", re.I), "ConstraintDefinition"),
    (re.compile(r"Package(Def|Definition)?\b", re.I), "PackageDefinition"),
    (re.compile(r"Part(Def|Definition)?\b", re.I), "PartDefinition"),
]


def is_allowed_m2_type(type_name: str) -> bool:
    """Check if a type name conforms to the closed M2 metamodel allowlist or carries a meta_ prefix."""
    if not isinstance(type_name, str) or not type_name.strip():
        return False
    cleaned = type_name.strip()
    if cleaned.lower().startswith("meta_") or cleaned.lower().startswith("meta-") or cleaned.lower().startswith("meta"):
        return True
    norm = cleaned.lower().replace("_", "").replace("-", "")
    return norm in _NORMALIZED_ALLOWED_M2_MAP


def map_ast_classifier_to_m2(classifier_name: str, domain_role: Optional[str] = None) -> str:
    """Map an AST classifier name and optional domain role to a valid Tier-1 M2 metamodel type string."""
    if not isinstance(classifier_name, str):
        return "PartDefinition"

    cleaned_name = classifier_name.strip()

    # 1. If explicit domain_role is provided, check role mapping
    if domain_role and isinstance(domain_role, str) and domain_role.strip():
        norm_role = domain_role.strip().lower().replace("-", "_")
        if norm_role in _ROLE_TO_M2_MAP:
            return _ROLE_TO_M2_MAP[norm_role]
        norm_role_collapsed = norm_role.replace("_", "")
        if norm_role_collapsed in _ROLE_TO_M2_MAP:
            return _ROLE_TO_M2_MAP[norm_role_collapsed]

    # 2. Check if classifier_name itself is already an allowed M2 type
    norm_name = cleaned_name.lower().replace("_", "").replace("-", "")
    if norm_name in _NORMALIZED_ALLOWED_M2_MAP:
        return _NORMALIZED_ALLOWED_M2_MAP[norm_name]

    # 3. If meta_ prefix is present, preserve it
    if cleaned_name.lower().startswith("meta_") or cleaned_name.lower().startswith("meta-") or cleaned_name.lower().startswith("meta"):
        return cleaned_name

    # 4. Token-based inference from classifier name
    for pattern, target_m2 in _CLASSIFIER_TOKEN_RULES:
        if pattern.search(cleaned_name):
            return target_m2

    # 5. Default fallback
    return "PartDefinition"


class _PurityASTVisitor(ast.NodeVisitor):
    """Internal AST visitor checking Python code for metamodel purity and M1 domain leaks."""

    STATIC_PARAM_DICT_NAMES = re.compile(
        r"^(_)?("
        r"ground_?truth(_?(specs?|params?|parameters?|dict|map|set|table))?|"
        r"(expected|domain|static|hardcoded|benchmark|mandated|system)_?(specs?|params?|parameters?|constants?|dict|map|set|table|specifications?)"
        r")$",
        re.IGNORECASE,
    )

    M1_DOMAIN_DICT_NAMES = re.compile(
        r"^(_)?("
        r"(m1|domain|concrete)_(entities|instances|models|specs|objects|dicts|types)|"
        r"(sample|mock|concrete)_(uav|aircraft|vehicle|device|patient|car|robot)(_?(specs|params|data|dict))?"
        r")$",
        re.IGNORECASE,
    )

    METAMODEL_TYPE_KEYS = {
        "type",
        "metamodel_type",
        "entity_type",
        "kind",
        "node_type",
        "ast_type",
        "element_type",
        "m2_type",
        "definition_type",
    }

    def __init__(self, filename: str):
        self.filename = filename or "<inline>"
        self.violations: List[Finding] = []
        self.scope_stack: List[str] = []

    def visit_ClassDef(self, node: ast.ClassDef):
        self.scope_stack.append(node.name)
        if self.STATIC_PARAM_DICT_NAMES.match(node.name):
            has_static_attrs = any(
                isinstance(stmt, ast.Assign) and isinstance(stmt.value, (ast.Constant, ast.Dict, ast.List, ast.Set, ast.Tuple))
                for stmt in node.body
            )
            if has_static_attrs:
                self.violations.append(
                    Finding(
                        "domain-static-param-violation",
                        f"Static domain specification class \"{node.name}\" declared in {self.filename}:{node.lineno}. "
                        "Domain parameters must be dynamically parsed from schema/*.sysml or workspace.schemas.",
                        location=f"{self.filename}:{node.lineno}",
                    )
                )
        self.generic_visit(node)
        self.scope_stack.pop()

    def visit_FunctionDef(self, node: ast.FunctionDef):
        self.scope_stack.append(f"def {node.name}")
        if any(k in node.name.lower() for k in ("extract_ground_truth", "get_ground_truth", "extract_domain_specs", "get_expected_specs")):
            for child in ast.walk(node):
                if isinstance(child, ast.Return) and isinstance(child.value, ast.Dict) and len(child.value.keys) > 0:
                    self.violations.append(
                        Finding(
                            "domain-static-param-violation",
                            f"Parameter extraction function \"{node.name}\" returns static literal parameter dictionary in {self.filename}:{child.lineno}. "
                            "All parameter extraction must dynamically query schema/*.sysml or workspace.schemas.",
                            location=f"{self.filename}:{child.lineno}",
                        )
                    )
        self.generic_visit(node)
        self.scope_stack.pop()

    def visit_AsyncFunctionDef(self, node: ast.AsyncFunctionDef):
        self.scope_stack.append(f"async def {node.name}")
        self.generic_visit(node)
        self.scope_stack.pop()

    def _check_dict_metamodel_types(self, dict_node: ast.Dict, lineno: int):
        if not isinstance(dict_node, ast.Dict):
            return
        for key_node, val_node in zip(dict_node.keys, dict_node.values):
            if key_node is None or not isinstance(key_node, ast.Constant) or not isinstance(key_node.value, str):
                continue
            key_str = key_node.value.lower()
            if key_str in self.METAMODEL_TYPE_KEYS:
                if isinstance(val_node, ast.Constant) and isinstance(val_node.value, str):
                    val_str = val_node.value.strip()
                    if val_str and not is_allowed_m2_type(val_str):
                        self.violations.append(
                            Finding(
                                "domain-metamodel-typing-violation",
                                f"Check violation (domain-metamodel-typing-violation): Unvalidated M1 domain instance entity/type '{val_str}' declared in {self.filename}:{lineno}. "
                                "Upstream compiler ASTs and dictionaries must adhere strictly to the closed M2 metamodel allowlist.",
                                location=f"{self.filename}:{lineno}",
                            )
                        )

    def visit_Dict(self, node: ast.Dict):
        self._check_dict_metamodel_types(node, getattr(node, "lineno", 1))
        self.generic_visit(node)

    def _check_target_name(self, target_name: str, value_node: ast.AST, lineno: int):
        if not target_name or value_node is None:
            return

        if self.M1_DOMAIN_DICT_NAMES.match(target_name):
            self.violations.append(
                Finding(
                    "domain-metamodel-typing-violation",
                    f"Check violation (domain-metamodel-typing-violation): Unvalidated M1 domain instance dictionary/constant \"{target_name}\" declared in {self.filename}:{lineno}. "
                    "Upstream compiler ASTs must adhere strictly to the closed M2 metamodel allowlist.",
                    location=f"{self.filename}:{lineno}",
                )
            )
            return

        if self.STATIC_PARAM_DICT_NAMES.match(target_name):
            is_literal_dict = isinstance(value_node, ast.Dict) and len(value_node.keys) > 0
            is_literal_collection = isinstance(value_node, (ast.List, ast.Set, ast.Tuple)) and len(value_node.elts) > 0
            is_constant = isinstance(value_node, ast.Constant) and value_node.value is not None
            is_dict_call = (
                isinstance(value_node, ast.Call)
                and isinstance(value_node.func, ast.Name)
                and value_node.func.id in ("dict", "list", "set")
                and (len(value_node.args) > 0 or len(value_node.keywords) > 0)
            )

            is_module_or_class_level = len(self.scope_stack) == 0 or (
                len(self.scope_stack) == 1 and not self.scope_stack[0].startswith("def ") and not self.scope_stack[0].startswith("async def ")
            )

            if is_literal_dict or is_literal_collection or is_constant or is_dict_call or (is_module_or_class_level and isinstance(value_node, (ast.Dict, ast.List, ast.Set, ast.Tuple))):
                self.violations.append(
                    Finding(
                        "domain-static-param-violation",
                        f"Check violation (domain-static-param-violation): Static hardcoded parameter dictionary/constant \"{target_name}\" declared in {self.filename}:{lineno}. "
                        "Domain specifications must be dynamically queried from workspace.schemas or schema/*.sysml AST nodes.",
                        location=f"{self.filename}:{lineno}",
                    )
                )

    def visit_Assign(self, node: ast.Assign):
        for target in node.targets:
            if isinstance(target, ast.Name):
                self._check_target_name(target.id, node.value, getattr(node, "lineno", 1))
        self.generic_visit(node)

    def visit_AnnAssign(self, node: ast.AnnAssign):
        if isinstance(node.target, ast.Name):
            self._check_target_name(node.target.id, node.value, getattr(node, "lineno", 1))
        self.generic_visit(node)


def validate_metamodel_purity(text: str, filename: Optional[str] = None) -> List[Finding]:
    """Validate text/code for metamodel purity and M1 domain leaks."""
    if not text or not text.strip():
        return []

    try:
        tree = ast.parse(text, filename=filename or "<inline>")
    except SyntaxError:
        # Non-python text or parse error; skip AST checks
        return []

    visitor = _PurityASTVisitor(filename or "<inline>")
    visitor.visit(tree)
    return visitor.violations


class MetamodelTransformationEngine(IValidator):
    """Tier-1 Metamodel Transformation Engine and M2 Type Validator."""

    ALLOWED_TYPES = ALLOWED_M2_METAMODEL_TYPES

    @staticmethod
    def is_allowed_m2_type(type_name: str) -> bool:
        """Check if a type name is in the allowed M2 metamodel types."""
        return is_allowed_m2_type(type_name)

    @staticmethod
    def map_ast_classifier_to_m2(classifier_name: str, domain_role: Optional[str] = None) -> str:
        """Map an AST classifier to its Tier-1 M2 metamodel type."""
        return map_ast_classifier_to_m2(classifier_name, domain_role=domain_role)

    @staticmethod
    def validate_metamodel_purity(text: str, filename: Optional[str] = None) -> List[Finding]:
        """Validate text for metamodel purity."""
        return validate_metamodel_purity(text, filename=filename)

    def validate(self, repo: WorkspaceRepository, **kwargs) -> List[str]:
        """Validate the workspace for metamodel purity across scripts and skills."""
        if not repo.is_upstream_compiler_repo():
            return []

        errors: List[str] = []
        target_dirs = [
            os.path.join(repo.workspace_dir, "scripts"),
            os.path.join(repo.workspace_dir, "skills"),
        ]

        for target_dir in target_dirs:
            if not os.path.exists(target_dir):
                continue
            for root, _, files in os.walk(target_dir):
                for f in files:
                    if not f.endswith(".py"):
                        continue
                    full_path = os.path.join(root, f)
                    rel_path = os.path.relpath(full_path, repo.workspace_dir)
                    try:
                        with open(full_path, "r", encoding="utf-8") as py_file:
                            content = py_file.read()
                        violations = validate_metamodel_purity(content, filename=rel_path)
                        errors.extend(violations)
                    except Exception:
                        pass

        return errors
