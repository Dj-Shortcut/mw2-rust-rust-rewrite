#!/usr/bin/env python3
"""Make clippy blocking one package at a time.

    cargo clippy --workspace --all-targets --message-format=json \\
        -- --cap-lints warn | scripts/clippy_gate.py

Reads cargo's JSON messages on stdin, prints every finding as cargo would,
then a per-package count. Exits 1 if any package named in
scripts/clippy_clean.txt has a finding; every other package is report-only.

A package goes on the list once it has no findings, so it cannot regress.
Fixing a package's findings changes existing code, so do that in its own
reviewed change, not to get a package onto this list.
"""

import json
import subprocess
import sys
from collections import Counter
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
CLEAN = ROOT / "scripts" / "clippy_clean.txt"


def workspace_packages():
    metadata = json.loads(subprocess.run(
        ["cargo", "metadata", "--no-deps", "--format-version", "1", "--locked"],
        cwd=ROOT, capture_output=True, text=True, check=True,
    ).stdout)
    return {p["manifest_path"]: p["name"] for p in metadata["packages"]}


def package_name(package_id):
    # path+file:///repo/skate/crates/skate-host#0.1.0, or ...#name@0.1.0
    path, _, version = package_id.partition("#")
    return version.split("@")[0] if "@" in version else path.rstrip("/").rsplit("/", 1)[-1]


def clean_list(known):
    names = []
    for line in CLEAN.read_text(encoding="utf-8").splitlines():
        name = line.split("#", 1)[0].strip()
        if not name:
            continue
        if name not in known:
            sys.exit(f"clippy_gate: {CLEAN.name} names unknown package {name}")
        names.append(name)
    return names


def main():
    packages = workspace_packages()
    clean = clean_list(set(packages.values()))
    findings = Counter()
    seen = set()
    for line in sys.stdin:
        try:
            message = json.loads(line)
        except json.JSONDecodeError:
            continue
        if message.get("reason") != "compiler-message":
            continue
        diagnostic = message["message"]
        if diagnostic["level"] not in ("warning", "error") or not diagnostic.get("spans"):
            continue
        rendered = diagnostic.get("rendered") or diagnostic["message"]
        package = packages.get(message.get("manifest_path")) or package_name(message["package_id"])
        # Lib and test targets of one package report the same finding twice.
        if (package, rendered) in seen:
            continue
        seen.add((package, rendered))
        findings[package] += 1
        print(rendered, end="" if rendered.endswith("\n") else "\n")

    print(f"clippy: {sum(findings.values())} findings in {len(findings)} packages")
    for package, count in sorted(findings.items(), key=lambda item: (-item[1], item[0])):
        print(f"  {count:5} {package}")
    regressed = [name for name in clean if findings[name]]
    print(f"blocking for {len(clean)} packages: {', '.join(clean)}")
    if regressed:
        print(f"clippy_gate: findings in packages that must stay clean: {', '.join(regressed)}")
        sys.exit(1)


if __name__ == "__main__":
    main()
