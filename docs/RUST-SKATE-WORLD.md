# Native Rust skate collision queries

Part of [the full mod](RUST-MW2-SKATE.md); [task #285](https://github.com/Dj-Shortcut/mw2-rust-rust-rewrite/issues/285).
Status: genuine-reference build passes; 127 temporary actual-DLL checks pass; native scene unverified.

## Boundary

`RustSkateWorld : ISkateWorld` queries actual server Unity physics for the existing skate core.
Original `SkateWorldQuery` maps bounded inputs/results without native calls.
The host authenticates the rider, owns the exact board and two disabled box probes,
and calls on the server main thread after establishing the physics synchronization boundary.
Queries never move transforms, synchronize physics or change global physics settings.
Binding loss, reentrancy, stale/disabled results and exceptions fail without movement.
Close revokes the binding; the host destroys its probes/board during lifecycle cleanup.
This slice changes no saved data, existing core API, stock survival/building/TC or player controls.

## Shape and query rules

Use only the existing mounted/standing axis-aligned hulls and identity rotation.
Centre is anchor + CentreOffset; both hulls extend 0.15m below the anchor.
Host probes are unparented, with exact hull dimensions, zero centre and unit world scale.
Use fixed `Rust.Layers.Server.PlayerMovement` and `QueryTriggerInteraction.Ignore`.
Ignore only the exact native rider/board roots and descendants, never a client-selected mask.
Check initial overlap separately: touching is allowed; positive penetration blocks.
A non-convex mesh overlap is conservatively refused because penetration ignores backfaces.
Rust terrain-filter ambiguity is refused, not converted into an empty world.
Use fixed 128-entry overlap/cast buffers; a full buffer fails even when stored hits are self.
Scan every cast result and choose the nearest non-self blocker; result order is irrelevant.
Zero-distance sweeps still classify initial overlap. Invalid normals/distances fail.
Native contacts have RailId 0; this adapter cannot capture a rail until trusted binding exists.
Reject anchor/end/centre float spacing above Skin/8 and sweeps above 16m; 4096m is refused.
Near-unit normals are normalized, never flipped; zero-distance tangent/receding contacts are skipped.
Selected normals also oppose the original core displacement, not only rounded native travel.

## Verification and remaining acceptance

Original net48/C#7.3 source builds against genuine server Mono/Unity references: 0 errors/warnings.
Actual DLL linkage confirms three Physics queries, both Rust Verify overloads and the fixed mask.
127 compiled mapping/filter/buffer/precision/core-atomicity cases pass; native calls executed: 0.
The original normal-rounding defect was reproduced in its compiled DLL, then corrected.
Two source reviews pass; parent-shear ambiguity is prevented by unparented probes.
No copied game implementation, fabricated native API, permanent test or game DLL is published.
These checks cannot execute Unity physics outside a running Rust scene.
Real contacts/slopes, terrain holes, concave interiors, probe shape/scale, hierarchy removal,
moving objects and native collision-driven mount/dismount remain a server runtime gate.
Visible board/rider, client input/camera/weapon adapters, rail bindings, replication,
lifecycle integration and complete two-client play remain required by #275.
No unfinished component is a playable release or an owner test request.
