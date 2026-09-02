from dataclasses import dataclass, field
from typing import List, Dict, Optional, Any

@dataclass
class MetaRules:
    version: str = "1.0.0"
    description: str = ""
    upstream_repository: str = ""
    troubleshooting_instruction: str = ""
    constitution_path: str = ""
    profiles_directory: str = ""
    walkthrough_directory: str = ""
    walkthrough_pattern: str = ""
    reconciliation_script_path: str = ""
    behavioral_triggers_path: str = ""

@dataclass
class BacklogDirectories:
    epics: str = "docs/epics"
    features: str = "docs/features"
    user_stories: str = "docs/user-stories"
    use_cases: str = "docs/use-cases"
    schemas: str = "schema"

@dataclass
class TargetDirectories:
    react: Optional[str] = None
    flutter: str = "app_flutter"

@dataclass
class ReactRules:
    file_extensions: List[str] = field(default_factory=lambda: [".ts", ".tsx", ".js", ".jsx", ".css", ".scss"])
    exclusions: List[str] = field(default_factory=lambda: ["node_modules", "build", "dist", "coverage", ".git"])
    ui_directories: List[str] = field(default_factory=lambda: ["components", "views"])
    network_directories: List[str] = field(default_factory=lambda: ["io"])
    forbidden_words: List[str] = field(default_factory=list)
    forbidden_words_message: str = "UI view/component but imports forbidden libraries directly. Calculations must run exclusively in a background Web Worker."
    write_lock_keywords: List[str] = field(default_factory=lambda: ["writelock", "lockwrite", "sendlock", "mutationlock"])
    selection_keywords: List[str] = field(default_factory=lambda: ["onSelect", "onNodeSelect", "onSelectionChange", "setSelectedNode", "setSelectedId", "dispatch"])
    interaction_keywords: List[str] = field(default_factory=lambda: ["onClick", "onDrag", "onMouseDown", "onPointerDown"])
    playhead_clamp_regex: List[str] = field(default_factory=lambda: ["0\\.9\\b", "1\\.1\\b"])
    playhead_clamp_range: List[float] = field(default_factory=lambda: [0.90, 1.10])
    ast_compliance_method: str = "stopPropagation"
    viewport_file_patterns: List[str] = field(default_factory=lambda: ["viewport"])
    network_file_patterns: List[str] = field(default_factory=lambda: ["gateway", "socket", "client", "connection"])

@dataclass
class FlutterRules:
    file_extensions: List[str] = field(default_factory=lambda: [".dart"])
    exclusions: List[str] = field(default_factory=lambda: ["build", ".dart_tool", ".git"])
    ui_directories: List[str] = field(default_factory=lambda: ["widgets", "screens"])
    network_directories: List[str] = field(default_factory=lambda: ["io"])
    selection_setters: List[str] = field(default_factory=lambda: ["set selected", "set active", "set selection", "setSelectedNode", "setActiveNode"])
    selection_triggers: List[str] = field(default_factory=lambda: ["onChanged", "onSelected", "notifyListeners", "dispatch"])
    loop_guard_keywords: List[str] = field(default_factory=lambda: ["userinitiated", "programmatic", "fromuser", "isuser", "userinteraction"])
    forbidden_words: List[str] = field(default_factory=list)
    forbidden_words_message: str = "UI widget/screen but references forbidden libraries directly. Calculations must run exclusively in a background Isolate."
    write_lock_keywords: List[str] = field(default_factory=lambda: ["writelock", "lockwrite", "sendlock", "mutationlock"])
    playhead_clamp_regex: List[str] = field(default_factory=lambda: ["0\\.9\\b", "1\\.1\\b"])
    ffi_keywords: List[str] = field(default_factory=lambda: ["dart:ffi"])
    ffi_finalizer_keywords: List[str] = field(default_factory=lambda: ["nativefinalizer"])
    ffi_refcount_keywords: List[str] = field(default_factory=lambda: ["refcount", "referencecount", "addref", "release", "finalizer"])
    viewport_file_patterns: List[str] = field(default_factory=lambda: ["viewport"])
    network_file_patterns: List[str] = field(default_factory=lambda: ["gateway", "socket", "client", "connection"])

@dataclass
class PythonRules:
    exclusions: List[str] = field(default_factory=lambda: ["node_modules", "build", "dist", "coverage", ".git", "skills", ".tessl-plugin"])
    scan_directories: Optional[List[str]] = None

