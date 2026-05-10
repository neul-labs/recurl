"""
recurl - Drop-in curl replacement with automatic anti-bot bypass

This is a thin Python wrapper that delegates to the platform-specific
binary downloaded at install time.
"""

import os
import platform
import subprocess
import sys
from pathlib import Path


def _find_binary(name: str) -> str:
    """Locate the downloaded recurl binary."""
    # 1. Check alongside this package
    package_dir = Path(__file__).parent
    bin_path = package_dir / "bin" / name
    if bin_path.exists():
        return str(bin_path)

    # 2. Check in PATH
    found = shutil.which(name)
    if found:
        return found

    raise FileNotFoundError(
        f"Could not find {name} binary. "
        "Try reinstalling: pip install --force-reinstall recurl-cli"
    )


def run(args: list[str] = None) -> int:
    """
    Run recurl with the given CLI arguments.

    Args:
        args: List of arguments (e.g., ["-s", "https://example.com"]).
              If None, uses sys.argv[1:].

    Returns:
        Exit code from the recurl process.
    """
    import shutil

    binary = _find_binary("recurl")
    cmd = [binary] + (args if args is not None else sys.argv[1:])
    result = subprocess.run(cmd)
    return result.returncode


def run_daemon(args: list[str] = None) -> int:
    """Run recurld with the given CLI arguments."""
    import shutil

    binary = _find_binary("recurld")
    cmd = [binary] + (args if args is not None else sys.argv[1:])
    result = subprocess.run(cmd)
    return result.returncode
