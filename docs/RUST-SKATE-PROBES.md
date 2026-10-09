# Owned Rust skate penetration probes

Part of [the full mod](RUST-MW2-SKATE.md); [task #325](https://github.com/Dj-Shortcut/mw2-rust-rust-rewrite/issues/325).
Status: source built and real server probes checked; connected skating remains unverified.

## Reproduction and selected boundary

The current server reported false for a disabled standing probe against a known
overlapping box. Enabling it reported true, depth 0.5 and a unit direction.
Inactive probes also reported false. A distinct local physics scene excluded its
probe from default queries, but its immediate cross-scene penetration failed.
That unsimulated route remains unproven, rather than established as impossible.
[Unity's example requires enabled colliders](https://docs.unity3d.com/6000.3/Documentation/ScriptReference/Physics.ComputePenetration.html).
The correction owns two probes in the default physics scene, disabled at rest.

## Ownership, measurement and lifecycle

Call `RustSkateProbeSet.TryCreate` on the actual server main thread. It returns
its allocation/cleanup record before native setup. A partially failed allocation
leaves a closed record with retained cleanup obligations.
Only exact owned, unparented, active objects and colliders are accepted: no
children, scripts or Rigidbody; zero centre, exact hull sizes, unit scale and
identity rotation. Layer 2 must be excluded from the actual PlayerMovement mask.
They remain parked at `(0, -10000, 0)`, beyond the bounded query domain including
both hull extents. This is parking, not global scene invisibility.

`TryPenetration` briefly enables only the selected probe. It observes activation,
computes with supplied poses, then always disables it in `finally` and observes
unchanged shape, layer and parking before publishing either contact or no contact.
No callback, yield, simulation, synchronization or component attachment occurs
while enabled. The target must be enabled, active, nontrigger and in a valid,
loaded default physics scene; cross-scene targets refuse instead of certifying
clear space. Invalid caller inputs refuse without invalidating healthy probes.
Native exceptions, observed probe mutations or uncertain activation/restoration
permanently close the set. Zero-depth contact does not block the world query.
Arbitrary AllLayers queries can still see the briefly enabled actor at parking.
No global collision matrix or physics settings are changed.

Use `RustSkateWorld.WithProbes` with the factory's exact probes. Legacy constructor
signatures remain source-compatible, but now close/refuse queries without an owned
set; their earlier runtime behavior is deliberately not preserved. Query entry
and publication check probe and rail freshness, with existing rider/board guards.
`Close` invalidates on any thread without Unity access. Cleanup runs on the
creator thread: `TryBeginCleanup` submits destruction, and `IsCleanupComplete`
observes the exact objects absent. Unknown children prevent whole-object cleanup;
their owner must resolve that hierarchy before retrying. Never acknowledge a
submitted Destroy, failed allocation or unknown cleanup as complete.

## Actual verification and remaining acceptance

On 2026-10-09 root compiled frozen source against the current 262 genuine server
references: net48/C# 7.3, zero warnings/errors; 273 source/project/reference pins
unchanged. The same production source was composed into a temporary Oxide probe
and compiled/loaded by the actual private server, with zero connected players.
Both hulls detected known overlap: standing depth 0.5, mounted approximately
0.6499634; unit direction, repeated alternation and restoration passed. Separated
poses succeeded without contact and returned zero direction/depth.

The first report preserves 125 passing checks and one failed fixture expectation:
exact touching returned true with depth 0, while the fixture expected false.
An independent review found no corresponding product defect. A separate native
follow-up confirmed touching depth 0 is nonblocking under the existing depth>0
rule, positive overlap depth 0.5 and separated no contact. This is probe output
and branch classification, not an authenticated world query or player movement.
All 43 initial and 3 follow-up owned objects were actually absent after deferred
cleanup; an independent scene inventory found no survivors. The distinct local
scene was gone and global physics settings were unchanged.

Mask exclusion, honest AllLayers visibility, invalid callers, wrong-thread
refusal, closed-set cleanup, shape/position/layer/scene/component mutations,
foreign-child retention and cleanup retry, actual collider/object destruction,
and irreversible expiry passed. Two private copies instrumented only the native
compute call with a deterministic throw or synchronous reentry and confirmed
refusal/restoration. They are not observations of an exception from the native
API. Partial allocation and activation/restoration failure paths remain source
reviewed, without an actual injected native failure.

Temporary probes, game references and raw logs remain private and ignored.
No permanent tests, rental, paid change, ShortcutLoadouts or saved inventory
behavior change was added. A production board/rider/input/effect host, visible
authored board, accepted client extension, replication and the complete two-client
#275 flow remain unfinished. These checks cannot establish connected skating.
