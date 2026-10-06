# Native Rust skate collision queries

Part of [the full mod](RUST-MW2-SKATE.md); [task #285](https://github.com/Dj-Shortcut/mw2-rust-rust-rewrite/issues/285).
Status: design draft; source, compilation and native behavior are not yet verified.

## Boundary

`RustSkateWorld : ISkateWorld` queries actual server Unity physics for the existing skate core.
Original `SkateWorldQuery` maps bounded inputs/results without native calls.
The host authenticates the rider, owns the exact board and two disabled box probes,
and calls on the server main thread after establishing the physics synchronization boundary.
Queries never move transforms, synchronize physics or change global physics settings.
Binding loss, reentrancy, stale/disabled results and exceptions fail without movement.
The host destroys its probes during lifecycle cleanup; the adapter never owns a native entity.
This slice changes no saved data, existing core API, stock survival/building/TC or player controls.

## Shape and query rules

Use only the existing mounted/standing axis-aligned hulls and identity rotation.
Centre is anchor + CentreOffset; both hulls extend 0.15m below the anchor.
Host probes have exact hull dimensions, zero local centre and unit world scale.
Use fixed `Rust.Layers.Server.PlayerMovement` and `QueryTriggerInteraction.Ignore`.
Ignore only the exact native rider/board roots and descendants, never a client-selected mask.
Check initial overlap separately: touching is allowed; positive penetration blocks.
A non-convex mesh overlap is conservatively refused because penetration ignores backfaces.
Rust terrain-filter ambiguity is refused, not converted into an empty world.
Use fixed 128-entry overlap/cast buffers; a full buffer fails even when stored hits are self.
Scan every cast result and choose the nearest non-self blocker; result order is irrelevant.
Zero-distance sweeps still classify initial overlap. Invalid normals/distances fail.
Native contacts have RailId 0 until a separate trusted collider/catalog binding is implemented.
Reject coordinates/conversions whose float spacing loses the core's 2mm contact skin.
Bounded numeric preparation and result selection use the actual compiled original helper.

## Verification and remaining acceptance

Compile net48/C#7.3 against genuine local RustDedicated Mono/Unity references.
Check actual DLL call linkage; exercise compiled mapping, filtering, nearest-hit selection,
full buffers, malformed data, precision limits and core failure atomicity in ignored probes.
No copied game implementation, fabricated native API, permanent test or game DLL is published.
These checks cannot execute Unity physics outside a running Rust scene.
Real contacts/slopes, terrain holes, concave interiors, probe shape/scale, hierarchy removal,
moving objects and native collision-driven mount/dismount remain a server runtime gate.
Visible board/rider, client input/camera/weapon adapters, rail bindings, replication,
lifecycle integration and complete two-client play remain required by #275.
No unfinished component is a playable release or an owner test request.
