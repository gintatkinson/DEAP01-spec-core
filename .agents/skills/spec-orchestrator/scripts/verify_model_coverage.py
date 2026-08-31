#!/usr/bin/env python3
# Copyright Gint Atkinson, gint.atkinson@gmail.com

import sys
import os

def sanitize_github_token_env():
    """
    Sanitize environment by removing dummy or placeholder GITHUB_TOKEN and GH_TOKEN
    values that interfere with git/gh terminal operations.
    """
    dummy_keywords = ("antigravity", "dummy", "placeholder", "invalid", "mock")
    for var in ("GITHUB_TOKEN", "GH_TOKEN"):
        val = os.environ.get(var)
        if val and any(kw in val.lower() for kw in dummy_keywords):
            os.environ.pop(var, None)

# NOTE: sanitize_github_token_env() is deliberately NOT called at module level.
# Doing so mutated os.environ on import, so any test importing this module stripped
# GITHUB_TOKEN/GH_TOKEN from the process and unrelated tests failed depending on
# import order (issue #276). It is invoked from __main__ below, which is the only
# context that shells out to git/gh.

sys.path.insert(0, os.path.abspath(os.path.join(os.path.dirname(__file__), "..", "parity_auditor", "src")))

from parity_auditor.cli import main
from parity_auditor.validators.doc_metadata_validator import DocMetadataValidator

if __name__ == "__main__":
    sanitize_github_token_env()
    main()

