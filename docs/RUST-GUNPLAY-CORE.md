# Shared gunplay timing and handling core

[Issue #279](https://github.com/Dj-Shortcut/mw2-rust-rust-rewrite/issues/279) is a source slice of the [full Rust mod](RUST-MW2-SKATE.md).
Original C#7.3 source under `mods/rust/shared/`; no native adapter yet.
This core does not supply complete MW2 gunplay or a playable mod.

## Host contract

Use a fixed 10ms simulation tick and validated original profiles with integer
fire/equip/reload/ADS/sprint-recovery ticks. Values are provisional authored tuning,
not copied MW2 data. The host owns time, caps catch-up and suppresses historical input;
network packets cannot advance the clock or supply ammo, poses, hits or damage.
Immutable state is per weapon instance; `TryTick` requires the next exact host tick. The host retains it for each item UID
through switches; bind ownership, reconcile stock before `TryTick`, revalidate at commit.
`TryCreate` initializes trusted actual ammo; it is not a per-packet reset API.
`TryBeginEquip` cancels reload/ADS and requires trigger release while preserving
ammo/sequence/cadence. Advance every holstered UID each host tick as unavailable.
`TryTick` returns a candidate state and optional shot intent; rejection is atomic.
The native owner must commit actual Rust ammo/native firing and candidate state
together; discard failed candidates, then settle that same tick with no input/unavailable.
`TryReconcileAmmo` accepts trusted stock, cancels changed reloads and preserves cadence.

## Controls and order

Host-confirmed walking/sprinting/skating/unavailable mode and moving/airborne
facts govern handling. Fire, aim and reload are boolean requests only.
Sprinting/skating/unavailable cancel reload and ADS; firing needs trigger release
and a bounded recovery after returning to walking. Native survival remains Rust's.
Cancellation precedes reload transfer; timers/completion then precede requests.
Valid reload uses a rising edge and precedes fire. Ammo moves only when
reload completes, limited to magazine space and available reserve; no free ammo.
Semi edges while blocked are consumed; armed auto may fire on reload completion.
At most one shot per tick, then the full cooldown is applied; no burst catch-up.
Equip/reload/recovery block shots. Empty fire consumes no ammo or shot sequence.
ADS ramps over the configured duration; recoil/spread and movement-scale outputs
are deterministic handling targets for later native presentation and authority.
ADS endpoints are exact; shot kicks are clamped deltas, handling is absolute; moving/airborne
spread/recoil are bounded. Apply recoil once; native adapters must respect stock semantics.

## Verification and limits

Compile actual production source against genuine framework references; temporary
probes exercise timing boundaries, semi/auto edges, reload conservation, switching,
mode transitions, handling outputs and malformed-input rejection. No fake game APIs.
No permanent tests, game binaries, offsets or probes ship. Record evidence in [TODO](../TODO.md).
Build: `dotnet build mods/rust/shared/Gunplay.csproj -p:RustManagedPath=...`.
Client IL2CPP compatibility, real recoil/ADS/input, inventory transactions, native
firing, survival interactions, networking and two-client play remain unverified.
The installed Shadow RustClient executable reports Unity6000.3.15x1-13;
that is engine metadata, not a complete Rust/loader compatibility tuple.
