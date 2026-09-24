import os
import re
import sys

def audit_all_md(root_dir):
    findings = []
    for root, dirs, files in os.walk(root_dir):
        # skip .git, .venv, etc.
        if any(p in root for p in [".git", ".venv", "node_modules"]):
            continue
        for f in files:
            if f.endswith(".md"):
                md_path = os.path.join(root, f)
                audit_file(md_path, findings)

    print(f"Total findings across repository markdown files: {len(findings)}")
    for item in findings:
        print(f"{item['file']}:{item['line_no']} (block starting {item['block_start']}, lang={item['lang']}) [{item['type']}]: {item['content']}")

def audit_file(path, findings):
    with open(path, 'r', encoding='utf-8', errors='ignore') as f:
        lines = f.readlines()

    in_block = False
    block_lang = ""
    block_start = 0

    for idx, line in enumerate(lines, 1):
        stripped = line.strip()
        if stripped.startswith("```"):
            if not in_block:
                in_block = True
                block_lang = stripped[3:].strip()
                block_start = idx
            else:
                in_block = False
        elif in_block:
            if block_lang in ["bash", "sh", "shell", "zsh", "console"]:
                stripped_line = line.strip()
                if stripped_line.startswith("#"):
                    unescaped_parens = re.findall(r"(?<!\\)[()]", stripped_line)
                    if unescaped_parens:
                        findings.append({
                            "file": path,
                            "type": "unescaped_paren_in_comment",
                            "line_no": idx,
                            "block_start": block_start,
                            "lang": block_lang,
                            "content": line.rstrip()
                        })
                angle_matches = re.finditer(r"<[^>]+>", line)
                for m in angle_matches:
                    match_str = m.group(0)
                    start_pos = m.start()
                    prefix = line[:start_pos]
                    single_quotes = prefix.count("'") % 2
                    double_quotes = prefix.count('"') % 2
                    backticks = prefix.count('`') % 2
                    if single_quotes == 0 and double_quotes == 0 and backticks == 0:
                        # ignore redirection like <<EOF or < file
                        if not re.match(r"^<[0-9]+$", match_str):
                            findings.append({
                                "file": path,
                                "type": "unquoted_angle_bracket",
                                "line_no": idx,
                                "block_start": block_start,
                                "lang": block_lang,
                                "content": line.rstrip(),
                                "match": match_str
                            })

if __name__ == "__main__":
    audit_all_md(".")
