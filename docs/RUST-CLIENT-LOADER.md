# Exact Rust Windows client-loader gate
Part of [full acceptance](RUST-MW2-SKATE.md) / [diagnosis #289](https://github.com/Dj-Shortcut/mw2-rust-rust-rewrite/issues/289): official loader failed, bare recovery passed.
This is development evidence, not installation or an owner-playtest request.
## Actual Windows observations, 6 October 2026
The local Mac launcher reached Shadow Windows with working mouse/typed commands.
Bare EAC-disabled RustClient.exe reached responsive Home/Options and normal exit:
no remaining process; Steam252490/build25681799, menu166652,
Unity6000.3.15x1-13(a91cf34396ee). Steam beta UI selection remains unchecked.
[Official BepInEx be.788+5b766a3](https://builds.bepinex.dev/projects/bepinex_be) Windows x64 ZIP:
SHA256 F4CC496BD098A0DF4164B81E3737297707F13A47C2478DBA2F60EEFAB784817A.
No pre-existing bootstrap paths; plugins/patchers empty; default configuration unchanged.
Loader menus responded and Quit to Desktop left no process, but generation failed:
Cpp2IL metadata39, IndexOutOfRangeException at RawPropertyType line44;
interop directory empty, then missing UnityEngine.CoreModule and fatal no-plugin chainloader.
Read-only official LoadFromFile on actual binary/metadata returned true.
A real getterless zero-parameter setter reproduced RawPropertyType array bounds,
without cache injection; the saved exception names the official DLL/source method.
All six added roots were moved to private staging; four original retail hashes matched.
An interrupted recovery was followed by a fresh-start hang logging NoSteamClient.
With Steam running, Home/Options, normal Quit to Desktop and no remaining process passed;
six bootstrap roots were absent, original hashes matched and the new transcript closed.
No authored plugin, server connection, retail-binary edit or security-setting change occurred.
## Bounded recovery and next diagnosis
Follow the [official IL2CPP guide](https://docs.bepinex.dev/master/articles/user_guide/installation/unity_il2cpp.html).
Inventory before every attempt; preserve same-name paths and never overwrite them.
For every probe launch only the existing direct EAC-disabled RustClient.exe, offline.
Do not connect to a server; the folder-wide bootstrap must stay out of Steam/EAC launches.
Default generation downloads Unity libraries and creates private config/cache/interop files.
Archive the whole added BepInEx/dotnet trees, four root files and any new preloader_*.log
only after Rust stops; restore prior paths, compare hashes and repeat bare menu/normal exit.
Keep private logs, generated references, binaries and probe code outside GitHub.
Use genuine pinned DLLs, recorded hashes/versions and original fixtures in isolated processes.
Validate table strides, owners and parameter spans before interpreting accessor failures.
Five original mechanism fixtures used explicit cache scaffolding; actual Windows
metadata parsing now separately found 175 empty and 938 nonempty getterless setters.
Checked owner/accessor/parameter relationships passed; full byte geometry is pending.
[Correction source/build](../mods/rust/client/generator/README.md) preserves indices/methods;
omits only noninferable wrappers; invalid relationships and output references fail.
Clean Mac source build passed (41 nullable/source-link warnings/0 errors), 54 DLL fixture checks
and two original-DLL reproductions. The first Windows source build stopped before
restore/build at a post-patch hash mismatch: observed bytes matched CRLF conversion
of the expected LF file. Per-invocation Git LF settings correct that build route;
fresh native build and corrected retail parsing/runtime remain pending.
Upstream #548 closed without merge; its closure establishes no current approval.
Review any generator correction/licence before retry; only passing empty-loader
generation/menu/normal-exit/rollback permits a log-only Load probe.
That callback proves no Unity frame hooks, input or native game adapters.
## Remaining acceptance
No compatible client extension/full mod is verified. A marker cannot prove weapon/camera/input adapters, visible board/rider,
network movement, lifecycle cleanup or the complete two-client flow.
Shadow remains client-only; a permitted controllable testserver is still missing.
The owner tests after source completion and our full verification; no intermediate playtest.
Choose/rent no host without owner choice and concrete cost approval.
