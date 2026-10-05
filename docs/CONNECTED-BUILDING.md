# Connected Wood building

Implemented [#259](https://github.com/Dj-Shortcut/mw2-rust-rust-rewrite/issues/259) development source; latest review fixes await full acceptance.
The existing connected Foundation costs 200 Wood; this slice adds a 100-Wood Wall.

## Player flow and controls

Gather finite Wood normally; no starter stock or local inventory grants.
B / Controller Back opens or closes building mode; Foundation is selected initially.
Left / Controller Left selects Foundation; Right / Controller Right selects Wall.
Q / Controller LB rotates the selected Wall between X-axis and Y-axis.
With Foundation selected, rotation asks to select Wall and changes nothing.
Kind/axis selection is local; ghosts, Wood and costs use the confirmed shared world.
Last notice mixes local feedback and receipts.
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
Confirmed pieces feed both world traces and movement/mantle collision backends.
The WallPlaced receipt must name a positive confirmed recipient-owned Wall ID.
Request replay returns its historical receipt without placing or paying twice.
Leave/rejoin keeps old structures; new actors get fresh ownership and empty stock.
All peers require envelope 4; imported protocol 95/snapshot schema 1 stay unchanged.
Offline building/saves stay separate; Wood ledgers public, carried stacks/trades private.

## Acceptance and limits

Current source: authority62 and2582 live command writebacks; codec392/input585 pass.
Mailbox664 and panel30 pass; current optimized Linux build and scoped fmt pass.
Mismatch/private capacity or identity fixtures are labelled, not normal play.
Earlier source TCP70/71 and native keyboard/Mesa110/113 are historical results.
Current-source TCP/native/ordinary/layout acceptance and exact-head CI/review are pending.
Earlier720p fixtures fit visible glyphs/ink/occupied lines; hanging whitespace is excluded.
Original strict OCR/raw-full-width failures remain preserved, never relabelled PASS.
Read-only native observer evidence remains separate from shipping/layout fixtures.
Floors/doors, demolition/upgrades/repair, TC/decay and server persistence stay open.
Physical controllers, Windows/internet gameplay and release readiness remain unproved.
