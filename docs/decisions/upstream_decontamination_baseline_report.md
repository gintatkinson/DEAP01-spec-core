# Upstream Decontamination & Baseline Refactor Report

**Repository**: `gintatkinson/DEAP-spec-core`  
**Location**: `docs/decisions/upstream_decontamination_baseline_report.md`  
**Branch**: `main`  
**Latest Unpushed Commit**: `b51f797d0022ad6126ca4f9b5b26d22454931206`  
**Commit Message**: `refactor(upstream): establish domain-agnostic 0...n tab baseline`  
**Remote Sync Status**: 1 commit ahead of `origin/main` (Unpushed)

---

## Executive Summary

The upstream pipeline has been decontaminated of hardcoded domain logic, geometry keywords, and fixed 3-tab layout assumptions. The architecture now strictly adheres to a domain-agnostic baseline where:

1. **YANG Compiler (`scripts/compile_yang.py`)**: Yields an empty `TabbedContainer` child array (`[]`) by default when no downstream `details_tabs` rules are configured.
2. **Layout JSON Assets (`.pipeline/logical-ui/logical-layout.json` & `app_flutter/assets/logical-layout.json`)**: Configured with generic schema metadata (`"schema_name": "generic_dashboard"`) and a 0-tab baseline (`"children": []`).
3. **Validator (`logical_ui_validator.py`)**: Stripped of domain-specific path checkers, geometry keyword lists (`coordinate_keywords`), and dead code (`allowed_component_names`). Retains pure structural parity enforcement (`TabbedContainer` children type assertion).

---

## 1. Commit Patch (`git log -1 -p`)

```diff
commit b51f797d0022ad6126ca4f9b5b26d22454931206
Author: gintatkinson <gintatkinson@gmail.com>
Date:   Sun Jul 26 17:50:00 2026 +0800

    refactor(upstream): establish domain-agnostic 0...n tab baseline

diff --git a/.pipeline/logical-ui/logical-layout.json b/.pipeline/logical-ui/logical-layout.json
index 5143c6d..b74288b 100644
--- a/.pipeline/logical-ui/logical-layout.json
+++ b/.pipeline/logical-ui/logical-layout.json
@@ -1,7 +1,7 @@
 {
   "meta": {
     "version": "1.0.0",
-    "schema_name": "systems_topology_dashboard"
+    "schema_name": "generic_dashboard"
   },
   "theme": {
     "modes": ["light", "dark", "system"],
@@ -101,47 +101,7 @@
               "props": {
                 "tab_position": "bottom"
               },
-              "children": [
-                {
-                  "type": "TableView",
-                  "id": "components_table",
-                  "props": {
-                    "label": "token:layout.labels.components",
-                    "sortable": true,
-                    "filterable": true,
-                    "high_density": true,
-                    "display_all_attributes": true
-                  },
-                  "bindings": {
-                    "data_source": "token:layout.data_sources.components"
-                  }
-                },
-                {
-                  "type": "TableView",
-                  "id": "relation_a_table",
-                  "props": {
-                    "label": "token:layout.labels.relation_a",
-                    "sortable": true,
-                    "filterable": true,
-                    "high_density": true,
-                    "display_all_attributes": true
-                  },
-                  "bindings": {
-                    "data_source": "token:layout.data_sources.relation_a"
-                  }
-                },
-                {
-                  "type": "TableView",
-                  "id": "relation_b_table",
-                  "props": {
-                    "label": "token:layout.labels.relation_b",
-                    "sortable": true,
-                    "filterable": true,
-                    "high_density": true,
-                    "display_all_attributes": true
-                  },
-                  "bindings": {
-                    "data_source": "token:layout.data_sources.relation_b"
-                  }
-                }
-              ]
+              "children": []
             }
           ]
         }

diff --git a/scripts/compile_yang.py b/scripts/compile_yang.py
index a95f73c..84d0ccd 100644
--- a/scripts/compile_yang.py
+++ b/scripts/compile_yang.py
@@ -279,14 +279,8 @@ def build_lui_json(data_defs, schema_name='unknown', yang_source=''):
 
     layout_rules = codebase_rules.get('layout_rules', {})
     details_tabs = layout_rules.get('details_tabs')
-    if not details_tabs:
-        details_tabs = [
-            {
-                "type": "TableView",
-                "id": "properties_table",
-                "props": { "title": "Properties" }
-            }
-        ]
+    if details_tabs is None:
+        details_tabs = []

     return {
         'meta': {

diff --git a/skills/spec-orchestrator/parity_auditor/src/parity_auditor/validators/logical_ui_validator.py b/skills/spec-orchestrator/parity_auditor/src/parity_auditor/validators/logical_ui_validator.py
index 8032aa6..847fa47 100644
--- a/skills/spec-orchestrator/parity_auditor/src/parity_auditor/validators/logical_ui_validator.py
+++ b/skills/spec-orchestrator/parity_auditor/src/parity_auditor/validators/logical_ui_validator.py
@@ -60,17 +60,6 @@ class LogicalUiValidator(IValidator):
         else:
             return [f"Logical UI Compliance: logical-layout.json not found at expected paths."]
             
-        # Allowed component names from logical-components.md
-        allowed_component_names = {
-            "HierarchyTree", 
-            "ResizableSplitter", 
-            "NavigationBreadcrumbs", 
-            "PropertyGrid", 
-            "TopologyMap", 
-            "DensityTable", 
-            "ContextualPanel"
-        }
-        
         features_dir = kwargs.get("features_dir")
         if not features_dir:
             features_dir = os.path.join(repo.workspace_dir, backlog_dirs.features)
@@ -83,12 +72,6 @@ class LogicalUiValidator(IValidator):
             
         feature_files = repo.get_feature_files(features_dir)
         
-        coordinate_keywords = [
-            "astronomical-body", "geometry-datum", "coordinate", 
-            "dim_0", "dim_1", "trajectory", "orbit", 
-            "dim_2", "geo-location"
-        ]
-        
         for feat in feature_files:
             content = feat.content
             # Use relative path starting with features directory to match doc-checking patterns
@@ -115,34 +98,6 @@ class LogicalUiValidator(IValidator):
                         parts = line.split(":", 1)
                         if len(parts) > 1:
                             container_val = parts[1].strip().strip("*`\"'[]() ")
