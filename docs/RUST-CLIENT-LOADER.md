# Exact Rust Windows client-loader gate
Part of [full acceptance](RUST-MW2-SKATE.md) / [diagnosis #289](https://github.com/Dj-Shortcut/mw2-rust-rust-rewrite/issues/289): official loader failed, bare recovery passed.
Development evidence only; no installation or owner playtest.
## Actual Windows observations, 6 October 2026
Mac native control reached Shadow Windows. Bare EAC-disabled RustClient.exe reached responsive Home/Options and normal exit:
no remaining process; Steam252490/build25681799, menu166652, Unity6000.3.15x1-13(a91cf34396ee); Steam beta selection unchecked.
[Official BepInEx be.788+5b766a3](https://builds.bepinex.dev/projects/bepinex_be) Windows x64 ZIP:
SHA256 F4CC496BD098A0DF4164B81E3737297707F13A47C2478DBA2F60EEFAB784817A.
No pre-existing bootstrap paths; plugins/patchers empty; default configuration unchanged.
Loader menus responded and Quit to Desktop left no process, but generation failed:
Cpp2IL metadata39, IndexOutOfRangeException at RawPropertyType line44;
interop directory empty, then missing UnityEngine.CoreModule and fatal no-plugin chainloader.
Official LoadFromFile returned true; a real getterless zero-parameter setter reproduced RawPropertyType array bounds,
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
Keep private logs/references/binaries/probes outside GitHub; use pinned genuine DLLs and fixtures in isolated processes.
Validate table strides, owners and parameter spans before interpreting accessor failures.
Five original mechanism fixtures used explicit cache scaffolding; actual Windows
metadata parsing now separately found 175 empty and 938 nonempty getterless setters.
Checked owner/accessor/parameter relationships passed; full byte geometry is pending.
[Correction source/build](../mods/rust/client/generator/README.md) preserves indices/methods;
omits only noninferable wrappers; invalid relationships and output references fail.
Previous correction (.1): clean Mac source build passed (41 nullable/source-link warnings/0 errors), 54 DLL fixture checks
and two original-DLL reproductions. The first Windows source build stopped before
restore/build at a post-patch hash mismatch: observed bytes matched CRLF conversion
of the expected LF file; per-invocation Git LF settings correct that build route.
The corrected Windows build passed with 41 warnings/0 errors; Core/Lib stay outside Rust.
Patched LoadFromFile returned true with unchanged retail hashes; original crash case passes.
All 71,422 RawPropertyType metadata getters were invoked: 44,622 resolved, 175 tagged getterless empty setters,
26,625 getter-plus-empty-setter exceptions, all classified by the explicit no-value guard.
Raw getter scan: 26,618 VOID, seven non-VOID, zero nulls/errors.
[Bounded accessor correction](../mods/rust/client/generator/README.md#bounded-accessor-correction)
omits VOID/empty-setter wrappers, preserves non-VOID getters and rejects suppressed-setter references.
The .2 Mac build passed (41 warnings, 0 errors) and 106 authored DLL checks passed,
including 16 serialized V29 payload cases. Generic local-index resolution is unsupported.
Actual .2 Windows build passed (41 warnings, 0 errors); all 71,422 raw-property calls
passed: 44,629 resolved, 175 getterless empty setters tagged, 26,618 VOID/empty-setter
tagged, zero exceptions. Full Core initialization completed with 162 assemblies;
AttributeInjector PreProcess/Process returned without exception once.
First managed BuildAssemblies then failed with four type failures, each
`Property type cannot be void`. Further native classification found 22,588 additional
raw VOID getters among the resolved count: 22,574 without setters, 14 with one-parameter
setters, none getterless. No output libraries or installation were produced.

The [remaining bounded correction](../mods/rust/client/generator/README.md#remaining-void-getter-correction)
omits these wrappers after relationship validation, retains methods/slots and rejects
managed VOID interface getters before association. Its .3 source builds on Mac with
41 warnings and zero errors; all 182 authored DLL checks pass. Fresh native .3 build,
full managed PE round trip, interop generation/runtime and loader retry remain open.
Upstream #548 closed without merge; its closure establishes no current approval.
Review corrections/licences before retry; only passing empty-loader generation/menu/exit/rollback permits a log-only Load probe.
That callback proves no Unity frame hooks, input or native game adapters.
## Remaining acceptance
No compatible full mod is verified. A marker cannot prove weapon/camera/input adapters, board/rider, movement, lifecycle or two-client flow.
Shadow remains client-only; a permitted controllable testserver is still missing.
The owner tests after source completion and full verification; choose/rent no host without owner choice and concrete cost approval.

