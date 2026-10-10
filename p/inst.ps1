# Install or update the owner's Rust Skate launcher on this PC. Builds the released commit named in
# p/release.txt, copies that plugin to play\, saves the launcher scripts locally (launcher\) and
# writes two files on the desktop: "Rust Skate.cmd" and "Rust Skate - remove mod files.cmd".
# The launcher itself needs no network and never builds anything.
$probe = "$HOME\Downloads\claude-loader-probe"; $L = "$probe\launcher"
$repo = 'Dj-Shortcut/mw2-rust-rust-rewrite'
$rel = ("" + (zg 'release.txt')).Trim() -split "`n" | ? { $_ -match '^[0-9a-f]{40}$' } | select -First 1
if (-not $rel) { 'ABORT: p/release.txt names no commit'; return }
$global:pcommit = $rel; $global:pnext = $null
zz d
$global:pcommit = $null
if ($pbuilt -ne $rel) { 'ABORT: the released commit did not build; the launcher was left as it was'; return }
New-Item -ItemType Directory -Force $L, "$probe\play" | Out-Null
Copy-Item "$probe\plugin\ShortcutSkateClient.dll" "$probe\play\ShortcutSkateClient.dll" -Force
$pb = ((gc 'C:\Program Files (x86)\Steam\steamapps\appmanifest_252490.acf' | sls '"buildid"').Line -split '"')[3]
"Skate mod " + $rel.Substring(0, 7) + ", prepared " + (Get-Date -Format 'yyyy-MM-dd HH:mm') + " for Rust build $pb" | Out-File "$probe\play\version.txt" -Encoding utf8
$enc = New-Object Text.UTF8Encoding($false)
foreach ($f in 'run.ps1', 'skate.ps1', 'watch.ps1', 'clean.ps1') {
  $t = "" + (irm "https://raw.githubusercontent.com/$repo/$rel/p/$f")
  if ($t.Length -lt 200) { "ABORT: could not fetch $f at the released commit"; return }
  [IO.File]::WriteAllText("$L\$f", $t, $enc)
}
$desk = [Environment]::GetFolderPath('Desktop')
$start = "@echo off`r`ntitle Rust Skate`r`npowershell -NoProfile -ExecutionPolicy Bypass -File `"%USERPROFILE%\Downloads\claude-loader-probe\launcher\skate.ps1`"`r`necho.`r`npause`r`n"
$clean = "@echo off`r`ntitle Rust Skate - remove mod files`r`npowershell -NoProfile -ExecutionPolicy Bypass -File `"%USERPROFILE%\Downloads\claude-loader-probe\launcher\clean.ps1`"`r`necho.`r`npause`r`n"
[IO.File]::WriteAllText("$desk\Rust Skate.cmd", $start, [Text.Encoding]::ASCII)
[IO.File]::WriteAllText("$desk\Rust Skate - remove mod files.cmd", $clean, [Text.Encoding]::ASCII)
'launcher files: ' + ((ls $L -File | % { "{0} {1}" -f $_.Name, $_.Length }) -join ', ')
'released plugin: ' + (gc "$probe\play\version.txt" -First 1) + ' | ' + (gi "$probe\play\ShortcutSkateClient.dll").Length + ' bytes'
'desktop: ' + ((ls $desk -Filter 'Rust Skate*' | % Name) -join ', ')
