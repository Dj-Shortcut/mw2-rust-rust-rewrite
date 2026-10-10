# In-world session: loader with the patched runtime, sound muted, world probe with push test.
# The client stays up until the plugin reports done or 25 minutes pass; connect by hand in F1.
$global:ptag='W'; $global:plisten=$false; $global:prt="$HOME\Downloads\claude-loader-probe\rt"; $global:pplug="$HOME\Downloads\claude-loader-probe\plugin"; $global:psteps='mute,worldpush'; $global:pwait=30; $global:pdone=1500; $global:ppreload=$false; $global:psteam=$false; $global:ptail=6
$global:pnext='run'; zz d
