#!/usr/bin/env python3
"""Check tracked binary content against the project's authored manifests.

What a pass establishes, and nothing more:

  * no tracked path has a game-install or engine-archive extension;
  * every model file is a .glb or .blend directly under assets/authored/, every
    .glb is listed in assets/authored/manifest.json with the same SHA-256 and
    size as the committed bytes, and every .blend sits next to a listed .glb;
  * every audio file is a .wav under assets/authored/audio/, listed in
    assets/authored/audio/manifest.json with the same SHA-256 (and size and
    format fields where the manifest gives them), and is a structurally valid
    RIFF/WAVE file;
  * nothing a manifest lists is missing from the tree.

What it does not establish: provenance, authorship or licence. The manifests
and the files are committed by the same hands, so a matching hash says only
that the file is the one its manifest describes and has not drifted since; it
says nothing about where either came from. An extension says nothing about
content. That question is answered by the generators and by review, not here.

It reads no source text, so constants, colour codes and masks are not its
concern; `make publish-check` covers source contents.
"""

import hashlib
import json
import struct
import subprocess
import sys
from pathlib import Path, PurePosixPath

AUTHORED = PurePosixPath("assets/authored")
MODEL_MANIFEST = AUTHORED / "manifest.json"
AUDIO = AUTHORED / "audio"
AUDIO_MANIFEST = AUDIO / "manifest.json"

GAME_DATA = {
    # IW engine install and extracted asset formats.
    "ff", "iwd", "iwi", "d3dbsp", "iwuv", "gsc", "csc", "ipak", "xpak",
    "sabs", "sabl", "xmodel", "xanim", "xmodelparts", "xmodelsurfs",
    # Common game archive, video and sound bank formats.
    "pak", "vpk", "bik", "fsb", "bnk", "wem", "xwm", "xwma",
}

MODELS = {
    "glb", "gltf", "blend", "fbx", "obj", "dae", "3ds", "max", "ma", "mb",
    "x", "md5mesh", "md5anim",
}
ALLOWED_MODELS = {"glb", "blend"}

AUDIO_FORMATS = {
    "wav", "wave", "ogg", "oga", "opus", "mp3", "flac", "aif", "aiff", "aifc",
    "wma", "m4a", "aac", "mp2",
}

WAVE_FORMATS = {1: "pcm", 3: "float", 0xFFFE: "extensible"}


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


def load_manifest(root: Path, rel: PurePosixPath, problems: list[str]) -> dict[str, dict]:
    """Entries keyed by file path relative to the manifest's directory.

    Accepts the entry list under "assets", "sounds" or "files"; each entry
    needs "file" and "sha256".
    """
    path = root / rel
    if not path.is_file():
        return {}
    try:
        data = json.loads(path.read_text())
    except ValueError as err:
        problems.append(f"{rel}: not valid JSON ({err})")
        return {}
    lists = [data.get(k) for k in ("assets", "sounds", "files") if isinstance(data, dict)]
    lists = [entries for entries in lists if isinstance(entries, list)]
    if len(lists) != 1:
        problems.append(f'{rel}: expected exactly one entry list ("assets", "sounds" or "files")')
        return {}
    entries: dict[str, dict] = {}
    for i, entry in enumerate(lists[0]):
        if not isinstance(entry, dict) or not isinstance(entry.get("file"), str) \
                or not isinstance(entry.get("sha256"), str):
            problems.append(f'{rel}: entry {i} needs string "file" and "sha256"')
            continue
        name = entry["file"]
        if name.startswith("/") or ".." in PurePosixPath(name).parts:
            problems.append(f"{rel}: entry {i} file {name!r} escapes the manifest directory")
            continue
        if name in entries:
            problems.append(f"{rel}: {name} listed twice")
        entries[name] = entry
    return entries


