#!/usr/bin/env bash
# rustfmt --check for the .rs files changed since a base revision.
#
# The project formats only the files it touches, so a whole-tree
# `cargo fmt --check` would fail on code nobody edited. rustfmt also descends
# into a file's child modules, so its findings are filtered back down to the
# changed files. skate/ and third_party/ sit outside the workspace and keep
# their upstream formatting.
#
# usage: scripts/check_fmt_changed.sh <base-rev>
set -euo pipefail

base=${1:?usage: $0 <base-rev>}
root=$(git rev-parse --show-toplevel)
cd "$root"

mapfile -t changed < <(
  git diff --name-only --diff-filter=ACMR "$base"...HEAD -- '*.rs' \
    | grep -v -e '^skate/' -e '^third_party/' || true
)

if [ ${#changed[@]} -eq 0 ]; then
  echo "fmt: no changed .rs files"
  exit 0
fi

out=$(rustfmt --check --edition 2024 "${changed[@]}" 2>&1 || true)

declare -A want
for f in "${changed[@]}"; do want[$f]=1; done

bad=()
while IFS= read -r path; do
  rel=${path#"$root"/}
  if [ -n "${want[$rel]:-}" ]; then bad+=("$rel"); fi
done < <(printf '%s\n' "$out" | sed -n 's/^Diff in \(.*\):[0-9]*:$/\1/p' | sort -u)

if printf '%s\n' "$out" | grep -q '^error'; then
  printf '%s\n' "$out" | grep -A5 '^error'
  exit 1
fi

if [ ${#bad[@]} -gt 0 ]; then
  echo "fmt: these changed files need rustfmt:"
  printf '  %s\n' "${bad[@]}"
  echo "run: rustfmt --edition 2024 ${bad[*]}"
  exit 1
fi

echo "fmt: ${#changed[@]} changed .rs files are formatted"
