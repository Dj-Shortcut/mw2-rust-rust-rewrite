# Rider lifecycle and control fencing

Part of [the full existing-Rust mod](RUST-MW2-SKATE.md); [task #283](https://github.com/Dj-Shortcut/mw2-rust-rust-rewrite/issues/283).
Status: source compiled against genuine server Mono references; 115 temporary actual-DLL cases pass.

## Boundary and API

Original C#7.3 `mods/rust/shared/RiderControlSession.cs`, transportless and bounded.
Its net48 project references actual [gunplay](RUST-GUNPLAY-CORE.md)/[skate](RUST-SKATE-CORE.md) projects.
It owns identity, release gates, one candidate and cleanup acknowledgement.
Weapon UID state, physics, inventory, firing and networking stay with the adapter.
The adapter serializes effects/lifecycle/settlement; historical `Current` authorizes no new effects.
Host-issued Guid identities/leases provide fencing, not authentication.
Create requires positive player ID, nonempty runtime/life IDs and bounded host tick.
Prepare requires the exact next 10ms tick, confirmed `GunPose` and board lease.
Walking/Sprinting require no lease; Skating requires one; Unavailable may retain one.
These facts describe confirmed physical mode; the gate does not mount/dismount.
An optional request supplies matching identity, increasing positive sequence,
`GunInput`/`SkateInput`; no clock, pose, ammo, world, hit or UID selection.
Steer/spin/flip must be finite in [-1,1]; enums and timing bounds are checked.
One opaque pending candidate binds owner, original state and epoch.
Foreign, reused, stale, overlapping and out-of-order operations publish nothing.

## Release and settlement

Fire/Reload/Jump start locked; mode/lease changes, missing input and rejection relock.
Only explicit false buttons clear gates; missing input is not a release.
Held Fire while locked uses Unavailable, preserving the release requirement.
Reload/Jump true are suppressed until explicit release; entering Skating relocks Jump.
No input gives no actions/Unavailable; the adapter rejects backlog and admits only fresh current-tick samples.
Confirmed mode reaches Gunplay before ticking, so mount/bail beats reload completion.
The adapter preserves every UID state and advances holstered guns at the same tick.
Applied publishes the prepared frame and release gates.
RejectedNoEffects consumes tick/sequence, retains confirmed mode/lease and relocks.
Its fallback is Unavailable/no actions; settle the same core tick from original/
reconciled state, preserving ammo/cadence. No core reset or native atomicity is claimed.
UnknownPartial closes control; acknowledgement requires trusted reconciliation of ambiguous native effects, with no rollback claim.

## Lifecycle and verification

Death/disconnect/reload/stop invalidates even without safe dismount; ambiguous work uses UnknownPartial.
A stable exact cleanup ticket retains committed and pending confirmed board leases
(at most two distinct leases). Retry is idempotent until exact acknowledgement.
Old/foreign tickets cannot acknowledge cleanup for replacement lives or boards.
Restart requires acknowledged cleanup and a fresh host-issued life ID.
Runtime may change; same-runtime time cannot rewind, including pending work.
Temporary fixtures exercise the actual compiled DLL and production core APIs:
identity/time/input/token rejection, real releases, all outcomes, pending invalidation,
cleanup retry, restart and reload-versus-mode ordering. Native board/rider, callbacks,
replication, stock rules and full gameplay remain open.
