# Shared skateboard rail contract

[Issue #281](https://github.com/Dj-Shortcut/mw2-rust-rust-rewrite/issues/281) extends the [skate source](RUST-SKATE-CORE.md) for the [full mod](RUST-MW2-SKATE.md).
Design before implementation: native rail gameplay is missing. Preserve existing calls and enum values in original C#7.3 source.

## Trusted contacts and definitions

The host supplies an immutable copied set of at most 64 unique positive rail IDs
and revisions: finite horizontal top-centrelines 0.25–100m long within world bounds.
A new `SkateHit` constructor carries the actual hit collider's rail ID; the existing
constructor and misses use zero. Reject negative IDs and nonzero IDs on misses.
The world provider must bind contacts to real colliders and the same catalog/world
snapshot. Unknown/zero IDs remain ordinary contacts; metadata alone is not support.
Client packets cannot supply rail geometry, contacts, poses or authority.

## Capture and travel

Capture only inside a genuine descending airborne sweep, using its matching ID,
near-horizontal normal, safe impact/flip, 0.12m lateral and 0.01m height tolerance.
Velocity and final facing must align within 25degrees of either rail direction;
along speed must be at least 1.5m/s and is bounded by 14m/s. Tuning is provisional.
Sweep to the seated pose and verify clearance; obstruction denies capture.
Store the exact captured ID/revision/geometry and signed distance/speed.
Every grinding step requires genuine matching support. Missing support or
removed/revised/moved geometry releases at the current pose without replacement snap.
Latched push/steer/spin/flip have no effect; 0.6m/s² drag and 18m/s² braking cannot
reverse travel. Release below 0.8m/s. Sweep travel and check final rider headroom;
rail side/endcap hits remain obstacles. Block/bail safely on obstructed travel.
A fresh jump releases with the normal ollie impulse; held jump does not.

## Time, failure and compatibility

Consume remaining capture/endpoint time once, with no extra zero-duration step.
Allow at most capture→travel→release→ordinary continuation per public call;
continuation cannot capture again and airborne jump edges cannot fire twice.
Endpoint release keeps momentum; ordinary world queries determine later support.
Same-ID 0.25s recapture cooldown advances once per successful public step, including
the existing overload with an empty catalog. New release starts a fresh cooldown.
Failed queries or invalid candidates preserve the original state and empty events.
Append grinding/capture/release enum values; keep existing numeric values unchanged.
No saved-state or packet deserialization contract is introduced.

## Verification and native limits
Root compiles the actual DLL against genuine framework references and preserves
all 69 existing scenarios. Ignored probes cover definition bounds/copying, actual
collider identity, capture directions/rejections, snap/travel/exit obstruction,
endpoints/remainder, jump edges, stale rails, cooldown and late-query atomic failure.
No permanent tests, game files, offsets or probes ship. Record results in [TODO](../TODO.md).
Native colliders/adapters, visible board/rider, animation, two-client authority,
cleanup/lifecycle and the full player flow remain unverified; this is not a playable mod.
