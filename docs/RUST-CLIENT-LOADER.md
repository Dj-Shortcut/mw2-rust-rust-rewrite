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
41 warnings and zero errors; all 182 authored DLL checks pass.
Actual .3 Windows build also passed (41 warnings, zero errors), with verified pinned
DLL identities. Genuine LoadFromFile returned true against unchanged retail inputs.
All 71,422 raw-property calls completed with zero exceptions: 22,041 typed,
175 getterless empty setters, 26,618 VOID/empty-setter wrappers and 22,588 additional
VOID-getter wrappers. Core initialized with 162 assemblies; AttributeInjector ran
once without exception. First managed BuildAssemblies failed at the explicit
interface getter guard with two encountered aggregate type causes:
`Interface getter has no value type`. Each assembly stops at its first failed type;
this is not a complete count of incompatible types. No output libraries, interop,
bootstrap installation or retry on the mutated failed context occurred.
That .3 attempt produced no interop output. Complete output validation, runtime
and loader retry remain open; the later .4 generation is recorded below.
[Upstream #548](https://github.com/SamboyCoding/Cpp2IL/pull/548) closed without merge; its closure establishes no current approval.
Review corrections/licences before retry; only passing empty-loader generation/menu/exit/rollback permits a log-only Load probe.
That callback proves no Unity frame hooks, input or native game adapters.
## Interface diagnosis before another correction

Read-only checks found matching parsed/header counts and zero row-size remainder
in six metadata sections (methods, parameters, types, properties, interface offsets
and generic containers). Three known system/Unity getters return their expected
managed/raw types. Index widths agree with the registered type count and method
row-size formula. Independent decoding of seven serialized method rows and binary
type-bit records matches the parser's return indices, type bits and method tokens:
two exact failure sources, three controls and two actual interface targets.
This check still uses Cpp2IL's selected registration and PE address mapping; it is
not an independent registration locator.

Leaf method names are reused. Six initial candidates were narrowed to one method
in each encountered failing type using its managed full name, with the complete
module/declaring-type/assembly tuple checked against the preserved exception.
Both source returns are raw VOID; their computed interface targets are nonVOID.
The preliminary candidates share some nonzero native pointers; this does not establish intentional stripping or signature
compatibility. `Overrides` reconstructs edges from vtable entries, interface offsets
and slot lookup; it is not a copied native MethodImplementation table.

Both failed sources have one applicable vtable/interface candidate, one target
slot match and exact target-definition identity. Neither target is a concrete
generic method context; both interface reflection records report IsGenericType=false.
Direct serialized vtable-word, interface-type-index and offset comparisons also
match for both candidates; this metadata-byte check does not use binary registration.
Both targets are ordinary method contexts with present declaring definitions and
empty interface-offset arrays. Consequently, the target filter from merged
[Cpp2IL #567](https://github.com/SamboyCoding/Cpp2IL/pull/567) leaves both pairs;
its separate source-side BaseMethod change has not been evaluated here.

No proven mapping correction follows from these checks. Registration provenance
and full native validation of the explicit source-model contract remain open.
Preserve strict assembly rejection while the mismatch is unexplained. Do not
guess a value type, silently discard incompatible edges or interpret wrapper
omission as permission to generate invalid interface metadata. Private
retail names, indices, addresses, files and transcripts remain outside publication.

## Explicit interop-only source models

The .4 source implements the explicitly selected `BuildInteropSourceModels`
entry point. It supplies in-memory metadata to the pinned official interop generator. Default
`BuildAssemblies`, registered DLL formats and `DoOutput` retain strict behavior.
The alternate route omits the whole synthetic explicit-interface MethodImpl and
property construction phase for every edge. It retains original type flags,
interface rows, constraints, method signatures/order, native attributes and
eligible original properties/semantics. It does not modify raw VOID returns.

The [be.788 manager](https://github.com/BepInEx/BepInEx/blob/5b766a3b7f6c164d4798924a93f3acf4db769d06/Runtimes/Unity/BepInEx.Unity.IL2CPP/Il2CppInteropManager.cs)
passes assembly models directly to the generator. The
[pinned metadata consumer](https://github.com/BepInEx/Il2CppInterop/blob/dbda1cb353b0f4253345dc45136d170b9e50a5a0/Il2CppInterop.Generator/MetadataAccess/AssemblyMetadataAccess.cs)
keeps those objects; its wrapper passes do not consume MethodImpl rows, remove
interface/abstract flags and do not copy original InterfaceImpl rows. Source
interfaces still support naming, constraints and awaiter generation.

This deliberately loses synthesized explicit-interface property APIs and their
semantics, and can alter generated method names, collision ordinals and rename
keys. It does not restore either contradictory value type or prove those native
calls safe. These input models must never be exported or installed as repaired
normal dummy DLLs. The existing BepInEx caller still selects strict generation;
a separate isolated offline consumer must opt in and prohibit dummy export.

Verify strict and alternate routes in fresh contexts: preserved source methods,
signatures/tokens, valid original properties/interfaces/constraints and existing
malformed/unresolved/named-write rejection for references still consumed; no
synthesized edge/property in the alternate model; exact interop consumer output
and its documented API losses. Omitting that entire phase also omits its
interface-specific reference guards. This does not relax the strict route.

A per-context attempt marker is set before any build mutation; success or failure
requires a fresh context for another attempt, including cross-mode or formatter
reuse. This does not make global Cpp2IL caches safe across concurrent applications;
use one builder and a fresh process after failure. Ordinary `DoOutput` remains
the existing strict semantic route; separate strict generated-PE validation is
still required.

The pinned .4 recipe built on the actual Mac with 41 upstream/archive warnings
and 0 errors; ten changed source hashes and four dependency lock hashes checked.
All 230 authored cases pass against the built libraries: 182 prior cases,
26 public builder cases and 22 selected official interop-consumer cases. The
fixture build has two nullable scaffold warnings and 0 errors. These cover
strict VOID rejection before association, omitted synthetic APIs, original
properties/indexers, constraints/native metadata and context-reuse rejection.
Selected passes and two CIL inspections do not prove the complete runner, PE
validity, native type resolution or native invocation; generic target identity
coverage is bounded. Actual Windows offline generation is recorded below;
complete generated-output validation must pass before installation. Empty offline
menus, normal exit and rollback remain later gates.

## Actual Windows generation and output validation

The pinned .4 recipe at `8ba5811230582c265e80cff185971c43f228a840`
built on Shadow Windows with 41 warnings and 0 errors. Its separate offline
consumer built with 7 warnings and 0 errors. A private observer confirmed the
staged CoreCLR 6.0.7 startup and expected zero-argument rejection. This proves
that host startup only; generated assemblies were never CLR-loaded.

The first full attempt stopped before source construction because the isolated
caller had not registered the official x86_64 instruction-set handler. The caller
bootstrap was corrected to register the official instruction sets and binary readers.
After that correction, a fresh process passed source-model
checks for 162 assemblies, 32,404 types, 710,535 methods and 22,041 typed original
properties, with 49,381 source properties omitted. The exact official interop
consumer wrote 191 DLLs and two database sidecars, logging zero warnings/errors.
Completion and those counters do not establish complete restoration coverage.
The original strict reread then failed on `unresolved-generated-assembly-reference`
before type traversal; zero strict PE round trips completed. The child exited
normally with code 1, with no timeout/kill and complete redirected streams.
Final 549 input hash checks, 522 archive-entry comparisons and inventories matched.
The candidate and its failure evidence remain private, outside the game.

A passive metadata audit inspected all 191 DLLs and 1,172 assembly references.
Its 182 unresolved references all use the unsigned neutral `Il2CppInterop.Runtime`
version `0.0.0.0`, against the exact official `1.5.3.0` dependency. No other
unresolved assembly identity was found in that scan. The pinned official
[generator deliberately emits this zero-version reference](https://github.com/BepInEx/Il2CppInterop/tree/dbda1cb353b0f4253345dc45136d170b9e50a5a0).
The exact-version diagnostic therefore needs a narrowly scoped resolver alias;
this finding alone does not approve generated types, members or CIL.

An independently reviewed existing-output diagnostic accepts only that unsigned
neutral zero-version identity against the exact path, SHA256 and identity of the
pinned official dependency. Every other assembly comparison and all original
type/member/CIL checks remain strict. It traverses existing DLL metadata and
attempts in-memory PE round trips without rewriting the candidate. Eight authored
identity boundary cases pass locally. The first native wrapper failed before
build/host on a report-hash transcription error; its correction reached a build
failure because the SDK selected an incompatible package-copy dependency before
the pinned reference. After isolating that search, the native build and exact
CoreCLR host ran, but the diagnostic failed on
`unresolved-generated-member-reference` during the fourth assembly's original
imported-member traversal. Three assemblies completed both traversals and strict
PE round trips. Before rejection the primary traversal counters reached 2,692
types, 102,616 methods and 5,971 properties; 2,658,443 type-resolution and 923,802
member-resolution visits are repeated visits, not unique metadata rows. The sole
pinned runtime alias resolved once. The child exited normally with code 1, with
no timeout/kill and complete streams. The diagnostic rechecked 422 files and
three inventories unchanged; wrapper final checks likewise preserved 3,749 input
hashes and inventories with no reported cleanup/recheck failures. This is a
failed partial audit, not complete validation of the 191-library set. The exact
member/signature context and the cause of its rejection remain to be diagnosed.
The public .4 recipe and stock BepInEx caller remain unchanged. No regeneration,
installation or loader retry follows from the alias diagnostic. Sidecar validity,
full runtime closure, native calls, offline menus/exit/rollback and full mod
acceptance remain separate open gates.

## Remaining acceptance
No compatible full mod is verified. A marker cannot prove weapon/camera/input adapters, board/rider, movement, lifecycle or two-client flow.
Shadow remains client-only; a permitted controllable testserver is still missing.
The owner tests after source completion and full verification; choose/rent no host without owner choice and concrete cost approval.
