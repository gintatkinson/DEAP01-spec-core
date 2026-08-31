import os
import re
import json
from typing import List
from .base import IValidator
from ..core.findings import Finding
from ..core.workspace import WorkspaceRepository

class LogicalUiValidator(IValidator):
    def validate(self, repo: WorkspaceRepository, **kwargs) -> List[str]:
        rules = repo.get_codebase_rules()
        backlog_dirs = rules.backlog_directories
        
        flutter_dir = rules.target_directories.flutter if (rules.target_directories and rules.target_directories.flutter) else "app_flutter"
        # Check workspace_dir / .pipeline / logical-ui / logical-layout.json first
        layout_path = os.path.join(repo.workspace_dir, ".pipeline", "logical-ui", "logical-layout.json")
        if not os.path.exists(layout_path):
            # Check flutter_dir / assets / logical-layout.json next
            layout_path = os.path.join(repo.workspace_dir, flutter_dir, "assets", "logical-layout.json")
            
        component_types = set()
        container_ids = set()
        container_to_type = {}
        tabbed_container_errors = []
        
        if os.path.exists(layout_path):
            try:
                with open(layout_path, "r", encoding="utf-8") as f:
                    layout_data = json.load(f)
                    
                def traverse(node):
                    if isinstance(node, dict):
                        # Extract component type if present
                        node_type = node.get("type")
                        if isinstance(node_type, str):
                            component_types.add(node_type)
                        # Extract container/component ID if present
                        node_id = node.get("id")
                        if isinstance(node_id, str):
                            container_ids.add(node_id)
                            if isinstance(node_type, str):
                                container_to_type[node_id] = node_type

                        ALLOWED_TABBED_CHILD_TYPES = {"TableView", "PropertyGrid", "DensityTable"}
                        if isinstance(node_type, str) and node_type == "TabbedContainer":
                            container_id_str = node_id if (isinstance(node_id, str) and node_id) else "unknown"
                            children = node.get("children")
                            if not isinstance(children, list):
                                tabbed_container_errors.append(Finding(
                                    "logical-ui-tabbed-container-child-must-be-a-table-view",
                                    f"TabbedContainer '{container_id_str}' contains non-TableView child 'unknown' of type 'unknown'",
                                    location=container_id_str,
                                ))
                            else:
                                for child in children:
                                    if not isinstance(child, dict):
                                        tabbed_container_errors.append(Finding(
                                            "logical-ui-tabbed-container-child-must-be-a-table-view",
                                            f"TabbedContainer '{container_id_str}' contains non-TableView child 'unknown' of type 'unknown'",
                                            location=container_id_str,
                                        ))
                                    else:
                                        raw_type = child.get("type")
                                        raw_id = child.get("id")
                                        child_type = raw_type if (isinstance(raw_type, str) and raw_type) else "unknown"
                                        child_id = raw_id if (isinstance(raw_id, str) and raw_id) else "unknown"
                                        if child_type not in ALLOWED_TABBED_CHILD_TYPES:
                                            tabbed_container_errors.append(Finding(
                                                "logical-ui-tabbed-container-child-must-be-a-table-view",
                                                f"TabbedContainer '{container_id_str}' contains non-TableView child '{child_id}' of type '{child_type}'",
                                                location=container_id_str,
                                            ))
                            
                        # Recurse on values
                        for val in node.values():
                            traverse(val)
                    elif isinstance(node, list):
                        for item in node:
                            traverse(item)
                            
                traverse(layout_data)
            except Exception as e:
                return [Finding(
                    "logical-ui-layout-manifest-must-parse",
                    f"Logical UI Compliance: Failed to parse logical-layout.json: {e}",
                    location=layout_path,
                )]
        else:
            return [Finding(
                "logical-ui-layout-manifest-must-exist",
                "Logical UI Compliance: logical-layout.json not found at expected paths.",
            )]
            
        features_dir = kwargs.get("features_dir")
        if not features_dir:
            features_dir = os.path.join(repo.workspace_dir, backlog_dirs.features)
            
        errors = []
        errors.extend(tabbed_container_errors)
        if not os.path.exists(features_dir):
            errors.append(Finding(
                "logical-ui-features-directory-must-exist",
                f"Logical UI Compliance: features directory not found at {features_dir}",
                location=features_dir,
            ))
            return errors
            
        feature_files = repo.get_feature_files(features_dir)
        
        for feat in feature_files:
            content = feat.content
            # Use relative path starting with features directory to match doc-checking patterns
            rel_path = os.path.join(backlog_dirs.features, feat.filename)
            
            fm = getattr(feat, "frontmatter", None)
            if not fm or not isinstance(fm, dict):
                fm = {}
                fm_match = re.match(r"^---\s*\n(.*?)\n---\s*\n", content, re.DOTALL)
                if fm_match:
                    try:
                        import yaml
                        parsed_fm = yaml.safe_load(fm_match.group(1).replace('\x01', ''))
                        if isinstance(parsed_fm, dict):
                            fm = parsed_fm
                    except Exception as e:
                        errors.append(Finding(
                            "logical-ui-feature-frontmatter-must-parse",
                            f"Logical UI Compliance: Failed to parse frontmatter YAML in '{rel_path}': {e}",
                            location=rel_path,
                        ))
            
            declared_interface_types = []
            if isinstance(fm, dict):
                raw_it = fm.get("interface_type")
                if isinstance(raw_it, str):
                    declared_interface_types.append(raw_it.lower())
                elif isinstance(raw_it, list):
                    declared_interface_types.extend([str(x).lower() for x in raw_it])
                
                raw_its = fm.get("interface_types")
                if isinstance(raw_its, list):
                    declared_interface_types.extend([str(x).lower() for x in raw_its])

            non_ui_types = {"api", "config", "persistence", "gate", "cli", "backend"}
            
            target_match = re.search(r"##\s*(?:\d+\.\s*)?Logical\s+UI\s+&\s+(?:Layout|Interface)\s+Bindings(.*?)(?=##|\Z)", content, re.DOTALL | re.IGNORECASE)
            
            if not target_match:
                has_ui_channel = False
                if not declared_interface_types:
                    has_ui_channel = True
                else:
                    for it in declared_interface_types:
                        if it not in non_ui_types:
                            has_ui_channel = True
                            break
                if has_ui_channel:
                    errors.append(Finding(
                        "logical-ui-feature-requires-layout-bindings-section",
                        f"Logical UI Compliance: Feature '{rel_path}' is a UI feature but lacks the 'Logical UI & Layout Bindings' section.",
                        location=rel_path,
                    ))
                    target_match = None
            
            if target_match:
                section_content = target_match.group(1)
                
                # Check for raw N/A fallback strings
                has_raw_na = False
                if re.search(r"\bN/A\b|\bn/a\b", section_content):
                    has_raw_na = True
                
                # Check for literal placeholder strings (#X, Task Y)
                if re.search(r"#X|Task\s+Y", section_content, re.IGNORECASE):
                    errors.append(Finding(
                        "logical-ui-prohibit-placeholder-string",
                        f"Logical UI Compliance: Feature '{rel_path}' contains literal placeholder string in interface bindings. Literal placeholder strings ('#X', 'Task Y') are strictly prohibited.",
                        location=rel_path,
                    ))

                specified_components = set()
                container_val = ""
                
                # Check if multi-interface binding table exists
                table_rows = []
                lines = [l.strip() for l in section_content.splitlines() if l.strip()]
                raw_table_lines = [l for l in lines if l.startswith("|") and l.endswith("|")]
                
                if len(raw_table_lines) >= 2:
                    for line in raw_table_lines:
                        if "---" in line or "interface channel" in line.lower() or "category" in line.lower():
                            continue
                        cells = [c.strip() for c in line.split("|")[1:-1]]
                        if len(cells) >= 4:
                            table_rows.append(cells)
                    
                    table_channels = set()
                    for cells in table_rows:
                        channel_val = cells[0].lower()
                        table_channels.add(channel_val)
                        if channel_val in ("ui", "gui"):
                            table_channels.add("gui")
                            table_channels.add("ui")
                        
                        comp_val = cells[2]
                        cont_val = cells[3]
                        ds_val = cells[4] if len(cells) > 4 else ""
                        
                        if comp_val.upper() == "N/A" or cont_val.upper() == "N/A" or ds_val.upper() == "N/A":
                            has_raw_na = True
                        
                        if channel_val in ("ui", "gui"):
                            cleaned_comp = re.sub(r'\[([^\]]+)\](?:\([^)]*\))?', r'\1', comp_val)
                            for item in cleaned_comp.split(','):
                                token = item.strip().strip("*`\"'[]() ")
                                if token:
                                    specified_components.add(token)
                            
                            cleaned_cont = re.sub(r'\[([^\]]+)\](?:\([^)]*\))?', r'\1', cont_val)
                            if cleaned_cont:
                                container_val = cleaned_cont.strip().strip("*`\"'[]() ")
                            
                        if ds_val:
                            ds_clean = ds_val.strip().strip("*`\"'[]() ")
                            if ds_clean and not self._is_unbound_or_deferred(ds_clean):
                                self._validate_ds_path(ds_clean, rel_path, errors)
                    
                    if declared_interface_types:
                        for chan in declared_interface_types:
                            if chan in non_ui_types:
                                continue
                            norm_chan = "gui" if chan == "ui" else chan
                            if norm_chan not in table_channels and chan not in table_channels:
                                errors.append(Finding(
                                    "logical-ui-missing-interface-channel-row",
                                    f"Logical UI Compliance: Feature '{rel_path}' specifies interface channel '{chan}' in frontmatter but lacks a corresponding row in the Multi-Interface Binding Table.",
                                    location=rel_path,
                                ))
                else:
                    for line in section_content.splitlines():
                        if "Target LUI Component" in line:
                            raw_val = ""
                            if "|" in line:
                                cells = [c.strip() for c in line.split("|")]
                                for idx, cell in enumerate(cells):
                                    cell_clean = re.sub(r'[*`]', '', cell).strip().lower()
                                    if "target lui component" in cell_clean:
                                        if idx + 1 < len(cells):
                                            raw_val = cells[idx + 1]
                                        break
                            elif ":" in line:
                                parts = line.split(":", 1)
                                if len(parts) > 1:
                                    raw_val = parts[1]
                            
                            if raw_val:
                                if raw_val.strip().upper() == "N/A":
                                    has_raw_na = True
                                cleaned = re.sub(r'\[([^\]]+)\](?:\([^)]*\))?', r'\1', raw_val)
                                for item in cleaned.split(','):
                                    token = item.strip().strip("*`\"'[]() ")
                                    if token:
                                        specified_components.add(token)
                        elif "Target Layout Container ID" in line:
                            raw_container = ""
                            if "|" in line:
                                cells = [c.strip() for c in line.split("|")]
                                for idx, cell in enumerate(cells):
                                    cell_clean = re.sub(r'[*`]', '', cell).strip().lower()
                                    if "target layout container id" in cell_clean:
                                        if idx + 1 < len(cells):
                                            raw_container = cells[idx + 1]
                                        break
                            elif ":" in line:
                                parts = line.split(":", 1)
                                if len(parts) > 1:
                                    raw_container = parts[1]
                            if raw_container:
                                if raw_container.strip().upper() == "N/A":
                                    has_raw_na = True
                                cleaned = re.sub(r'\[([^\]]+)\](?:\([^)]*\))?', r'\1', raw_container)
                                container_val = cleaned.strip().strip("*`\"'[]() ")
                        elif "Data Source Binding" in line:
                            parts = line.split(":", 1)
                            if len(parts) > 1:
                                ds_val = parts[1].strip().strip("*`\"'[]() ")
                                if ds_val.strip().upper() == "N/A":
                                    has_raw_na = True
                                if ds_val and not self._is_unbound_or_deferred(ds_val):
                                    self._validate_ds_path(ds_val, rel_path, errors)

                    if declared_interface_types:
                        for chan in declared_interface_types:
                            if chan in non_ui_types:
                                continue
                            norm_chan = "gui" if chan == "ui" else chan
                            if norm_chan not in ("gui", "ui"):
                                errors.append(Finding(
                                    "logical-ui-missing-interface-channel-row",
                                    f"Logical UI Compliance: Feature '{rel_path}' specifies interface channel '{chan}' in frontmatter but lacks a corresponding row in the Multi-Interface Binding Table.",
                                    location=rel_path,
                                ))
                
                if has_raw_na:
                    errors.append(Finding(
                        "logical-ui-prohibit-raw-na-fallback",
                        f"Logical UI Compliance: Feature '{rel_path}' contains raw 'N/A' fallback string in interface bindings. Raw N/A fallback strings are strictly prohibited.",
                        location=rel_path,
                    ))

                # Ensure specified target component is a valid layout component (if not N/A or deferred)
                for c in sorted(specified_components):
                    if not self._is_unbound_or_deferred(c) and c not in {"MCPToolHandler", "RESTEndpointHandler", "gRPCMethodHandler", "GraphQLQueryHandler", "RegisterBuffer", "DiscreteSignal", "SerialDataStream", "I2CBusDevice"}:
                        if c not in component_types and not (c == "TopologyMap" and ("topology_pane" in container_ids or "TopologyMap" in component_types)):
                            errors.append(Finding(
                                "logical-ui-component-type-must-exist-in-the-layout",
                                f"Logical UI Compliance: Feature '{rel_path}' specifies invalid component type '{c}'. It must be instantiated in logical-layout.json.",
                                location=rel_path,
                            ))
                    
                # Ensure specified target container ID is valid (if not N/A or deferred)
                if container_val and not self._is_unbound_or_deferred(container_val):
                    if container_val not in container_ids:
                        errors.append(Finding(
                            "logical-ui-container-id-must-exist-in-the-layout",
                            f"Logical UI Compliance: Feature '{rel_path}' specifies invalid container ID '{container_val}'.",
                            location=rel_path,
                        ))
                    else:
                        expected_type = container_to_type.get(container_val)
                        if expected_type and specified_components:
                            for c in sorted(specified_components):
                                if not self._is_unbound_or_deferred(c) and c != expected_type:
                                    if c == "TopologyMap" and container_val == "topology_pane":
                                        continue
                                    errors.append(Finding(
                                        "logical-ui-component-must-match-its-container-type",
                                        f"Logical UI Compliance: Feature '{rel_path}' specifies component type '{c}' but target container '{container_val}' is of type '{expected_type}'.",
                                        location=rel_path,
                                    ))

                # Enforce that geodetic/spatial features map to valid spatial view components
                VALID_SPATIAL_COMPONENTS = {"TopologyMap", "TopographicalView", "GeoSpatialViewer", "PropertyGrid", "TableView"}
                GEODETIC_REGEX = re.compile(r"\b(?:location|velocity|geo-location|geodetic|latitude|longitude|altitude|elevation|datum|position|spatial|reference-frame|geodetic-system|coordinates|velocity\s+vectors)\b", re.IGNORECASE)

                if target_match and GEODETIC_REGEX.search(content):
                    has_unbound = any(self._is_unbound_or_deferred(c) for c in specified_components)
                    if not has_unbound and (any(c not in VALID_SPATIAL_COMPONENTS for c in specified_components) or not specified_components):
                        errors.append(Finding(
                            "logical-ui-spatial-feature-requires-a-spatial-component",
                            f"Logical UI Compliance: Feature '{rel_path}' contains spatial/geodetic attributes but fails to map to a spatial view component ('TopologyMap', 'TopographicalView', 'GeoSpatialViewer', 'PropertyGrid', or 'TableView').",
                            location=rel_path,
                        ))

        return errors

    def _is_unbound_or_deferred(self, val: str) -> bool:
        if not val:
            return True
        v = val.strip().lower()
        return (
            v.startswith("unbound")
            or v.startswith("deferred")
        )

    def _validate_ds_path(self, ds_val: str, rel_path: str, errors: list):
        paths = [p.strip().strip(" *`\"'") for p in ds_val.split(',')]
        nil_elements = {"locations", "racks", "rack", "rack-location", "contained-chassis"}
        FORBIDDEN_CHOICE_NODES = {"location-choice", "cartesian", "ellipsoid", "choice", "case"}
        for path in paths:
            if not path or self._is_unbound_or_deferred(path):
                continue
            if ' ' in path or not (path.startswith('/') or path.startswith('schema:') or path.startswith('provider:')):
                errors.append(Finding(
                    "logical-ui-data-source-binding-must-be-a-schema-path",
                    f"Logical UI Compliance: Feature '{rel_path}' Data Source Binding '{path}' contains plain-text English instead of valid schema path.",
                    location=rel_path,
                ))
                continue

            segments = path.split('/')
            in_augmented_subtree = False
            for seg in segments:
                seg_clean = seg.strip()
                if not seg_clean:
                    continue
                if '[' in seg_clean:
                    seg_clean = seg_clean.split('[', 1)[0].strip()
                if ':' in seg_clean:
                    prefix, local_name = seg_clean.split(':', 1)
                else:
                    prefix, local_name = "", seg_clean
                
                if local_name in FORBIDDEN_CHOICE_NODES or local_name.endswith("-choice") or local_name.endswith("-case"):
                    errors.append(Finding(
                        "logical-ui-data-source-binding-must-omit-choice-and-case-nodes",
                        f"Logical UI Compliance: Feature '{rel_path}' Data Source Binding '{path}' contains forbidden YANG choice/case node '{local_name}'. Choice/case wrappers must be omitted from data paths.",
                        location=rel_path,
                    ))
                
                if prefix == "nil" or local_name in nil_elements:
                    in_augmented_subtree = True
                    if local_name in nil_elements and prefix != "nil":
                        errors.append(Finding(
                            "logical-ui-augmented-node-must-carry-its-module-prefix",
                            f"Logical UI Compliance: Feature '{rel_path}' Data Source Binding '{path}' contains un-prefixed augmented element '{local_name}'. Must use 'nil:{local_name}'.",
                            location=rel_path,
                        ))
                elif in_augmented_subtree and prefix == "":
                    errors.append(Finding(
                        "logical-ui-augmented-node-must-carry-its-module-prefix",
                        f"Logical UI Compliance: Feature '{rel_path}' Data Source Binding '{path}' contains un-prefixed augmented child segment '{local_name}' under augmented subtree. Must use 'nil:{local_name}'.",
                        location=rel_path,
                    ))



