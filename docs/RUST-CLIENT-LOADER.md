# Exact Rust Windows client-loader gate

Part of [full acceptance](RUST-MW2-SKATE.md); [task #287](https://github.com/Dj-Shortcut/mw2-rust-rust-rewrite/issues/287).
Status: unmodified client baseline verified; official empty-loader probe pending.
This document is development evidence, not an installation or owner-playtest request.

## Recorded baseline

On 6 October 2026 the local Mac launcher started the existing Shadow PC.
Native Windows mouse and typed commands work; clipboard/modifier input is unreliable.
Existing EAC-disabled RustClient.exe reached responsive Home/Gameplay Options menus
and exited through normal Quit to Desktop; Windows reported no remaining process.
Steam app252490/build25681799; Rust menu166652; Unity6000.3.15x1-13(a91cf34396ee).
Executable/GameAssembly/metadata hashes and this session's log date stay in ignored evidence.
No beta-key line was returned by the installed manifest query; Steam beta UI is unchecked.
Shadow remains client-only; a permitted controllable server is still missing.

## Official candidate and bounded procedure

Candidate: [BepInEx be.788+5b766a3](https://builds.bepinex.dev/projects/bepinex_be), Unity.IL2CPP Windows x64.
Use the [official IL2CPP guide](https://docs.bepinex.dev/master/articles/user_guide/installation/unity_il2cpp.html).
There is no verified support claim for this exact Rust/Unity tuple.
[Interop issue283](https://github.com/BepInEx/Il2CppInterop/issues/283) reports a different x1-8 client;
unmerged generic x1 fixes and another game's startup cannot prove x1-13 compatibility.
First unpack separately, pin the ZIP hash and inspect actual bootstrap contents.
Preserve any existing same-name paths; never overwrite them with the probe.
Start with no plugins or patchers and only inventoried bootstrap files.
Launch only the existing direct RustClient.exe, with no server connection.
Folder-wide bootstrap is not an executable sandbox; avoid other launches during the probe.
Do not modify retail binaries, EAC launchers or game/OS security settings.
Observe logs, a responsive menu and normal exit; generated interop alone is insufficient.
After every outcome move added bootstrap paths back to the separate probe directory,
restore pre-existing paths, verify original hashes and repeat the unmodified baseline.
Only after a passing empty-loader menu/exit consider an original log-only BasePlugin.Load
callback, without game hooks or Unity component injection; it does not prove frame callbacks.
Keep private logs, generated game references, binaries and temporary code outside GitHub.

## Remaining acceptance

Client-loader execution and rollback remain to be observed on the recorded installation.
A loader log marker would not prove weapon/camera/input adapters, visible board/rider,
network movement, lifecycle cleanup or the complete two-client flow.
The owner tests after source completion and our full verification; no intermediate playtest.
Choose/rent no host without owner choice and concrete cost approval.
