# Shared skateboard motion core

[Issue #277](https://github.com/Dj-Shortcut/mw2-rust-rust-rewrite/issues/277) is a source slice of the [full mod](RUST-MW2-SKATE.md).
This original C#7.3 module lives under `mods/rust/shared/`; no native adapter exists.
It does not resume Bevy, supply a visible board/rider, or satisfy playable acceptance.

## State and authority

Use metres, Y-up, yaw degrees (zero faces +Z), and a board-centre position.
Immutable state holds position/velocity, yaw, airborne spin/flip, grounded/air/bail
mode and previous jump button. No score, player identity, inventory or persistence.
The host queries axis-aligned standing-rider/flat-board boxes including headroom.
Native adapters must also validate actual animated/rotating board and rider geometry.
Queries use `SkateHull` half-extents and centre offset; hit fraction/unit normal are validated. The core derives positions; a packet cannot supply contacts.
A server adapter must bind input to its rider, validate sequence/rate/lifecycle,
query the real world and authorize movement. Client prediction is never authority.
Initial solid overlap differs from zero-fraction contact. Queries are synchronous, side-effect-free and may fail; failures leave state unchanged.

## Contract and controls

`TryMount(position, yaw, world, out state, out error)` requires nearby walkable support.
`TryStep(state, input, dt, world, out next, out events, out error)` copies on success.
`TryDismount(state, world, out feet, out error)` sweeps standing-rider side exits;
returned feet are 0.15m below the supported board anchor. Boxes ignore only skin contact.
Inputs: push/brake/jump buttons and normalized steer/spin/flip axes; native bindings
are deferred until real Rust input is available. Jump uses a rising edge on support.
Push/drag/brake never reverse travel; slope gravity can reverse a stopped board.
Steering follows bounded speed; spin360/flip720 degrees/s permit a flat ollie flip.
Gravity acts along ground travel and in air; sweeps resolve slopes/obstacles.
An upright trick aligned along either board direction lands; hard normal impact or unfinished orientation bails.
Bail stops the core until the host performs a safe dismount or authorized remount.
Dismount prefers either side, never teleports through a blocking sweep or into air.
Finite input/state/query checks, bounded speed/coordinates, and `0 < dt <= 0.05`
reject NaN/infinity, invalid traces and oversized steps atomically. Tuning is provisional.
[Rail capture/travel design](RUST-SKATE-RAILS.md) is tracked separately; native rails, rider animation and fall damage remain unverified.
There is no saved-state format or network protocol; adapters must not deserialize state
from an untrusted client. Runtime cleanup/death/rejoin remains the host's responsibility.

## Verification and native gate

Compile actual production source and exercise it with temporary analytic world fixtures:
push/steer/brake, jump edges, slopes, obstacles, gravity, trick landing/bail, both exits,
and malformed input/query rejection. Fixtures prove core rules, not Rust collision.
Build: `dotnet build mods/rust/shared/SkateMotion.csproj -p:RustManagedPath=...`.
No permanent tests, binary references or probes ship; results belong in [TODO](../TODO.md).
Current [Il2CppInterop #283](https://github.com/BepInEx/Il2CppInterop/issues/283) reports a Rust client startup crash;
[PR #286](https://github.com/BepInEx/Il2CppInterop/pull/286) is unmerged and does not verify Rust compatibility.
An authored BepInEx client remains a candidate requiring an exact-version native smoke;
server DLL compilation cannot prove client rendering, recoil/ADS, input or prediction.
Do not install or deploy this core as an Oxide plugin: it is a library for later adapters.
