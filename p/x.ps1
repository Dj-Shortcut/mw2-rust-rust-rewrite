# Scratch command for the current investigation; rewritten as needed. Run with `zz x`.
# Now: the last run's plugin log without the once-a-second status lines.
$global:pinc = '.'; $global:pexc = '^SKATE (Ground|Air|Grind|Bail|Off) speed'
zz lg
