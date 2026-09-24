import os
import subprocess
import tempfile
import sys
import re

print("Starting E2E Independent Reviewer Verification...")

INSTALLER_PATH = "/Users/perkunas/jail/DEAP01-spec-core/scripts/install_pipeline.sh"

with tempfile.TemporaryDirectory() as td:
    print(f"Sandbox created at {td}")
    
    # --- TEST 1: Tier 1 Domain Template Installation ---
    print("\n--- TEST 1: Tier 1 Domain Template Installation ---")
    domain_repo = os.path.join(td, "DEAP-uas-safety")
    os.makedirs(domain_repo)
    subprocess.run(["git", "init", domain_repo], check=True, capture_output=True)
    subprocess.run(["git", "-C", domain_repo, "remote", "add", "origin", "https://github.com/defense-contractor/DEAP-uas-safety.git"], check=True, capture_output=True)
    
    # Run installer from DEAP01-spec-core into domain_repo
    res1 = subprocess.run(["bash", INSTALLER_PATH, domain_repo], capture_output=True, text=True)
    if res1.returncode != 0:
        print("Tier 1 installation failed:")
        print(res1.stderr)
        sys.exit(1)
    
    assert os.path.exists(os.path.join(domain_repo, "README.md")), "README.md missing in domain repo"
    readme_tier1 = open(os.path.join(domain_repo, "README.md")).read()
    
    expected_clone = "git clone https://github.com/defense-contractor/DEAP-uas-safety.git ./.tmp-pipeline && bash ./.tmp-pipeline/scripts/install_pipeline.sh . && rm -rf ./.tmp-pipeline"
    assert expected_clone in readme_tier1, f"Expected clone command not found in README.md. Got:\n{readme_tier1[:1000]}"
    assert "DEAP01-spec-core" not in readme_tier1, "Found DEAP01-spec-core reference in domain README"
    print("Test 1 Passed: Domain README properly parameterized with domain remote URL.")
    
    # --- TEST 2: Add Domain Models & Commit in Domain Repo ---
    print("\n--- TEST 2: Add Domain Models in Domain Template ---")
    schema_dir = os.path.join(domain_repo, "schema")
    os.makedirs(os.path.join(schema_dir, "submodels"), exist_ok=True)
    with open(os.path.join(schema_dir, "UAS_SAFETY_MODEL.sysml"), "w") as f:
        f.write("package UasSafetyModel {}\n")
    with open(os.path.join(schema_dir, "submodels", "BATTERY_MONITOR.sysml"), "w") as f:
        f.write("package BatteryMonitor {}\n")
    
    subprocess.run(["git", "-C", domain_repo, "add", "."], check=True, capture_output=True)
    subprocess.run(["git", "-C", domain_repo, "commit", "-m", "add domain schemas"], check=True, capture_output=True)
    
    # --- TEST 3: Tier 2 Customer Onboarding with Pre-Existing schema/ (.gitkeep) ---
    print("\n--- TEST 3: Tier 2 Customer Onboarding with Pre-Existing schema/ (.gitkeep) ---")
    customer_repo1 = os.path.join(td, "customer_mission_alpha")
    os.makedirs(os.path.join(customer_repo1, "schema"))
    with open(os.path.join(customer_repo1, "schema", ".gitkeep"), "w") as f:
        f.write("")
    subprocess.run(["git", "init", customer_repo1], check=True, capture_output=True)
    
    # Run onboarding via .tmp-pipeline
    tmp_pipe1 = os.path.join(customer_repo1, ".tmp-pipeline")
    subprocess.run(["git", "clone", domain_repo, tmp_pipe1], check=True, capture_output=True)
    # Ensure origin in tmp_pipe1 is preserved as the domain remote URL
    subprocess.run(["git", "-C", tmp_pipe1, "remote", "set-url", "origin", "https://github.com/defense-contractor/DEAP-uas-safety.git"], check=True, capture_output=True)
    res2 = subprocess.run(["bash", os.path.join(tmp_pipe1, "scripts/install_pipeline.sh"), customer_repo1], capture_output=True, text=True)
    if res2.returncode != 0:
        print("Tier 2 installation failed:")
        print(res2.stderr)
        sys.exit(1)
    subprocess.run(["rm", "-rf", tmp_pipe1], check=True)
    
    # Verify schemas copied
    assert os.path.exists(os.path.join(customer_repo1, "schema", "UAS_SAFETY_MODEL.sysml")), "UAS_SAFETY_MODEL.sysml not copied!"
    assert os.path.exists(os.path.join(customer_repo1, "schema", "submodels", "BATTERY_MONITOR.sysml")), "Nested schema not copied!"
    assert not os.path.exists(os.path.join(customer_repo1, "schema", "schema")), "Unwanted nested schema/schema created!"
    print("Test 3 Passed: Pre-existing schema/ successfully received domain models without nesting.")

    # --- TEST 4: Tier 2 Customer Onboarding with Existing Customer Schema ---
    print("\n--- TEST 4: Tier 2 Customer Onboarding with Existing Customer Files ---")
    customer_repo2 = os.path.join(td, "customer_mission_beta")
    os.makedirs(os.path.join(customer_repo2, "schema"))
    with open(os.path.join(customer_repo2, "schema", "CUSTOM_PAYLOAD.sysml"), "w") as f:
        f.write("package CustomPayload {}\n")
    subprocess.run(["git", "init", customer_repo2], check=True, capture_output=True)
    
    tmp_pipe2 = os.path.join(customer_repo2, ".tmp-pipeline")
    subprocess.run(["git", "clone", domain_repo, tmp_pipe2], check=True, capture_output=True)
    res3 = subprocess.run(["bash", os.path.join(tmp_pipe2, "scripts/install_pipeline.sh"), customer_repo2], capture_output=True, text=True)
    if res3.returncode != 0:
        print("Tier 2 test 4 failed:")
        print(res3.stderr)
        sys.exit(1)
    subprocess.run(["rm", "-rf", tmp_pipe2], check=True)
    
    assert os.path.exists(os.path.join(customer_repo2, "schema", "CUSTOM_PAYLOAD.sysml")), "Custom schema deleted or lost!"
    assert os.path.exists(os.path.join(customer_repo2, "schema", "UAS_SAFETY_MODEL.sysml")), "Domain model missing!"
    print("Test 4 Passed: Pre-existing customer schemas preserved alongside domain models.")

    # --- TEST 5: Verify Generated README Syntax in Domain Repo ---
    print("\n--- TEST 5: Code Block Syntax Verification in Domain README ---")
    readme_lines = readme_tier1.splitlines()
    in_block = False
    block_lang = ""
    block_lines = []
    block_start = 0
    
    for idx, line in enumerate(readme_lines, 1):
        stripped = line.strip()
        if stripped.startswith("```"):
            if not in_block:
                in_block = True
                block_lang = stripped[3:].strip()
                block_start = idx
                block_lines = []
            else:
                in_block = False
                if block_lang in ["bash", "sh", "shell"]:
                    # Check bash -n
                    b_code = "\n".join(block_lines)
                    with tempfile.NamedTemporaryFile(mode="w", suffix=".sh", delete=False) as tf:
                        tf.write(b_code)
                        tname = tf.name
                    chk = subprocess.run(["bash", "-n", tname], capture_output=True, text=True)
                    assert chk.returncode == 0, f"bash -n failed on domain README block at line {block_start}: {chk.stderr}"
                    
                    # Check parens in comments
                    for l in block_lines:
                        sl = l.strip()
                        if sl.startswith("#"):
                            parens = re.findall(r"(?<!\\)[()]", sl)
                            assert len(parens) == 0, f"Unescaped paren in comment at line {idx}: {sl}"
                        # Check unquoted angle brackets
                        for m in re.finditer(r"<[^>]+>", l):
                            prefix = l[:m.start()]
                            assert prefix.count('"') % 2 != 0 or prefix.count("'") % 2 != 0, f"Unquoted angle bracket in {l}"
                block_lang = ""
                block_lines = []
        elif in_block:
            block_lines.append(line)
            
    print("Test 5 Passed: Domain README contains pure valid shell syntax, zero unescaped parens in comments, zero unquoted angle brackets.")

print("\nALL VERIFICATION TESTS COMPLETED SUCCESSFULLY!")
