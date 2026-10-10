# The tooling moved to mods/rust/client/tools. This file only switches a session that still has the
# old helper functions over to the new ones, on this branch.
$global:pref = 'claude-loader-probe'
iex (irm -Headers @{ Accept = 'application/vnd.github.raw' } "https://api.github.com/repos/Dj-Shortcut/mw2-rust-rust-rewrite/contents/mods/rust/client/tools/remote.ps1?ref=$pref")
