# Connected Cloth and Bandage crafting

Unimplemented design for [#254](https://github.com/Dj-Shortcut/mw2-rust-rust-rewrite/issues/254), extending the native join route.

## Player flow

Gather a finite authored Hemp plant with F / Controller Y to receive 10 Cloth.
Cloth is a carried item, separate from building Wood/Stone/Metal balances.
I / Controller D-pad Up opens a recipient-only inventory panel.
C / Controller X deliberately crafts one Bandage there, costing 4 Cloth.
The panel shows confirmed Cloth/stacks, actual pending/refusal feedback and controls.
Inventory releases mouse capture and neutralizes movement/look/gather/build.
The shared world and network keep running; closing recaptures after two neutral frames.
Pause, focus loss, dead/content/error/admission gates prevent new actions.
Toggles cancel only unsent intents; a sent action is never automatically retried.

## Authority and compatibility

Add typed GatherCloth/CraftBandage requests, with no client cost/item/actor payload.
The server derives the nearest visible target; only Tree/Hemp are supported.
Stage node depletion and carried Cloth together; refusal commits neither.
Craft on a candidate inventory: remove 4 Cloth, add 1 Bandage, then commit.
Consumed Cloth may free a slot; failed output leaves the original inventory intact.
No starter Cloth, local grants or Wood crafting charge. Replay preserves old receipts.
Append Item::Cloth and ResourceKind::Hemp; preserve old item tags and node IDs.
Add two non-solid finite Hemp plants after the existing deterministic node layout.
Older complete gathering saves gain these plants without changing saved old nodes.
All new offline Bandage crafting reserves Cloth, including queue/cancel/refund.
Older already-paid queued Bandages retain their legacy payment/refund compatibility.
The standalone envelope becomes version 2; imported protocol 95/schema 1 remain.
Only the recipient's carried stacks are sent; building resource ledgers remain public.
Fresh admissions receive new ownership and empty inventory, without restoration.

## Verification and remaining scope

Check exact finite Cloth → Bandage conservation, unchanged building resources,
atomic full-slot/refusal cases, replay/conflicts, stale/dead/init actors and regrowth.
Use two real transport clients with unequal positive recipient inventories.
Exercise old gathering saves and paid queued-job completion/cancel/death refunds.
Use two native windows for real keyboard gather/panel/craft, pending/refusals,
focus/pause/failure/late join, continued other-client progress and natural cleanup.
Review English UI at 720p; keep probes ignored and add no permanent tests.
Wood-for-Bandage trade is a later roadmap item, separate from Cloth crafting.
Connected timed crafting, Bandage healing, trading, other recipes, combat/skating,
private building balances and server persistence are outside this slice.
No Windows gameplay, physical controllers/audio, two-machine/internet or release proof.
