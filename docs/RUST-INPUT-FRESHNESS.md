# Held-input freshness between native callbacks

Part of [the full mod](RUST-MW2-SKATE.md); [task #331](https://github.com/Dj-Shortcut/mw2-rust-rust-rewrite/issues/331).
Status: additive source compiled; 382 actual server protocol checks pass; native host pending.

## Problem and compatibility

The session needs every exact 10ms core tick; genuine input callbacks need not
arrive at 100Hz. Legacy null relocks Fire/Reload/Jump and makes gun input unavailable,
while reusing its request violates increasing sequences. Legacy TryPrepare and
all existing public signatures retain these semantics. No timestep/tuning change.
## New observation path

Immutable RiderInputObservation retains the original host-issued RiderControlRequest,
admission slot and lifetime 1–10 logical slots. It retains no native message.
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
TTL proves no wall-time limit, native 100Hz cadence or correct effect scheduler.

## Publication and fences

Only Applied settlement commits retained input. No pending observation is public.
RejectedNoEffects consumes its tick/sequence but drops continuity; UnknownPartial
closes. Legacy settlement, expiry, mode/board change and lifecycle clear continuity.
A fresh replacement on the first expired slot covers it; TTL1 can accept new input
each tick. An uncovered expired slot relocks; later held input cannot hide that gap.
Only genuinely new false controls can clear locks. Without after-transition
ordering proof, same-transition-slot release cannot unlock the new mode/lease;
require a later capture/admission slot. Restart discards old-life observations; legacy cannot supply unlocked controls.
Wall freshness loss must be Unavailable before replacement; failed admission
cannot refresh any boundary.

## Verification and remaining host work

net48/C#7.3 builds against 262 genuine references: 0 warnings/errors, 274 pins unchanged.
Exact composed source ran in a private Oxide diagnostic: 382 checks passed, 0 failed
across 14 groups, including legacy/slower packets/TTL1/expiry/fences/rejection and
maximum clock bounds. Report: `319ad4946c2cf216dc995eb423c03ec385de8d80d2500b12b235d649e33c1aa0`.
Synthetic identities/ticks/poses and bookkeeping settlements only; 0 native objects,
0 connected captures or effects; physics unchanged, diagnostic unloaded/source removed.
Production hook, wall freshness/scheduler, mount/board/model, movement, replication
and client/two-client #275 acceptance remain open. Loadouts unchanged; no permanent test.
