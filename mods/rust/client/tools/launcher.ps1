# Install or update the owner's launcher on this PC. Builds the commit named in release.txt, keeps
# that plugin in play\, saves the launcher's own scripts locally in launcher\ and writes two files
# on the desktop: "Rust Skate.cmd" and "Rust Skate - remove mod files.cmd". The launcher itself
# needs no network and never builds anything, so what the owner plays is exactly this commit.
$probe = "$HOME\Downloads\claude-loader-probe"; $L = "$probe\launcher"
$repo = 'Dj-Shortcut/mw2-rust-rust-rewrite'
$rel = ("" + (zg 'release.txt')).Trim() -split "`n" | ? { $_ -match '^[0-9a-f]{40}$' } | select -First 1
if (-not $rel) { 'ABORT: release.txt names no commit'; return }
$global:pcommit = $rel; $global:pnext = $null
zz build
$global:pcommit = $null
if ($pbuilt -ne $rel) { 'ABORT: the released commit did not build; the launcher was left as it was'; return }
$enc = New-Object Text.UTF8Encoding($false)
$files = @{}
foreach ($f in 'session.ps1', 'skate.ps1', 'watch.ps1', 'clean.ps1') {
  $t = "" + (irm "https://raw.githubusercontent.com/$repo/$rel/mods/rust/client/tools/$f")
  if ($t.Length -lt 200) { "ABORT: could not fetch $f at the released commit; the launcher was left as it was"; return }
  $files[$f] = $t
}
New-Item -ItemType Directory -Force $L, "$probe\play" | Out-Null
# Everything in play\ is copied into the game's plugin folder, so only the plugin may be there.
Get-ChildItem "$probe\play" -File | ? { $_.Name -ne 'ShortcutSkateClient.dll' } | Remove-Item
Get-ChildItem $L -File -Filter *.ps1 | ? { -not $files.ContainsKey($_.Name) } | Remove-Item
Copy-Item "$probe\plugin\ShortcutSkateClient.dll" "$probe\play\ShortcutSkateClient.dll" -Force
foreach ($f in $files.Keys) { [IO.File]::WriteAllText("$L\$f", $files[$f], $enc) }
$pb = ((gc 'C:\Program Files (x86)\Steam\steamapps\appmanifest_252490.acf' | sls '"buildid"').Line -split '"')[3]
"Skate mod " + $rel.Substring(0, 7) + ", prepared " + (Get-Date -Format 'yyyy-MM-dd HH:mm') + " for Rust build $pb" | Out-File "$L\version.txt" -Encoding utf8
$desk = [Environment]::GetFolderPath('Desktop')
$start = "@echo off`r`ntitle Rust Skate`r`npowershell -NoProfile -ExecutionPolicy Bypass -File `"%USERPROFILE%\Downloads\claude-loader-probe\launcher\skate.ps1`"`r`necho.`r`npause`r`n"
$clean = "@echo off`r`ntitle Rust Skate - remove mod files`r`npowershell -NoProfile -ExecutionPolicy Bypass -File `"%USERPROFILE%\Downloads\claude-loader-probe\launcher\clean.ps1`"`r`necho.`r`npause`r`n"
[IO.File]::WriteAllText("$desk\Rust Skate.cmd", $start, [Text.Encoding]::ASCII)
[IO.File]::WriteAllText("$desk\Rust Skate - remove mod files.cmd", $clean, [Text.Encoding]::ASCII)
'launcher files: ' + ((ls $L -File | % { "{0} {1}" -f $_.Name, $_.Length }) -join ', ')
'released plugin: ' + (gc "$L\version.txt" -First 1) + ' | ' + (gi "$probe\play\ShortcutSkateClient.dll").Length + ' bytes | play folder: ' + ((ls "$probe\play" | % Name) -join ', ')
'desktop: ' + ((ls $desk -Filter 'Rust Skate*' | % Name) -join ', ')
