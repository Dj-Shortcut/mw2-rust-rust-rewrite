# Native player input for rider controls

Part of [the full existing-Rust mod](RUST-MW2-SKATE.md); [task #329](https://github.com/Dj-Shortcut/mw2-rust-rust-rewrite/issues/329).
Status: genuine reader source and bounded native decoding verified; live capture pending.

## Scope and admission

RustRiderInput binds one exact connected/alive BasePlayer to a trusted host-issued
RiderIdentity. Create it on the server main thread and call only inside that
player's genuine OnPlayerInput callback: these remain caller obligations, not
facts established by a captured thread ID or matching object references.
Require the exact callback player and InputState equal to player.serverInput,
valid current message and current runtime/life identity. A Guid fences an already
authenticated life; it does not authenticate. Close before death, disconnect,
respawn, reload or stop. Reentry, identity changes and uncertain publication close.

Copy held buttons immediately and retain no mutable InputState/InputMessage.
Refuse stale/duplicate host slots, unknown bits and uncertain native observations.
Issue positive increasing request sequences only after successful publication.
RiderControlRequest grants no mode, board, weapon or effect authority.

## Basic controls and clock boundary

Read held FirePrimary, FireSecondary, Reload and Jump, preserving the shared
session's explicit release gates. Copy raw held levels; if genuine IsDown reports
any supported button swallowed, refuse the whole sample. Never fabricate a
release or grant a stock-denied intent. Forward maps to Push, Backward to Brake,
Right-minus-Left to Steer. Spin/flip stay zero; trick input remains unfinished.
The callback is received input, not a guaranteed 100Hz clock. The host supplies
its current slot and owns the exact 10ms clock and freshness policy. Do not
replay packets, admit backlog, use client clocks/sequences or change missing-input
release behavior. [Packet-silence admission](RUST-INPUT-FRESHNESS.md) is separate. No hook/input suppression.

## Actual verification and remaining product work

The net48/C#7.3 project builds against 262 genuine current server references:
0 warnings/errors and all 273 authored source/project/reference hashes unchanged.
An ignored, separately composed Oxide diagnostic used the exact reader source:
20 checks passed, 0 failed; all 256 held combinations matched genuine IsDown,
repeated controls stayed held when genuine WasJustPressed reported no new edge;
all eight supported swallowed buttons retained raw bits but were absent from effective input. Decoding
preserved message references/buttons/effective state. Release, unmapped buttons,
actual enum-mask and static invalid binding refusals also passed.
Report SHA256: `0fe6a576de57c080eb1cc3daa7f7457cb5d35fcd1e4df85dfb7efda6fe754f63`.
No native object created; physics settings unchanged; diagnostic unloaded and
its source removed. No fake player, connected check or authenticated capture.
Instance slot/sequence/lifecycle success and genuine hook/main-thread provenance
still require a connected player and the production host; decoding is no playtest.
Mount/mode, movement, model, camera, tricks, replication, reconciliation and client
route remain open. Loadout/survival/TC behavior unchanged; no permanent test added.
