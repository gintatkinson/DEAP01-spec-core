#!/usr/bin/env python3
import os
import sys
import tempfile

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from verify_downstream_baseline import tag_restoration_point


def test_tag_success_in_git_repo():
    with tempfile.TemporaryDirectory() as tmp:
        init = os.system(f"cd {tmp} && git init --quiet && git commit --allow-empty -m init --quiet")
        assert init == 0, "Failed to initialize test git repo"

        result = tag_restoration_point()
        assert result is True, f"Expected True in git repo, got {result}"
        print("PASS: test_tag_success_in_git_repo")


def test_tag_fails_outside_git_repo():
    with tempfile.TemporaryDirectory() as tmp:
        orig_cwd = os.getcwd()
        try:
            os.chdir(tmp)
            result = tag_restoration_point()
            assert result is False, f"Expected False outside git repo, got {result}"
            print("PASS: test_tag_fails_outside_git_repo")
        finally:
            os.chdir(orig_cwd)


def test_tag_returns_bool_type():
    with tempfile.TemporaryDirectory() as tmp:
        init = os.system(f"cd {tmp} && git init --quiet && git commit --allow-empty -m init --quiet")
        assert init == 0, "Failed to initialize test git repo"

        result = tag_restoration_point()
        assert isinstance(result, bool), f"Expected bool return type, got {type(result).__name__}"
        print("PASS: test_tag_returns_bool_type")


if __name__ == "__main__":
    failures = []
    for test in [test_tag_success_in_git_repo, test_tag_fails_outside_git_repo, test_tag_returns_bool_type]:
        try:
            test()
        except AssertionError as e:
            print(str(e))
            failures.append(test.__name__)

    if failures:
        print(f"\n{failures} failed")
        sys.exit(1)
    else:
        print("\nAll tests passed.")
