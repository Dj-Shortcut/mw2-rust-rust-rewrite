# Scripted check in a world, sound muted: the rider and the board in each state, standing still,
# seen from fixed cameras; K steps to the next view. Ends with a short ride and closes the session
# by itself. Connect by hand in the F1 console, close the console and hold a key to wake the player.
$global:ptag='O'; $global:psteps='mute,pose'; $global:pdone=1500; $global:psteam=$false; $global:plisten=$false; $global:prt="$HOME\Downloads\claude-loader-probe\rt"; $global:pplug="$HOME\Downloads\claude-loader-probe\plugin"; $global:pwait=30; $global:ppreload=$false; $global:ptail=4
$global:pnext='session'; zz build
