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

## Verification boundary

Genuine Mono-reference net48/C#7.3 compilation passes with zero warnings/errors. Private Mac net8 execution of the actual net48 library passes 266,150 assertions: flat push to max, coasting distance, braking, downhill gain/uphill stall, wall block, jump/landing and stale support, steering rates, gravity, 360 headings, deterministic sampled steps and board-normal reconstruction. Valid and invalid 100,000-call loops each allocate zero measured bytes after warm-up.
These checks do not prove Unity physics, IL2CPP/Mono runtime allocation, controller feel, network movement acceptance or Rust client gameplay. Probes, logs and binaries stay in ignored `context/`.
