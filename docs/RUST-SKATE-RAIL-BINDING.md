# Trusted rail collider binding

Part of [the full mod](RUST-MW2-SKATE.md); [task #298](https://github.com/Dj-Shortcut/mw2-rust-rust-rewrite/issues/298).
Connects the [rail core](RUST-SKATE-RAILS.md) to [native world queries](RUST-SKATE-WORLD.md); shared source only, native adapter not wired.

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

## Native wiring (not done)

`RustSkateWorld` should accept a binding through a new constructor overload and
pass `Resolve(key)` for every non-self cast hit. That needs genuine RustDedicated
references and is left for the native server owner; until then native contacts stay ID 0.

## Verification and limits

`SkateMotion` and `RiderControlSession` build with 0 warnings/errors against .NET
Framework 4.8 reference assemblies; the genuine Mono references were not available.
78 temporary probes against the built DLLs pass: binding bounds and copying, box
maths including rotation, ID propagation and ambiguity, 20,000 randomized legacy
`TrySelect` comparisons identical to main, and capture, grind, release, rotated,
revision-change and lost-binding rides through an analytic world. The previous query
was shown to fail the capture step. No native query, server, client or permanent test ran.
