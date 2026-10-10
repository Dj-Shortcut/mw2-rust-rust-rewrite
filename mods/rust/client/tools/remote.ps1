# Helpers for an agent that works on this PC through typed keys, where a long command is slow and
# some characters cannot be typed at all: `zz name` runs name.ps1 from this folder and `zg file`
# returns a file from it. Both read the branch named in $pref (default main) through the GitHub
# contents API: the raw host caches a branch for minutes and would hand out a stale script.
if (-not $pref) { $global:pref = 'main' }
function global:zg($f) { irm -Headers @{ Accept = 'application/vnd.github.raw' } ("https://api.github.com/repos/Dj-Shortcut/mw2-rust-rust-rewrite/contents/mods/rust/client/tools/{0}?ref={1}" -f $f, $pref) }
function global:zz($n) { iex (zg "$n.ps1") }
$global:ErrorActionPreference = 'Continue'
"zz ready, branch $pref"
