import re
import sys

with open("/Users/perkunas/jail/DEAP01-spec-core/README.md", "r", encoding="utf-8") as f:
    lines = f.readlines()

in_block = False
lang = ""
block_start = 0
violations = []

for idx, line in enumerate(lines, 1):
    stripped = line.strip()
    if stripped.startswith("```"):
        if not in_block:
            in_block = True
            lang = stripped[3:].strip()
            block_start = idx
        else:
            in_block = False
            lang = ""
    elif in_block and lang in ["bash", "sh", "shell", "zsh"]:
        # Check unescaped parens in comment
        if stripped.startswith("#"):
            unescaped = re.findall(r"(?<!\\)[()]", stripped)
            if unescaped:
                violations.append((idx, "unescaped_paren_in_comment", line.rstrip()))
        # Check unquoted angle brackets
        angle_matches = re.finditer(r"<[^>]+>", line)
        for m in angle_matches:
            match_str = m.group(0)
            prefix = line[:m.start()]
            if prefix.count('"') % 2 == 0 and prefix.count("'") % 2 == 0:
                violations.append((idx, "unquoted_angle_bracket", line.rstrip()))

print(f"Total violations in README.md: {len(violations)}")
for v in violations:
    print(v)

if violations:
    sys.exit(1)
else:
    sys.exit(0)
