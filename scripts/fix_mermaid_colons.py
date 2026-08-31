#!/usr/bin/env python3
import os
import re
import sys

# Regex pattern for inline members: Class : +methodName(args) : ReturnType [multiplicity]
# Groups:
# 1: leading whitespace
# 2: class name (alphanumeric/underscore)
# 3: visibility prefix (+, -, #, ~)
# 4: method name (alphanumeric/underscore)
# 5: parameters list (anything inside parenthesis)
# 6: optional return type (anything except space, [, ])
INLINE_REGEX = re.compile(
    r'^(\s*)(\w+)\s*:\s*([+\-#~]?)\s*(\w+)\s*\(([^)]*)\)(?:\s*:?\s*([^\s\[\]]+))?\s*(?:\[[^\]]*\])?\s*$'
)

# Regex pattern for class-body members: +methodName(args) : ReturnType [multiplicity]
# Groups:
# 1: leading whitespace
# 2: visibility prefix (+, -, #, ~)
# 3: method name (alphanumeric/underscore)
# 4: parameters list (anything inside parenthesis)
# 5: optional return type (anything except space, [, ])
BODY_REGEX = re.compile(
    r'^(\s*)([+\-#~]?)\s*(\w+)\s*\(([^)]*)\)(?:\s*:?\s*([^\s\[\]]+))?\s*(?:\[[^\]]*\])?\s*$'
)

def transform_params(params_str):
    """
    Convert parameter list format from 'name : Type' to 'Type name'.
    """
    if not params_str.strip():
        return ""
    parts = params_str.split(",")
    new_parts = []
    for part in parts:
        part = part.strip()
        if not part:
            continue
        if ":" in part:
            name, ptype = part.split(":", 1)
            name = name.strip()
            ptype = ptype.strip()
            new_parts.append(f"{ptype} {name}")
        else:
            new_parts.append(part)
    return ", ".join(new_parts)

def transform_mermaid_line(line):
    """
    Transforms a single line of a Mermaid classDiagram if it matches the target signatures.
    Returns (new_line, modified_boolean).
    """
    # Try inline match first
    inline_match = INLINE_REGEX.match(line)
    if inline_match:
        leading_spaces = inline_match.group(1)
        class_name = inline_match.group(2)
        visibility = inline_match.group(3)
        method_name = inline_match.group(4)
        params = inline_match.group(5)
        ptype = inline_match.group(6)

        transformed_params = transform_params(params)
        
        # Format: Class : +ReturnType methodName(args)
        if ptype:
            new_line = f"{leading_spaces}{class_name} : {visibility}{ptype} {method_name}({transformed_params})"
        else:
            new_line = f"{leading_spaces}{class_name} : {visibility}{method_name}({transformed_params})"
        
        if new_line != line:
            return new_line, True

    # Try body match
    body_match = BODY_REGEX.match(line)
    if body_match:
        leading_spaces = body_match.group(1)
        visibility = body_match.group(2)
        method_name = body_match.group(3)
        params = body_match.group(4)
        ptype = body_match.group(5)

        transformed_params = transform_params(params)

        # Format: +ReturnType methodName(args)
        if ptype:
            new_line = f"{leading_spaces}{visibility}{ptype} {method_name}({transformed_params})"
        else:
            new_line = f"{leading_spaces}{visibility}{method_name}({transformed_params})"

        if new_line != line:
            return new_line, True

    return line, False

def process_file(file_path):
    """
    Reads the file, replaces invalid Mermaid signatures inside class diagram blocks,
    and writes the modified content back.
    """
    try:
        with open(file_path, "r", encoding="utf-8") as f:
            content = f.read()
    except Exception as e:
        print(f"Error reading {file_path}: {e}", file=sys.stderr)
        return

    lines = content.splitlines()
    in_mermaid = False
    is_class_diagram = False
    mermaid_block_lines = []
    mermaid_start_idx = -1
    
    modified = False
    changes = [] # list of (line_num, old_line, new_line)

    for idx, line in enumerate(lines):
        # We check for ```mermaid block start
        if line.strip().startswith("```mermaid"):
            in_mermaid = True
            mermaid_block_lines = [line]
            mermaid_start_idx = idx
            is_class_diagram = False
        elif in_mermaid:
            mermaid_block_lines.append(line)
            if "classDiagram" in line:
                is_class_diagram = True
            if line.strip().startswith("```"):
                in_mermaid = False
                if is_class_diagram:
                    # Transform the lines within this block (excluding the ```mermaid and ``` fences)
                    new_mermaid_lines = [mermaid_block_lines[0]]
                    for m_idx, ml in enumerate(mermaid_block_lines[1:-1], start=1):
                        new_line, line_modified = transform_mermaid_line(ml)
                        new_mermaid_lines.append(new_line)
                        if line_modified:
                            abs_line_num = mermaid_start_idx + m_idx + 1
                            changes.append((abs_line_num, ml, new_line))
                            modified = True
                    new_mermaid_lines.append(mermaid_block_lines[-1])
                    
                    # Apply changes back to lines list
                    for k, new_l in enumerate(new_mermaid_lines):
                        lines[mermaid_start_idx + k] = new_l
                
                mermaid_block_lines = []
                mermaid_start_idx = -1
                is_class_diagram = False

    if modified:
        new_content = "\n".join(lines)
        # Preserve trailing newline if original had it
        if content.endswith("\n") and not new_content.endswith("\n"):
            new_content += "\n"
        
        try:
            with open(file_path, "w", encoding="utf-8") as f:
                f.write(new_content)
            
            print(f"Modified file: {file_path}")
            for line_num, old, new in changes:
                print(f"  Line {line_num}:")
                print(f"    - {old}")
                print(f"    + {new}")
        except Exception as e:
            print(f"Error writing to {file_path}: {e}", file=sys.stderr)

def main():
    docs_dir = "docs"
    if not os.path.exists(docs_dir):
        print(f"Error: {docs_dir} directory not found in the current working directory.", file=sys.stderr)
        sys.exit(1)
        
    for root, dirs, files in os.walk(docs_dir):
        for file in files:
            if file.endswith(".md"):
                file_path = os.path.join(root, file)
                process_file(file_path)

if __name__ == "__main__":
    main()
