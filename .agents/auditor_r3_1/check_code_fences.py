#!/usr/bin/env python3
import re
import os
import glob

def check_file(fpath):
    with open(fpath, 'r', encoding='utf-8') as f:
        content = f.read()

    blocks = re.findall(r'```(?:bash|sh|shell|zsh)\n(.*?)```', content, re.DOTALL)
    issues = []

    for b_idx, block in enumerate(blocks):
        lines = block.splitlines()
        for l_idx, line in enumerate(lines):
            stripped = line.strip()
            # 1. Check for unescaped parentheses in comments
            if stripped.startswith('#'):
                parens = re.findall(r'(?<!\\)[()]', stripped)
                if parens:
                    issues.append((b_idx + 1, l_idx + 1, 'unescaped parenthesis in comment', stripped))

            # 2. Check for unquoted angle brackets <...> in shell command lines
            if not stripped.startswith('#'):
                parts = line.split('#', 1)
                code_part = parts[0]
                no_quotes = re.sub(r'"[^"]*"', '', code_part)
                no_quotes = re.sub(r"'[^']*'", '', no_quotes)
                angles = re.findall(r'<[a-zA-Z0-9_\-\./]+>', no_quotes)
                if angles:
                    issues.append((b_idx + 1, l_idx + 1, f'unquoted angle brackets {angles}', line))

    return len(blocks), issues

def main():
    md_files = []
    for root, dirs, files in os.walk('.'):
        if '.agents' in root or '.git' in root or 'node_modules' in root:
            continue
        for f in files:
            if f.endswith('.md'):
                md_files.append(os.path.normpath(os.path.join(root, f)))

    md_files.sort()
    total_issues = 0
    total_blocks = 0
    print(f"=== Scanning All {len(md_files)} Markdown Files in Repository ===")
    for fpath in md_files:
        num_blocks, issues = check_file(fpath)
        total_blocks += num_blocks
        if issues:
            print(f"File: {fpath} ({num_blocks} shell blocks)")
            for b_num, l_num, desc, line in issues:
                print(f"  [FAIL] Block {b_num}, Line {l_num}: {desc} -> {line.strip()}")
                total_issues += 1

    print(f"\nTotal markdown files scanned: {len(md_files)}")
    print(f"Total shell blocks scanned: {total_blocks}")
    print(f"Total violations found: {total_issues}")

if __name__ == '__main__':
    main()
