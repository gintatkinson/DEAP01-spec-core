# Copyright Gint Atkinson, gint.atkinson@gmail.com

from setuptools import find_packages, setup

if __name__ == "__main__":
    setup(
        name="deap01-spec-core",
        version="0.1.0",
        package_dir={"": "skills/spec-orchestrator/parity_auditor/src"},
        packages=find_packages(where="skills/spec-orchestrator/parity_auditor/src"),
        entry_points={
            "console_scripts": [
                "parity-auditor = parity_auditor.cli:main",
            ],
        },
    )
