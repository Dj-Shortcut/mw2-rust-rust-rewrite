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
A reentrant freshness read refuses without expiring a still-valid lease; the outer
read retains responsibility for observing mutation and registry closure.
A mutation restored between freshness reads is undetectable; hosts must revoke before mutation.
RustSkateWorld accepts binding plus an optional lease and requires reference equality.
Existing constructors retain their immutable-snapshot behavior for compatibility.
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
One additional disabled-probe ComputePenetration check failed; it is an existing adapter
precondition gate, not a passing skating result. Its focused diagnosis remains separate.
A focused review correction separates reentry refusal from permanent registry closure.
The unchanged old source reproduced valid-lease expiry after private checking-state
injection (8 checks pass, 1 fails); the corrected source passes all 9 focused checks
on a genuine owned rail/lease, including actual mutation/restoration/replacement
and closure. Both owned objects were actually absent after their separate runs;
settings unchanged and both probes unloaded with their own source removed.
The current-reference module and composed probe compile with zero warnings/errors.
This is guard-state injection, not a naturally observed Unity callback reentry.
Corrected native report SHA256:
`92b33c4506b810478832b45404995314b58f683b3b24ae3acf85bb7225d38b5b`.
Temporary probes, real server DLLs, addresses, credentials and raw logs remain ignored.
No permanent test is added; no rental or paid change is made.
This is server rail lifecycle work, not a visible skateboard or connected player test.
The production host must still bind/tick rider controls, motion and pose with real players.
Original board/rider presentation, input/camera, replication, collision-driven skating,
rails/tricks and complete two-client play remain gates under #275 and the client route #289.
An existing bike or sled does not satisfy the requested board experience.
