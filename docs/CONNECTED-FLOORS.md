# Connected Wood Floors: design

Design only for [#262](https://github.com/Dj-Shortcut/mw2-rust-rust-rewrite/issues/262).
Floor gameplay is not connected yet; [walls](CONNECTED-BUILDING.md) are integrated via #261.

## Player flow and controls
Gather finite Wood, select Floor, aim at the upper side of a supported Wall,
preview an overhead deck, then submit once; both clients show the confirmed piece.
Keep B / Back build mode, Left / DPadLeft Foundation and Right / DPadRight Wall.
Add Down / DPadDown Floor in build mode; left click / RT requests placement.
Q / LB rotates Wall only; other selections say “Select Wall to rotate”.
Keep inventory/trading priority, equivalent-intent deduplication and held-edge suppression.
Reject distinct simultaneous control intents; pending work blocks new build controls/actions.
Preserve focus/pause/death/content/admission/error gates and movement/look.
Show confirmed Wood, Floor cost 100, advisory ghost and English Last notice feedback.

## Authority and compatibility
Reuse existing Floor placement geometry: axis 0, levels 1..32, Wood health 250.
Existing Wall/Doorway support is geometric; another owner's Wall may support the Floor.
The server derives current actor, ray, socket, support, overlap, ownership and 100-Wood cost.
PlaceWoodFloor carries no geometry or owner; failed requests spend nothing.
Floor preview compares complete ordered confirmed player states before deriving geometry.
FloorPlaced must name a positive confirmed recipient-owned Floor ID.
Match only the typed Floor action/effect pair; reject every Floor cross-pair before fallback.
Validate the whole pending-ID receipt batch before clearing pending or publishing mutations.
Replay returns the historical receipt without a second piece/payment; conflicting IDs reject.
Append action/effect tag 9; envelope 4 becomes 5, requiring all peers to upgrade together.
Retain existing tags, imported protocol 95 and shared/building schema 1; offline format 3 unchanged.
Planned runtime seams: survival shared.rs, standalone protocol.rs and native connected.rs,
connected/input.rs, connected/network.rs and connected/scene.rs; claim them before editing.

## Acceptance before implementation is marked complete
Use two fresh worlds for supporting Wall axes 0 and 1; Floor itself always uses axis 0.
A naturally gathers Tree1's 300 Wood and pays Foundation200 plus Wall100.
B naturally gathers 100 from Tree6, then pays for a Floor on A's Wall; Tree6 retains 200.
Verify equal confirmed geometry/IDs, B ownership, exact costs, replay and occupied refusal.
Reject zero balance, level 0, unsupported sockets, current overlap and stale previews atomically.
Check complete player-replica mismatches, malformed codecs/IDs and mixed receipt batches.
Check Floor top/underside point/capsule traces, movement underneath and retained wall collision.
Run actual independent TCP clients, native windows and separately labelled ordinary smoke.
Review English success/failure/help/pending panels at 720p; retain strict failed runs.
Root owns frozen-source builds/execution/publication; ignored probes, no new permanent tests.

## Limits
This is an overhead deck/roof, not verified access to a second storey.
Existing jump/mantle rules do not establish climbing onto it; stairs/access need a separate task.
Doors, upgrades, demolition, TC/decay, private Wood balances and server persistence stay open.
No new assets or proprietary content. Compiler checks are not gameplay evidence;
Linux/Mesa/localhost acceptance will not prove Windows, hardware controllers or internet play.
