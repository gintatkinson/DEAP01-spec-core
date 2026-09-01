"""
Validator that enforces UML diagram compliance across feature, epic,
user-story, and use-case specification files.

Verifies class-diagram syntax, forbidden diagram types, required sections,
sequence-diagram lifeline/method alignment with global class definitions,
use-case diagram structural rules, and epic checklist formatting.
"""

import os
import re
from typing import List, Dict, Any
from .base import IValidator
from ..core.findings import Finding
from ..core.workspace import WorkspaceRepository

# Unresolved template placeholders (issue #281).
#
# Detection is pattern-based rather than literal-membership. The previous
# implementation held a hardcoded list -- ["*(none registered)*", "*to be
# populated*", "*tbd*", "*n/a*"] -- so near-variants slipped through: "*(None)*"
# differed from "*(none registered)*" only in case and wording, and "#[EpicID]"
# evaded a substring test for "IssueID" because "EpicID" does not contain it.
#
# Every pattern below is case-insensitive and each carries a human-readable label
# so the error names what is actually wrong rather than echoing a raw line.
# Never valid in any document, under any circumstances.
ALWAYS_INVALID_PLACEHOLDER_PATTERNS = [
    # Real references look like '#43'. Any bracketed token is unresolved.
    (re.compile(r"#\[[^\]]+\]"), "unresolved issue reference token"),
    (re.compile(r"\[(?:Feat(?:ure)?|Epic|US|UC|User\s*Story|Use\s*Case|Story|Issue)[-_\s]*(?:ID|IssueID)\]", re.I),
     "unresolved issue reference token"),
    (re.compile(r"\[(?:Epic|Feature|User\s*Story|Use\s*Case)\s+Title\]", re.I),
     "unpopulated template title"),
    (re.compile(r"\[Title\]", re.I),
     "unpopulated template title"),
    (re.compile(r"\{\{REQUIRED_(?:JUSTIFICATION|SOURCE_REF)\}\}", re.I),
     "unreplaced template escape token"),
    (re.compile(r"\(\s*semantic linkage justification[^)]*\)", re.I),
     "template text left in place of a written linkage justification"),
    (re.compile(r"\[POPULATE:", re.I),
     "unreplaced [POPULATE:] placeholder token"),
    (re.compile(r"\b(?:epic|feat|us|uc)-XX-name\b", re.I), "placeholder file path"),
    (re.compile(r"\[(?:Repository\s+Base\s+URL|Branch\s+Name|Spec\s+Reference|SysMLInteractionName|SysMLTestCaseName)\]", re.I),
     "unpopulated template token"),
    (re.compile(r"<blob_path>", re.I),
     "unpopulated template path"),
]

# Conditionally valid. "*(None registered)*" is a truthful statement when nothing is
# in fact registered, and a placeholder only when matching items exist. Issue #239
# established that distinction and covers it with two paired tests, so these patterns
# must stay gated on the caller's knowledge of what exists.
CONDITIONAL_STUB_PATTERNS = [
    (re.compile(r"\*\(\s*none(?:\s+registered)?\s*\)\*", re.I), "placeholder stub"),
    # Parentheses optional: "*(TBD)*" reads exactly like "*(None)*", which the
    # pattern above already accepts in both forms. Without this, the near-variant
    # slips through — the same gap the comment above records (#280).
    (re.compile(r"\*\s*\(?\s*(?:to be populated|tbd|n/a)\s*\)?\s*\*", re.I), "placeholder stub"),
]


def find_unresolved_placeholders(content: str, patterns=None):
    """Yield ``(line_number, label, line_text)`` for each unresolved placeholder."""
    if patterns is None:
        patterns = ALWAYS_INVALID_PLACEHOLDER_PATTERNS
    for lineno, line in enumerate(content.splitlines(), 1):
        for pattern, label in patterns:
            if pattern.search(line):
                yield lineno, label, line.strip()
                break

import yaml
from typing import Optional, Set, Tuple

# Import SysML v2 AST classes via the fail-closed loader (refs #76): resolve
# the real scripts dir or raise ImportError — never bind None silently.
from ..utils.sysml_loader import load_sysml_ast_members

_sysml_ast = load_sysml_ast_members([
    "SysMLPackage", "SysMLParser", "SysMLCapabilityDef", "ActionDef",
    "SysMLOperationDef", "SysMLConstraintDef", "SysMLInteractionDef",
    "SysMLTestCaseDef", "RequirementDef", "PartDef", "AttributeDef",
])
SysMLPackage = _sysml_ast.SysMLPackage
SysMLParser = _sysml_ast.SysMLParser
SysMLCapabilityDef = _sysml_ast.SysMLCapabilityDef
ActionDef = _sysml_ast.ActionDef
SysMLOperationDef = _sysml_ast.SysMLOperationDef
SysMLConstraintDef = _sysml_ast.SysMLConstraintDef
SysMLInteractionDef = _sysml_ast.SysMLInteractionDef
SysMLTestCaseDef = _sysml_ast.SysMLTestCaseDef
RequirementDef = _sysml_ast.RequirementDef
PartDef = _sysml_ast.PartDef
AttributeDef = _sysml_ast.AttributeDef


def _find_sysml_files_in_repo(repo: WorkspaceRepository, schemas_dir: Optional[str] = None) -> List[str]:
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


def _collect_all_sysml_elements(pkg: Any) -> Tuple[List[Any], List[Any], List[Any], List[Any], List[Any], List[Any], List[Any], List[Any], List[Any]]:
    """Recursively collect (packages, parts, capabilities, actions, operations, interactions, constraints, test_cases, requirements) from a SysMLPackage."""
    if not pkg:
        return [], [], [], [], [], [], [], [], []

    packages = [pkg]
    parts = list(getattr(pkg, "part_defs", []) or [])
    capabilities = list(getattr(pkg, "capability_defs", []) or [])
    actions = list(getattr(pkg, "action_defs", []) or [])
    operations = list(getattr(pkg, "operation_defs", []) or [])
    interactions = list(getattr(pkg, "interaction_defs", []) or [])
    constraints = list(getattr(pkg, "constraint_defs", []) or [])
    test_cases = list(getattr(pkg, "test_case_defs", []) or [])
    requirements = list(getattr(pkg, "requirement_defs", []) or [])

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
        for inter in (getattr(p, "interactions", []) or []):
            interactions.append(inter)
        for c in (getattr(p, "constraints", []) or []):
            constraints.append(c)
        for tc in (getattr(p, "test_cases", []) or []):
            test_cases.append(tc)
        for r in (getattr(p, "requirements", []) or []):
            requirements.append(r)

    for p in (getattr(pkg, "part_defs", []) or []):
        _traverse_part(p)

    for sub_pkg in (getattr(pkg, "sub_packages", []) or []):
        sub_pks, sub_pts, sub_caps, sub_acts, sub_ops, sub_inters, sub_cons, sub_tcs, sub_reqs = _collect_all_sysml_elements(sub_pkg)
        packages.extend(sub_pks)
        parts.extend(sub_pts)
        capabilities.extend(sub_caps)
        actions.extend(sub_acts)
        operations.extend(sub_ops)
        interactions.extend(sub_inters)
        constraints.extend(sub_cons)
        test_cases.extend(sub_tcs)
        requirements.extend(sub_reqs)

    return packages, parts, capabilities, actions, operations, interactions, constraints, test_cases, requirements


def _load_all_sysml_elements_full(repo: WorkspaceRepository, schemas_dir: Optional[str] = None):
    sysml_files = _find_sysml_files_in_repo(repo, schemas_dir)
    errors: List[Finding] = []
    all_pkgs: List[Any] = []
    all_parts: List[Any] = []
    all_caps: List[Any] = []
    all_actions: List[Any] = []
    all_ops: List[Any] = []
    all_interactions: List[Any] = []
    all_constraints: List[Any] = []
    all_test_cases: List[Any] = []
    all_requirements: List[Any] = []

    for sf in sysml_files:
        try:
            with open(sf, "r", encoding="utf-8") as file:
                content = file.read()
            if SysMLParser:
                pkg = SysMLParser.parse_text(content, default_name=os.path.splitext(os.path.basename(sf))[0])
            else:
                pkg = SysMLPackage(name=os.path.splitext(os.path.basename(sf))[0]) if SysMLPackage else None
            pks, pts, caps, acts, ops, inters, cons, tcs, reqs = _collect_all_sysml_elements(pkg)
            all_pkgs.extend(pks)
            all_parts.extend(pts)
            all_caps.extend(caps)
            all_actions.extend(acts)
            all_ops.extend(ops)
            all_interactions.extend(inters)
            all_constraints.extend(cons)
            all_test_cases.extend(tcs)
            all_requirements.extend(reqs)
        except Exception as exc:
            errors.append(Finding(
                "sysml-model-not-readable",
                f"Failed to read SysML model file '{os.path.basename(sf)}': {exc}",
                location="schemas"
            ))

    return sysml_files, all_pkgs, all_parts, all_caps, all_actions, all_ops, all_interactions, all_constraints, all_test_cases, all_requirements, errors

from ..core.models import FeatureFile
from ..parsers.mermaid import MermaidClassDiagramParser, MermaidFlowchartParser, MermaidSequenceDiagramParser

