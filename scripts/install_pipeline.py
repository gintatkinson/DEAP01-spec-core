import argparse
import json
import os
import sys


def discover_profiles(repo_root=None):
    if repo_root is None:
        script_dir = os.path.dirname(os.path.abspath(__file__))
        potential_roots = [os.getcwd(), os.path.dirname(script_dir)]
    else:
        potential_roots = [repo_root]

    for root in potential_roots:
        profiles_dir = os.path.join(root, ".pipeline", "profiles")
        if os.path.isdir(profiles_dir):
            profiles = [
                os.path.splitext(f)[0]
                for f in sorted(os.listdir(profiles_dir))
                if f.endswith(".md") and not f.startswith(".")
            ]
            if profiles:
                return profiles
    return []


def install(profile_name, target_dir="."):
    pipeline_dir = os.path.join(target_dir, ".pipeline")
    os.makedirs(pipeline_dir, exist_ok=True)
    config_path = os.path.join(pipeline_dir, "profile_config.json")
    config = {"active_profile": profile_name}
    with open(config_path, "w", encoding="utf-8") as f:
        json.dump(config, f, indent=2)
    print(f"Successfully configured pipeline for profile: {profile_name}")


def main():
    profiles = discover_profiles()
    parser = argparse.ArgumentParser(description="Modular Downstream Installer CLI")
    if profiles:
        parser.add_argument(
            "--profile",
            choices=profiles,
            required=True,
            help=f"Target implementation profile (available: {', '.join(profiles)})",
        )
    else:
        parser.add_argument(
            "--profile",
            required=True,
            help="Target implementation profile",
        )
    args = parser.parse_args()
    install(args.profile)


if __name__ == "__main__":
    main()
