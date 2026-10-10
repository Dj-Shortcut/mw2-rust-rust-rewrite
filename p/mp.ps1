# In-world session: loader with the patched runtime, sound muted, rider-model probe (model dump,
# animation control, cameras, test tone, console route, on-screen text). Ends by itself. Connect by
# hand in F1, set env.time 12, close the console and hold a key to wake the player.
$global:ptag='M'; $global:plisten=$false; $global:prt="$HOME\Downloads\claude-loader-probe\rt"; $global:pplug="$HOME\Downloads\claude-loader-probe\plugin"; $global:psteps='mute,modelprobe'; $global:pwait=30; $global:pdone=1500; $global:ppreload=$false; $global:psteam=$false; $global:ptail=4
$global:pnext='run'; zz d