-                    elif "Data Source Bindings" in line:
-                        parts = line.split(":", 1)
-                        if len(parts) > 1:
-                            ds_val = parts[1].strip().strip("*`\"'[]() ")
-                            if ds_val.upper() == "N/A":
-                                continue
-                            paths = [p.strip() for p in ds_val.split(',')]
-                            nil_elements = {"locations", "racks", "slotContainer", "slotContainer-location", "contained-slotContainer"}
-                            geo_elements = {"geo-location", "reference-frame", "geometry-system", "rateOfChange"}
-                            forbidden_nodes = {"cartesian", "geometry", "location-choice"}
-                            placeholder_words = {"from", "logical-layout.json", "container", "choice", "placeholder", "tbd"}
-                            for path in paths:
-                                if not path: continue
-                                if not (path.startswith("schema:") or path.startswith("provider:") or path.startswith("/")):
-                                    errors.append(f"Logical UI Compliance: Feature '{rel_path}' Data Source Binding '{path}' must start with 'schema:', 'provider:', or '/'.")
-                                path_lower = path.lower()
-                                for word in placeholder_words:
-                                    if word in path_lower:
-                                        errors.append(f"Logical UI Compliance: Feature '{rel_path}' Data Source Binding '{path}' contains invalid plain-text placeholder '{word}'.")
-                                segments = path.split('/')
-                                for seg in segments:
-                                    seg_clean = seg.strip()
-                                    if seg_clean in forbidden_nodes:
-                                        errors.append(f"Logical UI Compliance: Feature '{rel_path}' Data Source Binding '{path}' contains forbidden choice/case node '{seg_clean}'.")
-                                    if seg_clean in nil_elements:
-                                        errors.append(f"Logical UI Compliance: Feature '{rel_path}' Data Source Binding '{path}' contains un-prefixed augmented element '{seg_clean}'. Must use 'nil:{seg_clean}'.")
-                                    if seg_clean in geo_elements:
-                                        errors.append(f"Logical UI Compliance: Feature '{rel_path}' Data Source Binding '{path}' contains un-prefixed module element '{seg_clean}'. Must use 'geo:{seg_clean}'.")
                             
             # Ensure specified target component is a valid layout component (if not N/A)
             if comp_val.upper() != "N/A":
