# Owned Rust skate penetration probes

Part of [the full mod](RUST-MW2-SKATE.md); [task #325](https://github.com/Dj-Shortcut/mw2-rust-rust-rewrite/issues/325).
Status: design; correction source and native verification pending.

## Reproduction and selected boundary

Real current-server ComputePenetration reports false for a disabled standing probe
against a known overlapping box; enabling it reports true, depth0.5 and unit direction.
Inactive probes also report false. The raw-probe world must not advance a player.
A distinct local physics scene excluded its probe from default queries, but its immediate
cross-scene penetration failed. That unsimulated route remains unproven, not impossible.
[Unity requires enabled colliders](https://docs.unity3d.com/6000.3/Documentation/ScriptReference/Physics.ComputePenetration.html).
The selected correction owns two disabled probes in the default physics scene.

## Ownership, measurement and lifecycle

RustSkateProbeSet.TryCreate returns its allocation/cleanup record before native setup.
On partial failure the returned closed record remains a cleanup obligation, not null success.
Only exact owned unparented active objects/colliders are accepted: no children, scripts
or Rigidbody; zero centre, exact hull sizes, unit scale and identity rotation.
Fixed layer2 must be excluded from the actual PlayerMovement mask.
Park at(0,-10000,0), beyond the guarded query domain including both hull extents.
Only the selected probe is synchronously enabled during TryPenetration.
Observe activation before ComputePenetration; always disable in finally and confirm
unchanged shape/layer/parking before publishing any true or false result.
No callbacks, yields, simulation, synchronization or component attachment while active.
Uncertain activation/compute/restoration closes the set/world and refuses the query.
The actor is briefly visible to other AllLayers queries at its parking position;
this is not global scene invisibility or a global collision-matrix change.
RustSkateWorld.WithProbes uses the factory's exact probes; legacy signatures remain
but cannot query without the owned set. Rail freshness and existing guards remain.
Close invalidates on any thread without Unity access; cleanup runs on the owning thread.
TryBeginCleanup submits destruction; IsCleanupComplete observes exact objects absent.
Never acknowledge deferred Destroy, a failed allocation or unknown cleanup as complete.

## Verification and remaining acceptance

Root will compile against the current262 genuine server references and run a temporary
private probe on the existing externally closed server with zero connected players.
Check both hulls, positive/separated/touching geometry, mask exclusion and restoration,
shape/position/component mutation, thread/reentry/lifecycle refusal and actual cleanup.
Check global physics settings and all source/reference hashes before/after.
No permanent tests, proprietary binaries/media/raw logs, rental or paid change.
No ShortcutLoadouts or saved world/inventory behavior changes.
A real board/rider/input/effect host, visible authored board, accepted client extension,
replication and the complete two-client #275 flow remain unfinished.
These native probe results cannot establish connected skateboard gameplay.
