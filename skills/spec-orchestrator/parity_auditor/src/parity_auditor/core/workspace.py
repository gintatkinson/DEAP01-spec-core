import os
import json
import re
from typing import Dict, List, Set, Optional, Any
from .models import CodebaseRules, FeatureFile, load_from_dict

UPSTREAM_COMPILER_REPO_TYPE = "UPSTREAM_SPEC_CORE_COMPILER"


def is_upstream_compiler_repo(workspace_dir: str) -> bool:
    """
    Detect an Upstream Specification Core Compiler repository.

    Mirrors ``reconcile_backlog.py``'s detection (issue #68 mechanism) and the
    repository-classification sentinel documented in ``.pipeline/constitution.md``
    and ``AGENTS.md``: the presence of a ``.pipeline/upstream`` marker inside the
    workspace, or the ``DEAP_REPOSITORY_TYPE`` environment variable set to
    ``UPSTREAM_SPEC_CORE_COMPILER``.
    """
    env_type = os.environ.get("DEAP_REPOSITORY_TYPE")
    if env_type and env_type.strip() == UPSTREAM_COMPILER_REPO_TYPE:
        return True
    upstream_marker = os.path.join(os.path.abspath(workspace_dir), ".pipeline", "upstream")
    return os.path.exists(upstream_marker)


def has_configured_target_code_directories(repo: "WorkspaceRepository") -> bool:
    """
    True when at least one configured client-codebase target directory
    (react or flutter) exists inside the workspace.
    """
    rules = repo.get_codebase_rules()
    if rules is None:
        return False
    for name in (rules.target_directories.react, rules.target_directories.flutter):
        if name and os.path.isdir(os.path.join(repo.workspace_dir, name)):
            return True
    return False


