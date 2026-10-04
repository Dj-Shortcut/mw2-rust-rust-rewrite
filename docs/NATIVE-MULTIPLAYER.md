# Native connected PC client

UNIMPLEMENTED design; follows [direct TCP multiplayer](DIRECT-MULTIPLAYER.md).
Task: [#251](https://github.com/Dj-Shortcut/mw2-rust-rust-rewrite/issues/251).

## Start and content

Host: `cargo run -p launcher --profile play --locked -- server --bind IP:PORT`.
Player: `cargo run -p launcher --profile play --locked -- join IP:PORT`.
Join precedes imported-game setup and uses repository-authored assets.
The no-argument/`game` route remains the offline development sandbox.

## Authority and networking

A worker owns DirectClient, polling independently of rendering and pause.
A bounded mailbox keeps the latest complete validated snapshot/replica pair.
Separate ordered receipts never drop; capacity is256. No local authority ticks.
Send movement once per newly observed server tick, without catch-up bursts.
Allow one typed action until its actual receipt; no automatic retries/local grants.
Input older than250ms becomes neutral. After5s without a validated tick,
neutralize input and poll for2s grace before closing with an English error.

## View and controls

Follow the assigned actor; show remote authored operators, live nodes and foundations.
Remove departed players/depleted nodes; late joins render the current world.
Read-only gather hints and foundation ghosts reuse shared placement rules.
WASD/mouse move/look; Shift sprint, Space jump, Ctrl crouch.
F gathers a Tree; B toggles Wood foundation mode; LMB submits placement.
Esc pauses local controls/releases the mouse; F1 shows English help.
Controller: LS/RS move/look, LS-click sprint, A jump, B crouch,
Y gather, Back building mode, RT place, Start local pause.
Focus loss, pause, admission and failure neutralize movement and actions.
HUD shows confirmed Wood,200-Wood foundation cost and pending/refusal feedback.
Pause: "Input paused. Shared world continues." Players are unarmed.
Combat, inventory mutation, crafting, skating/editor actions, upgrades/doors
and save/load are outside this route; never create an offline Session as fallback.

## Verification and limits

Run the shipping server and two actual Bevy client windows on Xvfb/Mesa.
Use keyboard input for mutual movement and finite gather→foundation with exact costs.
Check assigned cameras, remote visibility, scene bounds/colliders, depletion,
join/leave/fresh-owner admission, focus/pause and pending/refusal UI.
Prove the server and second client advance while the first pauses.
Exercise missing content/server, full admission, disconnect and orderly shutdown.
Keep probes/logs/screenshots ignored; add no permanent test scenario.
Software-GPU Linux windows do not prove Windows, physical controllers/audio,
two physical machines, internet hosting, visual parity or release readiness.