@@ -153,16 +108,6 @@ class LogicalUiValidator(IValidator):
             if container_val.upper() != "N/A":
                 if container_val not in container_ids:
                     errors.append(f"Logical UI Compliance: Feature '{rel_path}' specifies invalid container ID '{container_val}'.")
-                    
-            # Coordinate/Reference-Frame constraint:
-            # If the feature file text contains coordinate/reference-frame terms and Target LUI Component is N/A
-            has_coordinate_term = any(re.search(rf"\b{re.escape(word)}\b", content, re.IGNORECASE) for word in coordinate_keywords)
-            if has_coordinate_term and comp_val.upper() != "N/A":
-                valid_geodetic_components = {"PropertyGrid", "TableView", "DensityTable"}
-                if comp_val not in valid_geodetic_components:
-                    errors.append(
-                        f"Logical UI Compliance: Feature '{rel_path}' contains geometry/coordinate concepts but "
-                        f"'Target LUI Component' is '{comp_val}'. Geolocation attributes must reside in details panels or tables, not topology or tree selectors."
-                    )
                 
         return errors
```

---

## 2. Complete Source: `skills/spec-orchestrator/parity_auditor/src/parity_auditor/validators/logical_ui_validator.py`

```python
import os
import re
import json
from typing import List
from .base import IValidator
from ..core.workspace import WorkspaceRepository

class LogicalUiValidator(IValidator):
    def validate(self, repo: WorkspaceRepository, **kwargs) -> List[str]:
        rules = repo.get_codebase_rules()
        backlog_dirs = rules.backlog_directories
        
        # 1. Locate layout configuration JSON file
        # Check workspace_dir / .pipeline / logical-ui / logical-layout.json first
        layout_path = os.path.join(repo.workspace_dir, ".pipeline", "logical-ui", "logical-layout.json")
        if not os.path.exists(layout_path):
            # Check app_flutter / assets / logical-layout.json next
            layout_path = os.path.join(repo.workspace_dir, "app_flutter", "assets", "logical-layout.json")
            
        component_types = set()
        container_ids = set()
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
                            
                        if isinstance(node_type, str) and node_type == "TabbedContainer":
                            children = node.get("children", [])
                            if isinstance(children, list):
                                for child in children:
                                    if isinstance(child, dict):
                                        child_type = child.get("type")
                                        child_id = child.get("id", "unknown")
                                        if child_type != "TableView":
                                            tabbed_container_errors.append(f"TabbedContainer '{node_id}' contains non-TableView child '{child_id}' of type '{child_type}'")
                            
                        # Recurse on values
                        for val in node.values():
                            traverse(val)
                    elif isinstance(node, list):
                        for item in node:
                            traverse(item)
                            
                traverse(layout_data)
            except Exception as e:
                return [f"Logical UI Compliance: Failed to parse logical-layout.json: {e}"]
        else:
            return [f"Logical UI Compliance: logical-layout.json not found at expected paths."]
            
        features_dir = kwargs.get("features_dir")
        if not features_dir:
            features_dir = os.path.join(repo.workspace_dir, backlog_dirs.features)
            
        errors = []
        errors.extend(tabbed_container_errors)
        if not os.path.exists(features_dir):
            errors.append(f"Logical UI Compliance: features directory not found at {features_dir}")
            return errors
            
        feature_files = repo.get_feature_files(features_dir)
        
        for feat in feature_files:
            content = feat.content
            # Use relative path starting with features directory to match doc-checking patterns
            rel_path = os.path.join(backlog_dirs.features, feat.filename)
            
            comp_val = "N/A"
            container_val = "N/A"
            
            # Extract Target LUI Component and Target Layout Container ID from ## 5. Logical UI & Layout Bindings
            match = re.search(r"##\s*(?:\d+\.\s*)?Logical\s+UI\s+&\s+Layout\s+Bindings(.*?)(?=##|\Z)", content, re.DOTALL | re.IGNORECASE)
            
            if not match:
                has_ui_concept = bool(re.search(r"(?:interface[-_]type:\s*ui|\bui\b|\binterface\b|\blayout\b|\bview\b|\bcomponent\b|\bwidget\b|\bscreen\b)", content, re.IGNORECASE))
                if has_ui_concept:
                    errors.append(f"Logical UI Compliance: Feature '{rel_path}' is a UI feature but lacks the 'Logical UI & Layout Bindings' section.")
            else:
                section_content = match.group(1)
                for line in section_content.splitlines():
                    if "Target LUI Component" in line:
                        parts = line.split(":", 1)
                        if len(parts) > 1:
                            comp_val = parts[1].strip().strip("*`\"'[]() ")
                    elif "Target Layout Container ID" in line:
                        parts = line.split(":", 1)
                        if len(parts) > 1:
                            container_val = parts[1].strip().strip("*`\"'[]() ")
                            
            # Ensure specified target component is a valid layout component (if not N/A)
            if comp_val.upper() != "N/A":
                if comp_val not in component_types:
                    errors.append(f"Logical UI Compliance: Feature '{rel_path}' specifies invalid component type '{comp_val}'. It must be instantiated in logical-layout.json.")
                    
            # Ensure specified target container ID is valid (if not N/A)
            if container_val.upper() != "N/A":
                if container_val not in container_ids:
                    errors.append(f"Logical UI Compliance: Feature '{rel_path}' specifies invalid container ID '{container_val}'.")
                
        return errors
