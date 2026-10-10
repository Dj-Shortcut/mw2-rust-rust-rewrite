# ShortcutSkate server component

Development source for [#335](https://github.com/Dj-Shortcut/mw2-rust-rust-rewrite/issues/335). This enables scoped movement acceptance; the client supplies its own board/controller. Native Oxide loading and client movement are not yet verified.

Copy only `ShortcutSkate.cs` to `oxide/plugins/` on the owner's matching Oxide test server. Use player F1 commands `skate.toggle`, `skate.on`, `skate.off`, `skate.status`; a bind can use `bind k skate.toggle`. Commands affect only their caller. Turning on requires dry physical support, a connected living awake human, no wound, mount or parent. Death, sleep, disconnect, mount, wound, swimming and permission loss end skating; a 0.1-second sweep covers lifecycle changes without a dedicated hook. Unload also ends every session.

Default config: `{"Version":1,"AllowEveryone":true,"MaximumHorizontalSpeed":14,"MaximumAirtime":1}`.
`AllowEveryone` grants effective access without changing saved Oxide permission grants. Set false to require `shortcutskate.use`, then use `oxide.grant user <SteamID64> shortcutskate.use`. Speed accepts 1–50 m/s; airtime accepts 0.1–2 seconds. Invalid/unknown config fields preserve the file and disable skating. Reload with `oxide.reload ShortcutSkate`.

## Movement boundary and verified server contract

The [current violation hook](https://docs.oxidemod.com/hooks/player/OnPlayerViolation) is `object OnPlayerViolation(BasePlayer, AntiHackType, float, GameObject)`. Any non-null return skips `AntiHack.AddViolation`. `SpeedHack` and `FlyHack` are the enum names (values 2 and 3).
Offline IL inspection of genuine cached Oxide 2.0.7815 and official 2.0.7820 (9 October 2026) proves that `ValidateMoves` separately consumes the completed speed/fly result arrays and selects movement rejection. Cancelling only the violation hook does **not** cancel those reject paths. The public pause methods provide larger finite forgiveness; they do **not** disable the tests.

This plugin uses Oxide's `Oxide.Core.Plugins.AutoPatchAttribute` with Harmony 2.3.1.1. A prefix on `AntiHack.ValidateMoves(in BasePlayer.PlayerServerStates.ReadOnly, NativeArray<int>.ReadOnly, NativeArray<BasePlayer.PositionChange>)` computes one decision per player/batch. Postfixes on `AreSpeeding(in ReadOnly, NativeArray<AntiHack.PlayerSpeedhackState>, NativeArray<int>.ReadOnly, NativeArray<bool>)` and `AreFlying(in ReadOnly, NativeArray<AntiHack.PlayerState>.ReadOnly, NativeArray<AntiHack.PlayerFlyhackState>, NativeArray<int>.ReadOnly, NativeArray<bool>)` clear only that eligible player's speed/fly result. The original checks run first; unrelated results and global convars are untouched. The finalizer closes the batch scope. Missing installed patches disable skating.

Every cached path segment contributes its horizontal length, preventing an out-and-back move from hiding behind its endpoint. The bound is horizontal path length divided by the **server validation interval**, not instantaneous per-packet speed: the cache has no segment timestamps. Intervals over 0.5 seconds, stale/corrected positions, nonfinite input, cumulative time inflation, horizontal excess or vertical path over 40 m/s end skating and receive no exemption.
Physical downward rays sample start, endpoints and interiors at spacing at most 0.1 m on Terrain/World/Construction/Deployed layers, ignore triggers and require a walkable normal. Client `modelState.onground` is never used. Any unsupported sample charges the whole interval as airtime. A fully supported batch resets airtime only after the previous airborne interval passes its limit. This deliberately conservative estimate can end skating on poor support or delayed packets.

`Shortcut.RustMod.SkateSessionBounds` in the same single file exposes `Begin(now,supported)`, `Observe(now,dt,horizontalPath,verticalPath,fullySupported,maxSpeed,maxAir)`, `End()`, `Active`, `AirSeconds` and pure `Eligible(...)`. No engine/server type appears in this class.

## Verification and remaining work

Build with SDK 8: `dotnet build mods/rust/plugins/ShortcutSkate.csproj -p:RustManagedPath=/absolute/genuine/Managed`. All outputs stay in ignored `context/`.
Genuine-reference net48/C#7.3 compilation passes with zero warnings/errors. A private consumer of that exact DLL passes 20,546 bounds/lifecycle/config assertions, including exact speed/airtime boundaries, overspeed, late landing, repeated activation, lifecycle exclusions, stale/timing/vertical/invalid inputs and malformed configuration.
The offline IL/API comparison proves result-array ordering and hook/patch signatures for both inspected server versions. It does not prove runtime Harmony installation, Unity support probes or actual correction-free skating.
TODO: verify native Oxide load, three patch installations, reload/unload restoration, ground/water support and bounded speed/jump acceptance with the client owner. Recheck signatures and IL after server/Oxide updates. No test server connection or client run was performed for this change.

For removal: unload the plugin and delete its `.cs`; stored grants/config can then be removed separately. The plugin changes no global anti-hack settings or saved world data.
