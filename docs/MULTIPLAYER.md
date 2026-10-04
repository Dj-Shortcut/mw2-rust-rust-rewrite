# Standalone multiplayer milestone

Design for [#243](https://github.com/Dj-Shortcut/mw2-rust-rust-rewrite/issues/243).
Multiplayer is a core requirement: players share one authored survival world.
Current `iw4l.exe game` is offline; this design adds no networking.

The net crate has authority/client roles, admission, commands, deltas and verdicts.
Reuse codecs/admission/master primitives; authored launch/content and actions need adapters.
UDP codecs are relay-only: use a configured QUIC master or implement direct I/O separately.
The master is an asset-independent relay/browser; current clients gate imported content.
`make duo` uses game data and does not prove standalone survival multiplayer.
The standalone multiplayer path must require no original game assets.

## Authority and ownership
One server owns one SimWorld, terrain, buildings, resources, editor and world timers.
Inventory, needs, crafting, clothing, skating and personal feedback belong to each player.
Session currently binds these to LOCAL0; define actor-scoped state before shared actions.
Do not clone independent Session worlds or map every actor to LOCAL0.
The target dummy is not a connected client; assign real peers distinct identities.
Collect commands and step once per server tick. Reconcile 20 Hz networking with 17 ms Session steps.
Bind requests to connection identity; validate reach, costs, ownership and repetition.
Derive building sockets/grounded state from the server actor view, not client assertions.
Route every mutation through authority; include inventory/skate and editor permission/history.
Replicate public state and owner-private inventory/needs with explicit schemas.
Existing building snapshots expose all resource ledgers; private balances need projection.
Client pause/inventory cannot stop the server; local F9 cannot roll back the world.
Persistence/reconnect need verified durable ownership before reusing connection identities.

## First connected milestones
1. Start one asset-independent authority and two clients with matching static content.
   Distinguish static content identity from mutable gathered/edited world baselines.
   Server-stamped movement and join/leave must be visible to both clients.
2. Gather from the same finite node, credit only the requesting player, and serialize
   simultaneous requests/depletion. Build a Wood foundation with server-owned costs;
   both clients must see it and collide with the same authoritative geometry.
3. Connect native presentation/input and per-player survival state; exercise the
   gather → craft → build → combat → skate loop while both players are connected.
4. Verify persistence, reconnect, latency/loss and real two-machine Windows/GPU play.

Claim runtime files separately before changing APIs or shared native code.

## Acceptance and remaining scope
Use one authority process and two independent client processes with assigned identities.
Show matching terrain, both players, actor-owned gathering and a shared building.
Check refusal, repetition and disconnect cases appropriate to the implemented slice.
Separate headless protocol, local graphical and real remote-PC evidence.
Offline windows, a dummy or imported-game networking do not satisfy this acceptance.
World/player replication, persistence, internet play and production scale remain open.
Original/open-source content only; player-facing text stays English.
Re-estimate after architecture and the first connected flow; no delivery date is set.
