# Rust PC skate client architecture

Scope: existing Rust PC game; the standalone rewrite remains parked. Sources are
[#337](https://github.com/Dj-Shortcut/mw2-rust-rust-rewrite/issues/337) and Claude's [in-world report in #289](https://github.com/Dj-Shortcut/mw2-rust-rust-rewrite/issues/289#issuecomment-6093680035), dated 10 October 2026, client Steam build 25824447.
These client observations were reported by Claude/the owner, not rerun by Codex.

## Reported working route

BepInEx IL2CPP loaded in a world through the owner's Steam shortcut. Game code is
obfuscated; engine APIs, component type names and Unity message patches remain usable.
The kinematic BasePlayer follows a separate `assets/prefabs/player/player_movement.prefab`
with PlayerWalkMovement, dynamic Rigidbody and CapsuleCollider. Keep the walk component
enabled: writing velocity after the game's 32 Hz fixed step moves the real body;
the game retains gravity, jumping and model handling. Disabling the component caused pullback.
The measured movement capsule (1.8 m high, 0.5 m radius) is not a skeleton measurement.
The local v0 has push/brake/view steering/slope motion, a primitives board and walk animation.
2.5 m/s for three seconds moved about 7.4 m without pullback; higher-speed runs had corrections.
The cause remains unexplained; server source changes do not establish a correction-free result.

Legacy Unity Input is unavailable; InputSystem Keyboard worked. The owner reported controller
keys through Steam Input on his shortcut. Direct executable launch reported no Gamepad.
Raycast/spherecast worked; stripped methods can throw. Probe each new mesh, material, audio,
text, Animator or camera call on the matching build. Shader discovery alone proves no material.
Interop must be regenerated after a Rust update; this world-load report does not close the
separate [generator/loader acceptance gates](RUST-CLIENT-LOADER.md).

## Shared integration contract and limits

[Shared APIs](../mods/rust/shared/README.md) now provide Drive, BoardMesh, Tricks, Rider,
Edges and Audio, with private execution of genuine-reference net48 DLLs. These offline
checks do not prove native integration, appearance, playback, feel or IL2CPP allocation.
For the reported game-owned vertical route, a Drive adapter must preserve achieved Y/game
jump, pass Jump=false and consume only desired X/Z after the game step. This adaptation
still needs client verification; full XYZ ownership requires one exclusive gravity/jump owner.
Feed actual support transitions and pre-contact impact to Tricks; apply spin to rider/board,
flip only the board. Keep Drive travel heading separate from nose heading on switch; measure/map the rig.
Edges needs real sampled heights plus contact/proximity/upright/clearance checks before capture;
Audio needs probed playback and cached clips. No pose, trick, grind, HUD or sound integration is proven.
[Server acceptance](../mods/rust/plugins/SKATE.md) defaults to automatic bounded eligibility;
the source's warmup/opt-out/practice behavior and native patch/cleanup still need real-server checks.

## Remaining acceptance

Claude owns client M1–M5: lifecycle/failure cleanup, native API probes, shared integration,
owner-run launcher and server verification. Restore owned body/model/input/camera/audio state
on dismount, bail, death, respawn, reconnect, swim, mount, unload and initialization failure.
K mounting was reported working; double-jump mounting, tricks, grind/bail and scoring remain targets.
The launch/install/remove/update/crash-recovery procedure awaits Claude's M4; none is supplied here.
Codex item 7 reviews the client PR when opened. Second-client observation remains open with one
available client/account. [Full gameplay acceptance](RUST-MW2-SKATE.md) and [TODO](../TODO.md) remain authoritative.
