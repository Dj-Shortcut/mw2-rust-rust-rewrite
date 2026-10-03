#!/usr/bin/env python3
"""Stage an unreleased development build: the game binary, the authored
runtime content it loads and the licence files that must travel with it.

    scripts/package_dev_build.py <game-binary> <output-dir>

The folder it writes runs from anywhere: the native frontend looks for
`assets/authored/` next to the executable. It is a CI artifact for building
and inspecting, not a release. `make release` / `make launcher windows` are the
shipping path and need signing material this script never touches.

What a pass establishes: the binary exists, a Windows binary keeps the 8 MiB
main-thread stack reserve from crates/launcher/build.rs, every authored .glb
and .wav listed in its manifest is copied, and the licence files are present.
It runs nothing, so it says nothing about whether the game starts or plays.
"""

import json
import shutil
import struct
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
AUTHORED = ROOT / "assets" / "authored"

LEGAL_FILES = [
    ("LICENSE", "LICENSE"),
    ("NOTICE", "NOTICE"),
    ("crates/ui/assets/OFL-Oxanium.txt", "OFL-Oxanium.txt"),
    ("crates/console/assets/COPYING-FreeFont.txt", "COPYING-FreeFont.txt"),
]

WANT_STACK_RESERVE = 8 * 1024 * 1024


def fail(message):
    print(f"package_dev_build: {message}", file=sys.stderr)
    sys.exit(1)


def pe_stack_reserve(data):
    if data[:2] != b"MZ":
        return None
    (pe,) = struct.unpack_from("<I", data, 0x3C)
    if data[pe:pe + 4] != b"PE\0\0":
        fail("binary starts with MZ but has no PE header")
    (magic,) = struct.unpack_from("<H", data, pe + 24)
    at = pe + 24 + 72
    if magic == 0x20B:
        return struct.unpack_from("<Q", data, at)[0]
    return struct.unpack_from("<I", data, at)[0]


def copy(source, target):
    target.parent.mkdir(parents=True, exist_ok=True)
    shutil.copy2(source, target)


def manifest_files(directory):
    manifest = directory / "manifest.json"
    if not manifest.is_file():
        return []
    listed = json.loads(manifest.read_text(encoding="utf-8"))
    entries = listed.get("assets") or listed.get("sounds") or listed.get("files") or []
    names = [entry["file"] for entry in entries]
    for name in names:
        if not (directory / name).is_file():
            fail(f"{directory.relative_to(ROOT)}/manifest.json lists missing {name}")
    return ["manifest.json", *names]


def commit():
    try:
        return subprocess.run(
            ["git", "rev-parse", "HEAD"],
            cwd=ROOT, capture_output=True, text=True, check=True,
        ).stdout.strip()
    except (OSError, subprocess.CalledProcessError):
        return "unknown"


def main():
    if len(sys.argv) != 3:
        fail("usage: package_dev_build.py <game-binary> <output-dir>")
    binary = Path(sys.argv[1])
    out = Path(sys.argv[2])
    if not binary.is_file():
        fail(f"no binary at {binary}")
    if out.exists():
        fail(f"{out} already exists")

    reserve = pe_stack_reserve(binary.read_bytes())
    if reserve is not None and reserve < WANT_STACK_RESERVE:
        fail(f"main-thread stack reserve {reserve} < {WANT_STACK_RESERVE}; the /STACK link arg is gone")

    out.mkdir(parents=True)
    shutil.copy2(binary, out / binary.name)

    content = out / "assets" / "authored"
    models = manifest_files(AUTHORED)
    if not models:
        fail("assets/authored/manifest.json lists no models")
    for name in [*models, "README.md", "CC0-1.0.txt"]:
        copy(AUTHORED / name, content / name)

    sounds = manifest_files(AUTHORED / "audio")
    if sounds:
        for name in sounds:
            copy(AUTHORED / "audio" / name, content / "audio" / name)

    for source, target in LEGAL_FILES:
        if not (ROOT / source).is_file():
            fail(f"missing licence file {source}")
        shutil.copy2(ROOT / source, out / target)

    (out / "BUILD-INFO.txt").write_text(
        "Unreleased development build.\n"
        f"Commit: {commit()}\n"
        "\n"
        "Built and packaged by CI. Nobody has run this binary as part of\n"
        "producing it; it may not start, and the game is unfinished. See\n"
        "TODO.md in the repository for what is and is not verified.\n",
        encoding="utf-8",
    )

    stack = f", stack reserve {reserve >> 20} MiB" if reserve is not None else ""
    print(
        f"package_dev_build: {out}: {binary.name}{stack}, "
        f"{len(models) - 1} models, {max(len(sounds) - 1, 0)} sounds, "
        f"{len(LEGAL_FILES)} licence files"
    )


if __name__ == "__main__":
    main()
