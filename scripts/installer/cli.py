#!/usr/bin/env python3
"""
CLI argument parsing, input validation, and execution entrypoint.
/// Realises: [InstallerCLI]
"""
import argparse
import os
import shutil
import sys
from pathlib import Path
from typing import List, Optional

from scripts.installer.metadata import (
    InstallOptions,
    RepositoryRole,
    capture_preserved_metadata,
    detect_repository_role,
    discover_project_metadata,
    get_git_remote_url,
    parse_git_remote_url,
    resolve_domain_remote_url,
    resolve_provider_and_metadata,
)
from scripts.installer.rollback import (
    CLIValidationError,
    InstallerError,
    RollbackManager,
)
from scripts.installer.scaffolding import (
    compile_active_rules_bundle,
    generate_downstream_readme,
    generate_env_template,
    install_git_hooks,
    purge_ds_store,
    scaffold_agents_and_governance,
    should_scaffold_readme,
    verify_safety_fixtures,
)
from scripts.installer.staging import StagingManager
from scripts.installer.tracker import (
    bootstrap_tracker_labels,
    configure_tracker_rules,
    deploy_gitlab_ci_template,
)


HELP_DESCRIPTION = """Usage: install_pipeline.sh [OPTIONS] [TARGET_DIR]

Installs the DEAP safety-critical engineering pipeline and governance baseline into a downstream project repository.

Primary Commercial Toolchain Integration Context:
  MATLAB / Simulink / Stateflow / Embedded Coder (Model-Based Design, Control Law Synthesis, DO-178C C/SPARK Ada code generation).

Examples:
  ./scripts/install_pipeline.sh
  ./scripts/install_pipeline.sh .
  ./scripts/install_pipeline.sh /path/to/downstream-project
  ./scripts/install_pipeline.sh --role domain-template
  ./scripts/install_pipeline.sh --role customer-project
  ./scripts/install_pipeline.sh --platform gitlab
  ./scripts/install_pipeline.sh --provider gitlab --gitlab-url https://gitlab.internal.defense.gov
  ./scripts/install_pipeline.sh --tracker jira --jira-url https://my-org.atlassian.net --jira-project PROJ
  ./scripts/install_pipeline.sh --provider github"""


class CustomHelpFormatter(argparse.RawDescriptionHelpFormatter):
    pass


def build_parser() -> argparse.ArgumentParser:
    """
    Constructs the ArgumentParser replicating 100% of CLI options and aliases
    from legacy install_pipeline.sh.
    """
    parser = argparse.ArgumentParser(
        prog="install_pipeline.sh",
        description=HELP_DESCRIPTION,
        formatter_class=CustomHelpFormatter,
        add_help=False,
    )

    # Positional target directory
    parser.add_argument(
        "target_pos",
        nargs="?",
        default=None,
        metavar="TARGET_DIR",
        help="Target project directory (default: current directory '.')",
    )

    # Options
    parser.add_argument(
        "--target",
        dest="target_opt",
        default=None,
        metavar="TARGET_DIR",
        help="Target project directory (equivalent to positional argument)",
    )
    parser.add_argument(
        "-r",
        "--role",
        dest="role",
        default=None,
        metavar="ROLE",
        help="Target repository role: 'domain-template' or 'customer-project' (auto-detected if omitted)",
    )
    parser.add_argument(
        "-p",
        "--provider",
        dest="provider",
        default=None,
        metavar="PROVIDER",
        help="Target issue tracker and CI/CD provider: 'github', 'gitlab', 'jira', or 'auto' (default: 'auto')",
    )
    parser.add_argument(
        "-t",
        "--tracker",
        dest="tracker",
        default=None,
        metavar="TRACKER",
        help="Alias for --provider: 'github', 'gitlab', 'jira', or 'auto'",
    )
    parser.add_argument(
        "--platform",
        dest="platform",
        default=None,
        metavar="PLATFORM",
        help="Alias for --provider: 'github', 'gitlab', 'jira', or 'auto'",
    )
    parser.add_argument(
        "--gitlab-url",
        dest="gitlab_url",
        default="https://gitlab.com",
        metavar="URL",
        help="GitLab instance base URL (default: 'https://gitlab.com')",
    )
    parser.add_argument(
        "--gitlab-group",
        dest="gitlab_group",
        default="",
        metavar="GROUP",
        help="GitLab namespace/group path (e.g. 'uas-safety', auto-detected from git remote if omitted)",
    )
    parser.add_argument(
        "--github-org",
        dest="github_org",
        default="",
        metavar="ORG",
        help="GitHub organization/user (auto-detected from git remote if omitted)",
    )
    parser.add_argument(
        "--jira-url",
        dest="jira_url",
        default="https://your-domain.atlassian.net",
        metavar="URL",
        help="Jira instance base URL (default: 'https://your-domain.atlassian.net')",
    )
    parser.add_argument(
        "--jira-project",
        dest="jira_project",
        default="",
        metavar="PROJECT",
        help="Jira project key code (e.g. 'UAS')",
    )
    parser.add_argument(
        "--jira-email",
        dest="jira_email",
        default="",
        metavar="EMAIL",
        help="Jira account email address (for Jira Cloud Basic Auth)",
    )
    parser.add_argument(
        "--domain-url",
        dest="domain_url",
        default="",
        metavar="URL",
        help="Explicit remote URL for upstream domain template repository",
    )
    parser.add_argument(
        "--domain-name",
        dest="domain_name",
        default="",
        metavar="NAME",
        help="Domain template or project name (e.g. 'DEAP-uas-infrastructure-safety')",
    )
    parser.add_argument(
        "-h",
        "--help",
        action="help",
        help="Display this help documentation and exit",
    )

    return parser


