#!/usr/bin/env python3
import subprocess
import re

diff = subprocess.check_output(['git', 'diff', 'origin/main'], text=True)

in_shell_block = False
violations = []

for line in diff.splitlines():
    if line.startswith('+++') or line.startswith('---'):
        continue
    if line.startswith('+'):
        raw_line = line[1:]
        stripped = raw_line.strip()
        if re.match(r'^```(?:bash|sh|shell|zsh)', stripped):
            in_shell_block = True
            continue
        elif in_shell_block and stripped.startswith('```'):
            in_shell_block = False
            continue

        if in_shell_block:
            if stripped.startswith('#'):
                if re.search(r'(?<!\\)[()]', stripped):
                    violations.append(('unescaped paren in comment', stripped))
            else:
                parts = raw_line.split('#', 1)
                code = parts[0]
                no_quotes = re.sub(r'"[^"]*"', '', code)
                no_quotes = re.sub(r"'[^']*'", '', no_quotes)
                angles = re.findall(r'<[a-zA-Z0-9_\-\./]+>', no_quotes)
                if angles:
                    violations.append(('unquoted angle brackets', angles, raw_line))

print(f"Violations in git diff added lines: {len(violations)}")
for v in violations:
    print(" ", v)
