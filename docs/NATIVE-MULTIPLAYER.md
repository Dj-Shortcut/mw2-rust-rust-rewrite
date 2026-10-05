# Native connected PC client

Latest wall review fixes compile; current-source graphical reruns are pending.

## Start and content
Host: `cargo run -p launcher --profile play --locked -- server --bind 127.0.0.1:28980`.
Player: `cargo run -p launcher --profile play --locked -- join 127.0.0.1:28980`.
Run join twice in separate terminals to connect two players.
Join uses repository-authored assets; no original game files or imported-data setup.
The no-argument/`game` route remains the offline development sandbox.

## Authority and networking
A worker owns DirectClient and polls independently of rendering and local pause.
The bounded mailbox retains the latest complete validated snapshot/replica pair;
ordered receipts never drop (capacity 256). No client authority ticks run.
Movement sends once per newly observed server tick, without catch-up bursts.
One typed action remains pending until its receipt; no automatic retries/local grants.
Input older than 250 ms becomes neutral. After 5 s without a valid tick,
input stays neutral for 2 s polling grace, then the connection closes.
A failed sent action may have an unresolved outcome; it is never retried automatically.
Fresh admissions receive new ownership; saved/reconnect restoration is absent.

## View and controls
Assigned camera shows authored peers, live nodes and confirmed Foundation/Wall pieces.
Departed players/depleted nodes disappear.
Read-only Tree/Hemp hints and Foundation/Wall ghosts use confirmed shared rules.
WASD/mouse move/look; Shift sprint, Space jump, Ctrl crouch.
F gathers Tree/Hemp; B / Back toggles build mode; Left click / RT submits placement.
Build: Left / Controller Left selects Foundation200; Right / Controller Right selects Wall100.
Q / LB rotates the selected Wall between X-axis/Y-axis; Foundation rotation changes nothing.
I / Controller Up opens inventory; C / Controller X crafts one Bandage for 4 Cloth.
Inventory: V / Right offers; Enter / A accepts; Backspace / B cancels or declines.
Esc pauses local controls/releases the mouse; F1 shows English help.
Controller: LS/RS move/look, LS-click sprint, A jump, B crouch,
Y gather, Back build mode, RT place, Start local pause.
Focus loss, pause, admission and failure neutralize movement and new actions.
Original operator loading gates input; load failure closes the connection with an error.
HUD confirms Wood, selected Foundation200/Wall100 cost, orientation and pending/refusals.
[Cloth](CONNECTED-CRAFTING.md)/[trading](CONNECTED-TRADING.md) are verified; [walls](CONNECTED-BUILDING.md) have headless proof.
Combat, timed crafting/healing, skating/editor, upgrades/doors and save/load
remain outside this route; no offline Session is created as fallback.

## Verification and limits
Earlier gather/build: 61 preview,70 software-input,46 actual worker/socket checks;
its24 shipping/101 observer-window checks are historical. Cloth/trade guides have their own scope.
`cargo test` succeeds with 0 permanent tests; these are ignored behavior probes.
The Cloth slice verifies unequal carried inventories, finite stock, costs and replay.
Earlier costs/axes/ghosts and visible720p text fit; latest-source reruns remain pending.
See [direct TCP limits](DIRECT-MULTIPLAYER.md). No proof of Windows gameplay,
physical controllers/audio, two physical machines, internet hosting or release readiness.
