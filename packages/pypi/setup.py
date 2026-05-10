"""
Custom setuptools installer for recurl.
Downloads the correct platform binary from GitHub Releases during install.
"""

import os
import platform
import shutil
import sys
import tarfile
import urllib.request
import zipfile
from pathlib import Path

from setuptools import setup
from setuptools.command.install import install

VERSION = "0.1.2"
GITHUB_REPO = "neul-labs/recurl"


def detect_platform():
    system = platform.system().lower()
    machine = platform.machine().lower()

    plat_map = {
        "darwin": "darwin",
        "linux": "linux",
        "windows": "windows",
    }
    arch_map = {
        "x86_64": "x86_64",
        "amd64": "x86_64",
        "arm64": "aarch64",
        "aarch64": "aarch64",
    }

    plat = plat_map.get(system)
    arch = arch_map.get(machine)

    if not plat or not arch:
        raise RuntimeError(
            f"Unsupported platform: {system}-{machine}. "
            "recurl supports: darwin-x86_64, darwin-arm64, linux-x86_64, linux-arm64, windows-x86_64"
        )

    return plat, arch


def download_binary(package_dir: Path):
    plat, arch = detect_platform()
    ext = "zip" if plat == "windows" else "tar.gz"
    asset_name = f"recurl-{plat}-{arch}.{ext}"
    url = f"https://github.com/{GITHUB_REPO}/releases/download/v{VERSION}/{asset_name}"

    bin_dir = package_dir / "bin"
    bin_dir.mkdir(parents=True, exist_ok=True)

    tmp_path = package_dir / asset_name
    print(f"[recurl] Downloading {asset_name}...")
    urllib.request.urlretrieve(url, tmp_path)

    print("[recurl] Extracting...")
    if ext == "tar.gz":
        with tarfile.open(tmp_path, "r:gz") as tar:
            tar.extractall(path=bin_dir)
    else:
        with zipfile.ZipFile(tmp_path, "r") as zf:
            zf.extractall(path=bin_dir)

    # The archive has a top-level folder; flatten it
    entries = [e for e in bin_dir.iterdir() if e.is_dir()]
    if entries:
        top = entries[0]
        for item in top.iterdir():
            shutil.move(str(item), str(bin_dir / item.name))
        top.rmdir()

    tmp_path.unlink(missing_ok=True)

    # Make executable on Unix
    if plat != "windows":
        for binary in ("recurl", "recurld"):
            binary_path = bin_dir / binary
            if binary_path.exists():
                binary_path.chmod(0o755)


class RecurlInstall(install):
    def run(self):
        install.run(self)
        package_dir = Path(self.install_lib) / "recurl"
        download_binary(package_dir)


setup(
    cmdclass={"install": RecurlInstall},
)
