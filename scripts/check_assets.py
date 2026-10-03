#!/usr/bin/env python3
"""Check tracked binary content against the project's authored manifests.

What a pass establishes, and nothing more:

  * no tracked path has a game-install or engine-archive extension;
  * every model file is a .glb or .blend directly under assets/authored/, every
    .glb is listed in assets/authored/manifest.json with the same SHA-256 and
    size as the committed bytes, and every .blend sits next to a listed .glb;
  * every audio file, recognised by extension or by its leading bytes, is a
    .wav under assets/authored/audio/, listed in
    assets/authored/audio/manifest.json with the same SHA-256 (and size and
    format fields where the manifest gives them), and is a structurally valid
    PCM or IEEE-float RIFF/WAVE file;
  * no model or audio path is a symlink, so every hash is of committed bytes;
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
import math
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
    "wav", "wave", "ogg", "oga", "opus", "mp3", "mp2", "flac", "aif", "aiff",
    "aifc", "wma", "m4a", "aac", "au", "snd", "caf", "mka", "ac3", "amr",
    "ape", "wv", "mid", "midi", "xm", "mod", "s3m", "it", "voc", "w64", "rf64",
}

# Leading bytes of audio containers, so a renamed sound is still caught.
AUDIO_MAGIC = (
    (0, b"RIFF", 8, b"WAVE"),
    (0, b"RF64", 8, b"WAVE"),
    (0, b"FORM", 8, b"AIFF"),
    (0, b"FORM", 8, b"AIFC"),
    (0, b"OggS", None, None),
    (0, b"fLaC", None, None),
    (0, b"ID3", None, None),
    (0, b".snd", None, None),
    (0, b"caff", None, None),
    (0, b"MThd", None, None),
    (0, b"#!AMR", None, None),
    (0, b"MAC ", None, None),
    (0, b"wvpk", None, None),
)

PCM, FLOAT, EXTENSIBLE = 1, 3, 0xFFFE
# KSDATAFORMAT_SUBTYPE_PCM / _IEEE_FLOAT share this GUID tail after the tag.
SUBFORMAT_TAIL = bytes.fromhex("000000001000800000aa00389b71")
VALID_BITS = {PCM: {8, 16, 24, 32}, FLOAT: {32, 64}}


def tracked(root: Path) -> list[tuple[PurePosixPath, bool]]:
    """Tracked paths with whether git stores each as a symlink."""
    out = subprocess.run(
        ["git", "ls-files", "-s", "-z"], cwd=root, check=True, capture_output=True
    ).stdout
    files = []
    for record in out.decode().split("\0"):
        if not record:
            continue
        meta, path = record.split("\t", 1)
        files.append((PurePosixPath(path), meta.split()[0] == "120000"))
    return files


def ext(path: PurePosixPath) -> str:
    return path.suffix.lower().lstrip(".")


def sniff_audio(path: Path) -> bool:
    with path.open("rb") as f:
        head = f.read(12)
    return any(
        head[a:a + len(m1)] == m1 and (m2 is None or head[b:b + len(m2)] == m2)
        for a, m1, b, m2 in AUDIO_MAGIC
    )


def sha256(path: Path) -> str:
    h = hashlib.sha256()
    with path.open("rb") as f:
        for chunk in iter(lambda: f.read(1 << 20), b""):
            h.update(chunk)
    return h.hexdigest()


def load_manifest(
    root: Path, rel: PurePosixPath, keys: tuple[str, ...], problems: list[str]
) -> dict[str, dict]:
    """Entries keyed by file path relative to the manifest's directory.

    The entry list must sit under exactly one of `keys`; each entry needs
    "file" and "sha256".
    """
    path = root / rel
    if not path.is_file():
        return {}
    try:
        data = json.loads(path.read_text())
    except ValueError as err:
        problems.append(f"{rel}: not valid JSON ({err})")
        return {}
    lists = [data.get(k) for k in keys] if isinstance(data, dict) else []
    lists = [entries for entries in lists if isinstance(entries, list)]
    if len(lists) != 1:
        names = ", ".join(f'"{k}"' for k in keys)
        problems.append(f"{rel}: expected exactly one entry list under {names}")
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
    """Parse the RIFF/WAVE framing and fmt chunk. Returns (fields, error)."""
    data = path.read_bytes()
    if len(data) < 12 or data[:4] != b"RIFF" or data[8:12] != b"WAVE":
        return None, "not a RIFF/WAVE file"
    riff_size = struct.unpack_from("<I", data, 4)[0]
    if riff_size + 8 != len(data):
        return None, f"RIFF size {riff_size + 8} differs from file size {len(data)}"
    fmt = None
    data_bytes = None
    pos = 12
    while pos < len(data):
        if pos + 8 > len(data):
            return None, f"{len(data) - pos} trailing bytes after the last chunk"
        cid, size = data[pos:pos + 4], struct.unpack_from("<I", data, pos + 4)[0]
        body = pos + 8
        end = body + size + (size & 1)
        if end > len(data):
            return None, f"chunk {cid!r} (with padding) runs past end of file"
        if cid == b"fmt ":
            if fmt is not None:
                return None, "more than one fmt chunk"
            fmt = data[body:body + size]
        elif cid == b"data":
            if data_bytes is not None:
                return None, "more than one data chunk"
            data_bytes = size
        pos = end
    if fmt is None:
        return None, "no fmt chunk"
    if data_bytes is None:
        return None, "no data chunk"
    if len(fmt) < 16:
        return None, "fmt chunk too short"
    tag, channels, rate, byte_rate, align, bits = struct.unpack_from("<HHIIHH", fmt)
    codec = tag
    if tag == EXTENSIBLE:
        if len(fmt) < 40:
            return None, "extensible fmt chunk shorter than 40 bytes"
        cb_size, valid_bits = struct.unpack_from("<HH", fmt, 16)
        if cb_size < 22:
            return None, f"extensible fmt extension size {cb_size}, expected 22"
        sub = fmt[24:40]
        codec = struct.unpack_from("<H", sub)[0]
        if sub[2:] != SUBFORMAT_TAIL or codec not in (PCM, FLOAT):
            return None, "extensible subformat is neither PCM nor IEEE float"
        if not 0 < valid_bits <= bits:
            return None, f"extensible valid bits {valid_bits} outside 1..{bits}"
    elif tag not in (PCM, FLOAT):
        return None, f"unsupported format tag {tag:#06x} (expected PCM or IEEE float)"
    if channels == 0 or rate == 0:
        return None, "zero channels or sample rate"
    if bits not in VALID_BITS[codec]:
        allowed = "/".join(map(str, sorted(VALID_BITS[codec])))
        return None, f"{bits}-bit samples invalid for this format (expected {allowed})"
    if align != channels * bits // 8:
        return None, f"block align {align} inconsistent with {channels} ch x {bits} bit"
    if byte_rate != rate * align:
        return None, f"byte rate {byte_rate} differs from sample rate x block align {rate * align}"
    if data_bytes % align:
        return None, "data chunk is not a whole number of frames"
    frames = data_bytes // align
    return {
        "format": "pcm" if codec == PCM else "float",
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
        value = entry["duration_seconds"]
        ok = isinstance(value, (int, float)) and not isinstance(value, bool) \
            and math.isfinite(value) and abs(value - info["duration_seconds"]) <= 1e-3
        if not ok:
            problems.append(
                f"{rel}: manifest duration_seconds={value!r}, "
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
    models = load_manifest(root, MODEL_MANIFEST, ("assets",), problems)
    sounds = load_manifest(root, AUDIO_MANIFEST, ("assets", "sounds", "files"), problems)
    seen_models: set[str] = set()
    seen_sounds: set[str] = set()
    counts = {"model": 0, "audio": 0}

    for rel, is_link in tracked(root):
        e = ext(rel)
        if e in GAME_DATA:
            problems.append(f"{rel}: original game data format (.{e})")
            continue
        path = root / rel
        if is_link:
            if e in MODELS or e in AUDIO_FORMATS or AUTHORED in rel.parents:
                problems.append(f"{rel}: symlink; models and audio must be committed files")
            continue
        if not path.is_file():
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
                    check_entry(rel, path, entry, MODEL_MANIFEST, problems)
            elif rel.with_suffix(".glb").name not in models:
                problems.append(f"{rel}: no matching .glb listed in {MODEL_MANIFEST}")

        elif e in AUDIO_FORMATS or sniff_audio(path):
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
            check_entry(rel, path, entry, AUDIO_MANIFEST, problems)
            info, err = read_wave(path)
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
