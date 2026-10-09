# Live Rust skate rails

Part of [the full mod](RUST-MW2-SKATE.md); [task #323](https://github.com/Dj-Shortcut/mw2-rust-rust-rewrite/issues/323).
Status: design; implementation and native scene verification pending.

## Server contract

The host selects at most 64 living enabled non-trigger BoxColliders; no client geometry.
`RustSkateRailRegistration` carries a positive rail ID/revision and the exact collider.
`RustSkateRailRegistry.TryReplace` runs only on its captured server main thread.
It revokes the previous lease before validating/publishing a new bounded snapshot.
Invalid replacement leaves no usable lease; explicit Close is irreversible and thread-safe.
World centre comes from the collider local centre and its real transform matrix.
Positive ancestor scales, orthogonal axes and a level yaw-only box are required.
Tilt, shear, mirrored/degenerate scale, inactive hierarchy and invalid geometry are refused.
Existing SkateRailBinding derives top centrelines and enforces catalog/key bounds.
Native signed GetInstanceID keys map only the host-selected collider to its rail.

## Lease and world boundary

`IRustSkateRailLease.Binding` is the immutable exact catalog; IsCurrent validates live state.
A lease permanently expires after collider movement, scaling, rotation, centre/size/layer
change, hierarchy replacement, enable/trigger change, deletion or registry replacement/closure.
Wrong-thread checks refuse before touching Unity; expiry never becomes current again.
RustSkateWorld accepts binding plus an optional lease and requires reference equality.
Existing constructors retain their immutable-snapshot behavior for compatibility.
A leased world checks freshness before queries and again before publishing results.
Hosts pass the world's exact Rails to SkateMotion; no stale catalog may advance motion.
Close the world before intentional rail mutation, then replace registry/world together.
No global physics settings, transforms, persistence, inventory or TC rules change here.

## Verification and unfinished integration

Root will compile against genuine RustDedicated managed references and execute a private,
temporary probe in a real server Unity scene with external game access closed.
Exercise transformed centres, signed identity, valid/invalid boxes, atomic replacement,
move/scale/enable/hierarchy/deletion expiry, wrong-thread rejection and unload cleanup.
Temporary probes, real server DLLs, addresses, credentials and raw logs remain ignored.
No permanent test is added; no rental or paid change is made.
This is server rail lifecycle work, not a visible skateboard or connected player test.
The production host must still bind/tick rider controls, motion and pose with real players.
Original board/rider presentation, input/camera, replication, collision-driven skating,
rails/tricks and complete two-client play remain gates under #275 and the client route #289.
An existing bike or sled does not satisfy the requested board experience.
