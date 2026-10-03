#!/usr/bin/env bash
# Start the native game on a virtual display and record what happened.
#
#   scripts/smoke_native.sh <game-binary> <output-dir> [seconds]
#
# Needs Xvfb, libxkbcommon-x11, a Vulkan driver (Mesa lavapipe works) and
# ImageMagick `import`.
# The output directory gets the game's log, a screenshot if the process was
# still running at the deadline, and result.txt with one of:
#
#   running   the process was alive after <seconds>; it was then stopped
#   exited    the process ended on its own before the deadline
#
# Either way it proves nothing about gameplay: no input is sent and nothing
# on screen is checked. Exit status is 0 for running, 1 for exited.
set -u

binary=$(realpath "$1")
out=$(realpath -m "$2")
seconds=${3:-30}
root=$(cd "$(dirname "$0")/.." && pwd)

mkdir -p "$out"
work=$(mktemp -d)
display=:${SMOKE_DISPLAY:-97}

Xvfb "$display" -screen 0 1280x720x24 -nolisten tcp >"$out/xvfb.log" 2>&1 &
xvfb=$!
trap 'kill "$xvfb" 2>/dev/null; rm -rf "$work"' EXIT
sleep 2

(
	cd "$work" &&
		DISPLAY=$display SURVIVAL_ASSETS="$root/assets" exec "$binary"
) >"$out/game.log" 2>&1 &
game=$!

waited=0
while [ "$waited" -lt "$seconds" ] && kill -0 "$game" 2>/dev/null; do
	sleep 1
	waited=$((waited + 1))
done

if kill -0 "$game" 2>/dev/null; then
	DISPLAY=$display import -window root "$out/screenshot.png" 2>>"$out/xvfb.log"
	kill "$game"
	wait "$game" 2>/dev/null
	echo "running after ${seconds}s" | tee "$out/result.txt"
	exit 0
fi

wait "$game"
code=$?
echo "exited with code $code after ${waited}s" | tee "$out/result.txt"
tail -n 5 "$out/game.log"
exit 1
