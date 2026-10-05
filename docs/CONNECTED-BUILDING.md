# Connected Wood building

Implemented [#259](https://github.com/Dj-Shortcut/mw2-rust-rust-rewrite/issues/259) development source; bounded flows verified, final integration pending.
The existing connected Foundation costs 200 Wood; this slice adds a 100-Wood Wall.

## Player flow and controls

Gather finite Wood normally; no starter stock or local inventory grants.
B / Controller Back opens or closes building mode; Foundation is selected initially.
Left / Controller Left selects Foundation; Right / Controller Right selects Wall.
Q / Controller LB rotates the selected Wall between X-axis and Y-axis.
With Foundation selected, rotation asks to select Wall and changes nothing.
Local kind/axis; ghosts/Wood/costs use confirmed shared world; Last notice mixes feedback/receipts.
Left click / Controller RT requests placement once on a fresh edge.
Mode/selection/rotation edges consume the frame without gathering or placing.
Distinct simultaneous building-control intents refuse without changing selection.
Equivalent keyboard/controller intent is deduplicated; held controls never repeat.
Pending work blocks selection/rotation/mode changes and another submitted action.
Inventory/trading, focus/pause/content/death/admission/error gates stay intact.
Movement/look unchanged; inventory neutral/free cursor; ghosts advisory, server confirms placement.

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

Current trace correction:32813 exact field/float-bit/ID comparisons pass, including4096 pieces.
Far queries allocate0; one local wall query2 vs8534 exhaustive requests in labelled fixtures.
Linear metadata/no timing claim. Full optimized Linux build, authority62/2582 and TCP70/71 pass.
Current observer115/118 pass with dual original-label reviews and all eight cost-image reviews.
Current shipping49/six original-cost reviews pass; scopes remain separate from authority/collision.
Earlier codec392/input585/mailbox664/panel30 retain their unchanged-module proof scope.
Earlier five720p layouts fit visible ink/lines; stock1000000/24slots were synthetic fixtures.
Strict failures preserved; no new tests/assets; final CI/review/guarded integration remain gates.
Floors/doors, demolition/upgrades/repair, TC/decay and server persistence stay open.
Physical controllers, Windows/internet gameplay and release readiness remain unproved.