def parse_args(argv: Optional[List[str]] = None) -> InstallOptions:
    """
    Parses CLI arguments into a validated InstallOptions instance.
    """
    parser = build_parser()
    args = parser.parse_args(argv)

    # Resolve target directory
    if args.target_opt and args.target_pos and args.target_opt != args.target_pos:
        raise CLIValidationError("Error: Both --target and positional TARGET_DIR provided with conflicting values.")

    target_dir_str = args.target_opt or args.target_pos or "."
    target_dir = Path(target_dir_str)

    # Resolve provider / tracker / platform aliases
    provider = args.provider or args.tracker or args.platform or "auto"

    # Resolve installer root (repository root of DEAP01-spec-core)
    installer_root = Path(__file__).resolve().parent.parent.parent

    return InstallOptions(
        installer_root=installer_root,
        target_dir=target_dir,
        cli_role=args.role,
        provider=provider,
        gitlab_url=args.gitlab_url,
        gitlab_group=args.gitlab_group,
        github_org=args.github_org,
        jira_url=args.jira_url,
        jira_project=args.jira_project,
        jira_email=args.jira_email,
        domain_url=args.domain_url,
        domain_name=args.domain_name,
    )


def validate_environment(options: InstallOptions) -> None:
    """
    Enforces self-installation prevention and directory accessibility checks.
    """
    target_dir = options.target_dir.resolve()
    installer_root = options.installer_root.resolve()

    if target_dir == installer_root:
        if (installer_root / ".pipeline" / "upstream").exists():
            raise InstallerError("REFUSING: target is the pipeline repository itself, not a downstream project.")
        print(f"Operating in-place on initialized downstream repository: {target_dir}")


def main(argv: Optional[List[str]] = None) -> int:
    """
    CLI entrypoint executed via `python3 -m scripts.installer.cli "$@"`.
    """
    try:
        options = parse_args(argv)
        validate_environment(options)

        target_dir = options.target_dir.resolve()
        target_dir.mkdir(parents=True, exist_ok=True)

        remote_url = get_git_remote_url(target_dir) or ""
        remote_info = parse_git_remote_url(remote_url) if remote_url else None

        options = resolve_provider_and_metadata(options, remote_info)
        detected_project = remote_info.project if remote_info else ""

        options.target_role = detect_repository_role(
            target_dir=target_dir,
            cli_role=options.cli_role,
            detected_project=detected_project,
            remote_url=remote_url,
            domain_url=options.domain_url,
            domain_name=options.domain_name,
        )

        print(f"Target repository role: {options.target_role.value}")

        preserved = capture_preserved_metadata(target_dir)

        with RollbackManager(target_dir) as rollback_mgr:
            with StagingManager(options, preserved) as staging_mgr:
                staging_mgr.assemble_assets()
                staging_mgr.enforce_clean_landing_zones()

                # Rules bundle compilation
                rules_src = options.installer_root / "rules"
                if not rules_src.is_dir() and (target_dir / "rules").is_dir():
                    rules_src = target_dir / "rules"
                compile_active_rules_bundle(
                    rules_src_dir=rules_src,
                    bundle_dest_file=staging_mgr.staging_path / ".pipeline" / "ACTIVE_RULES_BUNDLE.md",
                )

                # Tracker rules and GitLab CI
                configure_tracker_rules(staging_mgr.staging_path, options)
                deploy_gitlab_ci_template(options.installer_root, staging_mgr.staging_path, options)

                # Environment template
                generate_env_template(staging_mgr.staging_path / ".env.template")

                # Agent governance scaffolding
                scaffold_agents_and_governance(options.installer_root, staging_mgr.staging_path)

                # Dynamic README scaffolding
                meta = discover_project_metadata(target_dir, options, remote_info)
                meta.domain_remote_url = resolve_domain_remote_url(
                    installer_root=options.installer_root,
                    target_dir=target_dir,
                    options=options,
                    remote_info=remote_info,
                    meta=meta,
                )

                target_readme = target_dir / "README.md"
                if should_scaffold_readme(target_readme, options.target_role):
                    generate_downstream_readme(
                        dest_readme=staging_mgr.staging_path / "README.md",
                        target_role=options.target_role,
                        meta=meta,
                    )
                elif target_readme.is_file():
                    shutil.copy2(target_readme, staging_mgr.staging_path / "README.md")

                # Safety integrity test fixtures validation
                verify_safety_fixtures(staging_mgr.staging_path)

                # Non-destructive atomic promotion
                staging_mgr.promote_to_target(rollback_mgr)

        # Post-promotion tasks outside rollback context
        install_git_hooks(target_dir)
        bootstrap_tracker_labels(target_dir)
        purge_ds_store(target_dir)

        print("==> Digital Pipeline Installation Complete. 0 manual steps remaining.")
        return 0

    except SystemExit as e:
        return e.code if isinstance(e.code, int) else (0 if e.code is None else 1)
    except CLIValidationError as e:
        sys.stderr.write(f"{e}\n")
        return 1
    except InstallerError as e:
        sys.stderr.write(f"{e}\n")
        return 1
    except Exception as e:
        sys.stderr.write(f"Unexpected installation error: {e}\n")
        return 1


if __name__ == "__main__":
    sys.exit(main())