```

---

## 3. Complete Asset: `.pipeline/logical-ui/logical-layout.json`

```json
{
  "meta": {
    "version": "1.0.0",
    "schema_name": "generic_dashboard"
  },
  "theme": {
    "modes": ["light", "dark", "system"],
    "colors": {
      "primary_accent": "color.brand.primary",
      "background": {
        "light": "color.background.light",
        "dark": "color.background.dark"
      },
      "surface": {
        "light": "color.surface.light",
        "dark": "color.surface.dark"
      }
    },
    "typography": {
      "fonts": ["token.font.primary", "token.font.secondary", "system-ui"],
      "scale": "high-density"
    }
  },
  "navigation": {
    "sidebar": {
      "collapsible": true,
      "default_expanded": true,
      "items": "token:navigation.sidebar.items"
    }
  },
  "layout": {
    "root_container": {
      "type": "SidebarLayout",
      "id": "main_shell",
      "children": [
        {
          "type": "HierarchyTreeSelector",
          "id": "resource_tree",
          "props": {
            "width": "spacing.layout-sidebar-width",
            "hierarchy": [
              {
                "id": "CategoryA",
                "label": "Category A",
                "children": [
                  { "id": "ItemA1", "label": "Item A1" },
                  { "id": "ItemA2", "label": "Item A2" }
                ]
              },
              {
                "id": "CategoryB",
                "label": "Category B",
                "children": [
                  { "id": "ItemB1", "label": "Item B1" }
                ]
              }
            ]
          },
          "bindings": {
            "data_source": "token:layout.data_sources.resource_tree",
            "selection_target": "selected_managed_object"
          }
        },
        {
          "type": "SplitWorkspace",
          "id": "workspace_split",
          "props": {
            "axis": "horizontal",
            "resizable": true,
            "default_ratio": "token:layout_properties.default_ratio",
            "reconfigurable": true,
            "min_size_pixels": "spacing.layout-min-pane-size"
          },
          "children": [
            {
              "type": "TopographicalView",
              "id": "topology_pane",
              "props": {
                "dimensions": "token:layout_properties.dimensions",
                "enable_time_playback": true,
                "interactive": true,
                "allow_pan_zoom": true
              },
              "bindings": {
                "data_source": "token:layout.data_sources.topology",
                "selection_target": "active_focused_node",
                "coordinate_mapping": "token:layout.coordinate_mapping",
                "depth_selector": {
                  "default_hops": "token:layout_properties.default_hops",
                  "max_hops": "token:layout_properties.max_hops"
                },
                "relationship_filters": "token:layout_properties.relationship_filters"
              }
            },
            {
              "type": "TabbedContainer",
              "id": "details_and_relations_tab",
              "props": {
                "tab_position": "bottom"
              },
              "children": []
            }
          ]
        }
      ]
    }
  }
}
```

---

## 4. Verification Evidence

### Pytest Execution (`skills/spec-orchestrator/parity_auditor`)
```text
============================= test session starts ==============================
platform darwin -- Python 3.9.6, pytest-8.4.2, pluggy-1.6.0
collected 33 items

