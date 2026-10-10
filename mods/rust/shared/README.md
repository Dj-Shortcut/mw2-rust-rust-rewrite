# Physics-driven skate components

Engine-free C#7.3 source for [#335](https://github.com/Dj-Shortcut/mw2-rust-rust-rewrite/issues/335), namespace `Shortcut.RustMod`. These are client integration building blocks; no Unity/Oxide types occur here. Build `SkateMotion.csproj` with a genuine `RustManagedPath` as for the existing shared motion library. The separate [server acceptance plugin](../plugins/SKATE.md) handles scoped movement bounds.

## Drive API

Create `new SkateDriveState(initialYawDegrees)` and keep each successful returned state.
Construct `SkateDriveInput(push, brake, jump, steer, lookYawDegrees)`; Steer is [-1,1], +Z is forward, positive yaw turns toward +X.

```csharp
bool ok = SkateDrive.TryStep(state, input, actualVelocity, grounded,
    groundNormal, dt, maxSpeed, out SkateDriveResult result, out string error);
```

All vectors are `SkateVector`, velocities are world m/s and `dt` is seconds in (0,0.05]. `maxSpeed` is horizontal m/s in (0,14]. Grounded normals must be finite, unit and walkable; airborne may use a zero normal.
`result.State` is the next value state; `DesiredVelocity` drives the body. `BoardYaw`, `BoardPitch`, `BoardRoll` are degrees suitable for Unity `Quaternion.Euler(pitch,yaw,roll)`. `Events` uses existing `SkateEvents.Jumped`, `Landed`, `Blocked`. Rejection returns the supplied state, no events and a static English error. There are no per-call allocations.

Read actual body velocity immediately before each physics step, call once, then assign the desired velocity. **Disable the body's built-in gravity while this controller owns velocity**, since the step applies `SkateMotion.Gravity`; restore it when releasing control. Supply reliable ground support/normal measurements. The step moves no transform and performs no collision/ground query itself. The client must separately toggle the server skate state and restore walk control on exit.

Ground motion projects the achieved horizontal velocity along the heading, applies push, rolling loss, brake and tangent gravity, then returns ground-tangent velocity. A wall's lost velocity is not kept as stored propulsion. `Blocked` signals an achieved-speed shortfall exceeding both 0.1 m/s and 25% of the previous request. Upward slopes stall without reversing; downhill motion accelerates up to the horizontal cap.
LookYaw follows the shortest angular path; look and steer each contribute at most `90 * min(1, observed horizontal speed / 4)` degrees/s, including in air. Both turn rates are zero at rest. Air horizontal velocity remains the actual observed velocity, capped to maxSpeed; vertical velocity receives gravity with a downward fall cap. Air board pitch/roll are zero.
Jump starts only on a fresh grounded press. `TakeoffPending` preserves rising velocity when the support sensor remains true briefly after jumping. `Grounded` records the supplied support observation; `Landed` requires its false-to-true edge after a successful step. Held Jump cannot retrigger on landing.

## Board mesh API

