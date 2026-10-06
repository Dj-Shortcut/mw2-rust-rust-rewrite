#!/usr/bin/env python3
"""Guarded source preparation; installation does not verify a playable server."""

from __future__ import annotations

import argparse
import datetime
import gzip
import hashlib
import json
import os
import platform
from pathlib import Path, PurePosixPath
import re
import shutil
import signal
import stat
import subprocess
import sys
import tarfile
import urllib.request
import zipfile

STEAM_URL = "https://steamcdn-a.akamaihd.net/client/installer/steamcmd_linux.tar.gz"
RELEASE_API = "https://api.github.com/repos/OxideMod/Oxide.Rust/releases/tags/"
DEFAULT_RELEASE = "2.0.7801"
MAX_ARCHIVE = 64 * 1024 * 1024
MAX_EXPANDED = 256 * 1024 * 1024
MIN_RAM = 12 * 1024**3
MIN_DISK = 20 * 1024**3
PROTECTED = {".mod-bootstrap", "install-manifest.json", "server", "steamapps", "oxide"}


def root_argument(value: str) -> PurePosixPath:
    # SteamCMD has its own argument parser. Keep this path unambiguous for it.
    if not re.fullmatch(r"/[A-Za-z0-9_./-]+", value) or value == "/":
        raise ValueError("Root must be an absolute new path using letters, digits, /, _, - and . only.")
    if any(part in {"", ".", ".."} for part in value[1:].split("/")):
        raise ValueError("Root must have no empty, current-directory or parent-directory components.")
    return PurePosixPath(value)


def release_argument(value: str) -> str:
    if not re.fullmatch(r"[0-9]+\.[0-9]+\.[0-9]+", value):
        raise ValueError("Oxide release must be a stable numeric version such as 2.0.7801.")
    return value


def plan(root: Path, release: str) -> dict:
    return {
        "state": "plan-only-unverified",
        "root": str(root),
        "oxide_release": release,
        "steps": [
            "Require permitted nonroot Linux x86_64/glibc, available RAM/disk and new nonsymlink root outside Git.",
            "Download official SteamCMD; its measured SHA256 has no independent publisher comparison.",
            "Install anonymous Steam app 258550, public branch, without starting it.",
            "Verify official Oxide Linux release identity, archive size and publisher SHA256; overlay at server root.",
            "Record installed-unverified manifest; actual host/control/startup/client acceptance remains required.",
        ],
        "automatic_start": False,
        "automatic_packages_or_services": False,
        "network_or_filesystem_mutations_in_plan": False,
        "control_gate": "Linux console is noninteractive. Design authenticated RCON/private access on the chosen host.",
    }


def ancestors(path: Path):
    current = path
    while True:
        yield current
        parent = current.parent
        if parent == current:
            break
        current = parent


def no_symlinks(path: Path) -> None:
    for current in ancestors(path):
        if current.is_symlink():
            raise ValueError("Symlink path component refused: " + str(current))


def validate_new_root(root: Path) -> None:
    no_symlinks(root)
    if root.exists():
        raise ValueError("Existing destination refused; no update, overwrite or deletion is supported.")
    if not root.parent.is_dir() or not os.access(root.parent, os.W_OK | os.X_OK):
        raise ValueError("Root parent must already exist and be writable/searchable.")
    for parent in ancestors(root.parent):
        if (parent / ".git").exists() or (parent / ".git").is_symlink():
            raise ValueError("Installation inside a Git checkout is refused.")


def available_ram() -> int:
    text = Path("/proc/meminfo").read_text(encoding="ascii")
    match = re.search(r"^MemAvailable:\s+([0-9]+)\s+kB$", text, re.MULTILINE)
    if not match:
        raise ValueError("Cannot establish available RAM from Linux MemAvailable.")
    return int(match.group(1)) * 1024


def preflight(root: Path) -> None:
    if platform.system() != "Linux" or platform.machine().lower() not in {"x86_64", "amd64"}:
        raise ValueError("Actual installation requires Linux x86_64; use --plan on other platforms.")
    if platform.libc_ver()[0] != "glibc":
        raise ValueError("Actual installation requires a glibc Linux host with SteamCMD's 32-bit runtime dependencies.")
    if os.geteuid() == 0:
        raise ValueError("Run as a dedicated nonroot user; this installer never uses sudo or installs packages.")
    validate_new_root(root)
    if available_ram() < MIN_RAM:
        raise ValueError("At least 12 GiB available RAM is required before installation.")
    if shutil.disk_usage(root.parent).free < MIN_DISK:
        raise ValueError("At least 20 GiB free disk is required before installation.")