def read_wave(path: Path) -> tuple[dict | None, str | None]:
    """Parse the RIFF/WAVE header. Returns (format fields, error)."""
    data = path.read_bytes()
    if len(data) < 12 or data[:4] != b"RIFF" or data[8:12] != b"WAVE":
        return None, "not a RIFF/WAVE file"
    riff_size = struct.unpack_from("<I", data, 4)[0]
    if riff_size + 8 != len(data):
        return None, f"RIFF size {riff_size + 8} differs from file size {len(data)}"
    fmt = None
    data_bytes = None
    pos = 12
    while pos + 8 <= len(data):
        cid, size = data[pos:pos + 4], struct.unpack_from("<I", data, pos + 4)[0]
        body = pos + 8
        if body + size > len(data):
            return None, f"chunk {cid!r} runs past end of file"
        if cid == b"fmt ":
            if size < 16:
                return None, "fmt chunk too short"
            tag, channels, rate, _, align, bits = struct.unpack_from("<HHIIHH", data, body)
            fmt = (tag, channels, rate, align, bits)
        elif cid == b"data":
            data_bytes = size
        pos = body + size + (size & 1)
    if fmt is None:
        return None, "no fmt chunk"
    if data_bytes is None:
        return None, "no data chunk"
    tag, channels, rate, align, bits = fmt
    if tag not in WAVE_FORMATS:
        return None, f"unsupported format tag {tag:#06x} (expected PCM or IEEE float)"
    if channels == 0 or rate == 0 or bits == 0 or align == 0:
        return None, "zero channels, sample rate, bit depth or block align"
    if align != channels * ((bits + 7) // 8):
        return None, f"block align {align} inconsistent with {channels} ch x {bits} bit"
    if data_bytes % align:
        return None, "data chunk is not a whole number of frames"
    frames = data_bytes // align
    return {
        "format": WAVE_FORMATS[tag],
        "channels": channels,
        "sample_rate": rate,
        "bits_per_sample": bits,
        "frames": frames,
        "duration_seconds": frames / rate,
    }, None


def check_entry(rel: PurePosixPath, path: Path, entry: dict, manifest: PurePosixPath,
                problems: list[str]) -> None:
    if sha256(path) != entry["sha256"].lower():
        problems.append(f"{rel}: SHA-256 differs from {manifest}")
    if "bytes" in entry and entry["bytes"] != path.stat().st_size:
        problems.append(f"{rel}: size differs from {manifest}")


def check_wave_fields(rel: PurePosixPath, info: dict, entry: dict, problems: list[str]) -> None:
    for key in ("format", "channels", "sample_rate", "bits_per_sample", "frames"):
        if key in entry and entry[key] != info[key]:
            problems.append(f"{rel}: manifest {key}={entry[key]!r}, file has {info[key]!r}")
    if "duration_seconds" in entry:
        try:
            off = abs(float(entry["duration_seconds"]) - info["duration_seconds"])
        except (TypeError, ValueError):
            off = float("inf")
        if off > 1e-3:
            problems.append(
                f"{rel}: manifest duration_seconds={entry['duration_seconds']!r}, "
                f"file has {info['duration_seconds']:.6f}"
            )


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
    models = load_manifest(root, MODEL_MANIFEST, problems)
    sounds = load_manifest(root, AUDIO_MANIFEST, problems)
    files = tracked(root)
    seen_models: set[str] = set()
    seen_sounds: set[str] = set()
    counts = {"model": 0, "audio": 0}

    for rel in files:
        e = ext(rel)
        if e in GAME_DATA:
            problems.append(f"{rel}: original game data format (.{e})")
            continue
        if not (root / rel).is_file():
            continue

        if e in MODELS:
            counts["model"] += 1
            if rel.parent != AUTHORED:
                problems.append(f"{rel}: models belong directly under {AUTHORED}/")
            elif e not in ALLOWED_MODELS:
                problems.append(f"{rel}: only .glb and .blend are accepted in {AUTHORED}/")
            elif e == "glb":
                entry = models.get(rel.name)
                if entry is None:
                    problems.append(f"{rel}: not listed in {MODEL_MANIFEST}")
                else:
                    seen_models.add(rel.name)
                    check_entry(rel, root / rel, entry, MODEL_MANIFEST, problems)
            elif rel.with_suffix(".glb").name not in models:
                problems.append(f"{rel}: no matching .glb listed in {MODEL_MANIFEST}")

        elif e in AUDIO_FORMATS:
            counts["audio"] += 1
            if AUDIO not in rel.parents:
                problems.append(f"{rel}: audio belongs under {AUDIO}/")
                continue
            if e != "wav":
                problems.append(f"{rel}: only .wav is accepted in {AUDIO}/")
                continue
            key = str(rel.relative_to(AUDIO))
            entry = sounds.get(key)
            if entry is None:
                problems.append(f"{rel}: not listed in {AUDIO_MANIFEST}")
                continue
            seen_sounds.add(key)
            check_entry(rel, root / rel, entry, AUDIO_MANIFEST, problems)
            info, err = read_wave(root / rel)
            if err:
                problems.append(f"{rel}: {err}")
            else:
                check_wave_fields(rel, info, entry, problems)

    for name in sorted(set(models) - seen_models):
        problems.append(f"{MODEL_MANIFEST}: lists {name}, which is not a tracked .glb beside it")
    for name in sorted(set(sounds) - seen_sounds):
        problems.append(f"{AUDIO_MANIFEST}: lists {name}, which is not a tracked .wav under {AUDIO}/")

    if problems:
        print("asset check failed:")
        for p in problems:
            print(f"  {p}")
        return 1
    audio_note = "" if (root / AUDIO_MANIFEST).is_file() else f"; no {AUDIO_MANIFEST} yet"
    print(
        f"asset check passed ({counts['model']} model files, "
        f"{counts['audio']} audio files{audio_note})"
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
