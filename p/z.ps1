# In-world session: loader with the patched runtime, sound muted, movement probe that keeps the
# game's walk component on and writes the velocity after the game's fixed step (late component and
# Harmony postfix). The client stays up until the plugin reports done or 25 minutes pass; connect
# by hand in F1 and close the console; the player wakes on its own after the click.
$global:ptag='Z'; $global:plisten=$false; $global:prt="$HOME\Downloads\claude-loader-probe\rt"; $global:pplug="$HOME\Downloads\claude-loader-probe\plugin"; $global:psteps='mute,move'; $global:pwait=30; $global:pdone=1500; $global:ppreload=$false; $global:ptail=4
$global:pnext='run'; zz d
