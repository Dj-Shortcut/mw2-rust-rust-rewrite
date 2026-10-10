# In-world play session: loader with the patched runtime and the skate controller, sound on.
# K mounts and dismounts; W pushes, S brakes, the board follows the camera, A and D carve.
# The client stays up for at most an hour; quitting Rust ends the session and removes the loader.
$global:ptag='P'; $global:plisten=$false; $global:prt="$HOME\Downloads\claude-loader-probe\rt"; $global:pplug="$HOME\Downloads\claude-loader-probe\plugin"; $global:psteps='skate'; $global:pwait=30; $global:pdone=3600; $global:ppreload=$false; $global:ptail=4
$global:pnext='run'; zz d
