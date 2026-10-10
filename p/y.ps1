# In-world session: loader with the patched runtime, sound muted, movement probe that drives the
# real movement body (player_movement.prefab) with and without the game's walk component.
# The client stays up until the plugin reports done or 25 minutes pass; connect by hand in F1,
# wait about a minute, wake the player, then hold W for two seconds.
$global:ptag='Y'; $global:plisten=$false; $global:prt="$HOME\Downloads\claude-loader-probe\rt"; $global:pplug="$HOME\Downloads\claude-loader-probe\plugin"; $global:psteps='mute,move'; $global:pwait=30; $global:pdone=1500; $global:ppreload=$false; $global:psteam=$false; $global:ptail=4
$global:pnext='run'; zz d
