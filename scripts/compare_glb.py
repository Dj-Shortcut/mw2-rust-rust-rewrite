#!/usr/bin/env python3
"""Compare regenerated GLB files with the committed ones and say where they
differ.

    scripts/compare_glb.py <committed-dir> <regenerated-dir>

Each .glb in <committed-dir> gets one Markdown table row:

  identical   the files are byte for byte the same
  equivalent  same glTF structure, same integer data (joints; triangle index
              lists may list the same triangles in another order), and
              every float, in the JSON or in a float accessor, within
              TOLERANCE (scaled by magnitude above 1) of the committed one
  differs     anything else, with where it differs

manifest.json is compared the same way, ignoring its per-file sha256 and
byte counts, which change whenever a GLB is equivalent but not identical.
A .glb that was regenerated but is not committed also counts as a
difference. Exit status is 0 if every file is identical or equivalent.

Blender's float math is not bit-stable across CPUs, so a model regenerated on
another machine can land one float32 step away from the committed one and
emit its triangles in another order; that is what "equivalent" allows for,
and nothing larger.
"""

import json
import struct
import sys
from pathlib import Path

TOLERANCE = 1e-6

COMPONENTS = {5120: "b", 5121: "B", 5122: "h", 5123: "H", 5125: "I", 5126: "f"}
WIDTH = {"SCALAR": 1, "VEC2": 2, "VEC3": 3, "VEC4": 4, "MAT2": 4, "MAT3": 9, "MAT4": 16}


def close(a, b):
    return abs(a - b) <= TOLERANCE * max(1.0, abs(a), abs(b))


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


def json_diff(a, b, path="", ignore=()):
    """Paths where two JSON values differ beyond TOLERANCE, and the largest
    float difference seen."""
    number = (int, float)
    if isinstance(a, number) and isinstance(b, number) and not isinstance(a, bool) and not isinstance(b, bool):
        delta = abs(a - b)
        return ([] if close(a, b) else [path or "/"]), delta
    if type(a) is not type(b):
        return [path or "/"], 0.0
    if isinstance(a, dict):
        paths, worst = [], 0.0
        for key in sorted(set(a) | set(b)):
            if key in ignore:
                continue
            if key not in a or key not in b:
                paths.append(f"{path}/{key}")
                continue
            p, d = json_diff(a[key], b[key], f"{path}/{key}", ignore)
            paths += p
            worst = max(worst, d)
        return paths, worst
    if isinstance(a, list):
        if len(a) != len(b):
            return [f"{path} (length {len(a)} vs {len(b)})"], 0.0
        paths, worst = [], 0.0
        for i, (x, y) in enumerate(zip(a, b)):
            p, d = json_diff(x, y, f"{path}/{i}", ignore)
            paths += p
            worst = max(worst, d)
        return paths, worst
    return ([] if a == b else [path or "/"]), 0.0


def accessor_values(gltf, binary, accessor):
    view = gltf["bufferViews"][accessor["bufferView"]]
    code = COMPONENTS[accessor["componentType"]]
    size = struct.calcsize(code)
    width = WIDTH[accessor["type"]]
    stride = view.get("byteStride", size * width)
    start = view.get("byteOffset", 0) + accessor.get("byteOffset", 0)
    values = []
    for i in range(accessor["count"]):
        values += struct.unpack_from(f"<{width}{code}", binary, start + i * stride)
    return code, values


def binary_diff(gltf, a, b):
    """Accessors whose data differ beyond TOLERANCE, the largest float
    difference seen, and index accessors that hold the same triangles in a
    different order. The JSON must already match structurally."""
    bad, worst, reordered = [], 0.0, []
    triangle_lists = triangle_index_accessors(gltf)
    for index, accessor in enumerate(gltf["accessors"]):
        if "bufferView" not in accessor or "sparse" in accessor:
            if "sparse" in accessor:
                bad.append(f"accessor {index} (sparse, not compared)")
            continue
        code, x = accessor_values(gltf, a, accessor)
        _, y = accessor_values(gltf, b, accessor)
        if code == "f":
            deltas = [abs(p - q) for p, q in zip(x, y)]
            worst = max([worst, *deltas])
            if not all(close(p, q) for p, q in zip(x, y)):
                bad.append(f"accessor {index}")
        elif x != y:
            if index in triangle_lists and same_triangles(x, y):
                reordered.append(index)
            else:
                bad.append(f"accessor {index} (integer data)")
    return bad, worst, reordered


def triangle_index_accessors(gltf):
    """Accessors used only as index lists of triangle primitives."""
    found = set()
    for mesh in gltf.get("meshes", []):
        for primitive in mesh["primitives"]:
            if "indices" in primitive and primitive.get("mode", 4) == 4:
                found.add(primitive["indices"])
    return found


def same_triangles(x, y):
    """Whether two index lists hold the same triangles, with the same
    winding, in any order."""
    def canonical(values):
        triangles = []
        for i in range(0, len(values), 3):
            t = values[i:i + 3]
            k = t.index(min(t))
            triangles.append(tuple(t[k:] + t[:k]))
        return sorted(triangles)
    return len(x) == len(y) and len(x) % 3 == 0 and canonical(x) == canonical(y)


def listed(paths):
    shown = ", ".join(f"`{p}`" for p in paths[:3])
    return shown + (f" (+{len(paths) - 3})" if len(paths) > 3 else "")


def compare_glb(committed, regenerated):
    if not regenerated.is_file():
        return False, "differs: missing"
    a, b = committed.read_bytes(), regenerated.read_bytes()
    if a == b:
        return True, "identical"
    ja, ba = chunks(a)
    jb, bb = chunks(b)
    paths, worst = json_diff(ja, jb)
    if paths:
        return False, f"differs: JSON at {listed(paths)}"
    if len(ba) != len(bb):
        return False, f"differs: binary chunk {len(ba)} vs {len(bb)} bytes"
    bad, bin_worst, reordered = binary_diff(ja, ba, bb)
    if bad:
        return False, f"differs: {listed(bad)}"
    order = f", triangle order differs in {len(reordered)} index lists" if reordered else ""
    return True, f"equivalent (max float delta {max(worst, bin_worst):.3g}{order})"


def compare_manifest(committed, regenerated):
    if not regenerated.is_file():
        return False, "differs: missing"
    if committed.read_bytes() == regenerated.read_bytes():
        return True, "identical"
    a = json.loads(committed.read_text(encoding="utf-8"))
    b = json.loads(regenerated.read_text(encoding="utf-8"))
    paths, worst = json_diff(a, b, ignore=("sha256", "bytes"))
    if paths:
        return False, f"differs at {listed(paths)}"
    return True, f"equivalent apart from sha256/bytes (max float delta {worst:.3g})"


def main():
    if len(sys.argv) != 3:
        sys.exit("usage: compare_glb.py <committed-dir> <regenerated-dir>")
    committed, regenerated = Path(sys.argv[1]), Path(sys.argv[2])
    ok = True
    print("| file | result |")
    print("|---|---|")
    for path in sorted(committed.glob("*.glb")):
        same, note = compare_glb(path, regenerated / path.name)
        ok &= same
        print(f"| {path.name} | {note} |")
    for path in sorted(regenerated.glob("*.glb")):
        if not (committed / path.name).is_file():
            ok = False
            print(f"| {path.name} | differs: generated but not committed |")
    if (committed / "manifest.json").is_file():
        same, note = compare_manifest(committed / "manifest.json", regenerated / "manifest.json")
        ok &= same
        print(f"| manifest.json | {note} |")
    sys.exit(0 if ok else 1)


if __name__ == "__main__":
    main()