class WorkspaceRepository:
    def __init__(self, workspace_dir: Optional[str] = None):
        if not workspace_dir:
            workspace_dir = self._find_workspace_dir(os.getcwd())
        self.workspace_dir = os.path.abspath(workspace_dir)
        self._codebase_rules: Optional[CodebaseRules] = None
        self._behavioral_triggers: Optional[List[dict]] = None
        self._feature_files: Optional[List[FeatureFile]] = None
        self._design_tokens: Optional[Dict[str, Any]] = None
        self._forbidden_colors: Optional[Set[str]] = None

    def _find_workspace_dir(self, start_path: str) -> str:
        curr = os.path.abspath(start_path)
        while True:
            if os.path.exists(os.path.join(curr, ".pipeline", "logical-ui", "codebase_rules.json")):
                return curr
            parent = os.path.dirname(curr)
            if parent == curr:
                break
            curr = parent
        return os.path.abspath(start_path)

    def is_upstream_compiler_repo(self) -> bool:
        return is_upstream_compiler_repo(self.workspace_dir)

    def has_configured_target_code_directories(self) -> bool:
        return has_configured_target_code_directories(self)

    def get_codebase_rules_path(self) -> str:
        rules_path = os.environ.get("CODEBASE_RULES_PATH")
        if not rules_path:
            rules_path = os.path.join(self.workspace_dir, ".pipeline", "logical-ui", "codebase_rules.json")
        return rules_path

    def get_codebase_rules(self) -> CodebaseRules:
        if self._codebase_rules is not None:
            return self._codebase_rules
        
        rules_path = self.get_codebase_rules_path()
        if os.path.exists(rules_path):
            try:
                with open(rules_path, "r", encoding="utf-8") as f:
                    data = json.load(f)
                    self._codebase_rules = load_from_dict(data)
                    return self._codebase_rules
            except Exception as e:
                print(f"Warning: Failed to load codebase_rules.json: {e}")
        
        self._codebase_rules = CodebaseRules()
        return self._codebase_rules

    def get_behavioral_triggers(self, schema_dir: str) -> List[dict]:
        if self._behavioral_triggers is not None:
            return self._behavioral_triggers
        
        rules = self.get_codebase_rules()
        meta_trig_path = rules.meta.behavioral_triggers_path
        
        search_paths = []
        if meta_trig_path:
            search_paths.append(os.path.join(self.workspace_dir, meta_trig_path))
        search_paths.extend([
            os.path.join(schema_dir, "behavioral_triggers.json"),
            os.path.join(self.workspace_dir, "rules", "behavioral_triggers.json"),
            os.path.join(self.workspace_dir, "skills", "spec-orchestrator", "scripts", "behavioral_triggers.json")
        ])
        
        for path in search_paths:
            if os.path.exists(path):
                try:
                    with open(path, "r", encoding="utf-8") as f:
                        self._behavioral_triggers = json.load(f)
                        return self._behavioral_triggers
                except Exception as e:
                    print(f"Warning: Failed to load behavioral triggers from {path}: {e}")
        
        self._behavioral_triggers = []
        return self._behavioral_triggers

    def get_feature_files(self, features_dir: str) -> List[FeatureFile]:
        if self._feature_files is not None:
            return self._feature_files
            
        features = []
        if not os.path.exists(features_dir):
            self._feature_files = []
            return self._feature_files

        for filename in os.listdir(features_dir):
            if not filename.endswith(".md"):
                continue
            filepath = os.path.join(features_dir, filename)
            try:
                with open(filepath, "r", encoding="utf-8") as f:
                    content = f.read()
            except Exception as e:
                print(f"Warning: Failed to read feature file {filename}: {e}")
                continue

            labels = []
            frontmatter_dict = {}
            frontmatter_match = re.match(r"^---\s*\n(.*?)\n---\s*\n", content, re.DOTALL)
            if frontmatter_match:
                frontmatter_text = frontmatter_match.group(1)
                try:
                    import yaml
                    data = yaml.safe_load(frontmatter_text.replace('\x01', ''))
                    if isinstance(data, dict):
                        frontmatter_dict = data
                        if "labels" in data:
                            lbls = data["labels"]
                            if isinstance(lbls, list):
                                labels = [str(lbl).strip() for lbl in lbls]
                            elif isinstance(lbls, str):
                                labels_match = re.search(r"\[(.*?)\]", lbls)
                                if labels_match:
                                    labels = [lbl.strip().strip('"').strip("'") for lbl in labels_match.group(1).split(",")]
                                else:
                                    labels = [lbl.strip() for lbl in lbls.split(",") if lbl.strip()]
                except Exception:
                    for line in frontmatter_text.splitlines():
                        if line.startswith("labels:"):
                            labels_match = re.search(r"\[(.*?)\]", line)
                            if labels_match:
                                labels = [lbl.strip().strip('"').strip("'") for lbl in labels_match.group(1).split(",")]
            
            features.append(FeatureFile(
                filename=filename,
                labels=labels,
                content=content,
                frontmatter=frontmatter_dict
            ))
        self._feature_files = features
        return self._feature_files

    def get_design_tokens(self) -> Dict[str, Any]:
        if self._design_tokens is not None:
            return self._design_tokens
            
        rules = self.get_codebase_rules()
        tokens_path_rel = rules.spec_rules.design_tokens_path
        if not tokens_path_rel:
            self._design_tokens = {}
            return self._design_tokens
            
        tokens_path = os.path.join(self.workspace_dir, tokens_path_rel)
        if not os.path.exists(tokens_path):
            self._design_tokens = {}
            return self._design_tokens
            
        try:
            with open(tokens_path, "r", encoding="utf-8") as f:
                self._design_tokens = json.load(f)
                return self._design_tokens
        except Exception as e:
            print(f"Warning: Failed to load design tokens from {tokens_path}: {e}")
            self._design_tokens = {}
            return self._design_tokens

    def get_forbidden_colors(self) -> Set[str]:
        if self._forbidden_colors is not None:
            return self._forbidden_colors
            
        tokens_data = self.get_design_tokens()
        from ..utils.color_utils import extract_hex_colors_from_json
        self._forbidden_colors = extract_hex_colors_from_json(tokens_data)
        return self._forbidden_colors


