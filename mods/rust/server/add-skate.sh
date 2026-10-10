#!/usr/bin/env bash
# Adds the skate plugin to the private Rust/Oxide test server set up by bootstrap-ubuntu.sh.
# Run once as root on that server. The plugin is fetched from a fixed commit of this repository
# and checked against its SHA-256; Oxide loads a new file in its plugins folder by itself.
# Remove again with: rm /srv/rust/server/oxide/plugins/ShortcutSkate.cs
set -Eeuo pipefail
COMMIT=5a9ed6e550a405365c8a18746dfd276a310b6848
SHA256=078508fbdb67683f92d4943480250c7987bcbe051e60565713df06e41cbb465e
DIR=/srv/rust/server/oxide/plugins
[[ $EUID -eq 0 ]] || { echo "run as root"; exit 1; }
[[ -d $DIR ]] || { echo "no Oxide plugins folder at $DIR"; exit 1; }
tmp=$(mktemp)
curl -fsSL "https://raw.githubusercontent.com/Dj-Shortcut/mw2-rust-rust-rewrite/$COMMIT/mods/rust/plugins/ShortcutSkate.cs" -o "$tmp"
echo "$SHA256  $tmp" | sha256sum -c - >/dev/null || { echo "the downloaded plugin does not match its checksum; nothing installed"; rm -f "$tmp"; exit 1; }
install -o rust -g rust -m 0644 "$tmp" "$DIR/ShortcutSkate.cs"
rm -f "$tmp"
echo "ShortcutSkate.cs installed in $DIR; Oxide loads it within a few seconds."
echo "Check in the game's F1 console with: oxide.plugins"
