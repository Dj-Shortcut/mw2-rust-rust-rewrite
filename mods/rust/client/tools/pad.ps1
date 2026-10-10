# As play.ps1, but started through the owner's Steam shortcut "RustClient", so that the Steam
# Input layout made for that shortcut applies and the controller works.
$global:ptag='S'; $global:psteps='skate'; $global:pdone=3600; $global:psteam=$true; $global:pconnect=$false; $global:plisten=$false; $global:prt="$HOME\Downloads\claude-loader-probe\rt"; $global:pplug="$HOME\Downloads\claude-loader-probe\plugin"; $global:pwait=30; $global:ppreload=$false; $global:ptail=4
$global:pnext='session'; zz build
