# Held-input freshness between native callbacks

Part of [the full mod](RUST-MW2-SKATE.md); [task #331](https://github.com/Dj-Shortcut/mw2-rust-rust-rewrite/issues/331).
Status: design; additive packet-silence admission and verification pending.

## Problem and compatibility

The session needs every exact 10ms core tick; genuine input callbacks need not
arrive100Hz. Legacy null relocks Fire/Reload/Jump and makes gun input unavailable,
while reusing its request violates increasing sequences. Legacy TryPrepare and
all existing public signatures retain these semantics. No timestep/tuning change.

## New observation path

Add immutable RiderInputObservation: original host-issued RiderControlRequest,
admission slot and lifetime1–10 logical slots. It retains no native message.
The admission slot is chosen at genuine capture as the next eligible unprocessed
core slot; it is not a packet timestamp. First admission requires that exact tick
and increasing packet sequence/slot; no retagging, queued or future input.

Three states are distinct: NewObservation admits that new packet;
NoNewObservation continues only internally retained, unexpired held levels;
Unavailable (refused/swallowed/unknown capture or host freshness loss) immediately
drops continuity and relocks. Continuation retains the original packet sequence;
it creates no fresh callback, new edge, extended deadline or release of a lock.
Valid only while tick >= slot and tick-slot < lifetime, without overflow.
The trusted host enforces actual monotonic wall freshness independently; logical
TTL proves no wall-time limit, native100Hz cadence or correct effect scheduler.

## Publication and fences

Only Applied settlement commits retained input. No pending observation is public.
RejectedNoEffects consumes its tick/sequence but drops continuity; UnknownPartial
closes. Legacy settlement, expiry, mode/board change and lifecycle clear continuity.
A fresh replacement on the first expired slot covers it; TTL1 can accept new input
each tick. An uncovered expired slot relocks; later held input cannot hide that gap.
Only genuinely new false controls can clear locks. Without after-transition
ordering proof, same-transition-slot release cannot unlock the new mode/lease;
require a later capture/admission slot. Restart discards old-life observations.
Entering from legacy cannot inherit unlocked controls. Wall freshness loss must
be Unavailable before replacement. Failed admission cannot refresh any boundary.

## Verification and remaining host work

Root verifies actual compiled temporal behavior with ignored checks: legacy gap,
slower packets/held continuity, unchanged sequences, expiry before replacement,
release/fence/rejection/lifecycle and pending-publication failures; pin inputs.
No permanent test, fake callback/player, loadout change or native effect here.
Production hook, monotonic scheduler/wall freshness, mount/board/model, movement,
replication/client and complete two-client #275 acceptance remain open.
