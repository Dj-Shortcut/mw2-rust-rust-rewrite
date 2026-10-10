# In-world play session started through the owner's Steam shortcut "RustClient", so that the Steam
# Input layout made for that shortcut applies and the controller works. Loader with the patched
# runtime and the skate controller, sound on. K or two quick jumps mount and dismount; forward
# pushes, back brakes, the board follows the camera. The client stays up for at most an hour;
# quitting Rust ends the session and removes the loader.
$global:ptag='S'; $global:plisten=$false; $global:prt="$HOME\Downloads\claude-loader-probe\rt"; $global:pplug="$HOME\Downloads\claude-loader-probe\plugin"; $global:psteps='skate'; $global:pwait=30; $global:pdone=3600; $global:ppreload=$false; $global:ptail=4; $global:psteam=$true
$global:pnext='run'; zz d