def extract_metadata_from_content(content: str) -> Dict[str, Any]:
    """Extracts metadata dictionary from specification frontmatter or Markdown tables."""
    if not content:
        return {}

    # 1. Parse native Markdown tables starting with '| Attribute | Specification Detail |' or '| Metadata | Value |'
    table_header_re = re.compile(
        r'^\s*\|\s*(?:Attribute|Metadata)\s*\|\s*(?:Specification Detail|Value)\s*\|\s*$',
        re.IGNORECASE | re.MULTILINE
    )
    table_header_match = table_header_re.search(content)
    if table_header_match:
        data: Dict[str, Any] = {}
        table_start_idx = table_header_match.start()
        lines = content[table_start_idx:].splitlines()
        
        in_table = False
        for i, line in enumerate(lines):
            line_str = line.strip()
            if not line_str.startswith("|") or not line_str.endswith("|"):
                if in_table:
                    break
                continue

            if i == 0 or table_header_re.match(line_str):
                in_table = True
                continue

            if re.match(r'^\s*\|\s*:?-+:?\s*\|\s*:?-+:?\s*\|\s*$', line_str):
                continue

            parts = [p.strip() for p in line_str.split("|")]
            if len(parts) >= 4:
                raw_key = parts[1].strip()
                raw_val = parts[2].strip()

                clean_key = re.sub(r'[*`_]', '', raw_key).strip()
                norm_key = re.sub(r'[\s\-]+', '_', clean_key.lower())
                norm_key = re.sub(r'[^a-z0-9_]', '', norm_key).strip('_')

                if not norm_key:
                    continue

                if norm_key in ("issue_id", "issueid", "id"):
                    norm_key = "issue_id"
                elif norm_key in ("parent_epic", "parentepic", "parent_epic_id", "epic"):
                    norm_key = "epic"
                elif norm_key in ("specification_source", "spec_source", "specsource"):
                    norm_key = "spec_source"
                elif norm_key in ("sysml_test_case", "test_case", "testcase"):
                    norm_key = "sysml_test_case"
                elif norm_key in ("sysml_interaction", "interaction"):
                    norm_key = "sysml_interaction"
                elif norm_key in ("schema_containers", "schemacontainers", "schema_container"):
                    norm_key = "schema_containers"
                elif norm_key in ("interface_type", "interfacetype"):
                    norm_key = "interface_type"
                elif norm_key in ("generation_mode", "generationmode"):
                    norm_key = "generation_mode"

                val: Any = raw_val
                if norm_key == "issue_id":
                    clean_id = re.sub(r'["\'#]', "", str(val)).strip()
                    if clean_id.isdigit():
                        val = int(clean_id)
                    else:
                        val = clean_id
                else:
                    val = val.replace(r'\|', '|')
                    if norm_key in ("labels", "tags", "realizes"):
                        if val.startswith("[") and val.endswith("]"):
                            val = [item.strip().strip('"\'`') for item in val[1:-1].split(",") if item.strip()]
                        elif "," in val:
                            val = [item.strip().strip('"\'`') for item in val.split(",") if item.strip()]
                        elif val:
                            val = [val.strip().strip('"\'`')]
                        else:
                            val = []
                    elif norm_key == "schema_containers":
                        clean_str = val.strip().strip('"\'`')
                        if clean_str.startswith("[") and clean_str.endswith("]"):
                            val = [item.strip().strip('"\'`') for item in clean_str[1:-1].split(",") if item.strip()]
                        elif "," in clean_str:
                            val = [item.strip().strip('"\'`') for item in clean_str.split(",") if item.strip()]
                        elif clean_str:
                            val = [clean_str]
                        else:
                            val = []

                data[norm_key] = val
                if norm_key == "epic":
                    data["parent_epic"] = val
                elif norm_key == "spec_source":
                    data["specification_source"] = val
                elif norm_key == "sysml_test_case":
                    data["test_case"] = val
                    data["test_cases"] = [val] if isinstance(val, str) else val

        if data:
            return data

    # 2. Fallback to YAML frontmatter
    fm_match = re.search(r'^---\s*\n(.*?)\n---\s*\n', content, re.DOTALL)
    if fm_match:
        try:
            import yaml
            parsed = yaml.safe_load(fm_match.group(1))
            if isinstance(parsed, dict):
                return parsed
        except Exception:
            pass

    return {}
