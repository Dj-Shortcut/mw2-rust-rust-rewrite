# Native connected PC client

Implemented development route; the bounded Linux keyboard/Mesa flow is verified.
Task: [#251](https://github.com/Dj-Shortcut/mw2-rust-rust-rewrite/issues/251).

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
Read-only Tree hints and foundation ghosts use confirmed shared placement rules.
WASD/mouse move/look; Shift sprint, Space jump, Ctrl crouch.
F gathers a Tree; B toggles Wood foundation mode; Left click submits placement.
Esc pauses local controls/releases the mouse; F1 shows English help.
Controller: LS/RS move/look, LS-click sprint, A jump, B crouch,
Y gather, Back foundation mode, RT place, Start local pause.
Focus loss, pause, admission and failure neutralize movement and new actions.
Original operator loading gates input; load failure closes the connection with an error.
HUD shows confirmed Wood, 200-Wood foundation cost and pending/refusal feedback.
Pause: "Input paused. Shared world continues." Players are unarmed.
Combat, inventory mutation, crafting, skating/editor actions, upgrades/doors
and save/load are outside this route; no offline Session is created as fallback.

## Verification and limits
The optimized Linux shipping build passes. Separate headless probes pass
61 shared-preview, 70 Bevy software-input and 46 actual worker/socket checks.
They cover previews, software input and worker failures within their scopes.
`cargo test` succeeds with 0 permanent tests; these are ignored behavior probes.
Two unmodified shipping windows pass 24 narrow keyboard/Mesa smoke checks.
Two exact-source clients with a read-only observer pass 101 connected-flow checks:
finite gather/foundation, shared scene/colliders, pause, failures and cleanup.
Affected captures were reviewed for foundation/ghost surfaces and English UI.
See [direct TCP limits](DIRECT-MULTIPLAYER.md). No proof of Windows gameplay,
physical controllers/audio, two physical machines, internet hosting or release readiness.
