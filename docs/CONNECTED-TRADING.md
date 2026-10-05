# Connected player trading

Bounded [#257](https://github.com/Dj-Shortcut/mw2-rust-rust-rewrite/issues/257) trade extends [connected crafting](CONNECTED-CRAFTING.md): verified on Linux/localhost with keyboard/Mesa.

## Player flow and controls

The seller offers 1 Bandage for 25 buyer Wood within 100 units and clear sight.
Bandage crafting remains 4 Cloth; offering reserves or transfers nothing.
One live offer per two-player world; the seller must own the offered Bandage.
I / Controller Up opens inventory; C / Controller X crafts 1 Bandage.
Inside the eligible panel, V / Controller Right offers to the other player.
Only the addressed buyer can accept with Enter / Controller A.
Backspace / Controller B cancels your outgoing offer or declines an incoming one.
Fresh edges only: conflicting craft/trade edges reject all; pending blocks another.
Trade bindings apply inside eligible inventory, preserving existing world controls.
Movement/look stay neutral and cursor free; toggles/recovery keep neutral frames.
Focus/pause/content/death/admission/error gates stop requests; networking continues.
Pause or panel closure does not cancel a posted offer; sent actions may still settle.
Offers expire after 600 authority ticks of 50 ms (30 seconds), including local pause.
The panel shows confirmed own Wood/Bandages/Cloth and addressed seller/buyer roles.
Live terms/countdown come from the snapshot, ahead of the last terminal outcome.
Historical own-request feedback stays separately labelled; no peer receipt is invented.

## Authority and settlement

Offer IDs are nonzero, never reused in one world and bound to both actor generations.
The server selects the other initialized live actor; clients cannot choose price/target.
At post/accept require finite origin distance <=100 and clear eye-to-eye world trace.
Acceptance rechecks life, identities, stock, output room and checked Wood balances.
Stage both Inventories and BuildingWorld: seller gives 1 Bandage, buyer pays 25 Wood;
commit all three together after every check, preserving unrelated carried/world data.
A refusal transfers nothing and can leave a valid offer available for another attempt.
Cancel/decline/expiry/death/departure close without refunds because there is no escrow.
Expiry/invalidation precedes fresh actions; same-tick accept/cancel follows actor order.
Replay returns the historical receipt; terminal offer IDs cannot settle twice.
Addressed offers/outcomes and carried stacks stay private; Wood ledgers stay public.
Fresh admission inherits no old inventory/offer/outcome.

## Integration and verification

All peers require standalone envelope 4; imported protocol 95/snapshot schema 1 stay.
Server trades/inventories are not saved; offline NPC Fish trades/saves stay separate.
Trading-milestone source checks (historical): public authority 32, codec 304, panel 72, input 316, mailbox 429; settlement 15/private lifecycle 8 use explicit fixtures.
Real server plus two independent TCP processes: 55 checks pass; seller has 25 Wood/1 Bandage/2 Cloth, buyer 0 Wood/1 Bandage/0 Cloth.
Two native Bevy windows with read-only observer: 115 flow checks pass; 38 ordinary shipping-window smoke checks pass separately.
Labelled full-24-slot 1280x720 render fixtures: all 5 cases pass, including 512-character refusal and disconnect text.
Local fmt/check/test/clippy/asset/publish gates pass; integration requires exact-head GitHub checks/review in [PR #258](https://github.com/Dj-Shortcut/mw2-rust-rust-rewrite/pull/258).
Full inventory fixtures prove layout; they do not grant stock to the actual server.
Custom prices, vending/escrow, healing, internet/authentication and persistence remain open.
Physical controllers, Windows/two-machine GPU play and release readiness remain unproved.
