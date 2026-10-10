# Points zz and zg at another branch. A branch name has characters that cannot be typed from a
# distance, so $pto holds the name without them: claudeloaderprobe for claude-loader-probe.
$want = "$pto".ToLower() -replace '[^a-z0-9]', ''
if (-not $want) { "set pto to a branch name without its dashes and slashes; zz reads branch $pref"; return }
$all = @(); $page = 1
do {
  $got = @(irm "https://api.github.com/repos/$prepo/branches?per_page=100&page=$page")
  $all += $got; $page++
} while ($got.Count -eq 100)
$hit = @($all | % { $_.name } | ? { ($_.ToLower() -replace '[^a-z0-9]', '') -eq $want })
if ($hit.Count -ne 1) { "$($hit.Count) branches are called $want without their dashes and slashes; zz still reads branch $pref"; return }
$global:pref = $hit[0]
"zz reads branch $pref"
