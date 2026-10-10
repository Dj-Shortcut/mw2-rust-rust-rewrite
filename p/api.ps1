# Offline API survey of the generated interop assemblies for the installed build (reflection only;
# nothing from the game is executed). Inputs (globals):
#   $pt  comma separated entries Type or Type~memberRegex (exact simple or full type name),
#        or with $pfind a regex of type names
#   $pp  default regex on member names (default: everything)
#   $pfind  when set, only list matching type names across all interop assemblies
$dl = "$HOME\Downloads"; $probe = "$dl\claude-loader-probe"
$pb = ((gc 'C:\Program Files (x86)\Steam\steamapps\appmanifest_252490.acf' | sls '"buildid"').Line -split '"')[3]
$gen = "$probe\gen-$pb\out"
if (-not (Test-Path "$gen\Assembly-CSharp.dll")) { "no interop set for build $pb"; return }
$inner = "$probe\api-inner.ps1"
@'
param([string]$Gen, [string]$Core, [string]$Types, [string]$Pat, [string]$Find, [string]$Out)
$ErrorActionPreference = 'SilentlyContinue'
foreach ($n in 'Il2CppInterop.Common','Il2CppInterop.Runtime') { [void][Reflection.Assembly]::LoadFrom("$Core\$n.dll") }
[AppDomain]::CurrentDomain.add_AssemblyResolve([ResolveEventHandler]{ param($s, $e)
  $n = ($e.Name -split ',')[0]
  foreach ($d in $Gen, $Core) { $p = Join-Path $d "$n.dll"; if (Test-Path $p) { return [Reflection.Assembly]::LoadFrom($p) } }
  return $null })
function TypesOf($asm) { try { $asm.GetTypes() } catch { $_.Exception.InnerException.Types | ? { $_ } } }
function Short($t) { if ($null -eq $t) { return '?' }; $n = $t.Name; if ($t.IsGenericType) { $n = ($n -replace '`\d+','') + '<' + (($t.GetGenericArguments() | % { Short $_ }) -join ',') + '>' }; $n }
$lines = New-Object Collections.Generic.List[string]
if ($Find) {
  foreach ($f in (ls $Gen -Filter *.dll)) {
    $a = [Reflection.Assembly]::LoadFrom($f.FullName)
    foreach ($t in (TypesOf $a)) { if ($t.FullName -match $Find -and $t.FullName -notmatch '<|__c') { $lines.Add($f.BaseName + ': ' + $t.FullName) } }
  }
} else {
  $want = $Types -split ',' | % { $_.Trim() } | ? { $_ }
  $asms = 'Assembly-CSharp','Facepunch.Console','Facepunch.System','Facepunch.Unity','Facepunch.Input','Rust.Global','Rust.Data','Facepunch.Network','UnityEngine.CoreModule','UnityEngine.PhysicsModule','UnityEngine.InputLegacyModule' | % { "$Gen\$_.dll" } | ? { Test-Path $_ }
  $all = foreach ($p in $asms) { TypesOf ([Reflection.Assembly]::LoadFrom($p)) }
  $flags = [Reflection.BindingFlags]'Public,NonPublic,Instance,Static,DeclaredOnly'
  $defaultPat = $Pat
  foreach ($entry in $want) {
    $w, $own = $entry -split '~', 2
    $Pat = $defaultPat; if ($own) { $Pat = $own }
    $hits = @($all | ? { $_.Name -eq $w -or $_.FullName -eq $w })
    if (-not $hits) { $lines.Add("== $w : NOT FOUND"); continue }
    foreach ($t in $hits) {
      $lines.Add('== ' + $t.FullName + ' : ' + (Short $t.BaseType) + ' [' + $t.Assembly.GetName().Name + ']')
      $props = $t.GetProperties($flags) | ? { $_.Name -match $Pat } | sort Name | % { $(if (($_.GetGetMethod($true), $_.GetSetMethod($true) | ? { $_ } | select -First 1).IsStatic) { 'static ' } else { '' }) + $_.Name + ':' + (Short $_.PropertyType) + $(if ($_.CanWrite) { '' } else { '{get}' }) }
      if ($props) { $lines.Add('P ' + ($props -join ' | ')) }
      $meth = $t.GetMethods($flags) | ? { -not $_.IsSpecialName -and $_.Name -match $Pat } | sort Name | % { $(if ($_.IsStatic) { 'static ' } else { '' }) + $_.Name + '(' + (($_.GetParameters() | % { (Short $_.ParameterType) + ' ' + $_.Name }) -join ', ') + '):' + (Short $_.ReturnType) }
      if ($meth) { $lines.Add('M ' + ($meth -join ' | ')) }
      $flds = $t.GetFields($flags) | ? { $_.Name -match $Pat -and $_.Name -notmatch '^Native(Field|Method)InfoPtr_|^__' } | sort Name | % { $(if ($_.IsStatic) { 'static ' } else { '' }) + $_.Name + ':' + (Short $_.FieldType) }
      if ($flds) { $lines.Add('F ' + ($flds -join ' | ')) }
    }
  }
}
[IO.File]::WriteAllLines($Out, $lines)
'@ | Out-File $inner -Encoding utf8
$out = "$probe\api-out.txt"
$find = ''; if ($pfind) { $find = $pt }
$pat = '.'; if ($pp) { $pat = $pp }
& "$dl\codex-bep788-probe\pwsh-diag\pwsh.exe" -NoLogo -NoProfile -File $inner -Gen $gen -Core "$probe\bep788\BepInEx\core" -Types "$pt" -Pat $pat -Find $find -Out $out
$global:pfile = $out; $global:poff = 0
zz more