@dataclass
class SpecRules:
    dom_leak_patterns: List[str] = field(default_factory=lambda: ["\\baria-\\w+", "\\brole=[\"']\\w+"])
    pixel_leak_patterns: List[str] = field(default_factory=lambda: ["\\b\\d+px\\b"])
    spec_files: List[str] = field(default_factory=lambda: [".pipeline/logical-ui/logical-components.md"])
    design_tokens_path: str = ".pipeline/logical-ui/design-tokens.json"
    forbidden_standards_blocklist: List[str] = field(default_factory=list)

@dataclass
class ValidationRules:
    uml_primitives: List[str] = field(default_factory=lambda: ["String", "Integer", "Real", "Boolean"])
    visibility_prefixes: List[str] = field(default_factory=lambda: ["+", "-", "#", "~"])
    playhead_rate_limits: List[float] = field(default_factory=lambda: [0.90, 1.10])
    relationship_connectors: str = "(<\\|--|\\*--|o--|-->|\\.\\.>|--)"
    choice_stereotypes: List[str] = field(default_factory=lambda: ["<<choice>>"])
    sequence_replies: List[str] = field(default_factory=lambda: ["-->", "-->>"])
    fragment_keywords: List[str] = field(default_factory=lambda: ["alt", "loop", "opt", "par", "critical", "else", "option"])
    use_case_flow_limit: int = 2
    use_case_step_limit: int = 2
    max_body_characters: int = 65536
    schema_exclude_keywords: List[str] = field(default_factory=lambda: ["description", "reference", "organization", "contact", "revision", "import", "prefix", "namespace", "yang-version"])
    multiplicity_regex: str = "\\[[^\\]]+\\]"
    essential_feature_sections: List[str] = field(default_factory=lambda: ["Class Diagram", "Interface Requirements"])
    required_diagrams: Dict[str, List[str]] = field(default_factory=lambda: {
        "epic": ["classDiagram", "stateDiagram-v2"],
        "feature": ["classDiagram"],
        "user_story": ["sequenceDiagram"],
        "use_case": ["(?:graph|flowchart)", "stateDiagram"]
    })
    mermaid_dotted_link_regex: str = "-\\.-*->\\s*\\|"
    forbidden_diagram_types: List[str] = field(default_factory=lambda: ["erDiagram"])
    use_case_stadium_nodes_only: bool = True
    use_case_undirected_actor_links_only: bool = True
    use_case_extend_arrow_direction_check: bool = True
    naming_conventions: Dict[str, str] = field(default_factory=lambda: {
        "use_case": "^uc-\\d{2}-[a-z0-9\\-]+\\.md$"
    })
    test_data_shape_regex: str = "###\\s+1\\.\\s+Test\\s+Data\\s+Shape"
    test_data_block_regex: str = "```json"
    bdd_scenario_regexes: List[str] = field(default_factory=lambda: [
        "(?:Given|When|Then)",
        "As a\\s+.*\\s+I want to\\s+.*\\s+so that\\s+.*"
    ])
    required_features_matrix_regex: str = "##\\s+Required\\s+Features(?:\\s+Matrix)?(.*?)(?=##|\\Z)"
    checkbox_syntax_regex: str = "-\\s+\\[[ xX]\\]\\s+.*"
    use_case_alternate_flows_header: str = "## 5. Alternate and Exception Flows"
    use_case_numbered_step_regex: str = "\\b\\d+\\.\\s+\\S+"
    use_case_flow_list_regex: str = "(?:(?:-|\\*)\\s+\\*\\*|###\\s+)\\d+[a-zA-Z]+\\..*?(?=(?:\\n\\s*(?:(?:-|\\*)\\s+\\*\\*|###\\s+)\\d+[a-zA-Z]+\\.)|\\Z)"
    realization_matrix_header: str = "## 8. Realization Matrix"
    realization_stories_header: str = "### Required User Stories"
    realization_features_header: str = "### Required Features"
    alternative_schema_extensions: List[str] = field(default_factory=lambda: [".yaml", ".yml", ".json", ".proto", ".asn", ".asn1", ".msg", ".srv", ".xsd"])
    schema_patterns: Dict[str, Any] = field(default_factory=dict)
    required_sections: Dict[str, List[List[str]]] = field(default_factory=lambda: {})

@dataclass
class CodebaseRules:
    meta: MetaRules = field(default_factory=MetaRules)
    tracker_rules: Dict[str, Any] = field(default_factory=dict)
    backlog_directories: BacklogDirectories = field(default_factory=BacklogDirectories)
    target_directories: TargetDirectories = field(default_factory=TargetDirectories)
    react_rules: Optional[ReactRules] = None
    flutter_rules: FlutterRules = field(default_factory=FlutterRules)
    python_rules: PythonRules = field(default_factory=PythonRules)
    spec_rules: SpecRules = field(default_factory=SpecRules)
    validation_rules: ValidationRules = field(default_factory=ValidationRules)

