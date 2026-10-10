# Build the head of the branch and play it: the client is started directly (keyboard and mouse),
# sound on, and joins the private test server at start. Quitting Rust ends the session; it is
# ended after an hour at the latest.
$global:ptag='P'; $global:psteps='skate'; $global:pdone=3600; $global:psteam=$false; $global:pconnect=$true; $global:plisten=$false; $global:prt="$HOME\Downloads\claude-loader-probe\rt"; $global:pplug="$HOME\Downloads\claude-loader-probe\plugin"; $global:pwait=30; $global:ppreload=$false; $global:ptail=4
$global:pnext='session'; zz build
