# Shared board and rider pose

Part of [the full mod](RUST-MW2-SKATE.md); [task #307](https://github.com/Dj-Shortcut/mw2-rust-rust-rewrite/issues/307).
Turns a [skate core](RUST-SKATE-CORE.md) state into how the board and rider should look.
Shared source only: no entity, animation, rendering or network adapter exists.

## API

`SkatePose.TryCreate(state, input, events, stance, out pose, out error)` is a pure
function of one valid `SkateState`, the input and events of the same step, and
`SkateStance.Regular` or `Goofy`. It uses the core's own state checks and refuses
non-finite or out-of-range axes, unknown event bits and unknown stances.
It adds no state, persistence, network format or authority: the server still owns
the skate state, and a client may only render a pose derived from it.

## Board

Metres, Y-up, yaw zero faces +Z, as in the core. `BoardPosition` is the core position.
`BoardForward`, `BoardUp` and `BoardRight` are a unit basis with right = up x forward
(Unity's convention). `BoardRotation` is the matching quaternion (x, y, z, w with w >= 0),
so `rotation * (0,0,1)` is forward under Unity's `Quaternion * Vector3`.
Grounded and bailed boards follow the ground normal with the nose along the yaw.
Airborne boards turn by yaw plus spin about world up, then roll by flip about
their long axis; positive flip rolls up toward right. Grinding boards stay level.

## Rider

Regular puts the left foot at the nose and faces board right (heading + 90);
goofy puts the right foot at the nose and faces board left.
Feet sit 0.21m either side of the centre, 0.01m above the board position along its up.
During a flip they hold 0.12m above the unflipped board instead of following it.
Pushing (grounded, push held, no brake) puts the back foot on the ground beside the
toe edge, 0.25m behind centre, with the hip over the front foot.
`Hip` is the feet centre plus world up times 0.95m minus 0.4m per unit crouch.
Crouch: landing 0.7, trick or trick input in air 0.6, plain air 0.45, grind 0.35,
brake 0.3, riding 0.15, bail 0. `Lean` is steer x 20 degrees, scaled by speed up
to 4 m/s, toward board right; only on the ground. `Bailed` asks the adapter to ragdoll.
All offsets and crouch values are provisional tuning, not measured animation.

## Verification and limits

`SkateMotion` (now including `SkatePose.cs`) and `RiderControlSession` build with
0 warnings/errors against .NET Framework 4.8 reference assemblies; genuine
RustDedicated Mono references were not available in the cloud session.
Temporary probes against the built DLL pass: identity and yaw-90 quaternions,
quaternion-rotated axes equal the basis for 20,000 random grounded, airborne and
bailed states, slope, flip and spin geometry, both stances, pushing, lean,
crouch per mode, grind, bail, refusals, and two 400-step rides through the real
core on flat and sloped planes with a landed kickflip. No native entity, animation,
client, replication or play test ran; no permanent tests ship.
