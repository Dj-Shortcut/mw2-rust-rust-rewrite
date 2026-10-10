# Scripted check in a world, sound muted: the ride with a controller, played into the plugin without
# one. Gets on with Y, pushes, steers, ollies and flips from the right stick, holds a manual, grabs,
# brakes and gets off. Ends by itself after half a minute. Joins the private test server at start;
# press a key to wake the player.
$global:ptag='F'; $global:psteps='mute,pad'; $global:pdone=1500; $global:psteam=$false; $global:pconnect=$true; $global:pkeep=$true; $global:plisten=$false; $global:prt="$HOME\Downloads\claude-loader-probe\rt"; $global:pplug="$HOME\Downloads\claude-loader-probe\plugin"; $global:pwait=30; $global:ppreload=$false; $global:ptail=4
$global:pnext='session'; zz build