def load_from_dict(data: dict) -> CodebaseRules:
    meta_data = data.get("meta", {})
    meta = MetaRules(**{k: v for k, v in meta_data.items() if k in MetaRules.__dataclass_fields__})
    
    bd_data = data.get("backlog_directories", {})
    backlog_directories = BacklogDirectories(**{k: v for k, v in bd_data.items() if k in BacklogDirectories.__dataclass_fields__})
    
    td_data = data.get("target_directories", {})
    target_directories = TargetDirectories(**{k: v for k, v in td_data.items() if k in TargetDirectories.__dataclass_fields__})
    
    react_data = data.get("react_rules")
    if react_data is not None:
        react_rules = ReactRules(**{k: v for k, v in react_data.items() if k in ReactRules.__dataclass_fields__})
    else:
        react_rules = None
    
    flutter_data = data.get("flutter_rules", {})
    flutter_rules = FlutterRules(**{k: v for k, v in flutter_data.items() if k in FlutterRules.__dataclass_fields__})
    
    py_data = data.get("python_rules", {})
    python_rules = PythonRules(**{k: v for k, v in py_data.items() if k in PythonRules.__dataclass_fields__})
    
    spec_data = data.get("spec_rules", {})
    spec_rules = SpecRules(**{k: v for k, v in spec_data.items() if k in SpecRules.__dataclass_fields__})
    
    val_data = data.get("validation_rules", {})
    validation_rules = ValidationRules(**{k: v for k, v in val_data.items() if k in ValidationRules.__dataclass_fields__})
    
    return CodebaseRules(
        meta=meta,
        tracker_rules=data.get("tracker_rules", {}),
        backlog_directories=backlog_directories,
        target_directories=target_directories,
        react_rules=react_rules,
        flutter_rules=flutter_rules,
        python_rules=python_rules,
        spec_rules=spec_rules,
        validation_rules=validation_rules
    )

# Parsed Diagram Models
@dataclass
class FlowchartNode:
    id: str
    shape: Optional[str] = None
    label: Optional[str] = None
    subgraph: Optional[str] = None

@dataclass
class FlowchartConnection:
    from_node: str
    to_node: str
    style: str
    label: Optional[str] = None

@dataclass
class FlowchartSubgraph:
    id: str
    label: str
    parent: Optional[str] = None
    nodes: List[str] = field(default_factory=list)

@dataclass
class ParsedFlowchart:
    nodes: Dict[str, FlowchartNode] = field(default_factory=dict)
    connections: List[FlowchartConnection] = field(default_factory=list)
    subgraphs: Dict[str, FlowchartSubgraph] = field(default_factory=dict)
    parse_errors: List[str] = field(default_factory=list)

@dataclass
class ClassAttribute:
    visibility: Optional[str]
    name: str
    type: Optional[str]
    multiplicity: Optional[str]
    constraints: List[str]
    raw: str

@dataclass
class ClassMethod:
    visibility: Optional[str]
    name: str
    parameters: List[Dict[str, Optional[str]]]
    return_type: Optional[str]
    constraints: List[str]
    raw: str

@dataclass
class ClassInfo:
    name: str
    namespace: Optional[str] = None
    attributes: List[ClassAttribute] = field(default_factory=list)
    methods: List[ClassMethod] = field(default_factory=list)
    notes: List[str] = field(default_factory=list)

@dataclass
class ClassRelationship:
    type: str
    from_class: str
    to_class: str
    from_multiplicity: Optional[str] = None
    to_multiplicity: Optional[str] = None
    direction: str = "none"
    label: Optional[str] = None
    raw: str = ""

@dataclass
class ClassNamespace:
    name: str
    classes: List[str] = field(default_factory=list)

@dataclass
class ParsedClassDiagram:
    classes: Dict[str, ClassInfo] = field(default_factory=dict)
    relationships: List[ClassRelationship] = field(default_factory=list)
    namespaces: Dict[str, ClassNamespace] = field(default_factory=dict)
    parse_errors: List[str] = field(default_factory=list)

@dataclass
class SequenceMessage:
    sender: str
    receiver: str
    arrow: str
    arrow_type: str
    activation: Optional[str] = None
    operation: Optional[str] = None
    parameters: List[Dict[str, Optional[str]]] = field(default_factory=list)
    assignment: Optional[str] = None
    raw: str = ""
    fragment_context: List[Dict[str, str]] = field(default_factory=list)

