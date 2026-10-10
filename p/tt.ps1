# In-world session: loader with the patched runtime, sound muted, scripted trick ride (ollies,
# flips, spins, a grab and two presses that come too late) seen from a fixed camera. Ends by
# itself after half a minute. Connect by hand in F1, close the console and hold a key to wake the
# player.
$global:ptag='T'; $global:plisten=$false; $global:prt="$HOME\Downloads\claude-loader-probe\rt"; $global:pplug="$HOME\Downloads\claude-loader-probe\plugin"; $global:psteps='mute,tricktest'; $global:pwait=30; $global:pdone=1500; $global:ppreload=$false; $global:psteam=$false; $global:ptail=4
$global:pnext='run'; zz d
