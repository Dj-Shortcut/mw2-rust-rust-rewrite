# Build the head of the branch and play it: the client is started directly (keyboard and mouse),
# sound on, and joins the private test server at start. Quitting Rust ends the session; it is
# ended after an hour at the latest, or $plimit seconds after its start. $pquiet turns the sound down.
$global:ptag='P'; $global:psteps='skate'; if ($pquiet) { $global:psteps='mute,skate' }; $global:pdone=3600; if ($plimit) { $global:pdone=[int]$plimit }; $global:psteam=$false; $global:pconnect=$true; $global:pkeep=$false; $global:plisten=$false; $global:prt="$HOME\Downloads\claude-loader-probe\rt"; $global:pplug="$HOME\Downloads\claude-loader-probe\plugin"; $global:pwait=30; $global:ppreload=$false; $global:ptail=4
$global:pnext='session'; zz build