@dataclass
class SequenceFragmentBranch:
    guard: Optional[str]
    messages: List[SequenceMessage] = field(default_factory=list)

@dataclass
class SequenceFragment:
    type: str
    branches: List[SequenceFragmentBranch] = field(default_factory=list)
    nested: List['SequenceFragment'] = field(default_factory=list)

@dataclass
class SequenceLifeline:
    name: str
    role: str
    instance_name: str
    classifier_name: Optional[str] = None
    label: str = ""

@dataclass
class ParsedSequenceDiagram:
    lifelines: Dict[str, SequenceLifeline] = field(default_factory=dict)
    messages: List[SequenceMessage] = field(default_factory=list)
    fragments: List[SequenceFragment] = field(default_factory=list)
    parse_errors: List[str] = field(default_factory=list)

@dataclass
class FeatureFile:
    filename: str
    labels: List[str]
    content: str
    frontmatter: Dict[str, Any] = field(default_factory=dict)

# Cited Research Inventory & Declared-Total Population Register Models (CORE #97)
@dataclass
class NormativeStandard:
    standard_id: str
    issuing_body: str
    title: str
    applicable_clauses: str
    obligation_category: str
    declared_total: int
    clause_citation: str
    raw: Dict[str, str] = field(default_factory=dict)

@dataclass
class PopulationRegisterEntry:
    category: str
    standard_id: str
    obligation_count: int
    verification_mechanism: str
    clause_citation: str
    obligation_id: Optional[str] = None
    target_metric: Optional[str] = None
    raw: Dict[str, str] = field(default_factory=dict)

@dataclass
class ExternalAdditionEntry:
    category: str
    standard_id: str
    declared_total: int
    verification_mechanism: str
    clause_citation: str
    extension_id: Optional[str] = None
    target_metric: Optional[str] = None
    justification: Optional[str] = None
    raw: Dict[str, str] = field(default_factory=dict)

@dataclass
class ClauseAllocationEntry:
    population_id: str
    standard_id: str
    clause_citation: str
    clause_title: str
    specification_phase: str
    downstream_spec_file: str
    raw: Dict[str, str] = field(default_factory=dict)

@dataclass
class ResearchInventoryDocument:
    filepath: str = ""
    metadata: Dict[str, Any] = field(default_factory=dict)
    standards: List[NormativeStandard] = field(default_factory=list)
    population_register: List[PopulationRegisterEntry] = field(default_factory=list)
    external_additions: List[ExternalAdditionEntry] = field(default_factory=list)
    clause_allocations: List[ClauseAllocationEntry] = field(default_factory=list)
    gap_analysis: Dict[str, Any] = field(default_factory=dict)
    parse_errors: List[str] = field(default_factory=list)

    def get_totals_by_standard(self) -> Dict[str, int]:
        totals: Dict[str, int] = {}
        for std in self.standards:
            totals[std.standard_id] = totals.get(std.standard_id, 0) + std.declared_total
        return totals

    def get_totals_by_category(self) -> Dict[str, int]:
        totals: Dict[str, int] = {}
        for std in self.standards:
            cat = std.obligation_category or "Uncategorized"
            totals[cat] = totals.get(cat, 0) + std.declared_total
        return totals

    def get_total_declared_obligations(self) -> int:
        return sum(std.declared_total for std in self.standards)


