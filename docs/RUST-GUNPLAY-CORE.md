# Shared gunplay timing and handling core

[Issue #279](https://github.com/Dj-Shortcut/mw2-rust-rust-rewrite/issues/279) is a source slice of the [full Rust mod](RUST-MW2-SKATE.md).
Planned original C#7.3 source under `mods/rust/shared/`; no native adapter yet.
This design does not supply complete MW2 gunplay or a playable mod.

## Host contract

Use a fixed 10ms simulation tick and validated original profiles with integer
fire/equip/reload/ADS/sprint-recovery ticks. Values are provisional authored tuning,
not copied MW2 data. The native scheduler owns time and caps catch-up work;
network packets cannot advance the clock or supply ammo, poses, hits or damage.
Immutable state is per weapon instance. The host retains it for each item UID
through switches, binds requests to its actual owner and checks game conditions.
`TryCreate` initializes trusted actual ammo; it is not a per-packet reset API.
`TryBeginEquip` cancels reload/ADS and requires trigger release while preserving
ammo, shot sequence and fire cooldown. Inactive states keep their timing ledger.
`TryTick` returns a candidate state and optional shot intent; rejection is atomic.
The native owner must commit actual Rust ammo/native firing and candidate state
as one operation, or discard the candidate. Do not decrement stock ammo twice.
There are no item transfers, save format, damage application or native hooks here.

## Controls and order

Host-confirmed walking/sprinting/skating/unavailable mode and moving/airborne
facts govern handling. Fire, aim and reload are boolean requests only.
Sprinting/skating/unavailable cancel reload and ADS; firing needs trigger release
and a bounded recovery after returning to walking. Native survival remains Rust's.
Timers advance first; equip completion then reload completion precede requests.
Reload uses a rising edge and takes priority over fire. Ammo moves only when
reload completes, limited to magazine space and available reserve; no free ammo.
Semi fire uses a rising edge; auto uses held fire after release arms the trigger.
At most one shot per tick, then the full cooldown is applied; no burst catch-up.
Equip/reload/recovery block shots. Empty fire consumes no ammo or shot sequence.
ADS ramps over the configured duration; recoil/spread and movement-scale outputs
are deterministic handling targets for later native presentation and authority.
Recoil accumulates bounded authored pitch/yaw kicks and decays; moving/airborne
spread remains bounded. Native adapters must also respect stock weapon semantics.

## Verification and limits

Compile actual production source against genuine framework references; temporary
probes exercise timing boundaries, semi/auto edges, reload conservation, switching,
mode transitions, handling outputs and malformed-input rejection. No fake game APIs.
No permanent tests, game binaries, offsets or probes ship. Record evidence in [TODO](../TODO.md).
Build target will be `Gunplay.csproj`; source and verification are not present yet.
Client IL2CPP compatibility, real recoil/ADS/input, inventory transactions, native
firing, survival interactions, networking and two-client play remain unverified.
The installed Shadow RustClient executable reports Unity6000.3.15x1-13;
that is engine metadata, not a complete Rust/loader compatibility tuple.