`SkateBoardMesh.Create()` returns fresh arrays: `Positions` and `Normals` are xyz floats, `Triangles` are zero-based indices, `Colours` are rgba floats per vertex. Readonly fields protect references; each caller can edit its own array elements. Create once and cache the mesh; generation allocates.
Metres, +Z forward, +Y up, origin at ground contact centre. The deck is 0.80 x 0.20 m with raised nose/tail and dark grip on top; two metal trucks carry four 0.055 m diameter wheels touching y=0. Overall bounds: x +/-0.102, y 0–0.142, z +/-0.40 m. There are 1,524 vertices and 508 triangles, with flat outward normals and [Unity clockwise front-face winding](https://docs.unity.com/en-us/engine/6000.7/manual/assets-and-media/asset-types/mesh/get-started-with-meshes/anatomyofa/index-data).
Client integration converts triples to engine vectors and quadruples to colours, then assigns the mesh arrays. Use a material/shader that displays vertex colours to see the grip/wood/metal/wheel colours; shader names alone do not prove support. No UVs, textures, submeshes, collider or engine material is supplied.

## Tricks and scoring API

Start with `new SkateTrickState(ridingSwitch)` (default means regular). Call `SkateTricks.TryStep(state, new SkateTrickInput(spin, flip, grab, airborne, grinding, impactSpeed), dt, out result, out error)` once per observation, with axes in [-1,1], seconds in (0,0.05], mutually exclusive airborne/grinding and a finite nonnegative downward impact speed. Capture velocity **before** the physics landing response removes it; ImpactSpeed is only used on the airborne-to-supported transition. Keep `result.State` only on success. Invalid calls retain it and produce no events.

`BoardSpin`/`BoardFlip` are accumulated degrees relative to takeoff, at the existing Motion rates. Spin rotates board and rider; flip rotates only the board about its long axis. On landing, add BoardSpin to the client's persistent board heading, then use the next step's reset angles. Whole flips and half spins within 25 degrees land at impact <=8 m/s; an odd half-spin toggles `result.Switch`. Positive spin names are Frontside, negative Backside; positive flip is Kickflip, negative Heelflip. The adapter maps its physical axes accordingly. Names are static English strings, including combined and Grab variants. More than two whole flips or half-spins use `Multiple Rotation Trick` (with optional Grab); base scoring caps rotations at two.

Events: `Landed`, `Bailed`, `GrindEnded`, `ComboBanked`. `TrickName`/`Points` identify each event; `GrindSeconds` exposes live duration or the completed duration on exit (name `Grind`, 100 points/second, rounded, minimum 1). A supported grind capture can land an air trick and begin the grind in the same call. Each completed trick/grind increases the combo multiplier, capped at 16. `ComboPoints` is open base points times multiplier; 1.5 seconds of plain rolling banks it into `TotalPoints` and reports `BankedPoints` with name `Combo`. Air or grind suspends that rolling timer. Bail discards the open combo and retains the banked total. Call `TryBail` for collision/lifecycle bails outside touchdown; do not manufacture a landing. Totals saturate at 10^15. These value-state calls allocate no memory after static initialization.

Private execution of the actual compiled net48 DLL passes 2,439 assertions for all named spin/flip/grab variants, tolerance and impact boundaries, switch reversal, chaining/timeout, grind duration/capture, explicit and touchdown bails, atomic rejection, plus zero measured bytes across 100,000 valid steps on Mac net8. This does not establish Unity/IL2CPP behavior, controller feel or actual impact sensing.

## Rider joint API

`SkateRiderRig(pelvisHeight, hipWidth, thighLength, shinLength, footLength, spineLength, shoulderWidth, upperArmLength, forearmLength, neckToHeadLength)` stores finite metre measurements in [0.01,5]. Widths are full spans; foot is ankle-to-toe, spine pelvis-to-chest, head neck-to-head. Neck is 0.12*spine above chest. These require the real skeleton's measurements; no Rust dimensions are presumed.

`SkateRiderInput(boardPosition, boardForward, boardUp, stance, ridingSwitch, speed, leanDegrees, crouch, airborne, pushing, pushPhase, flipDegrees, grab, bailed, lookYawDegrees)` uses contact centre within +/-100,000 m, board **nose** forward including spin, and **unflipped** support up (Y>0). The frame is near-unit/perpendicular (squared-length/dot tolerance 0.0001). Pass flip independently; a visual flipped BoardUp would wrongly rotate the rider. Speed [0,14], lean [-20,20] degrees, crouch/phase [0,1], flip/look magnitude <=3,600,000 degrees. Yaw zero looks +Z; positive turns +X.

`SkateRider.TryCreate(rig,input,out pose,out error)` returns world-space value-type `SkateRiderPose`: Pelvis; Left/RightHip, Knee, Ankle, Toe; Chest, Neck, Head; Left/RightShoulder, Elbow, Hand; unit PelvisForward/Up, ChestForward/Up and HeadForward. Failure returns default pose/static English error. The adapter converts chains into bone rotations. The ground plane passes through BoardPosition along BoardUp; a tilted board can place a joint lower in world Y while above that plane.

Regular places left ankle at +SkatePose.FootOffset, goofy at -offset; the other ankle mirrors. Sideways toes retain exact FootLength. Switch keeps anchors fixed relative to the nose, reverses travel and chooses the rear pushing foot. Closed push phase 0/1 matches the deck; the foot swings beside/behind it using SkatePose offsets. Airborne/bail suppress push. Non-whole air flips lift feet up to FlipFootLift; whole flips return them. Grab aims a rear hand toward the deck subject to arm reach. Bail supplies a protective pose; body/ragdoll/lifecycle transitions remain the adapter's job.

Crouch lowers requested pelvis height; a common height interval keeps both legs reachable. Root-to-ankle distance uses the conservative shell sqrt(abs(thigh²-shin²))+1e-6 through thigh+shin-1e-6, avoiding folds through an endpoint. Impossible fixed-foot rig spans reject. Hands clamp into the arm triangle shell; exceptional proportions use a ground-safe horizontal balance target. Analytic two-bone IK preserves exact segment lengths and adjusts poles to keep knees/elbows above the plane. Grabs need not touch the board if unreachable. Every call is allocation-free after initialization.

Root reran the integrated net48 DLL: 4,085,197 assertions, 49,744 accepted random rig/input cases, 256 rejected impossible spans; 512 accepted/512 rejected min/max corners; exact limb/span/ground/orientation, stance/switch, seams and 2,000 continuity pairs. One million valid and one million invalid warm calls each allocated zero measured bytes on Mac net8. Independent geometry review passes. Actual skeleton application, IL2CPP allocation and in-game pose remain unverified.

## Verification boundary

Genuine Mono-reference net48/C#7.3 compilation passes with zero warnings/errors. Private Mac net8 execution of the actual net48 library passes 266,150 assertions: flat push to max, coasting distance, braking, downhill gain/uphill stall, wall block, jump/landing and stale support, steering rates, gravity, 360 headings, deterministic sampled steps and board-normal reconstruction. Valid and invalid 100,000-call loops each allocate zero measured bytes after warm-up.
The board passes 31,323 private assertions covering bounds, indices, normal length, deterministic independent arrays, wheel diameter/contact, truck attachment, manifold solids and every triangle facing outward relative to its own component. A software top/underside preview confirms projected clockwise front faces; this is no Rust render.
These checks do not prove Unity physics/materials, IL2CPP/Mono runtime allocation, controller feel, network movement acceptance or Rust client gameplay. Probes, logs and binaries stay in ignored `context/`.