def select_oxide_asset(metadata: dict, release: str) -> dict:
    if not isinstance(metadata, dict) or metadata.get("tag_name") != release or \
            metadata.get("draft") is not False or metadata.get("prerelease") is not False:
        raise ValueError("Official stable release identity is missing or mismatched.")
    assets = metadata.get("assets")
    if not isinstance(assets, list):
        raise ValueError("Official release assets are missing.")
    matches = [a for a in assets if isinstance(a, dict) and a.get("name") == "Oxide.Rust-linux.zip"]
    if len(matches) != 1:
        raise ValueError("Exactly one official Linux Oxide archive is required.")
    asset = matches[0]
    expected_url = "https://github.com/OxideMod/Oxide.Rust/releases/download/" + release + "/Oxide.Rust-linux.zip"
    size = asset.get("size")
    digest = asset.get("digest")
    if asset.get("state") != "uploaded" or asset.get("browser_download_url") != expected_url or \
            type(asset.get("id")) is not int or asset["id"] <= 0 or \
            type(size) is not int or not 0 < size <= MAX_ARCHIVE or \
            not isinstance(digest, str) or not re.fullmatch(r"sha256:[0-9a-fA-F]{64}", digest):
        raise ValueError("Linux archive URL, size or publisher SHA256 is invalid.")
    return {"id": asset.get("id"), "name": asset["name"], "url": expected_url,
            "size": size, "sha256": digest[7:].lower()}


def download(url: str, destination: Path, limit: int) -> tuple[int, str]:
    request = urllib.request.Request(url, headers={"User-Agent": "mw2-rust-server-preparation"})
    measured = hashlib.sha256()
    total = 0
    with urllib.request.urlopen(request, timeout=60) as response, destination.open("xb") as output:
        while True:
            block = response.read(1024 * 1024)
            if not block:
                break
            total += len(block)
            if total > limit:
                raise ValueError("Download exceeded its allowed byte limit.")
            measured.update(block)
            output.write(block)
    if total == 0:
        raise ValueError("Empty download refused.")
    return total, measured.hexdigest()


def archive_path(name: str, directory: bool) -> PurePosixPath | None:
    while name.startswith("./"):
        name = name[2:]
    if directory:
        name = name.rstrip("/")
    if not name and directory:
        return None
    if not name or name.startswith("/") or "\\" in name or ":" in name or \
            any(ord(c) < 32 or ord(c) == 127 for c in name) or \
            any(part in {"", ".", ".."} for part in name.split("/")):
        raise ValueError("Unsafe archive path refused.")
    return PurePosixPath(name)


def checked_entries(entries: list[tuple[str, bool, bool, int]]) -> list[PurePosixPath | None]:
    if len(entries) > 10000:
        raise ValueError("Too many archive entries.")
    paths = []
    seen = set()
    total = 0
    for name, directory, regular, size in entries:
        if not directory and not regular:
            raise ValueError("Archive links and special entries are refused.")
        path = archive_path(name, directory)
        if size < 0 or size > MAX_EXPANDED:
            raise ValueError("Invalid archive entry size.")
        total += size
        if total > MAX_EXPANDED or (path is not None and path in seen):
            raise ValueError("Excessive expansion or duplicate archive path refused.")
        if path is not None:
            seen.add(path)
        paths.append(path)
    # Refuse file/directory ancestor collisions before any extraction writes.
    files = {path for path, entry in zip(paths, entries) if path is not None and not entry[1]}
    if any(parent in files for path in seen for parent in path.parents if parent != PurePosixPath(".")):
        raise ValueError("Archive file/directory path collision refused.")
    return paths


def destination(root: Path, relative: PurePosixPath, directory: bool) -> Path:
    target = root.joinpath(*relative.parts)
    no_symlinks(target)
    if target.exists() and ((directory and not target.is_dir()) or (not directory and not target.is_file())):
        raise ValueError("Archive destination is not the expected regular file/directory.")
    target.parent.mkdir(parents=True, exist_ok=True)
    return target


class LimitedReader:
    """Cap actual decompression, including PAX/long-name metadata before TarInfo exists."""
    def __init__(self, source):
        self.source = source
        self.total = 0

    def read(self, size: int) -> bytes:
        if size < 0 or size > MAX_EXPANDED:
            raise ValueError("Excessive archive read refused.")
        data = self.source.read(min(size, MAX_EXPANDED - self.total + 1))
        self.total += len(data)
        if self.total > MAX_EXPANDED:
            raise ValueError("Actual archive decompression exceeds its limit.")
        return data


