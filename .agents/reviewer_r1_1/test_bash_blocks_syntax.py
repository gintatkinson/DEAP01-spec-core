import re
import sys
import subprocess
import tempfile

with open("/Users/perkunas/jail/DEAP01-spec-core/README.md", "r", encoding="utf-8") as f:
    content = f.read()

lines = content.splitlines()

in_block = False
lang = ""
current_block = []
block_start = 0
failed_blocks = 0

for idx, line in enumerate(lines, 1):
    stripped = line.strip()
    if stripped.startswith("```"):
        if not in_block:
            in_block = True
            lang = stripped[3:].strip()
            block_start = idx
            current_block = []
        else:
            in_block = False
            if lang in ["bash", "sh", "shell", "zsh"]:
                block_code = "\n".join(current_block)
                with tempfile.NamedTemporaryFile(mode="w", suffix=".sh", delete=False) as tf:
                    tf.write(block_code)
                    tf_name = tf.name
                
                res = subprocess.run(["bash", "-n", tf_name], capture_output=True, text=True)
                if res.returncode != 0:
                    print(f"FAILED bash -n check at README.md line {block_start}:")
                    print(res.stderr)
                    print(block_code)
                    failed_blocks += 1
                else:
                    print(f"PASSED bash -n check at README.md line {block_start} ({lang})")
            lang = ""
            current_block = []
    elif in_block:
        current_block.append(line)

print(f"\nTotal failed blocks: {failed_blocks}")
if failed_blocks > 0:
    sys.exit(1)
sys.exit(0)
