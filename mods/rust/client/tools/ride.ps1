# Scripted check in a world, sound muted: the board rides out, turns round and rides back (about
# 15 seconds) and the session ends by itself. Joins the private test server at start; press a key
# to wake the player.
$global:ptag='R'; $global:psteps='mute,ride'; $global:pdone=1500; $global:psteam=$false; $global:pconnect=$true; $global:pkeep=$true; $global:plisten=$false; $global:prt="$HOME\Downloads\claude-loader-probe\rt"; $global:pplug="$HOME\Downloads\claude-loader-probe\plugin"; $global:pwait=30; $global:ppreload=$false; $global:ptail=4
$global:pnext='session'; zz build
