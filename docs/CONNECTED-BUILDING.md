# Connected Wood building

Unimplemented design for [#259](https://github.com/Dj-Shortcut/mw2-rust-rust-rewrite/issues/259), extending [native multiplayer](NATIVE-MULTIPLAYER.md).
The existing connected Foundation costs 200 Wood; this slice adds a 100-Wood Wall.

## Player flow and controls

Gather finite Wood normally; no starter stock or local inventory grants.
B / Controller Back opens or closes building mode; Foundation is selected initially.
Left / Controller Left selects Foundation; Right / Controller Right selects Wall.
Q / Controller LB rotates the selected Wall between X-axis and Y-axis.
With Foundation selected, rotation asks to select Wall and changes nothing.
The ghost, selected kind/orientation, confirmed Wood and cost use confirmed state.
Left click / Controller RT requests placement once on a fresh edge.
Mode/selection/rotation edges consume the frame without gathering or placing.
Distinct simultaneous building-control intents refuse without changing selection.
Equivalent keyboard/controller intent is deduplicated; held controls never repeat.
Pending work blocks selection/rotation/mode changes and another submitted action.
Inventory/trading, focus/pause/content/death/admission/error gates stay intact.
World movement/look keep their bindings; inventory remains neutral with a free cursor.
Client ghosts are advisory; only a confirmed server result creates the wall.

## Authority and compatibility

The server derives actor/owner, sight ray, socket, support, overlap and resource cost.
Wall intent carries axis 0 or 1 only; reject other axes before helper normalization.
Wall preview must reject a replica with mismatched confirmed player states.
Acceptance reruns current placement after movement; an earlier valid ghost is no grant.
Foundation costs 200 Wood and Wall 100; failed requests change no balance or pieces.
Current support is geometric: another player's Foundation may support your Wall.
Both peers receive the confirmed piece, owner/socket/geometry and collision bounds.
The WallPlaced receipt must name a positive confirmed recipient-owned Wall ID.
Request replay returns its historical receipt without placing or paying twice.
Leave/rejoin keeps old structures; new actors get fresh ownership and empty stock.
No removal flow is introduced; historical placement receipts retain existing rules.
All peers require envelope 4; imported protocol 95/snapshot schema 1 stay unchanged.
Offline building/saves stay separate; Wood ledgers public, carried stacks/trades private.

## Acceptance and limits

Run each wall axis in a fresh world: 300 gathered Wood becomes Foundation + Wall.
Check support, occupied socket, actor/terrain overlap, reach, cost, life and axis refusal.
Check replay/conflicts, stale handles, preview parity and unchanged Cloth/trading flows.
Use one real server/two independent TCP processes; inspect both replicas and collisions.
Then use two native windows: selection/rotation, ghosts, placement and blocked walking.
Read-only observer evidence stays separate from ordinary executable smoke/layout.
Label private fixtures; keep all probes ignored and permanent-test policy unchanged.
Required local gates, exact-head CI/review and guarded integration precede completion.
Floors/doors, demolition/upgrades/repair, TC/decay and server persistence stay open.
Physical controllers, Windows/internet gameplay and release readiness remain unproved.