# Coverage-Digest & Obligation-Witness Models (CORE #98, closing #92 & #93)
@dataclass
class CoverageDigest:
    """
    Coverage Digest tracking population metrics against declared obligations (Gate 28 / #92).
    """
    total_declared_obligations: int = 0
    total_realized_obligations: int = 0
    realization_percentage: float = 0.0
    declared_by_standard: Dict[str, int] = field(default_factory=dict)
    realized_by_standard: Dict[str, int] = field(default_factory=dict)
    declared_by_category: Dict[str, int] = field(default_factory=dict)
    realized_by_category: Dict[str, int] = field(default_factory=dict)
    obligation_realization_map: Dict[str, List[str]] = field(default_factory=dict)
    unrealized_obligations: List[str] = field(default_factory=list)
    phantom_realizations: List[str] = field(default_factory=list)

    def is_fully_realized(self) -> bool:
        """Returns True if every declared obligation is realized with zero phantom realizations."""
        return (
            self.total_declared_obligations > 0
            and len(self.unrealized_obligations) == 0
            and len(self.phantom_realizations) == 0
        )

    def to_dict(self) -> Dict[str, Any]:
        return {
            "total_declared_obligations": self.total_declared_obligations,
            "total_realized_obligations": self.total_realized_obligations,
            "realization_percentage": round(self.realization_percentage, 2),
            "declared_by_standard": dict(self.declared_by_standard),
            "realized_by_standard": dict(self.realized_by_standard),
            "declared_by_category": dict(self.declared_by_category),
            "realized_by_category": dict(self.realized_by_category),
            "obligation_realization_map": {k: list(v) for k, v in self.obligation_realization_map.items()},
            "unrealized_obligations": list(self.unrealized_obligations),
            "phantom_realizations": list(self.phantom_realizations),
            "is_fully_realized": self.is_fully_realized(),
        }

    def generate_markdown_summary(self) -> str:
        lines: List[str] = [
            "| Metric Parameter | Value | Compliance Status |",
            "| :--- | :--- | :--- |",
            f"| Declared Total Obligations | {self.total_declared_obligations} | Baseline |",
            f"| Realized Total Obligations | {self.total_realized_obligations} | {'Conforming' if len(self.unrealized_obligations) == 0 else 'Gaps Identified'} |",
            f"| Population Realization Coverage | {self.realization_percentage:.1f}% | {'100% Conforming' if self.realization_percentage >= 100.0 else 'Incomplete'} |",
            f"| Phantom Realizations | {len(self.phantom_realizations)} | {'Zero (Conforming)' if len(self.phantom_realizations) == 0 else 'Non-Conforming'} |",
            f"| Unrealized Obligations | {len(self.unrealized_obligations)} | {'Zero (Conforming)' if len(self.unrealized_obligations) == 0 else 'Non-Conforming'} |",
        ]
        return "\n".join(lines)


@dataclass
class ObligationWitnessRecord:
    """
    Multidimensional witness tracking record for an individual obligation (Gate 29 / #93).
    """
    obligation_id: str
    standard_id: str = ""
    category: str = ""
    clause_citation: str = ""
    verification_mechanism: str = ""
    spec_witnesses: List[str] = field(default_factory=list)
    test_witnesses: List[str] = field(default_factory=list)
    code_witnesses: List[str] = field(default_factory=list)
    model_witnesses: List[str] = field(default_factory=list)

    @property
    def total_witnesses(self) -> int:
        return len(self.spec_witnesses) + len(self.test_witnesses) + len(self.code_witnesses) + len(self.model_witnesses)

    @property
    def is_witnessed(self) -> bool:
        return self.total_witnesses > 0

    @property
    def is_fully_witnessed(self) -> bool:
        has_spec = len(self.spec_witnesses) > 0
        has_test = len(self.test_witnesses) > 0
        has_impl = (len(self.code_witnesses) + len(self.model_witnesses)) > 0
        return has_spec and has_test and has_impl

    def to_dict(self) -> Dict[str, Any]:
        return {
            "obligation_id": self.obligation_id,
            "standard_id": self.standard_id,
            "category": self.category,
            "clause_citation": self.clause_citation,
            "verification_mechanism": self.verification_mechanism,
            "spec_witnesses": list(self.spec_witnesses),
            "test_witnesses": list(self.test_witnesses),
            "code_witnesses": list(self.code_witnesses),
            "model_witnesses": list(self.model_witnesses),
            "total_witnesses": self.total_witnesses,
            "is_witnessed": self.is_witnessed,
            "is_fully_witnessed": self.is_fully_witnessed(),
        }


@dataclass
class ObligationWitnessRegistry:
    """
    Obligation-Witness Registry managing all obligation witnesses across the workspace (Gate 29 / #93).
    """
    records: Dict[str, ObligationWitnessRecord] = field(default_factory=dict)
    phantom_witnesses: Dict[str, List[str]] = field(default_factory=dict)

    def get_record(self, obligation_id: str) -> Optional[ObligationWitnessRecord]:
        return self.records.get(obligation_id)

    def total_declared(self) -> int:
        return len(self.records)

    def total_witnessed(self) -> int:
        return sum(1 for r in self.records.values() if r.is_witnessed)

    def total_fully_witnessed(self) -> int:
        return sum(1 for r in self.records.values() if r.is_fully_witnessed)

    def witness_coverage_percentage(self) -> float:
        if not self.records:
            return 100.0
        return (self.total_witnessed() / len(self.records)) * 100.0

    def to_dict(self) -> Dict[str, Any]:
        return {
            "total_declared": self.total_declared(),
            "total_witnessed": self.total_witnessed(),
            "total_fully_witnessed": self.total_fully_witnessed(),
            "witness_coverage_percentage": round(self.witness_coverage_percentage(), 2),
            "records": {k: v.to_dict() for k, v in self.records.items()},
            "phantom_witnesses": {k: list(v) for k, v in self.phantom_witnesses.items()},
        }


