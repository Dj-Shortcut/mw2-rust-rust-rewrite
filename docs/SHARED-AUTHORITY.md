# Shared survival authority core

Design for [#246](https://github.com/Dj-Shortcut/mw2-rust-rust-rewrite/issues/246).
This stage supplies a reusable server core, not connected or graphical multiplayer.
See [MULTIPLAYER.md](MULTIPLAYER.md) for the complete two-client milestone.

`survival::SharedSession` owns one authored world, terrain, editor and finite nodes.
It admits at most two actors with no dummy or starter items/building resources.
Each receives an opaque generation-bound handle and separate carried Inventory.
Building owner IDs are monotonic and distinct from reusable client slots.
Disconnect invalidates the handle; old structures/resources remain under the old owner.
Fresh admission is not persistence or authenticated reconnect restoration.

## Inputs and transactions
The authority advances once per 50 ms tick, with at most one command per actor.
Server-stamp time, force weapon 0 and whitelist movement/stance inputs.
Missing commands stop movement while retaining view; reject invalid/duplicate handles.
The server derives reach, occlusion, socket, grounding and player overlap from its world.
Semantic requests are GatherTree and PlaceWoodFoundation, without client geometry/costs.
Barehand Tree gathering yields 25 wood from a finite 300-wood node.
A Wood foundation costs 200 wood; zero starter wood makes gathering necessary.
Serialize requests by actor/requestID and validate before committing stock or balances.
Cache up to 256 request outcomes per actor, including refusals and the original tick.
Same ID/payload replays its outcome; conflicting, expired or stale requests cannot mutate.
Bound requests/admissions and checked clocks/IDs; resource balances cannot exceed 1,000,000.
Regrowth runs once with 50 ms and checks shared player/building/editor blockage.

## Snapshots and compatibility
Capture snapshots after gather/build; both actors observe the same players and geometry.
Include current nodes and owner mapping, and only the recipient's carried Inventory.
Legacy building resource ledgers remain public in this bounded stage.
There is no mutable-world or client-supplied actor-handle API.
Offline Session retains LOCAL0, its dummy, starter grants and 17 ms stepping.
Offline save format, native controls and the current game command stay unchanged.

## Verification and remaining scope
Use ignored probes against the actual API, with two actors sharing one authority.
Check movement, neutral input, finite-stock races, replay/refusal, ownership and collision.
Compare recipient snapshots, stale handles, fresh owners and offline behavior/save loading.
Compiler/CI checks and in-process probes do not prove transported multiplayer.
No host/join command or native connected UI is added by this core stage.
Transport, full survival/combat/skate/editor and persistence remain unfinished.
Next: a bounded direct transport and separate server plus two real client processes.
All content remains authored; all player-facing text remains English.
