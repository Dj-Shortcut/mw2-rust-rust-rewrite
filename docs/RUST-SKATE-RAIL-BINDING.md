# Trusted rail collider binding

Part of [the full mod](RUST-MW2-SKATE.md); [task #298](https://github.com/Dj-Shortcut/mw2-rust-rust-rewrite/issues/298).
Connects the [rail core](RUST-SKATE-RAILS.md) to [native world queries](RUST-SKATE-WORLD.md).
Shared binding and native adapter wiring compile and pass local checks; native scene/host integration remains unverified.

## Binding snapshot

`SkateRailBinding.TryCreate` copies at most 64 rails and 128 host collider keys.
Keys are nonzero host values, for example a collider instance ID chosen by the server.
Each key is unique and names one catalog rail; every rail needs at least one collider.
`Rails` is the exact catalog to pass to `SkateMotion.TryStep` with that world binding.
`Resolve(key)` returns the bound rail ID, or 0 for an unbound or zero key.
The snapshot is immutable. Moving, revising or removing a rail means a new snapshot;
the core's stale-rail checks then release the rider at the current pose.
Client packets never supply keys, rail IDs or geometry.

`TryRailFromBox` derives a rail from a level box: world centre, scaled local
half extents and yaw about +Y (zero faces +Z). The top centreline runs along the
longer horizontal local axis. Boxes wider than 0.6m, square footprints, invalid
values and lengths outside 0.25–100m are refused. The host must check the actual
collider has no pitch or roll. Width and box rules are provisional tuning.

## Contact identity

`SkateWorldContact` has a new constructor with `RailId`; the old one still means 0.
`SkateWorldQuery.TrySelect` refuses negative IDs and carries the selected contact's ID.
Another non-self blocker at the same nearest distance with a different ID makes
the result an ordinary contact (ID 0), never a capture. Self contacts are ignored.
With only ID-0 contacts the result is unchanged from before.

`TryPrepare` now checks a move that rounds to the same native point as a
zero-distance overlap instead of failing. The rail seat snap produces such moves:
the old rule failed the whole skate step at the first real capture.
Moves above Skin/8 that collapse to zero are still refused.

## Native wiring

[Task #301](https://github.com/Dj-Shortcut/mw2-rust-rust-rewrite/issues/301) wires
`RustSkateWorld` to this immutable binding. The new constructor rejects null and
its `Rails` getter returns the exact catalog; the old constructor uses Empty.
Only after collider, self and Rust verification does a non-self cast contact map
its signed native instance ID through `Resolve`. Self/unbound contacts remain 0;
penetration never supplies a rail ID. Existing contact selection/revocation guards remain.

The live host must build the registration and pass this world's exact `Rails` to
`SkateMotion.TryStep`. Close/rebuild the world and snapshot before any bound collider
transform/scale/enabled-state change, removal/replacement, or rail geometry/revision
change. The adapter does not create registrations or detect stale host snapshots.
No production host yet creates/ticks the world; native physics/lifecycle, registration,
client presentation and connected two-client gameplay remain separate gates.

## Verification and limits

The initial shared-only `SkateMotion` and `RiderControlSession` checks built with 0
warnings/errors against .NET Framework 4.8 reference assemblies. Those checks used
framework reference assemblies rather than genuine server Mono references.
78 temporary probes against the built DLLs pass: binding bounds and copying, box
maths including rotation, ID propagation and ambiguity, 20,000 randomized legacy
`TrySelect` comparisons identical to main, and capture, grind, release, rotated,
revision-change and lost-binding rides through an analytic world. The previous query
was shown to fail the capture step. No native query, server, client or permanent test ran.

The native wiring now builds net48/C#7.3 against genuine RustDedicated/Mono/Unity
references with 0 warnings/errors. 170 actual query/core/binding behavior checks and
44 passive compiled-linkage checks pass, with 269 source/reference pins unchanged.
The signed identity flow, legacy constructor/null rejection, exact catalog and
unchanged native guards are verified locally. These checks invoke no native Unity
physics or world lifecycle and do not establish a running host or playable mod.
