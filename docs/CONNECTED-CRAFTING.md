# Connected Cloth and Bandage crafting

Implemented development slice for [#254](https://github.com/Dj-Shortcut/mw2-rust-rust-rewrite/issues/254), extending the native join route.

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

Typed GatherCloth/CraftBandage requests carry no client cost/item/actor payload.
The server derives the nearest visible target; only Tree/Hemp are supported.
Stage node depletion and carried Cloth together; refusal commits neither.
Craft on a candidate inventory: remove 4 Cloth, add 1 Bandage, then commit.
Consumed Cloth may free a slot; failed output leaves the original inventory intact.
No starter Cloth, local grants or Wood crafting charge. Replay preserves old receipts.
Append Item::Cloth and ResourceKind::Hemp; preserve old item tags and node IDs.
Add two non-solid finite Hemp plants after the existing deterministic node layout.
Older complete gathering saves gain these plants without changing saved old nodes.
All new offline Bandage crafting reserves Cloth, including queue/cancel/refund.
Death refunds fit available storage; excess reserved Cloth is discarded.
Older already-paid queued Bandages retain their legacy payment/refund compatibility.
Current standalone envelope is 3 after trading; imported protocol 95/schema 1 stay.
Only the recipient's carried stacks are sent; building resource ledgers remain public.
Fresh admissions receive new ownership and empty inventory, without restoration.

## Verification and remaining scope

Linux checks pass: 8 catalog and 10 actual Session groups, 21 craft-helper,
26 authority, 81 codec, 42 pure-panel and 138 Bevy software-input checks.
One shipping server and two independent DirectClient processes pass 36 checks:
A holds 6 Cloth/1 Bandage, B holds 2 Cloth/2 Bandages; 20 = 8 + 3 × 4.
Old saves/paid jobs, finite stock, full-slot atomicity and replay are covered.
Two actual-source native windows pass 98 keyboard/Mesa flow checks.
Two unmodified shipping windows pass 32 bounded keyboard/Mesa smoke checks.
English panels/Hemp are reviewed; full24 layout is verified in the linked trade guide.
All probes stay ignored; cargo test has 0 permanent test scenarios.
[Wood-for-Bandage trading](CONNECTED-TRADING.md) now exists separately from crafting.
Connected timed crafting, Bandage healing, other recipes, combat/skating,
private building balances and server persistence are outside this slice.
No Windows gameplay, physical controllers/audio, two-machine/internet or release proof.
