# Temporary: scripted check of when the server puts a rider back who starts to move.
$global:ptag='B'; $global:psteps='mute,starts'; $global:pdone=1500; $global:psteam=$false; $global:pconnect=$true; $global:plisten=$false; $global:prt="$HOME\Downloads\claude-loader-probe\rt"; $global:pplug="$HOME\Downloads\claude-loader-probe\plugin"; $global:pwait=30; $global:ppreload=$false; $global:ptail=4
$global:pnext='session'; zz build
