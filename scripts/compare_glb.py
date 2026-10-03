#!/usr/bin/env python3
"""Compare regenerated GLB files with the committed ones and say where they
differ.

    scripts/compare_glb.py <committed-dir> <regenerated-dir>

For every .glb in <committed-dir> it prints one Markdown table row: whether
the files are identical, and if not, whether the JSON chunk, the binary chunk
or both differ, how many bytes of the binary chunk differ and the largest
difference between float32 values at the same offset. It also says whether
manifest.json matches. Exit status is 0 only if everything is identical.
"""

import json
import struct
import sys
from pathlib import Path


def chunks(data):
    if data[:4] != b"glTF":
        raise ValueError("not a GLB file")
    (length,) = struct.unpack_from("<I", data, 8)
    found = {}
    at = 12
    while at < length:
        size, kind = struct.unpack_from("<I4s", data, at)
        found[kind] = data[at + 8:at + 8 + size]
        at += 8 + size
    return json.loads(found[b"JSON"]), found.get(b"BIN\0", b"")


def float_delta(a, b):
    count = min(len(a), len(b)) // 4
    worst = 0.0
    for x, y in zip(struct.iter_unpack("<f", a[:count * 4]), struct.iter_unpack("<f", b[:count * 4])):
        d = abs(x[0] - y[0])
        if d == d and d > worst:
            worst = d
    return worst


def json_paths(a, b, path=""):
    if type(a) is not type(b):
        return [path or "/"]
    if isinstance(a, dict):
        out = []
        for key in sorted(set(a) | set(b)):
            if key not in a or key not in b:
                out.append(f"{path}/{key}")
            else:
                out += json_paths(a[key], b[key], f"{path}/{key}")
        return out
    if isinstance(a, list):
        if len(a) != len(b):
            return [f"{path} (length {len(a)} vs {len(b)})"]
        out = []
        for i, (x, y) in enumerate(zip(a, b)):
            out += json_paths(x, y, f"{path}/{i}")
        return out
    return [] if a == b else [path or "/"]


def compare(committed, regenerated):
    if not regenerated.is_file():
        return False, "missing"
    a, b = committed.read_bytes(), regenerated.read_bytes()
    if a == b:
        return True, "identical"
    ja, ba = chunks(a)
    jb, bb = chunks(b)
    notes = [f"{len(a)} vs {len(b)} bytes"]
    paths = json_paths(ja, jb)
    if paths:
        shown = ", ".join(f"`{p}`" for p in paths[:3])
        more = f" (+{len(paths) - 3})" if len(paths) > 3 else ""
        notes.append(f"JSON differs at {shown}{more}")
    if ba != bb:
        changed = sum(x != y for x, y in zip(ba, bb)) + abs(len(ba) - len(bb))
        notes.append(f"BIN differs in {changed} bytes, max float32 delta {float_delta(ba, bb):.3g}")
    return False, "; ".join(notes)


def main():
    if len(sys.argv) != 3:
        sys.exit("usage: compare_glb.py <committed-dir> <regenerated-dir>")
    committed, regenerated = Path(sys.argv[1]), Path(sys.argv[2])
    ok = True
    print("| file | result |")
    print("|---|---|")
    for path in sorted(committed.glob("*.glb")):
        same, note = compare(path, regenerated / path.name)
        ok &= same
        print(f"| {path.name} | {note} |")
    manifest = committed / "manifest.json"
    if manifest.is_file():
        other = regenerated / "manifest.json"
        same = other.is_file() and manifest.read_bytes() == other.read_bytes()
        ok &= same
        print(f"| manifest.json | {'identical' if same else 'differs'} |")
    sys.exit(0 if ok else 1)


if __name__ == "__main__":
    main()