def extract_steam(archive: Path, root: Path) -> None:
    entries = []
    declared = 0
    with gzip.open(archive, "rb") as raw, tarfile.open(fileobj=LimitedReader(raw), mode="r|") as source:
        for entry in source:
            entries.append(entry)
            declared += entry.size
            if len(entries) > 10000 or entry.size < 0 or declared > MAX_EXPANDED:
                raise ValueError("SteamCMD archive count/size limit exceeded.")
    paths = checked_entries([(e.name, e.isdir(), e.isreg(), e.size) for e in entries])
    files = {path: e for path, e in zip(paths, entries) if e.isreg() and e.size > 0}
    if not {PurePosixPath("steamcmd.sh"), PurePosixPath("linux32/steamcmd")}.issubset(files):
        raise ValueError("SteamCMD archive lacks expected nonempty launcher/client files.")
    # A second bounded streaming pass extracts only after complete metadata validation.
    with gzip.open(archive, "rb") as raw, tarfile.open(fileobj=LimitedReader(raw), mode="r|") as source:
        for entry, path in zip(source, paths):
            if path is None:
                continue
            target = destination(root, path, entry.isdir())
            if entry.isdir():
                target.mkdir(exist_ok=True)
            else:
                with source.extractfile(entry) as incoming, target.open("xb") as outgoing:
                    shutil.copyfileobj(incoming, outgoing)
                target.chmod(0o700 if entry.mode & 0o111 else 0o600)


def overlay_oxide(archive: Path, root: Path) -> None:
    with zipfile.ZipFile(archive) as source:
        entries = source.infolist()
        paths = checked_entries([(e.filename,
            e.is_dir() and stat.S_IFMT(e.external_attr >> 16) in {0, stat.S_IFDIR},
            not e.is_dir() and stat.S_IFMT(e.external_attr >> 16) in {0, stat.S_IFREG},
            e.file_size) for e in entries])
        required = {PurePosixPath("RustDedicated_Data/Managed/Assembly-CSharp.dll"),
                    PurePosixPath("RustDedicated_Data/Managed/Oxide.Rust.dll")}
        files = {path for path, e in zip(paths, entries) if not e.is_dir() and e.file_size > 0}
        if not required.issubset(files) or any(p is not None and p.parts[0] in PROTECTED for p in paths):
            raise ValueError("Oxide archive layout is missing framework files or touches protected installation data.")
        for entry, path in zip(entries, paths):
            if path is None:
                continue
            target = destination(root, path, entry.is_dir())
            if entry.is_dir():
                target.mkdir(exist_ok=True)
            else:
                # Intended regular-file overlay after official Steam installation, in a fresh root only.
                fd = os.open(target, os.O_WRONLY | os.O_CREAT | os.O_TRUNC | os.O_NOFOLLOW, 0o600)
                with source.open(entry) as incoming, os.fdopen(fd, "wb") as outgoing:
                    shutil.copyfileobj(incoming, outgoing)
                if (entry.external_attr >> 16) & 0o111:
                    target.chmod(0o700)
        for path in required:
            target = root.joinpath(*path.parts)
            no_symlinks(target)
            if not target.is_file() or target.stat().st_size == 0:
                raise ValueError("Expected regular framework DLL is missing after overlay.")


def run_process(arguments: list[str], cwd: Path, logfile: Path, timeout: int = 1800) -> None:
    with logfile.open("xb") as log:
        process = subprocess.Popen(arguments, cwd=cwd, stdout=log, stderr=subprocess.STDOUT, start_new_session=True)
        try:
            code = process.wait(timeout=timeout)
        except BaseException:
            try:
                # Stop the complete installer group, including bootstrap/client children.
                # No server/world has been started; partial downloads are intentionally retained.
                os.killpg(process.pid, signal.SIGKILL)
            except ProcessLookupError:
                pass
            process.wait()
            raise
        if code != 0:
            raise RuntimeError("SteamCMD failed with exit code " + str(code) + "; inspect retained log.")


def steam_build(root: Path) -> str:
    manifest = root / "steamapps/appmanifest_258550.acf"
    executable = root / "RustDedicated"
    no_symlinks(manifest)
    no_symlinks(executable)
    text = manifest.read_text(encoding="utf-8")
    app = re.findall(r'^\s*"appid"\s+"([0-9]+)"\s*$', text, re.MULTILINE)
    build = re.findall(r'^\s*"buildid"\s+"([0-9]+)"\s*$', text, re.MULTILINE)
    state = re.findall(r'^\s*"StateFlags"\s+"([0-9]+)"\s*$', text, re.MULTILINE)
    if app != ["258550"] or len(build) != 1 or state != ["4"] or \
            not executable.is_file() or executable.stat().st_size == 0 or not os.access(executable, os.X_OK):
        raise ValueError("Expected Rust executable/appmanifest identity and numeric build are missing.")
    return build[0]


