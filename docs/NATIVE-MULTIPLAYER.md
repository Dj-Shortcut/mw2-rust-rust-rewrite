# Native connected PC client

Implemented development route; the bounded Linux keyboard/Mesa flow is verified.
Gather/build task: [#251](https://github.com/Dj-Shortcut/mw2-rust-rust-rewrite/issues/251).

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
Camera follows the assigned actor; scene uses authored remote operators,
live resource nodes and shared foundations. Departed players/depleted nodes disappear.
Read-only Tree/Hemp hints and foundation ghosts use confirmed shared placement rules.
WASD/mouse move/look; Shift sprint, Space jump, Ctrl crouch.
F gathers Tree/Hemp; B toggles Wood foundation mode; Left click submits placement.
I / Controller Up opens inventory; C / Controller X crafts one Bandage for 4 Cloth.
Esc pauses local controls/releases the mouse; F1 shows English help.
Controller: LS/RS move/look, LS-click sprint, A jump, B crouch,
Y gather, Back foundation mode, RT place, Start local pause.
Focus loss, pause, admission and failure neutralize movement and new actions.
Original operator loading gates input; load failure closes the connection with an error.
HUD shows confirmed Wood, 200-Wood foundation cost and pending/refusal feedback.
[Cloth gathering and recipient inventory](CONNECTED-CRAFTING.md) are implemented.
Combat, timed crafting/healing, skating/editor, upgrades/doors and save/load
remain outside this route; no offline Session is created as fallback.

## Verification and limits
The earlier gather/build slice passed 61 preview, 70 software-input and
46 actual worker/socket checks; its bounded 24 shipping and 101 observer
window checks cover that slice. Current Cloth evidence is in the linked guide.
`cargo test` succeeds with 0 permanent tests; these are ignored behavior probes.
The Cloth slice uses one server and two independent clients, with unequal
positive inventories, finite stock, exact costs and historical receipt replay.
Affected captures were reviewed for foundation/ghost surfaces and English UI.
See [direct TCP limits](DIRECT-MULTIPLAYER.md). No proof of Windows gameplay,
physical controllers/audio, two physical machines, internet hosting or release readiness.
