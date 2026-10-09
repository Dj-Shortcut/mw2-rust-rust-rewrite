# Native player input for rider controls

Part of [the full existing-Rust mod](RUST-MW2-SKATE.md); [task #329](https://github.com/Dj-Shortcut/mw2-rust-rust-rewrite/issues/329).
Status: design; genuine input reader and native decoding verification pending.

## Scope and admission

RustRiderInput binds one exact connected/alive BasePlayer to a trusted host-issued
RiderIdentity. The production host calls it only inside that player's genuine
OnPlayerInput callback. Require the exact callback player and InputState reference
equal to player.serverInput, valid current input message and the host's current
runtime/life identity. A Guid fences an existing authenticated life; it does not
authenticate a caller. Close on death, disconnect, respawn, reload or stop.

Capture on the server main thread, with reentry/closure/publication guards. Copy
held button values immediately and retain no InputState/InputMessage. Refuse
invalid identities, stale/duplicate host slots and uncertain native observations.
Generate increasing positive request sequences on the host. Copy produces the
existing RiderControlRequest; it grants no mode, board, weapon or effect authority.

## Basic controls and clock boundary

Read held FirePrimary, FireSecondary, Reload and Jump levels, preserving the
shared RiderControlSession's explicit release gates. Forward maps to Push,
Backward to Brake and Right-minus-Left to Steer. Spin/flip stay zero in this
basic reader; native trick controls are a separate unfinished gate.

The callback is received player input, not a guaranteed 100Hz clock. The host
supplies its current slot and still owns the exact 10ms session clock and freshness
policy. Never replay one packet as new samples, admit backlog, derive a clock or
sequence from client data, or silently change missing-input release behavior.
Reader code does not install a hook or suppress normal Rust input processing.

## Verification and remaining product work

Root builds against the genuine current server references. Temporary ignored
checks may exercise genuine InputState/button decoding and refusal logic. A fake
player cannot prove authenticated callback capture; that still requires a real
connected player. Keep observed input decoding distinct from native hook success.

No board/model is selected or created here. Mount, mode observation, rider/board
movement, camera/animation, trick input, replication, reconciliation, client/model
delivery and the full two-client #275 flow remain open. ShortcutLoadouts and
survival/TC behavior remain unchanged; no permanent test or paid change.
