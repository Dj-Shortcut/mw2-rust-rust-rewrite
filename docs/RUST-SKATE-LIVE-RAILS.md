# Live Rust skate rails

Part of [the full mod](RUST-MW2-SKATE.md); [task #323](https://github.com/Dj-Shortcut/mw2-rust-rust-rewrite/issues/323).
Status: implemented; live rail checks pass on the real server; connected skating remains unverified.

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
A lease permanently expires on observed collider movement, scaling, rotation, centre/size/layer
change, hierarchy replacement, enable/trigger change, deletion or registry replacement/closure.
Wrong-thread checks refuse before touching Unity; expiry never becomes current again.
A mutation restored between freshness reads is undetectable; hosts must revoke before mutation.
RustSkateWorld accepts binding plus an optional lease and requires reference equality.
At the #323 checkpoint, existing constructors retained their immutable snapshot.
The [#325 probe migration](RUST-SKATE-PROBES.md) keeps those signatures but makes
raw-probe queries refuse; use `WithProbes` and the owned factory for native queries.
A leased world checks freshness before queries and again before publishing results.
Hosts pass the world's exact Rails to SkateMotion; no stale catalog may advance motion.
Close the world before intentional rail mutation, then replace registry/world together.
No global physics settings, transforms, persistence, inventory or TC rules change here.

## Verification and unfinished integration

The current genuine RustDedicated/Oxide 262-reference build has zero warnings/errors.
A private temporary probe executed in the real externally closed Rust scene: 117 checks pass.
They cover transformed centres, real signed IDs, 64 rails/128-ancestor bounds, native BoxCast/
Overlap/Rust Verify, replacement/observed expiry, worker refusal, world freshness guards
and actual deletion of all 230 owned objects. No connected rider or world movement ran.
At that checkpoint one additional disabled-probe ComputePenetration check failed.
The separate #325 correction now uses owned parked probes with observed activation
and restoration. Its real overlap/contact/cleanup results are recorded in the
[probe guide](RUST-SKATE-PROBES.md); connected collision-driven skating is still unverified.
Temporary probes, real server DLLs, addresses, credentials and raw logs remain ignored.
No permanent test is added; no rental or paid change is made.
This is server rail lifecycle work, not a visible skateboard or connected player test.
The production host must still bind/tick rider controls, motion and pose with real players.
Original board/rider presentation, input/camera, replication, collision-driven skating,
rails/tricks and complete two-client play remain gates under #275 and the client route #289.
An existing bike or sled does not satisfy the requested board experience.
