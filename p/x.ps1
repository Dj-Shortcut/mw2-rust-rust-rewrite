# In-world session: loader with the patched runtime, sound muted, movement probe (component dump,
# walking reference window, six push methods). The client stays up until the plugin reports done or
# 25 minutes pass; connect by hand in F1, wake the player, then hold W for a few seconds.
$global:ptag='X'; $global:plisten=$false; $global:prt="$HOME\Downloads\claude-loader-probe\rt"; $global:pplug="$HOME\Downloads\claude-loader-probe\plugin"; $global:psteps='mute,move'; $global:pwait=30; $global:pdone=1500; $global:ppreload=$false; $global:ptail=4
$global:pnext='run'; zz d
