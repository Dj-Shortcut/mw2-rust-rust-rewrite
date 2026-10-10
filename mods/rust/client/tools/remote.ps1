# Helpers for an agent that works on this PC through typed keys, where a long command is slow and
# some characters cannot be typed at all: `zz name` runs name.ps1 from this folder and `zg file`
# returns a file from it. Both read the branch named in $pref (default main) through the GitHub
# contents API: the raw host caches a branch for minutes and would hand out a stale script.
# The repository is private. Every request for it carries the read-only key that the owner keeps in
# github-key.txt in the staging folder; the key is sent to no other address and is never shown.
if (-not $pref) { $global:pref = 'main' }
$global:prepo = 'Dj-Shortcut/mw2-rust-skate-rewrite'
# The headers of a request for this repository; $accept says in which form the answer is wanted.
function global:zh($accept) {
  $h = @{ Accept = $accept }
  $key = "$HOME\Downloads\claude-loader-probe\github-key.txt"
  if (Test-Path -LiteralPath $key) { $h.Authorization = 'Bearer ' + ("" + (Get-Content -LiteralPath $key -First 1)).Trim() }
  $h
}
# The address of a file of the repository at a branch or commit.
function global:zu($path, $ref) { "https://api.github.com/repos/$prepo/contents/{0}?ref={1}" -f $path, $ref }
function global:zg($f) { irm -Headers (zh 'application/vnd.github.raw') (zu "mods/rust/client/tools/$f" $pref) }
function global:zz($n) { iex (zg "$n.ps1") }
$global:ErrorActionPreference = 'Continue'
"zz ready, branch $pref"
