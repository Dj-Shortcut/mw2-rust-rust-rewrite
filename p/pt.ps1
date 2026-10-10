# In-world session: loader with the patched runtime, sound muted, scripted pose check (rider pose
# first person, from a side camera with the pose on and off, from the chase camera, then the console
# command route). Ends by itself. Connect by hand in F1, close the console and hold a key to wake
# the player.
$global:ptag='R'; $global:plisten=$false; $global:prt="$HOME\Downloads\claude-loader-probe\rt"; $global:pplug="$HOME\Downloads\claude-loader-probe\plugin"; $global:psteps='mute,posetest'; $global:pwait=30; $global:pdone=1500; $global:ppreload=$false; $global:psteam=$false; $global:ptail=4
$global:pnext='run'; zz d