tests/test_bug188.py::test_empty_schemas_dir_no_enforcement PASSED       [  3%]
tests/test_bug188.py::test_empty_schemas_dir_with_gitkeep_no_enforcement PASSED [  6%]
tests/test_cli.py::TestIsPresentInCodebase::test_non_common_word_matches_in_code_not_comment PASSED [  9%]
tests/test_cli.py::TestIsPresentInCodebase::test_non_common_word_only_in_comment_returns_false PASSED [ 12%]
tests/test_cli.py::TestIsPresentInCodebase::test_non_common_word_only_in_block_comment_returns_false PASSED [ 15%]
tests/test_cli.py::TestIsPresentInCodebase::test_non_common_word_only_in_string_does_not_match PASSED [ 18%]
tests/test_cli.py::TestIsPresentInCodebase::test_common_word_in_comment_not_matched_as_code PASSED [ 21%]
tests/test_cli.py::TestIsPresentInCodebase::test_common_word_only_in_comment_returns_false PASSED [ 24%]
tests/test_cli.py::TestIsPresentInCodebase::test_common_word_in_block_comment_only_returns_false PASSED [ 27%]
tests/test_cli.py::TestIsPresentInCodebase::test_common_word_in_actual_code_matches PASSED [ 30%]
tests/test_codebase_validator.py::test_codebase_validator_color_constants PASSED [ 33%]
tests/test_logical_ui_validator_issue222.py::test_logical_ui_validator_tabbed_container_constraint PASSED [ 36%]
tests/test_mermaid_parsers_bug171.py::test_sequence_diagram_parser_skips_code_fences_and_autonumber_and_notes PASSED [ 39%]
tests/test_mermaid_parsers_bug171.py::test_class_diagram_parser_skips_code_fences_and_notes PASSED [ 42%]
tests/test_mermaid_parsers_bug171.py::test_flowchart_parser_skips_code_fences_and_notes PASSED [ 45%]
tests/test_mermaid_parsers_bug171.py::test_sequence_diagram_note_with_semicolon_rejected PASSED [ 48%]
tests/test_mermaid_parsers_bug171.py::test_sequence_diagram_message_with_semicolon_rejected PASSED [ 51%]
tests/test_uml_validator.py::test_method_return_without_multiplicity_rejected PASSED [ 54%]
tests/test_uml_validator.py::test_method_return_with_valid_multiplicity_accepted PASSED [ 57%]
tests/test_uml_validator.py::test_method_return_void_skipped PASSED      [ 60%]
tests/test_uml_validator.py::test_brackets_in_method_name_not_false_positive PASSED [ 63%]
tests/test_uml_validator.py::test_brackets_in_parameter_types_not_false_positive PASSED [ 66%]
tests/test_uml_validator.py::test_flowchart_parser_reports_parse_errors PASSED [ 69%]
tests/test_uml_validator.py::test_sequence_parser_reports_parse_errors PASSED [ 72%]
tests/test_uml_validator.py::test_flowchart_parser_no_errors_for_valid_input PASSED [ 75%]
tests/test_uml_validator.py::test_sequence_parser_no_errors_for_valid_input PASSED [ 78%]
tests/test_uml_validator.py::test_sanitize_rel_connectors_adds_missing_dot_right_arrow PASSED [ 81%]
tests/test_uml_validator.py::test_sanitize_rel_connectors_sorts_longest_first PASSED [ 84%]
tests/test_uml_validator.py::test_sanitize_rel_connectors_idempotent PASSED [ 87%]
tests/test_uml_validator.py::test_extend_arrow_correct_direction_accepted PASSED [ 90%]
tests/test_uml_validator.py::test_extend_arrow_reversed_direction_flagged PASSED [ 93%]
tests/test_uml_validator.py::test_extend_label_without_stereotype_ignored PASSED [ 96%]
tests/test_uml_validator.py::test_extend_arrow_base_with_ext_in_name_no_false_positive PASSED [100%]

============================== 33 passed in 0.11s ==============================
```

### Baseline Script Verification (`verify_downstream_baseline.py --no-domain`)
```text
Success: App bundled to app_flutter_release.zip
Success: Build and test suite execution passed. Conformance gate verified.
Tagging restoration point...
Cleaning up workspace...
```
