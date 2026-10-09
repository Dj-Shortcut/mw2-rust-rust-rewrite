# Native Rust skate collision queries

Part of [the full mod](RUST-MW2-SKATE.md); [task #285](https://github.com/Dj-Shortcut/mw2-rust-rust-rewrite/issues/285).
Status: shared native collision and calculated skate flow observed; connected world movement unverified.

## Boundary and guards

`RustSkateWorld : ISkateWorld` queries actual server Unity physics for the skate core.
The host authenticates the exact connected/alive rider and owns its board/lease/probes.
Use the captured server main thread after the host physics synchronization boundary.
Queries never move transforms, synchronize physics or change global physics settings.
They briefly enable only their owned parked penetration collider and restore it
before publication; arbitrary AllLayers queries can see that actor at parking.
Close revokes the world before lifecycle cleanup; reentry/invalid bindings fail closed.
After authenticating those exact bindings, delegate geometry to the internal
RustSkateCollisionScene with fresh rider/board roots for this query. Its actor-free
entry grants no actor authority and performs no movement. Both entries share the
same queries, probe restoration, rail mapping and publication fences.
Use only existing mounted/standing axis-aligned hulls and identity rotation.
Centre is anchor + CentreOffset; both hulls extend 0.15m below the anchor.
Use [the owned probe factory](RUST-SKATE-PROBES.md) and `WithProbes` on the server
main thread. Probes remain unparented, active, zero-centred, unit-scale, exactly
hull-sized and disabled at rest; layer 2 and fixed parking are checked. Targets
must belong to the valid, loaded default physics scene. Probe freshness is checked
at query entry and publication. Legacy raw-probe signatures remain source-compatible
but close/refuse queries; this deliberately changes their earlier runtime behavior.
Exact zero-depth contact does not establish penetration or block clearance.
Fixed Rust.Layers.Server.PlayerMovement mask and QueryTriggerInteraction.Ignore apply.
Ignore only the exact rider/board transforms and descendants, never transform.root.
Fixed 128-entry buffers reject saturation; inspect every result, not native ordering.
Check initial overlap separately; reject concave meshes and Rust Verify ambiguity.
Reject invalid normal/distance, float spacing above Skin/8, and sweeps above 16m.
Normalize near-unit normals; skip zero-distance tangent/receding contacts.
Select the nearest non-self blocker whose normal opposes the original displacement.

## Rails and freshness

Immutable [SkateRailBinding](RUST-SKATE-RAIL-BINDING.md) wiring comes from #301/PR #305.
Existing constructors retain Empty or the caller's immutable binding; null is refused.
Map only verified non-self signed native collider IDs; self/unbound/overlap contacts stay 0.
Pass this world's exact Rails catalog to SkateMotion.TryStep with this same world.
The [live registry](RUST-SKATE-LIVE-RAILS.md) derives actual geometry and a freshness lease.
An additive binding-plus-lease constructor requires reference equality of the catalog.
Check freshness before query and result publication; stale/throwing leases close the world.
Revoke before intentional changes; observed changes expire permanently.
A change restored between reads is undetectable without the host revocation contract.
No native IDs/leases are persistent or client-selected; stock survival/TC remain unchanged.

## Evidence and remaining acceptance

Current 262 genuine server references: net48/C#7.3 build, zero warnings/errors.
Private real Rust scene: 117 rail/guard checks pass; 230 owned objects actually deleted.
Real BoxCast/Overlap/Rust Verify and signed-hit resolution pass. The original
disabled-probe failure was reproduced and corrected by #325. Its native suite
preserves 125 passing checks and one invalid exact-touching expectation; a focused
follow-up confirms zero-depth contact, positive overlap and separated results.
Owned probe cleanup observed 43 + 3 objects actually absent; no settings changed.
Private instrumented throw/reentry checks do not prove native API exceptions.
Null-player world checks exercise freshness/thread/reentry guards, never player movement.
Earlier 170 analytical behavior/44 passive linkage checks executed no Unity scene calls.
Earlier normal-rounding defect was reproduced/corrected; no copied game code is published.
Temporary probes, game DLLs, addresses, credentials and raw logs remain private/ignored.
The [native core/scene flow](RUST-SKATE-NATIVE-MOTION.md) now passes 75 checks,
978 calculated steps and 987 poses against actual owned fixtures, including
mount, push/brake/steering, obstacles, ollie/landing/bail, two rail orientations,
standing exits, a walkable slope and ledge. All 167 objects actually removed,
settings unchanged. The retained first analytical-height oracle failed; a focused
actual-cast diagnosis corrected only the temporary expectation, not production.
Full terrain/holes, concave interiors and connected collision-driven mount/dismount remain.
Production host, visible board/rider, client input/camera, replication and two-client play
are still required by #275; this component is not a playable release or owner test request.
