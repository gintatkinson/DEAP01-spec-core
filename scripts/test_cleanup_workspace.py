#!/usr/bin/env python3
import os
import shutil
import tempfile
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

from verify_downstream_baseline import cleanup_workspace


def test_cleanup_db_journal():
    tmp = tempfile.mkdtemp()
    try:
        os.makedirs(os.path.join(tmp, "xcode", "Build"), exist_ok=True)
        journal = os.path.join(tmp, "xcode", "Build", "build.db-journal")
        with open(journal, "w") as f:
            f.write("journal")
        assert os.path.isfile(journal), "journal file not created"

        cleanup_workspace(tmp)

        assert not os.path.isfile(journal), f"FAIL: .db-journal file not cleaned: {journal}"
        print("PASS: test_cleanup_db_journal")
    finally:
        shutil.rmtree(tmp, ignore_errors=True)


def test_cleanup_build_dir():
    tmp = tempfile.mkdtemp()
    try:
        build_dir = os.path.join(tmp, "build")
        os.makedirs(os.path.join(build_dir, "Release"), exist_ok=True)
        with open(os.path.join(build_dir, "Release", "app.framework"), "w") as f:
            f.write("fake")

        cleanup_workspace(tmp)

        assert not os.path.isdir(build_dir), f"FAIL: build/ directory not cleaned: {build_dir}"
        print("PASS: test_cleanup_build_dir")
    finally:
        shutil.rmtree(tmp, ignore_errors=True)


def test_cleanup_db_shm_and_wal_still_work():
    tmp = tempfile.mkdtemp()
    try:
        shm = os.path.join(tmp, "test.db-shm")
        wal = os.path.join(tmp, "test.db-wal")
        with open(shm, "w") as f:
            f.write("shm")
        with open(wal, "w") as f:
            f.write("wal")

        cleanup_workspace(tmp)

        assert not os.path.isfile(shm), "FAIL: .db-shm not cleaned"
        assert not os.path.isfile(wal), "FAIL: .db-wal not cleaned"
        print("PASS: test_cleanup_db_shm_and_wal_still_work")
    finally:
        shutil.rmtree(tmp, ignore_errors=True)


def test_cleanup_dart_tool_and_flutter_plugins():
    tmp = tempfile.mkdtemp()
    try:
        os.makedirs(os.path.join(tmp, ".dart_tool"), exist_ok=True)
        with open(os.path.join(tmp, ".dart_tool", "package_config.json.lock"), "w") as f:
            f.write("{}")
        os.makedirs(os.path.join(tmp, ".flutter-plugins"), exist_ok=True)
        os.makedirs(os.path.join(tmp, ".flutter-plugins-dependencies"), exist_ok=True)

        cleanup_workspace(tmp)

        assert os.path.isdir(os.path.join(tmp, ".dart_tool")), "FAIL: .dart_tool was unexpectedly deleted"
        assert not os.path.isfile(os.path.join(tmp, ".dart_tool", "package_config.json.lock")), "FAIL: package_config.json.lock not cleaned"
        assert not os.path.isdir(os.path.join(tmp, ".flutter-plugins")), "FAIL: .flutter-plugins not cleaned"
        assert not os.path.isdir(os.path.join(tmp, ".flutter-plugins-dependencies")), "FAIL: .flutter-plugins-dependencies not cleaned"
        print("PASS: test_cleanup_dart_tool_and_flutter_plugins")
    finally:
        shutil.rmtree(tmp, ignore_errors=True)


if __name__ == "__main__":
    failures = []
    for test in [test_cleanup_db_journal, test_cleanup_build_dir, test_cleanup_db_shm_and_wal_still_work, test_cleanup_dart_tool_and_flutter_plugins]:
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
