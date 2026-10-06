# Exact Rust Windows client-loader gate

Part of [full acceptance](RUST-MW2-SKATE.md); [diagnosis #289](https://github.com/Dj-Shortcut/mw2-rust-rust-rewrite/issues/289).
Status: official empty-loader probe failed; native recovery verification remains open.
This is development evidence, not installation or an owner-playtest request.

## Actual Windows observations, 6 October 2026
The local Mac launcher reached Shadow Windows with working mouse/typed commands.
Bare EAC-disabled RustClient.exe reached responsive Home/Options and exited normally,
with no remaining process: Steam252490/build25681799, menu166652,
Unity6000.3.15x1-13(a91cf34396ee). Steam beta UI selection remains unchecked.
[Official BepInEx be.788+5b766a3](https://builds.bepinex.dev/projects/bepinex_be) Windows x64 ZIP:
SHA256 F4CC496BD098A0DF4164B81E3737297707F13A47C2478DBA2F60EEFAB784817A.
No pre-existing bootstrap paths; plugins/patchers empty; default configuration unchanged.
Loader menus responded and Quit to Desktop left no process, but generation failed:
Cpp2IL metadata39, IndexOutOfRangeException at RawPropertyType line44;
interop directory empty, then missing UnityEngine.CoreModule and fatal no-plugin chainloader.
These stages are observed; the invalid accessor/index's actual cause is not established.
All six added roots were moved to private staging; four original retail hashes matched.
The bare client restarted with responsive menus; native control then timed out during exit.
Its final exit and private transcript closure were not observed; recovery remains open.
No authored plugin, server connection, retail-binary edit or security-setting change occurred.

## Bounded recovery and next diagnosis

Follow the [official IL2CPP guide](https://docs.bepinex.dev/master/articles/user_guide/installation/unity_il2cpp.html).
Inventory before every attempt; preserve same-name paths and never overwrite them.
The bootstrap covers the game folder, not one executable; avoid other launches during probes.
Default generation downloads Unity libraries and creates private config/cache/interop files.
Archive the whole added BepInEx/dotnet trees, four root files and any new preloader_*.log
only after Rust stops; restore prior paths, compare hashes and repeat bare menu/normal exit.
Keep private logs, generated references, binaries and probe code outside GitHub.
Use genuine pinned DLLs, recorded hashes/versions and original fixtures in isolated processes.
Validate table strides, owners and parameter spans before interpreting accessor failures.
Fixture-only cache/reflection scaffolding must be explicit; it does not prove Rust metadata.
Present-but-invalid accessors must fail explicitly; do not guess a signature or swallow errors.
[Upstream property guards](https://github.com/SamboyCoding/Cpp2IL/pull/548) were closed without merge;
this is a research lead, not an accepted fix or proof of our exact cause.
Review any coordinated generator correction and its licence before another native attempt.
Only a passing empty-loader generation/menu/normal-exit/rollback permits a log-only Load probe.
That callback still does not prove Unity frame hooks, client input or native game adapters.

## Remaining acceptance

No compatible client extension or complete mod is verified on this exact installation.
A loader marker cannot prove weapon/camera/input adapters, visible board/rider,
network movement, lifecycle cleanup or the complete two-client flow.
Shadow remains client-only; a permitted controllable testserver is still missing.
The owner tests after source completion and our full verification; no intermediate playtest.
Choose/rent no host without owner choice and concrete cost approval.
