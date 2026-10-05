# Connected player trading

Unimplemented design for [#257](https://github.com/Dj-Shortcut/mw2-rust-rust-rewrite/issues/257), extending [connected crafting](CONNECTED-CRAFTING.md).

## Flow and controls

The first slice exchanges 1 seller Bandage for 25 buyer Wood within 100 units.
Offers expire after 30 seconds of authority time. Bandage crafting costs 4 Cloth.
One live offer per two-player world; the seller must own the offered Bandage.
I / Controller Up opens inventory; V / Controller Right offers to the other player.
The buyer sees the seller and fixed terms, then Enter / Controller A accepts.
Backspace / Controller B cancels your outgoing offer or declines an incoming offer.
C / Controller X crafts; show confirmed own Wood/Bandages and addressed trade status.
Use fresh edges; conflicting craft/trade edges reject all; pending blocks another.
Trade controls apply only inside the eligible inventory panel, preserving world bindings.
Movement/look stay neutral and cursor free; toggle/recovery retain neutral frames.
Focus/pause/content/death/admission/error gates stop new requests; networking continues.
An already posted offer stays live during pause/panel closure until canceled/expired.
A sent action may settle after pause; keep its receipt, without automatic retry.

## Authority and settlement

Posting expresses seller consent; only the specifically addressed buyer can accept.
Bind a nonzero, never-reused world offer ID to both exact actor generations/owners.
Server selects the sole other initialized live actor; requests carry intent/offer ID.
At post and accept, require origin distance <=100 and clear eye-to-eye world trace.
Reject startsolid/allsolid; at accept recheck both handles, life and fixed terms.
Expire before fresh actions at checked creation tick +600, using 50 ms server ticks.
No escrow/debit at posting; cancel/decline/expiry/death/disconnect refund nothing.
Stage both Inventories and BuildingWorld: seller removes 1 Bandage, buyer adds 1,
buyer pays 25 Wood, seller receives 25; commit all three only after every check.
Check stock, output space, checked additions and MAX_RESOURCE_BALANCE before commit.
Preserve Cloth, unrelated items/wear, pieces/colliders, Stone/Metal and world nodes.
Business refusal transfers nothing and may leave the offer live until cancel/expiry.
Death/departure invalidates offers; fresh generations inherit no offers/outcomes.
Same-tick accept/cancel follows actor order; exact-ID closes cannot affect a newer offer.
Same request/action replays its historical receipt; terminal IDs cannot settle twice.
Project addressed current (client, owner) terms plus one last terminal outcome.
Validate historical peer identity structurally; it may be retired, with changed stock.
Never expose peer carried stacks; current Wood ledgers stay public. Both see outcomes.

## Integration and verification

Shared authority/trading helper, bounded standalone DTOs and native input/panel are seams.
Require standalone envelope 3; imported protocol 95 and snapshot schema 1 stay unchanged.
Offline NPC Fish trades and session saves stay unchanged; server trades are not saved.
Verify staged capacity/caps, real server/two-client conservation, deliberate consent,
replay/races/expiry/stale owners and malformed/privacy cases; then two native windows.
Check gates/pause, both outcomes and full 24-slot 720p layout with labelled fixtures.
Keep probes ignored. Vending/custom prices/escrow/healing/internet/release remain open.
