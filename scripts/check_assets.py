#!/usr/bin/env python3
"""Refuse tracked files that look like original game data or unvetted models.

Complements `make publish-check`: that gate reads file contents for offsets and
leaks; this one only looks at paths and, for models, at the authored manifest.
It never reads source text, so colour codes, masks and constants are not its
concern.

Rules:
  1. Game-install and engine archive formats are refused anywhere.
  2. 3D model and scene files may only live under assets/authored/.
  3. Each .glb there must be listed in manifest.json with a matching SHA-256.
  4. Each .blend there must be the editable source of a listed .glb.

Passing this proves nothing about provenance; it only catches accidents.
"""

import hashlib
import json
import subprocess
import sys
from pathlib import Path, PurePosixPath

AUTHORED = PurePosixPath("assets/authored")
MANIFEST = AUTHORED / "manifest.json"

GAME_DATA = {
    # IW engine install and extracted asset formats.
    "ff", "iwd", "iwi", "d3dbsp", "iwuv", "gsc", "csc", "ipak", "xpak",
    "sabs", "sabl", "xmodel", "xanim", "xmodelparts", "xmodelsurfs",
    # Common game archive, video and sound bank formats.
    "pak", "vpk", "bik", "fsb", "bnk", "wem",
}

MODELS = {
    "glb", "gltf", "blend", "fbx", "obj", "dae", "3ds", "max", "ma", "mb",
    "x", "md5mesh", "md5anim",
}

ALLOWED_AUTHORED_MODELS = {"glb", "blend"}


def tracked(root: Path) -> list[PurePosixPath]:
    out = subprocess.run(
        ["git", "ls-files", "-z"], cwd=root, check=True, capture_output=True
    ).stdout
    return [PurePosixPath(p) for p in out.decode().split("\0") if p]


def ext(path: PurePosixPath) -> str:
    return path.suffix.lower().lstrip(".")


def sha256(path: Path) -> str:
    h = hashlib.sha256()
    with path.open("rb") as f:
        for chunk in iter(lambda: f.read(1 << 20), b""):
            h.update(chunk)
    return h.hexdigest()


def load_manifest(root: Path, problems: list[str]) -> dict[str, str]:
    path = root / MANIFEST
    if not path.is_file():
        return {}
    try:
        data = json.loads(path.read_text())
        return {a["file"]: a["sha256"] for a in data["assets"]}
    except (ValueError, KeyError, TypeError) as err:
        problems.append(f"{MANIFEST}: unreadable manifest ({err})")
        return {}


def main() -> int:
    root = Path(
        subprocess.run(
            ["git", "rev-parse", "--show-toplevel"],
            check=True,
            capture_output=True,
            text=True,
        ).stdout.strip()
    )
    problems: list[str] = []
    manifest = load_manifest(root, problems)
    models = 0

    for rel in tracked(root):
        e = ext(rel)
        if e in GAME_DATA:
            problems.append(f"{rel}: original game data format (.{e})")
            continue
        if e not in MODELS:
            continue
        models += 1
        if rel.parent != AUTHORED:
            problems.append(f"{rel}: models belong directly under {AUTHORED}/")
            continue
        if e not in ALLOWED_AUTHORED_MODELS:
            problems.append(f"{rel}: only .glb and .blend are accepted in {AUTHORED}/")
            continue
        if not (root / rel).is_file():
            continue
        if e == "glb":
            want = manifest.get(rel.name)
            if want is None:
                problems.append(f"{rel}: not listed in {MANIFEST}")
            elif sha256(root / rel) != want:
                problems.append(f"{rel}: SHA-256 differs from {MANIFEST}")
        elif rel.with_suffix(".glb").name not in manifest:
            problems.append(f"{rel}: no matching .glb listed in {MANIFEST}")

    if problems:
        print("asset check failed:")
        for p in problems:
            print(f"  {p}")
        return 1
    print(f"asset check passed ({models} authored model files)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