class UmlValidator(IValidator):
    def validate(self, repo: WorkspaceRepository, **kwargs) -> List[str]:
        global_classes = kwargs.get("global_classes")
        
        rules = repo.get_codebase_rules()
        val_rules = rules.validation_rules
        backlog_dirs = rules.backlog_directories
        
        features_dir = os.path.join(repo.workspace_dir, backlog_dirs.features)
        user_stories_dir = os.path.join(repo.workspace_dir, backlog_dirs.user_stories)
        use_cases_dir = os.path.join(repo.workspace_dir, backlog_dirs.use_cases)
        epics_dir = kwargs.get("epics_dir")
        if epics_dir is None:
            epics_dir = os.path.join(repo.workspace_dir, backlog_dirs.epics) if backlog_dirs.epics else None
        
        errors = []
        
        if epics_dir and not os.path.exists(epics_dir):
            errors.append(
                Finding("epic-directory-must-exist-when-configured", f"Warning: Epic directory '{epics_dir}' configured in backlog_dirs.epics "
                f"does not exist on disk. Epic class diagrams will be excluded from the "
                f"global class registry. Sequence diagram lifeline validation may produce "
                f"false positive errors for classifiers defined in epic specifications. "
                f"Create the directory and populate epic files before running validation.")
            )
        
        def get_md_files(d):
            if not d or not os.path.exists(d):
                return []
            return [os.path.join(d, f) for f in os.listdir(d) if f.endswith(".md")]
            
        if global_classes is None:
            global_classes = self.build_global_classes(repo, features_dir, epics_dir)
            
        dotted_link_pattern = val_rules.mermaid_dotted_link_regex
        forbidden_diagram_types = val_rules.forbidden_diagram_types

        mermaid_block_re = re.compile(r'```mermaid\s*\n(.*?)```', re.DOTALL)

        def validate_dotted_links(content, doc_type, filename, errors_list):
            for block_match in mermaid_block_re.finditer(content):
                block = block_match.group(1)
                block_start_line = content[:block_match.start()].count('\n') + 1
                for line_idx, line in enumerate(block.splitlines()):
                    if re.search(dotted_link_pattern, line):
                        errors_list.append(
                            f"{doc_type} {filename} contains invalid Mermaid dotted link label "
                            f"syntax at line {block_start_line + line_idx + 1}: "
                            f"'{line.strip()}'. Use standard label formatting."
                        )
                        break

        def validate_forbidden_diagram_types(content, doc_type, filename, errors_list):
            for block_match in mermaid_block_re.finditer(content):
                block = block_match.group(1)
                block_start_line = content[:block_match.start()].count('\n') + 1
                for ftype in forbidden_diagram_types:
                    if re.search(ftype, block):
                        for line_idx, line in enumerate(block.splitlines()):
                            if re.search(ftype, line):
                                errors_list.append(
                                    f"{doc_type} {filename} contains forbidden '{ftype}' "
                                    f"diagram type at line {block_start_line + line_idx + 1}: "
                                    f"'{line.strip()}'"
                                )
                                break
                        break
        required_sections = val_rules.required_sections
        required_diagrams = val_rules.required_diagrams
        
        uml_primitives = set(val_rules.uml_primitives)
        visibility_prefixes = set(val_rules.visibility_prefixes)
        relationship_connectors = MermaidClassDiagramParser._sanitize_rel_connectors(val_rules.relationship_connectors)
        choice_stereotypes = val_rules.choice_stereotypes
        multiplicity_regex = val_rules.multiplicity_regex
        essential_feature_sections = val_rules.essential_feature_sections
        
        test_data_shape_regex = val_rules.test_data_shape_regex
        test_data_block_regex = val_rules.test_data_block_regex
        bdd_scenario_regexes = val_rules.bdd_scenario_regexes
        required_features_matrix_regex = val_rules.required_features_matrix_regex
        checkbox_syntax_regex = val_rules.checkbox_syntax_regex
        use_case_alternate_flows_header = val_rules.use_case_alternate_flows_header
        use_case_numbered_step_regex = val_rules.use_case_numbered_step_regex
        use_case_flow_list_regex = val_rules.use_case_flow_list_regex
        realization_matrix_header = val_rules.realization_matrix_header
        realization_stories_header = val_rules.realization_stories_header
        realization_features_header = val_rules.realization_features_header
        
        class_parser = MermaidClassDiagramParser(repo)
        flowchart_parser = MermaidFlowchartParser()
        sequence_parser = MermaidSequenceDiagramParser()
        
        feature_files = get_md_files(features_dir)
        for filepath in feature_files:
            filename = os.path.basename(filepath)
            try:
                with open(filepath, "r", encoding="utf-8") as f:
                    content = f.read()
            except Exception as e:
                errors.append(Finding("backlog-document-must-be-readable", f"System Error: Failed to read feature file '{filename}': {e}", location=filename))
                continue
                
            self._validate_subagent_isolation(content, "Feature", filename, errors)
            self._validate_placeholders_and_links(content, "Feature", filename, errors, checkbox_syntax_regex)
                
            validate_dotted_links(content, "Feature", filename, errors)
            validate_forbidden_diagram_types(content, "Feature", filename, errors)
                    
            interface_type = "ui"
            frontmatter_match = re.match(r"^---\s*\n(.*?)\n---\s*\n", content, re.DOTALL)
            if frontmatter_match:
                frontmatter_text = frontmatter_match.group(1)
                try:
                    import yaml
                    data = yaml.safe_load(frontmatter_text.replace('\x01', ''))
                    if isinstance(data, dict):
                        for k, v in data.items():
                            if k in ("interface_type", "interface-type"):
                                interface_type = str(v).lower()
                except Exception:
                    for fm_line in frontmatter_text.splitlines():
                        if ":" in fm_line:
                            fm_parts = fm_line.split(":", 1)
                            fm_key = fm_parts[0].strip()
                            fm_val = fm_parts[1].strip().strip('"').strip("'")
                            if fm_key in ("interface_type", "interface-type"):
                                interface_type = fm_val.lower()
                            
            req_key = f"feature_{interface_type}"
            required_feature_sections = required_sections.get(req_key)
            if required_feature_sections is None:
                required_feature_sections = required_sections.get("feature")
            if required_feature_sections is None:
                errors.append(Finding("uml-validator-configuration-must-be-complete", f"System Error: Missing '{req_key}' or 'feature' required sections config."))
                continue
                
            has_essential_sections = True
            for pattern, header_name in required_feature_sections:
                if not re.search(pattern, content, re.IGNORECASE):
                    errors.append(Finding("backlog-document-requires-its-configured-sections", f"Feature {filename} is missing section '{header_name}'.", location=filename))
                    if any(essential in header_name for essential in essential_feature_sections):
                        has_essential_sections = False
                        
            if not has_essential_sections:
                continue
                
            feature_req_diagrams = required_diagrams.get("feature")
            if feature_req_diagrams is None:
                errors.append(Finding("uml-validator-configuration-must-be-complete", "System Error: Missing required_diagrams.feature config."))
                continue
                
            has_diag_error = False
            for diag_type in feature_req_diagrams:
                if not re.search(r"```mermaid\s*\n\s*" + diag_type, content):
                    errors.append(Finding("backlog-document-requires-its-configured-diagrams", f"Feature {filename} is missing a valid diagram of type '{diag_type}'.", location=filename))
                    has_diag_error = True
            if has_diag_error:
                continue
                
            self._validate_class_diagram(
                "Feature", filename, content, errors, class_parser, val_rules,
                uml_primitives, visibility_prefixes, relationship_connectors,
                choice_stereotypes, multiplicity_regex
            )
                        
            if re.search(test_data_shape_regex, content, re.IGNORECASE):
                if not re.search(test_data_shape_regex + r".*?" + test_data_block_regex, content, re.DOTALL | re.IGNORECASE):
                    errors.append(Finding("feature-requires-a-test-data-payload-example", f"Feature {filename} is missing a payload example ({test_data_block_regex} block) under Test Data Shape.", location=filename))
                    
        story_files = get_md_files(user_stories_dir)
        for filepath in story_files:
            filename = os.path.basename(filepath)
            try:
                with open(filepath, "r", encoding="utf-8") as f:
                    content = f.read()
            except Exception as e:
                errors.append(Finding("backlog-document-must-be-readable", f"System Error: Failed to read user story file '{filename}': {e}", location=filename))
                continue
                
            self._validate_subagent_isolation(content, "User Story", filename, errors)
            self._validate_placeholders_and_links(content, "User Story", filename, errors, checkbox_syntax_regex)
                
            validate_dotted_links(content, "User Story", filename, errors)
            validate_forbidden_diagram_types(content, "User Story", filename, errors)
                    
            required_story_sections = required_sections.get("user_story")
            if required_story_sections is None:
                errors.append(Finding("uml-validator-configuration-must-be-complete", "System Error: Missing required_sections.user_story config."))
                continue
                
            has_essential_sections = True
            for pattern, header_name in required_story_sections:
                if not re.search(pattern, content, re.IGNORECASE):
                    errors.append(Finding("backlog-document-requires-its-configured-sections", f"User Story {filename} is missing section '{header_name}'.", location=filename))
                    if "Sequence Diagram" in header_name:
                        has_essential_sections = False
                        
            if not has_essential_sections:
                continue
                
            story_req_diagrams = required_diagrams.get("user_story")
            if story_req_diagrams is None:
                errors.append(Finding("uml-validator-configuration-must-be-complete", "System Error: Missing required_diagrams.user_story config."))
                continue
                
            has_seq = False
            seq_diagram_matches = []
            for diag_type in story_req_diagrams:
                for match in re.finditer(r"```mermaid\s*\n\s*" + diag_type + r"(.*?)(?=```|\Z)", content, re.DOTALL):
                    has_seq = True
                    seq_diagram_matches.append(match)
                    
            for seq_match in seq_diagram_matches:
                seq_code = seq_match.group(0)
                parsed = sequence_parser.parse(seq_code)
                for err in parsed.parse_errors:
                    errors.append(Finding("sequence-diagram-must-parse", f"User Story {filename} sequence diagram parse error: {err}", location=filename))
                lifelines = parsed.lifelines
                messages = parsed.messages
                
                for alias, lf in lifelines.items():
                    label = lf.label
                    if not lf.classifier_name:
                        errors.append(Finding("sequence-lifeline-requires-name-and-classifier", f"User Story {filename} sequence diagram lifeline '{alias}' is missing the name : Classifier pattern in its label: '{label}'", location=filename))
                    else:
                        cls_name = lf.classifier_name
                        # Exemption keys on UML role, not name spelling (issue #277).
                        #
                        # A tuple of suffixes previously exempted classifiers ending in
                        # Manager, System, Actor and similar. That was arbitrary --
                        # PaymentManager passed while PaymentHandler did not -- and it
                        # disabled a second check: an exempt classifier never enters
                        # global_classes, so the operation-signature guard below was
                        # also false and every message to that lifeline went unverified.
                        #
                        # An `actor` lifeline represents an entity outside the system
                        # boundary and is correctly absent from the structural models.
                        # Every other lifeline, including one auto-created by being
                        # referenced without declaration, must resolve.
                        is_external_actor = (lf.role or "").lower() == "actor"
                        if not is_external_actor and cls_name not in global_classes:
                            errors.append(Finding("sequence-lifeline-classifier-must-be-defined", f"User Story {filename} sequence diagram lifeline '{alias}' specifies classifier '{cls_name}' which is not defined in any feature class diagram.", location=filename))
                            
                for msg in messages:
                    if msg.arrow_type in ("sync", "async"):
                        op_name = msg.operation
                        if not op_name:
                            errors.append(Finding("sequence-message-requires-an-operation-signature", f"User Story {filename} sequence diagram message '{msg.raw}' is missing an operation signature.", location=filename))
                            continue
                        receiver = msg.receiver
                        rx_lf = lifelines.get(receiver)
                        rx_cls = rx_lf.classifier_name if rx_lf else None
                        if rx_cls:
                            if rx_cls in global_classes:
                                cls_methods = global_classes[rx_cls]["methods"]
                                method_found = None
                                for m in cls_methods:
                                    if m["name"] == op_name:
                                        method_found = m
                                        break
                                if not method_found:
                                    errors.append(Finding("sequence-message-operation-must-exist-on-the-receiver", f"User Story {filename} sequence diagram message '{msg.raw}' calls operation '{op_name}' which is not defined on class '{rx_cls}' in any class diagram.", location=filename))
                                elif method_found["visibility"] != "+":
                                    errors.append(Finding("sequence-message-operation-must-be-public", f"User Story {filename} sequence diagram message '{msg.raw}' calls non-public operation '{op_name}' on class '{rx_cls}' (visibility must be '+').", location=filename))
                                    
                    if msg.arrow_type == "reply":
                        sequence_replies = val_rules.sequence_replies
                        if msg.arrow not in sequence_replies:
                            errors.append(Finding("sequence-return-message-requires-a-reply-arrow", f"User Story {filename} sequence diagram return message '{msg.raw}' uses invalid reply arrow '{msg.arrow}'. Return arrows must strictly use standard open arrowhead {', '.join(sequence_replies)}.", location=filename))
                            
                        raw_msg_text = msg.raw.split(":", 1)[1].strip() if ":" in msg.raw else msg.raw
                        if "(" in raw_msg_text or ")" in raw_msg_text:
                            errors.append(Finding("sequence-return-message-must-not-be-an-operation-call", f"User Story {filename} sequence diagram return message '{msg.raw}' looks like an operation call (contains parentheses). Return messages must be simple assignments or return values (e.g. status : Status).", location=filename))
                            
                for line in seq_code.splitlines():
                    line_clean = line.strip()
                    line_clean = re.sub(r'%%.*$', '', line_clean).strip()
                    if not line_clean:
                        continue
                    fragment_keywords = val_rules.fragment_keywords
                    frag_pattern = r'^\s*(' + '|'.join(re.escape(k) for k in fragment_keywords) + r')(?:\s+(.*))?$'
                    frag_match = re.match(frag_pattern, line_clean, re.IGNORECASE)
                    if frag_match:
                        keyword = frag_match.group(1).lower()
                        guard_part = frag_match.group(2)
                        if guard_part:
                            guard_part = guard_part.strip()
                            if guard_part and not (guard_part.startswith('[') and guard_part.endswith(']')):
                                errors.append(Finding("sequence-combined-fragment-guard-requires-square-brackets", f"User Story {filename} sequence diagram contains a combined fragment '{keyword}' with guard '{guard_part}' that is not enclosed in square brackets [].", location=filename))
                                
            if not has_seq:
                errors.append(Finding("backlog-document-requires-its-configured-diagrams", f"User Story {filename} is missing a required diagram matching pattern(s): {', '.join(story_req_diagrams)}", location=filename))
                
            bdd_scenario_present = any(re.search(pat, content, re.DOTALL | re.IGNORECASE) for pat in bdd_scenario_regexes)
            if not bdd_scenario_present:
                errors.append(Finding("user-story-requires-a-bdd-scenario", f"User Story {filename} must contain a valid BDD scenario (Given-When-Then or As a/I want to/So that).", location=filename))
                
            rf_match = re.search(required_features_matrix_regex, content, re.DOTALL | re.IGNORECASE)
            if not rf_match:
                errors.append(Finding("user-story-requires-a-required-features-matrix", f"User Story {filename} is missing '## Required Features Matrix' section.", location=filename))
            else:
                rf_section = rf_match.group(1)
                checkboxes = re.findall(checkbox_syntax_regex, rf_section)
                if not checkboxes:
                    errors.append(Finding("user-story-matrix-requires-a-feature-reference", f"User Story {filename} must have at least one feature reference checklist item in its Required Features Matrix.", location=filename))
                for cb in checkboxes:
                    url_match = re.search(r"\]\((https?://[^)]+)\)", cb)
                    if not url_match:
                        errors.append(Finding("checklist-item-requires-an-absolute-url", f"User Story {filename} contains a checklist item with a missing or non-absolute URL: '{cb.strip()}'.", location=filename))
                    else:
                        link = url_match.group(1)
                        if not re.match(r"^https?://[a-zA-Z0-9.-]+/", link):
                            errors.append(Finding("checklist-item-requires-an-absolute-url", f"User Story {filename} contains a non-absolute/invalid URL in Required Features Matrix: '{link}'.", location=filename))
                            
                    justification_match = re.search(r"\s+\(([^)]+)\)$", cb)
                    if not justification_match or (url_match and justification_match.group(1) == url_match.group(1)):
                        errors.append(Finding("checklist-item-requires-a-semantic-justification", f"User Story {filename} contains a checklist item with a missing or invalid parenthetical semantic justification at the end: '{cb.strip()}'.", location=filename))
                        
        usecase_files = get_md_files(use_cases_dir)
        use_case_naming = val_rules.naming_conventions.get("use_case", r"^uc-\d{2}-[a-z0-9\-]+\.md$")
        for filepath in usecase_files:
            basename = os.path.basename(filepath)
            
            if not re.match(use_case_naming, basename):
                errors.append(Finding("use-case-filename-must-follow-the-naming-convention", f"Use Case file '{basename}' does not follow the naming convention '{use_case_naming}'.", location=basename))
                
            try:
                with open(filepath, "r", encoding="utf-8") as f:
                    content = f.read()
            except Exception as e:
                errors.append(Finding("backlog-document-must-be-readable", f"System Error: Failed to read use case file '{basename}': {e}", location=basename))
                continue
                
            self._validate_subagent_isolation(content, "Use Case", basename, errors)
            self._validate_placeholders_and_links(content, "Use Case", basename, errors, checkbox_syntax_regex)
                
            validate_dotted_links(content, "Use Case", basename, errors)
            validate_forbidden_diagram_types(content, "Use Case", basename, errors)
                    
            required_usecase_sections = required_sections.get("use_case")
            if required_usecase_sections is None:
                errors.append(Finding("uml-validator-configuration-must-be-complete", "System Error: Missing required_sections.use_case config."))
                continue
                
            has_essential_sections = True
            for pattern, header_name in required_usecase_sections:
                if not re.search(pattern, content, re.IGNORECASE):
                    errors.append(Finding("backlog-document-requires-its-configured-sections", f"Use Case {basename} is missing section '{header_name}'.", location=basename))
                    if "Diagrams" in header_name:
                        has_essential_sections = False
                        
            if not has_essential_sections:
                continue
                
            usecase_req_diagrams = required_diagrams.get("use_case")
            if usecase_req_diagrams is None:
                errors.append(Finding("uml-validator-configuration-must-be-complete", "System Error: Missing required_diagrams.use_case config."))
                continue
                
            for diag_type in usecase_req_diagrams:
                diag_matches = list(re.finditer(r"```mermaid\s*\n\s*" + diag_type + r"(.*?)(?=```|\Z)", content, re.DOTALL))
                if not diag_matches:
                    errors.append(Finding("backlog-document-requires-its-configured-diagrams", f"Use Case {basename} is missing a valid diagram matching pattern '{diag_type}'.", location=basename))
                elif "graph" in diag_type or "flowchart" in diag_type:
                    for match in diag_matches:
                        diagram_code = match.group(0)
                        parsed = flowchart_parser.parse(diagram_code)
                        for err in parsed.parse_errors:
                            errors.append(Finding("use-case-flowchart-must-parse", f"Use Case {basename} flowchart parse error: {err}", location=basename))
                        
                        boundary_sub = None
                        for sub_id, sub_info in parsed.subgraphs.items():
                            if "boundary" in sub_id.lower() or "system" in sub_id.lower() or \
                               (sub_info.label and ("boundary" in sub_info.label.lower() or "system" in sub_info.label.lower())):
                                boundary_sub = sub_info
                                break
                                
                        if not boundary_sub:
                            errors.append(Finding("use-case-requires-a-system-boundary-subgraph", f"Use Case {basename} is missing a system boundary subgraph (e.g. ID or label containing 'boundary' or 'system').", location=basename))
                            continue
                            
                        boundary_sub_id = boundary_sub.id
                        
                        def is_actor_node(node):
                            if not node:
                                return False
                            return (node.shape == "circle") or \
                                   ("actor" in node.id.lower()) or \
                                   (node.label and "actor" in node.label.lower())
                                   
                        for node_id, node in parsed.nodes.items():
                            is_actor = is_actor_node(node)
                            if is_actor:
                                if node.subgraph is not None:
                                    errors.append(Finding("use-case-actor-must-be-outside-the-system-boundary", f"Use Case {basename} actor node '{node_id}' must be placed outside the system boundary subgraph (found in subgraph '{node.subgraph}').", location=basename))
                            else:
                                if node.subgraph != boundary_sub_id:
                                    errors.append(Finding("use-case-node-must-be-inside-the-system-boundary", f"Use Case {basename} use case node '{node_id}' must be defined inside the system boundary subgraph '{boundary_sub_id}'.", location=basename))
                                    
                                if val_rules.use_case_stadium_nodes_only:
                                    if node.shape != "stadium":
                                        errors.append(Finding("use-case-node-must-use-the-stadium-shape", f"Use Case {basename} use case node '{node_id}' must use the Mermaid stadium/oval shape ('stadium').", location=basename))
                                        
                        for conn in parsed.connections:
                            src_id = conn.from_node
                            tgt_id = conn.to_node
                            src_node = parsed.nodes.get(src_id)
                            tgt_node = parsed.nodes.get(tgt_id)
                            
                            src_is_actor = is_actor_node(src_node)
                            tgt_is_actor = is_actor_node(tgt_node)
                            
                            if val_rules.use_case_undirected_actor_links_only:
                                if (src_is_actor and not tgt_is_actor) or (not src_is_actor and tgt_is_actor):
                                    if "arrow" in conn.style:
                                        errors.append(Finding("use-case-actor-association-must-be-undirected", f"Use Case {basename} connection from '{src_id}' to '{tgt_id}' between Actor and Use Case must use an undirected link, not '{conn.style}'.", location=basename))
                                        
                            if val_rules.use_case_extend_arrow_direction_check:
                                has_extend_stereotype = bool(conn.label and re.search(r'(?:<<|&lt;&lt;|«)\s*extend\s*(?:>>|&gt;&gt;|»)', conn.label, re.I))
                                if has_extend_stereotype:
                                    pass
                                        
            flows_block_match = re.search(re.escape(use_case_alternate_flows_header) + r"(.*?)(?=##\s+6\.\s+Postconditions|\Z)", content, re.DOTALL | re.IGNORECASE)
            if flows_block_match:
                flows_block = flows_block_match.group(1)
                # Parse flows using configured regex
                flows = re.findall(use_case_flow_list_regex, flows_block, re.DOTALL)
                use_case_flow_limit = val_rules.use_case_flow_limit
                use_case_step_limit = val_rules.use_case_step_limit
                
                # Count validation/negative constraints across referenced features
                total_constraints = 0
                features_section_match = re.search(r"###\s+Required\s+Features(.*?)(?=###\s+Required\s+User\s+Stories|##\s+Source\s+References|\Z)", content, re.DOTALL | re.IGNORECASE)
                if features_section_match:
                    feature_checkboxes = re.findall(r"(?:-|\*)\s+\[[ xX]\]\s+.*", features_section_match.group(1))
                    
                    def norm_t(t):
                        if not t: return ""
                        t = t.strip().strip("\"'\u201c\u201d")
                        t = re.sub(r"^(epic|feature|feat|user[- ]story|use[- ]case|us|uc)[s]?(?:[- ]*\d+\s*[:\-]?|:)\s*", "", t, flags=re.IGNORECASE)
                        t = t.replace("-", " ")
                        t = re.sub(r"[^\w\s]", "", t)
                        return " ".join(t.split()).lower()

                    def ext_t(c):
                        tm = re.search(r"^title:\s*(['\"]?)(.*?)\1\s*$", c, re.MULTILINE)
                        if tm: return tm.group(2).strip()
                        hm = re.search(r"^#\s+(.*?)$", c, re.MULTILINE)
                        if hm: return hm.group(1).strip()
                        return None

                    features_dir = kwargs.get("features_dir")
                    if not features_dir:
                        features_dir = os.path.join(repo.workspace_dir, backlog_dirs.features)
                    
                    title_to_feature_path = {}
                    try:
                        for f_file in repo.get_feature_files(features_dir):
                            title_to_feature_path[f_file.filename.lower()] = os.path.join(features_dir, f_file.filename)
                            f_title = ext_t(f_file.content)
                            if f_title:
                                title_to_feature_path[norm_t(f_title)] = os.path.join(features_dir, f_file.filename)
                    except Exception as e:
                        print(f"Warning: Failed to scan features directory: {e}")

                    for cb in feature_checkboxes:
                        feat_path = None
                        feat_file_match = re.search(r"/(feat-\d{2,}-[a-z0-9\-]+\.md)", cb)
                        if feat_file_match:
                            feat_filename = feat_file_match.group(1)
                            feat_path = os.path.join(features_dir, feat_filename)
                            if not os.path.exists(feat_path):
                                # fallback: search recursively
                                for root, _, files in os.walk(repo.workspace_dir):
                                    if feat_filename in files:
                                        feat_path = os.path.join(root, feat_filename)
                                        break
                        else:
                            # Try matching by link text (Feature Title)
                            link_text_match = re.search(r"\[([^\]]+)\]\((?:https?://[^)]+)\)", cb)
                            if link_text_match:
                                feat_path = title_to_feature_path.get(norm_t(link_text_match.group(1)))
                        
                        if feat_path and os.path.exists(feat_path):
                            try:
                                with open(feat_path, "r", encoding="utf-8") as f:
                                    feat_content = f.read()
                                constraints_match = re.search(
                                    r"#{1,4}\s+(?:\d+(?:\.\d+)*\.?\s+)?Validation\s+(?:&|and)\s+Constraints\b(.*?)(?=\n#{1,4}\s+|\Z)",
                                    feat_content,
                                    re.DOTALL | re.IGNORECASE
                                )
                                if constraints_match:
                                    constraints_block = constraints_match.group(1)
                                    constraints = re.findall(
                                        r"^\s*(?:[-*+]|\d+[\.\)]|[a-zA-Z][\.\)])\s+\S+",
                                        constraints_block,
                                        re.MULTILINE
                                    )
                                    total_constraints += len(constraints)
                            except Exception as e:
                                print(f"Warning: Failed to parse feature constraints for {feat_path}: {e}")
                
                required_flow_count = max(use_case_flow_limit, total_constraints)
                if len(flows) < required_flow_count:
                    errors.append(Finding("use-case-requires-alternate-and-exception-flows", f"Use Case {basename} must contain at least {required_flow_count} detailed Alternate/Exception flows. Found only {len(flows)} flows. (Referenced features define {total_constraints} schema validation constraints, requiring at least that many alternate flows, with a minimum floor of {use_case_flow_limit}.)", location=basename))
                else:
                    for idx, flow in enumerate(flows):
                        steps = re.findall(use_case_numbered_step_regex, flow)
                        if len(steps) < use_case_step_limit:
                            errors.append(Finding("use-case-alternate-flow-requires-numbered-steps", f"Use Case {basename} alternate flow {idx+1} is too thin (must contain at least {use_case_step_limit} numbered steps).", location=basename))
            else:
                errors.append(Finding("use-case-requires-an-alternate-flows-block", f"Use Case {basename} is missing '{use_case_alternate_flows_header}' content block.", location=basename))
                
            if re.search(realization_matrix_header, content, re.IGNORECASE):
                if not re.search(realization_stories_header, content, re.IGNORECASE):
                    errors.append(Finding("use-case-requires-a-complete-realization-matrix", f"Use Case {basename} is missing '{realization_stories_header}' under Realization Matrix.", location=basename))
                if not re.search(realization_features_header, content, re.IGNORECASE):
                    errors.append(Finding("use-case-requires-a-complete-realization-matrix", f"Use Case {basename} is missing '### Required Features' under Realization Matrix.", location=basename))
                    
                stories_section_match = re.search(r"###\s+Required\s+User\s+Stories(.*?)(?=###\s+Required\s+Features|##\s+Source\s+References|\Z)", content, re.DOTALL | re.IGNORECASE)
                features_section_match = re.search(r"###\s+Required\s+Features(.*?)(?=###\s+Required\s+User\s+Stories|##\s+Source\s+References|\Z)", content, re.DOTALL | re.IGNORECASE)
                
                story_checkboxes = []
                if stories_section_match:
                    story_checkboxes = re.findall(r"-\s+\[[ xX]\]\s+.*", stories_section_match.group(1))
                    
                feature_checkboxes = []
                if features_section_match:
                    feature_checkboxes = re.findall(r"-\s+\[[ xX]\]\s+.*", features_section_match.group(1))
                    
                if not story_checkboxes:
                    errors.append(Finding("use-case-realization-matrix-requires-checklist-entries", f"Use Case {basename} Realization Matrix contains no User Story checkboxes under '### Required User Stories'.", location=basename))
                if not feature_checkboxes:
                    errors.append(Finding("use-case-realization-matrix-requires-checklist-entries", f"Use Case {basename} Realization Matrix contains no Feature checkboxes under '### Required Features'.", location=basename))
                    
                all_checkboxes = story_checkboxes + feature_checkboxes
                for cb in all_checkboxes:
                    url_match = re.search(r"\]\((https?://[^)]+)\)", cb)
                    if not url_match:
                        errors.append(Finding("checklist-item-requires-an-absolute-url", f"Use Case {basename} contains a checklist item with a missing or non-absolute markdown link URL: '{cb.strip()}'.", location=basename))
                    else:
                        url_str = url_match.group(1)
                        if not re.match(r"^https?://[a-zA-Z0-9.-]+/", url_str):
                            errors.append(Finding("checklist-item-requires-an-absolute-url", f"Use Case {basename} contains an invalid URL in realization matrix: '{url_str}'.", location=basename))
                            
                    justification_match = re.search(r"\s+\(([^)]+)\)$", cb)
                    if not justification_match or (url_match and justification_match.group(1) == url_match.group(1)):
                        errors.append(Finding("checklist-item-requires-a-semantic-justification", f"Use Case {basename} contains a checklist item with a missing or invalid parenthetical semantic justification at the end: '{cb.strip()}'.", location=basename))
                        
        epic_files = get_md_files(epics_dir)
        for filepath in epic_files:
            filename = os.path.basename(filepath)
            try:
                with open(filepath, "r", encoding="utf-8") as f:
                    content = f.read()
            except Exception as e:
                errors.append(Finding("backlog-document-must-be-readable", f"System Error: Failed to read epic file '{filename}': {e}", location=filename))
                continue
                
            self._validate_subagent_isolation(content, "Epic", filename, errors)
            self._validate_placeholders_and_links(
                content, "Epic", filename, errors, checkbox_syntax_regex,
                has_usecases=bool(usecase_files), has_userstories=bool(story_files)
            )
                
            validate_dotted_links(content, "Epic", filename, errors)
            validate_forbidden_diagram_types(content, "Epic", filename, errors)
                    
            required_epic_sections = required_sections.get("epic")
            if required_epic_sections is None:
                errors.append(Finding("uml-validator-configuration-must-be-complete", "System Error: Missing required_sections.epic config."))
                continue
            for pattern, header_name in required_epic_sections:
                if not re.search(pattern, content, re.IGNORECASE):
                    errors.append(Finding("backlog-document-requires-its-configured-sections", f"Epic {filename} is missing section '{header_name}'.", location=filename))
                    
            epic_req_diagrams = required_diagrams.get("epic")
            if epic_req_diagrams is None:
                errors.append(Finding("uml-validator-configuration-must-be-complete", "System Error: Missing required_diagrams.epic config."))
                continue
            for diag_type in epic_req_diagrams:
                if not re.search(r"```mermaid\s*\n\s*" + diag_type, content):
                    errors.append(Finding("backlog-document-requires-its-configured-diagrams", f"Epic {filename} is missing a valid diagram of type '{diag_type}'.", location=filename))
                elif diag_type == "classDiagram":
                    self._validate_class_diagram(
                        "Epic", filename, content, errors, class_parser, val_rules,
                        uml_primitives, visibility_prefixes, relationship_connectors,
                        choice_stereotypes, multiplicity_regex
                    )

        is_sysml = kwargs.get("is_sysml", False)
        if is_sysml:
            errors.extend(self.validate_user_story_interactions_and_lifelines(repo, **kwargs))
            errors.extend(self.validate_safety_invariants_and_rta_constraints(repo, **kwargs))
            errors.extend(self.validate_acceptance_criteria_and_test_cases(repo, **kwargs))
                    
        return errors

    def _validate_subagent_isolation(self, content: str, doc_type: str, filename: str, errors: List[str]):
        frontmatter_match = re.match(r"^---\s*\n(.*?)\n---\s*\n", content, re.DOTALL)
        has_subagent_tag = False
        if frontmatter_match:
            frontmatter_text = frontmatter_match.group(1)
            try:
                import yaml
                data = yaml.safe_load(frontmatter_text.replace('\x01', ''))
                if isinstance(data, dict):
                    data_lower = {str(k).lower(): str(v).lower() for k, v in data.items() if v is not None}
                    if data_lower.get("generation_mode") == "subagent" or data_lower.get("generation-mode") == "subagent":
                        has_subagent_tag = True
                    elif data_lower.get("subagent_drafted") == "true" or data_lower.get("subagent-drafted") == "true":
                        has_subagent_tag = True
            except Exception:
                for fm_line in frontmatter_text.splitlines():
                    if ":" in fm_line:
                        fm_parts = fm_line.split(":", 1)
                        fm_key = fm_parts[0].strip().lower()
                        fm_val = fm_parts[1].strip().strip('"').strip("'").lower()
                        if fm_key in ("generation_mode", "generation-mode") and fm_val == "subagent":
                            has_subagent_tag = True
                            break
                        if fm_key in ("subagent_drafted", "subagent-drafted") and fm_val == "true":
                            has_subagent_tag = True
                            break
        if not has_subagent_tag:
            errors.append(Finding("specification-requires-the-subagent-generation-mode-marker", f"{doc_type} {filename} violates the Item-Level Subagent Context Isolation mandate. Specifications must be drafted strictly inside a context-isolated subagent with 'generation_mode: subagent' in the frontmatter.", location=filename))

    def _validate_placeholders_and_links(
        self,
        content: str,
        doc_type: str,
        filename: str,
        errors: List[str],
        checkbox_syntax_regex: str,
        has_usecases: bool = False,
        has_userstories: bool = False
    ):
        # Applies to EVERY document type. The previous implementation ran the stub
        # scan only for Epic, Use Case and User Story, so Feature files were checked
        # for the literal string "IssueID" alone -- which is why 8 Feature files in
        # the live corpus carried entirely unpopulated '## Parent Epic' sections and
        # still passed validation (issue #281).
        for lineno, label, line_text in find_unresolved_placeholders(content):
            if doc_type == "Epic" and (
                re.search(r"\(\s*semantic linkage justification", line_text, re.I)
                or re.search(r"\[POPULATE:", line_text, re.I)
                or re.search(r"\{\{REQUIRED_JUSTIFICATION\}\}", line_text, re.I)
            ):
                errors.append(
                    Finding(
                        "epic-prohibit-unreplaced-placeholder-text",
                        f"Epic {filename}:{lineno} contains unreplaced placeholder text '{line_text}'. "
                        f"Epic specifications must replace all placeholder tokens with concise semantic justifications.",
                        location=filename,
                    )
                )
            else:
                errors.append(
                    Finding(
                        "specification-must-not-contain-template-placeholders",
                        f"{doc_type} {filename}:{lineno} contains {label}: '{line_text}'. "
                        f"Specification templates must be populated before registration.",
                        location=filename,
                    )
                )

        # Conditional stubs. Gated exactly as before, preserving issue #239: an Epic
        # may legitimately say "*(None registered)*" when no Use Cases or User Stories
        # exist, and only lies once they do.
        check_stubs = (
            (has_usecases or has_userstories) if doc_type == "Epic"
            else doc_type in ("Use Case", "User Story")
        )
        if check_stubs:
            for lineno, label, line_text in find_unresolved_placeholders(
                content, CONDITIONAL_STUB_PATTERNS
            ):
                errors.append(
                    Finding("specification-must-not-contain-unresolved-registration-tokens", f"{doc_type} {filename}:{lineno} contains {label}: '{line_text}'. "
                    f"All referenced items must be explicitly registered.", location=filename)
                )

        if doc_type == "Epic":
            req_match = re.search(r"##\s+2\.\s+Requirements\s+&\s+Checklist(.*?)(?=##|\Z)", content, re.DOTALL | re.IGNORECASE)
            if req_match:
                req_section = req_match.group(1)
                checkboxes = re.findall(checkbox_syntax_regex, req_section)
                for cb in checkboxes:
                    if not re.search(r"\[[^\]]+\]\(https?://[^)]+\)", cb):
                        errors.append(Finding("epic-checklist-item-requires-a-feature-file-link", f"Epic {filename} checklist item '{cb.strip()}' must be a valid markdown link pointing to the feature file absolute URL.", location=filename))

    def _validate_class_diagram(self, doc_type: str, filename: str, content: str, errors: List[str], class_parser, val_rules, uml_primitives, visibility_prefixes, relationship_connectors, choice_stereotypes, multiplicity_regex):
        class_diagram_matches = re.finditer(r"```mermaid\s*\n\s*classDiagram(.*?)(?=```|\Z)", content, re.DOTALL)
        for match in class_diagram_matches:
            diagram_body = match.group(1)
            diagram_full = match.group(0)
            
            # Issue 17: Flag curly braces conflict
            for line_idx, line in enumerate(diagram_body.splitlines()):
                line_strip = line.strip()
                if not line_strip:
                    continue
                if "{" in line_strip or "}" in line_strip:
                    is_block_start = re.match(r'^(class|namespace)\s+(?:`[^`]+`|[a-zA-Z0-9_\-.]+)\s*\{', line_strip, re.IGNORECASE)
                    is_block_end = (line_strip == "}")
                    if not is_block_start and not is_block_end:
                        errors.append(Finding("class-diagram-member-must-not-contain-braces", f"{doc_type} {filename} contains a syntax conflict in classDiagram on line {line_idx+1}: '{line_strip}'. Curly braces '{{}}' inside members/attributes are prohibited due to Mermaid parse errors. Use standard attribute notation or separate notes for constraints.", location=filename))
                if line_strip.lower().startswith("note ") or line_strip.lower().startswith("note\t") or line_strip.lower() == "note":
                    note_match = re.match(r'^\s*note\s+(?:for\s+(?:`[^`]+`|[a-zA-Z0-9_\-.]+)\s*)?(.*)$', line_strip, re.IGNORECASE)
                    if note_match:
                        note_content = note_match.group(1).strip()
                        if note_content.startswith(':'):
                            note_content = note_content[1:].strip()
                        if ':' in note_content:
                            errors.append(Finding("class-diagram-note-must-not-contain-colons", f"{doc_type} {filename} contains a syntax conflict in classDiagram on line {line_idx+1}: '{line_strip}'. Colons ':' inside note strings are prohibited due to Mermaid rendering issues.", location=filename))

            try:
                parsed_cd = class_parser.parse(diagram_full)
            except Exception as e:
                errors.append(Finding("class-diagram-must-parse", f"{doc_type} {filename} contains an unparsable UML Class Diagram: {e}", location=filename))
                continue

            for err in parsed_cd.parse_errors:
                errors.append(Finding("class-diagram-must-parse", f"{doc_type} {filename} class diagram parse error: {err}", location=filename))

            if not re.search(relationship_connectors, diagram_body):
                if not parsed_cd.relationships:
                    errors.append(Finding("class-diagram-requires-relationships", f"{doc_type} {filename} contains a UML Class Diagram with no relationships. Isolated classes are prohibited; you must illustrate containment/inheritance/choice composition.", location=filename))
                else:
                    errors.append(
                        Finding("class-diagram-connector-must-be-recognised", f"{doc_type} {filename} contains UML Class Diagram relationships "
                        f"using connector formats not recognized by the configured relationship_connectors. "
                        f"The parser detected {len(parsed_cd.relationships)} relationship(s) in non-standard format. "
                        f"Verify connectors match the configured set.", location=filename)
                    )
                
            classes = parsed_cd.classes
            relationships = parsed_cd.relationships
            
            for rel in relationships:
                if rel.label and any(stereo in rel.label for stereo in ["<<", ">>", "&lt;&lt;", "&gt;&gt;", "«", "»"]):
                    errors.append(Finding("class-diagram-relationship-must-not-carry-a-stereotype", f"{doc_type} {filename} contains invalid stereotype/double angle brackets on relationship line between '{rel.from_class}' and '{rel.to_class}': '{rel.label}'. Relationship labels must not contain stereotypes.", location=filename))
                if rel.type in ("composition", "aggregation"):
                    if not rel.from_multiplicity and not rel.to_multiplicity:
                        errors.append(Finding("class-diagram-relationship-requires-multiplicity", f"{doc_type} {filename} contains an aggregation or composition relationship without multiplicity tags. You must enforce multiplicity tags (e.g., '1', '0..1', '0..*') on association ends between '{rel.from_class}' and '{rel.to_class}'.", location=filename))            
            adj = {c: set() for c in classes}
            for rel in relationships:
                u = rel.from_class
                v = rel.to_class
                if u not in adj:
                    adj[u] = set()
                if v not in adj:
                    adj[v] = set()
                adj[u].add(v)
                adj[v].add(u)
                
            for c, neighbors in adj.items():
                if len(neighbors) == 0:
                    errors.append(Finding("class-diagram-class-must-not-be-isolated", f"{doc_type} {filename} contains class '{c}' with zero relationships. Isolated classes are prohibited.", location=filename))
                    
            if classes:
                start_node = next(iter(classes))
                visited = set()
                queue = [start_node]
                visited.add(start_node)
                while queue:
                    curr = queue.pop(0)
                    for neighbor in adj.get(curr, []):
                        if neighbor not in visited:
                            visited.add(neighbor)
                            queue.append(neighbor)
                unvisited = set(classes.keys()) - visited
                if unvisited:
                    errors.append(Finding("class-diagram-must-be-connected", f"{doc_type} {filename} contains a disconnected UML Class Diagram. Classes {list(unvisited)} are not structurally connected to '{start_node}'.", location=filename))
                    
            for cls_name, cls_info in classes.items():
                is_enum = any("<<enumeration>>" in (a.name or "") or "<<enumeration>>" in (a.raw or "") for a in cls_info.attributes)
                for attr in cls_info.attributes:
                    if attr.raw and "<<" in attr.raw and ">>" in attr.raw:
                        continue
                    if is_enum:
                        continue
                    attr_type = attr.type
                    if not attr_type:
                        errors.append(Finding("class-attribute-requires-a-type", f"{doc_type} {filename} class '{cls_name}' attribute '{attr.name}' is missing a type.", location=filename))
                        continue
                        
                    base_type = re.sub(multiplicity_regex + r'$', '', attr_type).strip()
                    if base_type not in uml_primitives and base_type not in classes:
                        errors.append(Finding("class-attribute-type-must-be-a-uml-primitive", f"{doc_type} {filename} class '{cls_name}' attribute '{attr.name}' has invalid type '{attr_type}'. UML primitive types must be {', '.join(sorted(uml_primitives))} (case-sensitive), or reference another class.", location=filename))
                        
            choice_classes = set()
            for line in diagram_full.splitlines():
                line_clean = line.strip()
                for st in choice_stereotypes:
                    st_esc = re.escape(st)
                    m1 = re.search(st_esc + r"\s*([a-zA-Z0-9_\-.:]+)", line_clean, re.IGNORECASE)
                    if m1:
                        choice_classes.add(m1.group(1))
                    m2 = re.search(r"class\s+([a-zA-Z0-9_\-.:]+)\s+.*" + st_esc, line_clean, re.IGNORECASE)
                    if m2:
                        choice_classes.add(m2.group(1))
            for cls_name, cls_info in classes.items():
                matched_stereotype = False
                for st in choice_stereotypes:
                    if st in cls_name:
                        matched_stereotype = True
                        break
                    for attr in cls_info.attributes:
                        if attr.raw and (f"<<{st}>>" in attr.raw or f"&lt;&lt;{st}&gt;&gt;" in attr.raw):
                            matched_stereotype = True
                            break
                    for method in cls_info.methods:
                        if method.raw and (f"<<{st}>>" in method.raw or f"&lt;&lt;{st}&gt;&gt;" in method.raw):
                            matched_stereotype = True
                            break
                    if matched_stereotype:
                        break
                if matched_stereotype:
                    choice_classes.add(cls_name)
                            
            for choice_cls in choice_classes:
                has_subclass = False
                for rel in relationships:
                    if rel.type == "generalization":
                        is_parent = False
                        if rel.direction == "backward" and rel.from_class == choice_cls:
                            is_parent = True
                        elif rel.direction == "forward" and rel.to_class == choice_cls:
                            is_parent = True
                        if is_parent:
                            has_subclass = True
                            break
                if not has_subclass:
                    errors.append(Finding("choice-class-requires-a-generalization-subclass", f"{doc_type} {filename} choice class '{choice_cls}' must have at least one subclass inheriting from it via generalization (<|--).", location=filename))
                    
            for cls_name, cls_info in classes.items():
                is_enum = any("<<enumeration>>" in (a.name or "") or "<<enumeration>>" in (a.raw or "") for a in cls_info.attributes)
                for attr in cls_info.attributes:
                    if attr.raw and "<<" in attr.raw and ">>" in attr.raw:
                        continue
                    if is_enum:
                        continue
                    if attr.visibility not in visibility_prefixes:
                        errors.append(Finding("class-member-requires-a-visibility-prefix", f"{doc_type} {filename} class '{cls_name}' attribute '{attr.name}' is missing a valid UML visibility prefix ({', '.join(sorted(visibility_prefixes))}).", location=filename))
                    has_mult = bool(attr.multiplicity)
                    if not has_mult and attr.type:
                        if re.search(multiplicity_regex + r'$', attr.type):
                            has_mult = True
                    if not has_mult:
                        errors.append(Finding("class-member-requires-a-multiplicity", f"{doc_type} {filename} class '{cls_name}' attribute '{attr.name}' is missing a multiplicity (e.g. [1], [0..1], [0..*]).", location=filename))
                        
                for method in cls_info.methods:
                    if method.visibility not in visibility_prefixes:
                        errors.append(Finding("class-member-requires-a-visibility-prefix", f"{doc_type} {filename} class '{cls_name}' method '{method.name}' is missing a valid UML visibility prefix ({', '.join(sorted(visibility_prefixes))}).", location=filename))
                    if not method.return_type or method.return_type.lower() in ("void", "none"):
                        continue
                    has_mult = False
                    if method.return_type:
                        if '[' in method.return_type or ']' in method.return_type:
                            if re.search(multiplicity_regex, method.return_type):
                                has_mult = True
                        elif '[' in method.raw or ']' in method.raw:
                            if ")" in method.raw:
                                return_suffix = method.raw.rsplit(")", 1)[-1]
                            else:
                                return_suffix = method.raw
                            if '[' in return_suffix or ']' in return_suffix:
                                if (re.search(r'\)\s*' + multiplicity_regex, method.raw) or re.search(multiplicity_regex + r'\s*$', method.raw)):
                                    has_mult = True
                    if not has_mult:
                        errors.append(Finding("class-member-requires-a-multiplicity", f"{doc_type} {filename} class '{cls_name}' method '{method.name}' is missing a multiplicity (e.g. [1], [0..1], [0..*]) in its return signature.", location=filename))

            for cls_name, cls_info in classes.items():
                is_component = any("<<component>>" in (a.name or "") or "<<component>>" in (a.raw or "") for a in cls_info.attributes)
                if not is_component:
                    if "<<component>>" in cls_name or "&lt;&lt;component&gt;&gt;" in cls_name:
                        is_component = True
                if not is_component:
                    for line in diagram_full.splitlines():
                        line_clean = line.strip()
                        if ("<<component>>" in line_clean or "&lt;&lt;component&gt;&gt;" in line_clean) and cls_name in line_clean:
                            is_component = True
                            break
                if is_component:
                    real_attributes = [a for a in cls_info.attributes if not (a.raw and "<<" in a.raw and ">>" in a.raw)]
                    if not real_attributes and not cls_info.methods:
                        errors.append(Finding("subsystem-component-class-must-declare-members", f"{doc_type} {filename} subsystem component class '{cls_name}' is empty. Subsystem components must define at least one attribute or operation.", location=filename))

            # Validate schema containers
            import yaml
            schema_containers = []
            try:
                frontmatter_match = re.match(r"^---\s*\n(.*?)\n---\s*\n", content, re.DOTALL)
                if frontmatter_match:
                    fm_data = yaml.safe_load(frontmatter_match.group(1).replace('\x01', ''))
                    if isinstance(fm_data, dict):
                        schema_containers = fm_data.get("schema_containers", [])
            except Exception:
                pass

            if schema_containers:
                classes_lower_map = {c.lower(): c for c in classes.keys()}
                for sc_entry in schema_containers:
                    if isinstance(sc_entry, dict):
                        path = sc_entry.get("path", "")
                    else:
                        path = sc_entry
                    if not path:
                        continue
                    segments = [s for s in path.split("/") if s]
                    
                    if segments:
                        seg = segments[-1]
                        seg_clean = re.sub(r"^[^:]+:", "", seg)
                        seg_parts = re.split(r'[-_]', seg_clean)
                        exp_cls = "".join(p.capitalize() for p in seg_parts if p)
                        
                        fallback_cls = None
                        if ":" in seg:
                            prefix, local_name = seg.split(":", 1)
                            prefix_c = "".join(p.capitalize() for p in re.split(r'[-_]', prefix) if p)
                            local_c = "".join(p.capitalize() for p in re.split(r'[-_]', local_name) if p)
                            fallback_cls = f"{prefix_c}_{local_c}"
                            
                        if exp_cls.lower() not in classes_lower_map and (not fallback_cls or fallback_cls.lower() not in classes_lower_map):
                            if fallback_cls:
                                errors.append(Finding("class-diagram-must-model-the-schema-container-path", f"[{doc_type.upper()}] [{filename}] UML Class Diagram is missing class node '{exp_cls}' or '{fallback_cls}' for schema container target node '{seg}' in path '{path}'.", location=filename))
                            else:
                                errors.append(Finding("class-diagram-must-model-the-schema-container-path", f"[{doc_type.upper()}] [{filename}] UML Class Diagram is missing class node '{exp_cls}' for schema container target node '{seg}' in path '{path}'.", location=filename))

                    # Containment edges between consecutive segments.
                    #
                    # rules/uml-model-integrity.md documents this beside the node rule
                    # above -- "consecutive segments of that path must be joined by a
                    # relationship representing containment" -- but only the node half
                    # was implemented. Present nodes with absent edges reproduce the
                    # schema's vocabulary without its structure, which is exactly what
                    # the rule says must not happen.
                    #
                    # Only pairs whose classes are BOTH present are checked. A missing
                    # class is already reported by the node rule above, and reporting it
                    # twice under two rule ids would make one defect look like two.
                    resolved = []
                    for seg_i in segments:
                        seg_i_clean = re.sub(r"^[^:]+:", "", seg_i)
                        cls_i = "".join(
                            p.capitalize() for p in re.split(r'[-_]', seg_i_clean) if p
                        )
                        resolved.append(classes_lower_map.get(cls_i.lower()))

                    connected = set()
                    for rel in relationships:
                        connected.add((rel.from_class, rel.to_class))
                        connected.add((rel.to_class, rel.from_class))

                    for parent, child in zip(resolved, resolved[1:]):
                        if not parent or not child or parent == child:
                            continue
                        if (parent, child) not in connected:
                            errors.append(Finding(
                                "class-diagram-must-model-the-schema-containment-relationships",
                                f"[{doc_type.upper()}] [{filename}] UML Class Diagram declares "
                                f"classes '{parent}' and '{child}' but no relationship between "
                                f"them, while schema container path '{path}' makes '{child}' a "
                                f"child of '{parent}'. Add the containment relationship "
                                f"(e.g. '{parent} *-- {child}').",
                                location=filename,
                            ))

        
    def build_global_classes(self, repo: WorkspaceRepository, features_dir: str, epics_dir: str = None) -> Dict[str, Any]:
        """
        Build a global class dictionary from class diagrams in feature and epic files.

        Parses all feature spec files, then optionally epic spec files, merging
        their UML class diagrams into a single dictionary keyed by class name.
        Duplicate attributes and methods are skipped (first-writer wins).

        Args:
            repo: WorkspaceRepository for accessing feature files.
            features_dir: Path to the directory containing feature markdown files.
            epics_dir: Optional path to epic markdown files; when provided, class
                       diagrams from epic files are also merged.

        Returns:
            Dict mapping class names to dicts with keys ``name``, ``attributes``,
            and ``methods``.  Empty dict if no UML class diagrams are found.
        """
        global_classes = {}
        feature_files = repo.get_feature_files(features_dir)
        parser = MermaidClassDiagramParser(repo)
        for feat in feature_files:
            content = feat.content
            class_diagram_matches = re.finditer(r"```mermaid\s*\n\s*classDiagram(.*?)(?=```|\Z)", content, re.DOTALL)
            for match in class_diagram_matches:
                parsed_cd = parser.parse(match.group(0))
                for class_name, class_info in parsed_cd.classes.items():
                    if class_name not in global_classes:
                        global_classes[class_name] = {
                            "name": class_name,
                            "attributes": [],
                            "methods": [],
                            "notes": []
                        }
                    if "notes" not in global_classes[class_name]:
                        global_classes[class_name]["notes"] = []
                    for note in getattr(class_info, "notes", []):
                        if note not in global_classes[class_name]["notes"]:
                            global_classes[class_name]["notes"].append(note)
                    existing_attrs = {a["name"] for a in global_classes[class_name]["attributes"]}
                    for attr in class_info.attributes:
                        if attr.name and "<<" in attr.name and ">>" in attr.name:
                            continue
                        if attr.name not in existing_attrs:
                            global_classes[class_name]["attributes"].append({
                                "name": attr.name,
                                "visibility": attr.visibility,
                                "type": attr.type,
                                "multiplicity": attr.multiplicity,
                                "constraints": attr.constraints,
                                "raw": attr.raw
                            })
                    existing_methods = {m["name"] for m in global_classes[class_name]["methods"]}
                    for method in class_info.methods:
                        if method.name not in existing_methods:
                            global_classes[class_name]["methods"].append({
                                "name": method.name,
                                "visibility": method.visibility,
                                "parameters": method.parameters,
                                "return_type": method.return_type,
                                "constraints": method.constraints,
                                "raw": method.raw
                            })
        if epics_dir and os.path.exists(epics_dir):
            epic_files = [os.path.join(epics_dir, f) for f in os.listdir(epics_dir) if f.endswith(".md")]
            for ep_path in epic_files:
                try:
                    with open(ep_path, "r", encoding="utf-8") as f:
                        content = f.read()
                except Exception:
                    continue
                class_diagram_matches = re.finditer(r"```mermaid\s*\n\s*classDiagram(.*?)(?=```|\Z)", content, re.DOTALL)
                for match in class_diagram_matches:
                    parsed_cd = parser.parse(match.group(0))
                    for class_name, class_info in parsed_cd.classes.items():
                        if class_name not in global_classes:
                            global_classes[class_name] = {
                                "name": class_name,
                                "attributes": [],
                                "methods": [],
                                "notes": []
                            }
                        if "notes" not in global_classes[class_name]:
                            global_classes[class_name]["notes"] = []
                        for note in getattr(class_info, "notes", []):
                            if note not in global_classes[class_name]["notes"]:
                                global_classes[class_name]["notes"].append(note)
                        existing_attrs = {a["name"] for a in global_classes[class_name]["attributes"]}
                        for attr in class_info.attributes:
                            if attr.name and "<<" in attr.name and ">>" in attr.name:
                                continue
                            if attr.name not in existing_attrs:
                                global_classes[class_name]["attributes"].append({
                                    "name": attr.name,
                                    "visibility": attr.visibility,
                                    "type": attr.type,
                                    "multiplicity": attr.multiplicity,
                                    "constraints": attr.constraints,
                                    "raw": attr.raw
                                })
                        existing_methods = {m["name"] for m in global_classes[class_name]["methods"]}
                        for method in class_info.methods:
                            if method.name not in existing_methods:
                                global_classes[class_name]["methods"].append({
                                    "name": method.name,
                                    "visibility": method.visibility,
                                    "parameters": method.parameters,
                                    "return_type": method.return_type,
                                    "constraints": method.constraints,
                                    "raw": method.raw
                                })
        return global_classes
        
    def build_classes_from_features(self, matching_features: List[FeatureFile], repo: WorkspaceRepository) -> Dict[str, Any]:
        classes = {}
        parser = MermaidClassDiagramParser(repo)
        for feat in matching_features:
            content = feat.content
            class_diagram_matches = re.finditer(r"```mermaid\s*\n\s*classDiagram(.*?)(?=```|\Z)", content, re.DOTALL)
            for match in class_diagram_matches:
                parsed_cd = parser.parse(match.group(0))
                for class_name, class_info in parsed_cd.classes.items():
                    if class_name not in classes:
                        classes[class_name] = {
                            "name": class_name,
                            "attributes": [],
                            "methods": []
                        }
                    existing_attrs = {a["name"] for a in classes[class_name]["attributes"]}
                    for attr in class_info.attributes:
                        if attr.name not in existing_attrs:
                            classes[class_name]["attributes"].append({
                                "name": attr.name,
                                "visibility": attr.visibility,
                                "type": attr.type,
                                "multiplicity": attr.multiplicity,
                                "constraints": attr.constraints,
                                "raw": attr.raw
                            })
                    existing_methods = {m["name"] for m in classes[class_name]["methods"]}
                    for method in class_info.methods:
                        if method.name not in existing_methods:
                            classes[class_name]["methods"].append({
                                "name": method.name,
                                "visibility": method.visibility,
                                "parameters": method.parameters,
                                "return_type": method.return_type,
                                "constraints": method.constraints,
                                "raw": method.raw
                            })
        return classes

    def validate_user_story_interactions_and_lifelines(self, repo: WorkspaceRepository, **kwargs) -> List[Finding]:
        """
        Check 20: User Story Interaction & Sequence Lifeline Audit.
        Verifies that User Story sequence diagrams bind to valid SysML parts and interaction message steps.
        Ensures internal participant lifelines map to SysML PartDefs and messages map to SysML interaction
        message flows or part operations/actions.
        """
        rules = repo.get_codebase_rules()
        backlog_dirs = rules.backlog_directories
        schemas_dir_rel = getattr(backlog_dirs, "schemas", None)
        schemas_dir = os.path.join(repo.workspace_dir, schemas_dir_rel) if schemas_dir_rel else os.path.join(repo.workspace_dir, "schemas")
        user_stories_dir_rel = getattr(backlog_dirs, "user_stories", None)
        user_stories_dir = os.path.join(repo.workspace_dir, user_stories_dir_rel) if user_stories_dir_rel else os.path.join(repo.workspace_dir, "docs", "user-stories")

        sysml_files, all_pkgs, all_parts, _, all_actions, all_ops, all_interactions, _, _, _, errors = _load_all_sysml_elements_full(repo, schemas_dir)
        if errors:
            return errors
        if not sysml_files:
            return []

        if not user_stories_dir or not os.path.exists(user_stories_dir):
            story_files = []
        else:
            story_files = [f for f in sorted(os.listdir(user_stories_dir)) if f.endswith(".md")]

        if not story_files:
            return []

        # Collect valid SysML part names & global classes
        valid_parts: Set[str] = set()
        for p in all_parts:
            p_name = getattr(p, "name", "")
            if p_name:
                valid_parts.add(p_name)

        features_dir_rel = getattr(backlog_dirs, "features", None)
        features_dir = os.path.join(repo.workspace_dir, features_dir_rel) if features_dir_rel else None
        epics_dir_rel = getattr(backlog_dirs, "epics", None)
        epics_dir = os.path.join(repo.workspace_dir, epics_dir_rel) if epics_dir_rel else None
        global_classes = kwargs.get("global_classes") or self.build_global_classes(repo, features_dir, epics_dir)
        valid_classifiers = set(valid_parts) | set(global_classes.keys())

        sequence_parser = MermaidSequenceDiagramParser()
        covered_interactions: Set[str] = set()

        for filename in story_files:
            filepath = os.path.join(user_stories_dir, filename)
            try:
                with open(filepath, "r", encoding="utf-8") as f:
                    content = f.read()
            except Exception as exc:
                errors.append(Finding(
                    "user-story-not-readable",
                    f"Failed to read User Story specification '{filename}': {exc}",
                    location="user-stories"
                ))
                continue

            fm = None
            fm_match = re.match(r"^---\s*\n(.*?)\n---\s*\n", content, re.DOTALL)
            if fm_match:
                try:
                    fm = yaml.safe_load(fm_match.group(1).replace('\x01', ''))
                except Exception:
                    pass

            seq_matches = list(re.finditer(r"```mermaid\s*\n\s*sequenceDiagram(.*?)(?=```|\Z)", content, re.DOTALL))
            for sm in seq_matches:
                seq_code = sm.group(0)
                parsed_seq = sequence_parser.parse(seq_code)

                # 1. Lifeline Part Binding
                for alias, lf in parsed_seq.lifelines.items():
                    role = (lf.role or "").lower()
                    is_external_actor = (role == "actor")
                    cls_name = lf.classifier_name

                    if not is_external_actor:
                        if cls_name and (cls_name not in valid_classifiers and cls_name not in valid_parts):
                            errors.append(Finding(
                                "user-story-lifeline-part-invalid",
                                f"User Story '{filename}': Sequence diagram lifeline '{alias}' specifies classifier '{cls_name}' which does not bind to any valid SysML part definition.",
                                location="user-stories"
                            ))

                # 2. Message Flow Operation Matching
                for msg in parsed_seq.messages:
                    if msg.arrow_type in ("sync", "async") and msg.operation:
                        op_name = msg.operation
                        rx_lf = parsed_seq.lifelines.get(msg.receiver)
                        rx_cls = rx_lf.classifier_name if rx_lf else None
                        rx_role = (rx_lf.role or "").lower() if rx_lf else ""

                        # External actors are exempt from internal interface checking
                        if rx_role == "actor":
                            continue

                        # Determine if operation exists on the specific receiver classifier or part
                        cls_has_method = False
                        if rx_cls:
                            if rx_cls in global_classes:
                                cls_has_method = any(m["name"] == op_name for m in global_classes[rx_cls]["methods"])
                            if not cls_has_method:
                                for part in all_parts:
                                    if getattr(part, "name", None) == rx_cls:
                                        part_ops = [getattr(op, "name", None) for op in (getattr(part, "operations", []) or [])]
                                        part_acts = [getattr(act, "name", None) for act in (getattr(part, "actions", []) or [])]
                                        if op_name in part_ops or op_name in part_acts:
                                            cls_has_method = True
                                            break

                        # Fallback: check if message is a declared SysML interaction message step
                        if not cls_has_method:
                            for inter in all_interactions:
                                inter_msgs = getattr(inter, "messages", []) or []
                                inter_trgs = getattr(inter, "triggers", []) or []
                                if op_name in inter_msgs or op_name in inter_trgs:
                                    cls_has_method = True
                                    break

                        if not cls_has_method:
                            errors.append(Finding(
                                "user-story-interaction-step-invalid",
                                f"User Story '{filename}': Sequence diagram message '{op_name}' is not declared on receiver '{rx_cls}' or in SysML interaction message flows.",
                                location="user-stories"
                            ))

            # 3. Track covered SysML interactions
            for inter in all_interactions:
                inter_name = getattr(inter, "name", "")
                if not inter_name:
                    continue
                if fm and isinstance(fm, dict) and fm.get("interaction") == inter_name:
                    covered_interactions.add(inter_name)
                elif re.search(rf"\b(?:interaction|Interaction|SysML\s+Interaction\s+Def)\s*:?\s*`?{re.escape(inter_name)}`?\b", content):
                    covered_interactions.add(inter_name)
                elif re.search(rf"\b{re.escape(inter_name)}\b", content):
                    covered_interactions.add(inter_name)

        # 4. Bidirectional Check: SysML interaction realization
        for inter in all_interactions:
            inter_name = getattr(inter, "name", "")
            if inter_name and inter_name not in covered_interactions:
                errors.append(Finding(
                    "sysml-interaction-uncovered",
                    f"SysML interaction def '{inter_name}' is not realized by any User Story sequence diagram.",
                    location="user-stories"
                ))

        return errors

    def validate_safety_invariants_and_rta_constraints(self, repo: WorkspaceRepository, **kwargs) -> List[Finding]:
        """
        Check 21: Safety Invariant & RTA Constraint Assertion Audit.
        Verifies that all STPA Unsafe Control Actions (UCAs) have corresponding 'assert constraint'
        declarations in the SysML AST for Run-Time Assurance (RTA) mathematical verification with
        Simulink Design Verifier (SLDV).
        """
        rules = repo.get_codebase_rules()
        backlog_dirs = rules.backlog_directories
        schemas_dir_rel = getattr(backlog_dirs, "schemas", None)
        schemas_dir = os.path.join(repo.workspace_dir, schemas_dir_rel) if schemas_dir_rel else os.path.join(repo.workspace_dir, "schemas")

        sysml_files, all_pkgs, all_parts, _, _, _, _, all_constraints, _, _, errors = _load_all_sysml_elements_full(repo, schemas_dir)
        if errors:
            return errors
        if not sysml_files:
            return []

        # Discover STPA UCAs
        stpa_content = kwargs.get("stpa_content")
        discovered_ucas: List[Dict[str, Any]] = []

        if stpa_content:
            try:
                from scripts.compile_sysml import parse_stpa_ucas
                discovered_ucas.extend(parse_stpa_ucas(stpa_content))
            except ImportError:
                pass
        else:
            # Scan safety documentation directories
            safety_candidates = []
            for sub_dir in ["docs/safety", "docs/architecture/blueprints", "docs/requirements"]:
                full_dir = os.path.join(repo.workspace_dir, sub_dir)
                if os.path.exists(full_dir):
                    for f in sorted(os.listdir(full_dir)):
                        if f.endswith(".md"):
                            safety_candidates.append(os.path.join(full_dir, f))

            stpa_matrix_file = os.path.join(repo.workspace_dir, "docs", "safety", "STPA_MATRIX.md")
            if os.path.exists(stpa_matrix_file) and stpa_matrix_file not in safety_candidates:
                safety_candidates.append(stpa_matrix_file)

            try:
                from scripts.compile_sysml import parse_stpa_ucas
            except ImportError:
                parse_stpa_ucas = None

            for sc_path in safety_candidates:
                try:
                    with open(sc_path, "r", encoding="utf-8") as f:
                        s_text = f.read()
                    if parse_stpa_ucas:
                        file_ucas = parse_stpa_ucas(s_text)
                    else:
                        file_ucas = []
                        for m in re.finditer(r'\b(UCA(?:-[A-Za-z0-9_]+)?-\d+)\b', s_text):
                            file_ucas.append({"id": m.group(1)})
                    for u in file_ucas:
                        if not any(existing["id"] == u["id"] for existing in discovered_ucas):
                            discovered_ucas.append(u)
                except Exception:
                    pass

        if not discovered_ucas:
            return []

        # Check each UCA against assert constraint definitions
        assert_constraints = [c for c in all_constraints if getattr(c, "is_assertion", False)]
        
        for uca in discovered_ucas:
            uca_id = uca.get("id", "")
            if not uca_id:
                continue
            clean_uca_id = re.sub(r'[^a-zA-Z0-9_]', '_', uca_id)

            found_assert = False
            for c in assert_constraints:
                c_name = getattr(c, "name", "")
                c_doc = getattr(c, "doc", "")
                c_expr = getattr(c, "expression", "")
                
                if (
                    clean_uca_id.lower() in c_name.lower()
                    or uca_id.lower() in c_name.lower()
                    or clean_uca_id.lower() in c_doc.lower()
                    or uca_id.lower() in c_doc.lower()
                    or clean_uca_id.lower() in c_expr.lower()
                    or uca_id.lower() in c_expr.lower()
                ):
                    found_assert = True
                    break

            if not found_assert:
                errors.append(Finding(
                    "stpa-uca-missing-assert-constraint",
                    f"STPA Unsafe Control Action '{uca_id}' has no corresponding 'assert constraint' declaration in SysML AST.",
                    location="safety"
                ))

        # Check that all assert constraint expressions are non-empty for SLDV RTA verification
        for c in assert_constraints:
            c_name = getattr(c, "name", "")
            c_expr = getattr(c, "expression", "")
            if not c_expr or not str(c_expr).strip():
                errors.append(Finding(
                    "rta-constraint-expression-empty",
                    f"SysML assert constraint '{c_name}' has an empty expression; required for Simulink Design Verifier (SLDV) RTA verification.",
                    location="schemas"
                ))

        return errors

    def validate_acceptance_criteria_and_test_cases(self, repo: WorkspaceRepository, **kwargs) -> List[Finding]:
        """
        Check 22: Acceptance Criteria Test Case & Verification Binding Audit.
        Verifies that every BDD scenario is bound to a SysML 'test case def' and verifies
        its parent safety requirement via formal 'verify requirement' bindings.
        """
        rules = repo.get_codebase_rules()
        backlog_dirs = rules.backlog_directories
        schemas_dir_rel = getattr(backlog_dirs, "schemas", None)
        schemas_dir = os.path.join(repo.workspace_dir, schemas_dir_rel) if schemas_dir_rel else os.path.join(repo.workspace_dir, "schemas")
        user_stories_dir_rel = getattr(backlog_dirs, "user_stories", None)
        user_stories_dir = os.path.join(repo.workspace_dir, user_stories_dir_rel) if user_stories_dir_rel else os.path.join(repo.workspace_dir, "docs", "user-stories")

        sysml_files, all_pkgs, all_parts, _, _, _, _, _, all_test_cases, all_requirements, errors = _load_all_sysml_elements_full(repo, schemas_dir)
        if errors:
            return errors
        if not sysml_files:
            return []

        if not user_stories_dir or not os.path.exists(user_stories_dir):
            story_files = []
        else:
            story_files = [f for f in sorted(os.listdir(user_stories_dir)) if f.endswith(".md")]

        if not story_files:
            return []

        # Build AST lookup maps
        test_cases_by_name: Dict[str, Any] = {}
        for tc in all_test_cases:
            tc_name = getattr(tc, "name", "")
            if tc_name:
                test_cases_by_name[tc_name] = tc

        reqs_by_id_or_name: Dict[str, Any] = {}
        for r in all_requirements:
            r_name = getattr(r, "name", "")
            r_id = getattr(r, "req_id", "")
            if r_name:
                reqs_by_id_or_name[r_name] = r
            if r_id:
                reqs_by_id_or_name[r_id] = r

        bound_test_cases: Set[str] = set()

        for filename in story_files:
            filepath = os.path.join(user_stories_dir, filename)
            try:
                with open(filepath, "r", encoding="utf-8") as f:
                    content = f.read()
            except Exception as exc:
                errors.append(Finding(
                    "user-story-not-readable",
                    f"Failed to read User Story specification '{filename}': {exc}",
                    location="user-stories"
                ))
                continue

            fm = None
            fm_match = re.match(r"^---\s*\n(.*?)\n---\s*\n", content, re.DOTALL)
            if fm_match:
                try:
                    fm = yaml.safe_load(fm_match.group(1).replace('\x01', ''))
                except Exception:
                    pass

            # Extract test case bindings
            tc_refs = []
            if fm and isinstance(fm, dict):
                tc_val = fm.get("test_case") or fm.get("test_case_def") or fm.get("test_cases")
                if isinstance(tc_val, list):
                    tc_refs.extend([str(t) for t in tc_val])
                elif tc_val:
                    tc_refs.append(str(tc_val))

            # Markdown extraction
            header_tc_refs = re.findall(r'(?:SysML\s+Test\s+Case\s+Def\s*:\s*`?|test\s+case\s+(?:def\s+)?)([A-Za-z0-9_]+)', content)
            tc_refs.extend(header_tc_refs)

            # TC_ prefix scan
            tc_prefix_matches = re.findall(r'\b(TC_[A-Za-z0-9_]+)\b', content)
            tc_refs.extend(tc_prefix_matches)

            # Filter out non-testcase generic tokens
            tc_refs = [t for t in set(tc_refs) if t not in ("TestCase", "Name", "test_case", "test_case_def")]

            if not tc_refs:
                errors.append(Finding(
                    "user-story-missing-test-case-binding",
                    f"User Story '{filename}': BDD Acceptance Criteria scenario is missing formal SysML 'test case def' binding.",
                    location="user-stories"
                ))
                continue

            for tc_name in tc_refs:
                if tc_name not in test_cases_by_name:
                    errors.append(Finding(
                        "user-story-test-case-not-in-ast",
                        f"User Story '{filename}': Bound test case '{tc_name}' is not defined in SysML AST.",
                        location="user-stories"
                    ))
                else:
                    bound_test_cases.add(tc_name)
                    tc_obj = test_cases_by_name[tc_name]
                    verified_reqs = getattr(tc_obj, "verified_requirements", []) or []

                    if not verified_reqs:
                        errors.append(Finding(
                            "test-case-missing-verify-requirement",
                            f"User Story '{filename}': Test case def '{tc_name}' does not declare any 'verify requirement' bindings for safety requirements.",
                            location="user-stories"
                        ))
                    else:
                        if reqs_by_id_or_name:
                            for v_req in verified_reqs:
                                if v_req not in reqs_by_id_or_name:
                                    errors.append(Finding(
                                        "test-case-verify-requirement-invalid",
                                        f"User Story '{filename}': Test case def '{tc_name}' specifies verify requirement '{v_req}' which is not defined in SysML AST.",
                                        location="user-stories"
                                    ))

        # Bidirectional Check: SysML test cases bound across User Stories
        for tc in all_test_cases:
            tc_name = getattr(tc, "name", "")
            if tc_name and tc_name not in bound_test_cases:
                errors.append(Finding(
                    "sysml-test-case-unbound",
                    f"SysML test case def '{tc_name}' is not bound to any User Story BDD acceptance criteria.",
                    location="user-stories"
                ))

        return errors