def write_manifest(root: Path, manifest: dict) -> None:
    manifest["updated_utc"] = datetime.datetime.now(datetime.timezone.utc).isoformat()
    temporary = root / ".mod-bootstrap/manifest.tmp"
    no_symlinks(temporary)
    with temporary.open("x", encoding="utf-8") as output:
        json.dump(manifest, output, indent=2, allow_nan=False)
        output.write("\n")
    os.replace(temporary, root / "install-manifest.json")


def install(root: Path, release: str) -> dict:
    preflight(root)
    root.mkdir(mode=0o700)  # Atomic refusal if the destination appeared during preflight.
    staging = root / ".mod-bootstrap"
    staging.mkdir(mode=0o700)
    manifest = {"status": "preparing", "stage": "downloads", "root": str(root),
                "app_id": 258550, "branch": "public", "oxide_release": release,
                "runtime_verified": False, "world_started": False}
    try:
        write_manifest(root, manifest)
        metadata_file = staging / "oxide-release.json"
        download(RELEASE_API + release, metadata_file, 1024 * 1024)
        asset = select_oxide_asset(json.loads(metadata_file.read_text(encoding="utf-8")), release)
        oxide_file = staging / "Oxide.Rust-linux.zip"
        size, digest = download(asset["url"], oxide_file, asset["size"])
        if size != asset["size"] or digest != asset["sha256"]:
            raise ValueError("Oxide archive size or publisher SHA256 mismatch; no overlay performed.")
        manifest["oxide_asset"] = asset
        manifest["oxide_sha256_verified"] = True
        steam_file = staging / "steamcmd_linux.tar.gz"
        size, digest = download(STEAM_URL, steam_file, MAX_ARCHIVE)
        manifest["steamcmd_archive"] = {"url": STEAM_URL, "size": size, "measured_sha256": digest,
                                        "independent_publisher_digest_verified": False}
        steam_root = staging / "steamcmd"
        steam_root.mkdir(mode=0o700)
        extract_steam(steam_file, steam_root)
        manifest["stage"] = "steam-install"
        write_manifest(root, manifest)
        run_process([str(steam_root / "steamcmd.sh"), "+@ShutdownOnFailedCommand", "1",
                     "+@NoPromptForPassword", "1", "+force_install_dir", str(root), "+login", "anonymous",
                     "+app_update", "258550", "-beta", "public", "validate", "+quit"],
                    steam_root, staging / "steamcmd-install.log")
        manifest["steam_build"] = steam_build(root)
        manifest["stage"] = "oxide-overlay"
        write_manifest(root, manifest)
        overlay_oxide(oxide_file, root)
        manifest.update(status="installed-unverified", stage="complete")
        write_manifest(root, manifest)
        return manifest
    except BaseException as error:
        manifest.update(status="failed", error=type(error).__name__ + ": " + str(error))
        try:
            write_manifest(root, manifest)
        except (OSError, ValueError) as manifest_error:
            print("Failed to record failure manifest: " + str(manifest_error), file=sys.stderr)
        raise


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", required=True, type=root_argument)
    parser.add_argument("--oxide-release", default=DEFAULT_RELEASE, type=release_argument)
    parser.add_argument("--plan", action="store_true", help="Print a portable plan without network or file changes.")
    args = parser.parse_args(argv)
    previous = {}
    def interrupted(signum, frame):
        raise InterruptedError("Installation interrupted by signal " + str(signum) + ".")
    try:
        if not args.plan and platform.system() == "Linux":
            for signum in (signal.SIGTERM, signal.SIGHUP):
                previous[signum] = signal.signal(signum, interrupted)
        result = plan(args.root, args.oxide_release) if args.plan else install(Path(args.root), args.oxide_release)
        print(json.dumps(result, indent=2, allow_nan=False))
        return 0
    except (OSError, ValueError, RuntimeError, subprocess.SubprocessError, tarfile.TarError, zipfile.BadZipFile) as error:
        print("Installation refused/failed; partial installation is retained if created. " + str(error), file=sys.stderr)
        return 1
    except KeyboardInterrupt:
        print("Installation interrupted; partial installation is retained.", file=sys.stderr)
        return 130
    finally:
        for signum, handler in previous.items():
            signal.signal(signum, handler)


if __name__ == "__main__":
    sys.exit(main())
