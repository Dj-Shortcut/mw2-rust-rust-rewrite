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

## Pointer-byref generator source correction, 7 October 2026

The [separate pinned Il2CppInterop recipe](../mods/rust/client/generator/interop/README.md)
implements the two-predicate design from #295. Direct pointer elements use existing
byref storage-address handling in ordinary methods and the shared unstrip helper;
no pointer-object temporary or constructor copyback is emitted. Original source
projects, version 1.5.3.0 and dependency boundary remain intact. Only Generator is
staged, with complete LGPL corresponding source; nothing is installed in Rust.

Actual Linux x64 SDK8.0.425 source and recipe builds pass with 4 upstream warnings
and 0 errors. Fifteen temporary recipe guard cases and an automatic-CRLF Git-clone
hash check pass. Actual original/patched Generator DLLs produce 108 ordinary methods
and 64 unstrip invokers on authored metadata models; each path covers 32 ref/out
pointer cases. All 108 unaffected control outputs retain normalized IL, locals,
signatures and flags. Independent serialized metadata decoding finds 8 pointer-owned
constructors in the original PE reads and 0 in patched reads. Four authored PE
round trips preserve complete method/body inventory and normalized IL/locals;
4,247 original and 4,218 patched assertions pass, with zero fixture-build warnings/errors.

The minimal authored corlib is declared metadata scaffolding; generated IL/native
calls are not executed and external Runtime import resolution is not attempted.
This cloud run has neither the private native fixtures nor Mac/Shadow control.
It does not trace the private rejected caller, validate sidecars or regenerate the
191 retail assemblies. The rejected 3/191 candidate stays rejected. Fresh complete
native generation/output validation, loader/menu/exit/rollback and mod gameplay
remain open; no owner intermediate playtest or server rental is requested.

## Actual Windows pointer-generator build

The separate pointer/byref source build completed on Shadow Windows on 7 October
2026. The bounded process exited normally with code 0, confirmed root exit and
complete captured streams, no timeout, and no recheck or cleanup failures. All
1,347 input hashes and 15 build/staging output hashes were unchanged; the output
inventory was unchanged. These are build outputs, not the 191 generated game
assemblies. Independent process-tree termination is not established.

The actual staged Generator SHA256 is
`364e6baa9c6ad50456a45aa056d97879ec759c9404ec3a98d2864bbea33c591e`,
with informational/product version 1.5.3. The native observation digest was read
back as `1729fdaa5c6a70ed5088c776783c6dca6c1cbec2188ece7df5fffe254d124795`.
The result and digest were read from the live Windows console; the full report
has not been copied to the Mac. No Windows warning count is inferred from the
separate Linux build.

This pointer-build checkpoint establishes native source-build/staging only. Later
authenticated fresh generation is recorded below;
complete metadata and sidecar acceptance, runtime/loader checks and mod gameplay
remain open. No candidate was accepted, installed or launched, and the earlier
3/191 candidate remains rejected.

## Actual Windows passive caller diagnosis

The preserved diagnostic now has an observed native Windows result on the exact
CLR6.0.7 host. Its passive metadata/CIL reader visited and decoded all 513,677
expected method bodies in the rejected assembly: 7,513,644 instructions, one use
of the rejected pointer-owned constructor, no truncated caller output or errors.
The static caller has six parameters, ending in an out pointer-byref parameter.
Its signature and local pattern match the authored out-pointer reproduction;
precise emitting-pass attribution still needs the caller instruction window.
Raw names, tokens, signatures, offsets and candidate bytes remain private.

The helper reports completed scanning and all four pinned inputs preserved.
Its outer process exited normally with code 0, no timeout or kill, complete streams,
193 before/final file checks, unchanged host inventory and no cleanup/hash errors.
Process-tree termination is not independently established. The inspected generated
assembly was never CLR-loaded and no generated/native method was invoked.
This diagnoses one rejected member in one assembly; it does not validate all 191
assemblies, sidecars, runtime closure, the replacement build or the loader.
The old rejected candidate remains rejected. Later authenticated fresh generation
is recorded below; complete output acceptance remains required before installation
or a playtest.

## Nested method restoration

[Issue #303](https://github.com/Dj-Shortcut/mw2-rust-rust-rewrite/issues/303) follows the preserved fresh-output rejection in #289.
The actual Windows original-member diagnostic captured a missing static method on an empty
nested marshalling type;193 generated files and1517 outer input checks remain preserved.
The exact bound official Unity archive independently contains that eligible managed method
and nine source-only call sites. Pinned type restoration is recursive, while method restoration
walks only top-level types and the following body pass uses only its earlier queue.
This identifies a source traversal gap; the actual generated caller and earlier failure phase
are not attributed by this source-only read.

The source correction retains all existing top-level work as the original global prefix,
then visits source nested owners through their mapped parent and registered target context.
It reuses method eligibility/signature/body emission; exclusions, field restoration,
prior pointer-byref hunks and strict output guards remain unchanged. Missing parents create
no new context or orphan. No candidate DLL rewriting, resolver relaxation or installed-file edit.

Actual SDK8.0.425 source builds pass with four upstream warnings/zero errors and unchanged
projects, SDK policy and complete locks. Isolated actual-generator authored baseline/candidate
processes each pass32 checks and three strict PE round trips. The twelve-method original
queue prefix and28 exported controls remain identical; a live assertion verifies the exact
fourteen-method nested tail. Twelve candidate bodies translate; two negative cases preserve
the exception fallback and are not counted as successful translations. The actual parent
getter route and its strict reread binding pass. The baseline reproduces nine missing methods;
both actual serialized boundaries require zero candidate misses. Input hashes remain preserved.
These are macOS source/authored checks, not complete recipe staging or native generation.
[Detailed scope and build limits](../mods/rust/client/generator/interop/README.md#authored-nested-method-verification).
The later authenticated Windows recipe/staging and
[fresh generation](#actual-windows-authenticated-preparation-and-fresh-generation) are recorded below.
Full191-assembly original/member checks before strict roundtrips, sidecars and
loader/menu/rollback acceptance remain pending. No playtest.

## Current Windows preservation export

On 7 October 2026, the passive exporter completed on Shadow Windows in the
preserved PowerShell7.4.20 process with normal exit0. All1935 current file pins,
12 complete inventories and seven known historical anchors passed before/final
checks. The preserved1457/1517 checkpoints and old193 generated files remain
unchanged. Root independently read the three complete report digests from
Windows, transferred all three original reports to the Mac, and verified their
bytes and full contents. The complete historical1517-hash map is unavailable;
this authenticates the current union and known anchors, not every historical hash.

The reviewed build-only package copies all1935 file records and12 inventories
unchanged, then adds the three authenticated export reports and their inventory:
1938 pins and13 inventories. Its ten source payload files match canonical
PR [#306](https://github.com/Dj-Shortcut/mw2-rust-rust-rewrite/pull/306).
The build-only package was subsequently executed; its authenticated result is recorded below.
At that export checkpoint, the separate all191 original-member check had reviewed
source and a successful root compilation, but had not executed natively. Its later
native results are recorded below.

Later [fresh generation/output pinning](#actual-windows-authenticated-preparation-and-fresh-generation)
is recorded below. All191 original MemberRef checks before any strict PE roundtrip,
sidecars and loader/menu/exit/rollback acceptance remain pending. Prior rejected
outputs stay rejected. No installation, game launch or owner playtest follows from this export.

## Actual Windows nested-generator build and consumer preparation

On 8 October 2026, root fully authenticated the existing Shadow Windows build
without restarting it. The bounded child exited normally with code 0; root exit
and complete captured streams were confirmed, with no timeout or kill and no
preservation, output, source or cleanup failures. Actual build stdout reports
four warnings/zero errors; stderr is empty. All 1938 prior file pins and 13
inventories, all 1955 total before/final input checks and 193 old generated files
remain unchanged. All 15 build/staging outputs and ten protected/patched/lock
files were verified against the frozen public recipe. Independent process-tree
termination and complete historical 1517-hash equality remain unproven.

Root independently read the native hashes of all seven transferred files and
the ZIP, transferred the original full observation/output/anchor reports,
build manifest, stdout/stderr and Generator DLL to the Mac, and verified their
digests, ZIP entries/CRC and complete reports. The measured Generator SHA256 is
`a962c0a467d9f70b855b013d4683b59c5bba8df241efa92c58601e73899d7688`.
A separate pure PE-metadata read confirms the unique informational version 1.5.3,
assembly identity, target framework, flags and reference bindings against the
native records without loading the inspected DLL. The built Common remains
build evidence; the pinned official Common and Runtime remain unchanged.

The private generation-only consumer was rendered from the reviewed template
by exactly two constant substitutions. Root compiled it on macOS with
SDK8.0.425 against 33 pinned references: actual compiler exit 0, nine warnings,
zero errors and all 5318 compiler/input files preserved. Independent source/render
review passed. At this compilation checkpoint the consumer had not executed;
its later authenticated fresh-generation run is recorded below.

This checkpoint establishes source-build/staging and consumer compilation only.
The later [authenticated fresh generation](#actual-windows-authenticated-preparation-and-fresh-generation)
is recorded below. All191 original MemberRef checks before any strict PE roundtrip,
complete sidecar validation, loader/menu/exit/rollback and connected MW2/skate
gameplay remain pending. Prior rejected outputs remain rejected.
No candidate approval, installation, game launch or owner playtest is implied.

## Actual Windows authenticated preparation and fresh generation

On 8 October 2026, copy-only preparation completed on Windows with normal exit0.
All 229 planned consumer/host files were copied and final-checked; all 2019 prior
inputs and 2057 total before/final checks, with complete inventories, stayed equal.
Preparation did not execute the consumer or approve any loader/gameplay gate.

The subsequent fresh generation completed with normal child exit0, confirmed root
exit and fully drained streams, without timeout, kill or cleanup failure.
All 2019 prior inputs and 2478 total before/final checks remained equal; 193 fresh
generated files were pinned/preserved and the 197-artifact execution inventory
passed before/final checks. Original input/output inventories were preserved.
The exact custom Generator remains version 1.5.3 with SHA256
`a962c0a467d9f70b855b013d4683b59c5bba8df241efa92c58601e73899d7688`.

Root read the complete ZIP's SHA256 from Windows before transfer, then verified
the original 72,096,854-byte archive and all 200 received files on the Mac.
ZIP CRC errors are 0; all 193 generated files and 197 execution artifacts were
individually rehashed. The original observation, manifest and external anchor form a matching digest
chain inside the archive authenticated by its independently read Windows hash.
Independent read-only review passed 2200 assertions/0 mismatches.

| authenticated artifact | SHA256 |
| --- | --- |
| complete generation ZIP | `01f9294d4f9d6cdf5eb28d839a254c732d44091219243a15da0304ec30982e68` |
| generation observation | `1a9fffa61a0331ebb2dc63dfb7e028365448b2a66e31a1c6ac2d92412f3bb46b` |
| fresh output manifest | `a92b97ceb1b1b2a945a1a96dd81d04063788ac0b6ce2263f4ea110cd9157fb2c` |
| external manifest anchor | `c9d65a2bb20f5982d67bac2f20a0908bd31673239f17a8c1487e101a38253484` |

Generation diagnostics still report 4564 unrestored methods and 6695 failed IL
bodies; these informational counters do not establish successful restoration.
This proves preparation, fresh generation and byte-preserving transfer only.
All 191 original MemberRef checks, strict PE roundtrips (0 completed), sidecar
validation, candidate/loader approval, generated CLR/native-method execution,
installation, game launch, loader/menu/exit/rollback and two-client MW2/skate
acceptance remain unverified. Prior rejected candidates remain rejected.
Complete historical 1517-hash equality and independent full-process-tree
termination remain unproven. No owner playtest is implied.

Later on 8 October 2026, the original-member audit wrapper ran on Shadow Windows
and exited 1 before the original-member scan. The visible console reported 2500
before/final hash checks, 2019 prior checks, all 193 old generated files preserved
and no hash recheck failures; nevertheless, the generation directory and its
generated subdirectory both failed baseline and final inventory comparisons.
Equal hash counters do not establish complete inventory acceptance.
At that checkpoint, the receipt was partial: the complete native result had not
been transferred, and the proposed wrapper correction had not run on Windows.
All 191 original-member checks, strict PE roundtrips, sidecar validation, loader
approval and full MW2/skate acceptance remained pending. The later corrected run
is recorded below; the earlier failure does not authorize an owner playtest.

## Actual Windows original-member audit and automatic handoff

Later on 8 October 2026, the Windows diagnosis confirmed root-prefix casing and
an incorrectly included inventory root as the earlier inventory mismatch causes.
All 13 diagnosis checks passed and 201 before/final input hashes remained equal.
The corrected wrapper passed inventory preflight and preserved all 2505
before/final input checks, 2019 prior inputs and 193 old generated files.
The child preserved 431 files and four inventories, with no preservation or
cleanup failures, but exited normally with code 1 at
`original-module-member-reference-count-limit`. It read metadata for all 191
original assemblies; no original members were scanned or resolved and no strict
PE roundtrip ran. Root exit and drained streams were confirmed; independent
full-process-tree termination remains unproven. At that checkpoint, the private
count-limit correction was preparation only and had not run on Windows; its later
native result is recorded below.

Root subsequently completed both package-download and result-upload transfers
without owner intervention, read both ZIP hashes from the live Windows console,
and authenticated the full returned failure reports on the Mac. The return ZIP
contains ten entries with zero CRC errors; its saved observation, child summary,
stdout and preservation records agree. The native audit used the earlier
owner-copied identical package; the newly downloaded copy was not executed.
Both temporary transfer listeners and tunnels closed with exit 0.

| authenticated handoff artifact | SHA256 |
| --- | --- |
| source package ZIP | `34817f779bc6b55f3c8a3ebb304d5eef5a556b715ffec0f1798d65579a2d4331` |
| complete result ZIP | `26d73e1c3302f8e73ad2e9f4f358bee72e03a92245178e4c54a108e991825ab5` |

This handoff requires an active Codex session, an awake Mac and reachable Shadow;
no scheduler or persistent remote agent was installed. Complete original-member
resolution, strict PE roundtrips, sidecars, loader and full two-client MW2/skate
acceptance remain pending. No candidate acceptance, installation, game launch or
owner playtest follows from the preserved failure.

## Subsequent Windows count-limit correction and partial member audit

Root subsequently downloaded, freshly extracted and ran the reviewed count-limit
package on Shadow Windows, then returned its complete reports to the Mac without
requiring the owner to transfer files. Diagnosis again passed 13 checks with 201
hashes preserved.
The child exited normally with code 1 at `unresolved-generated-member-reference`.
It read metadata from all 191 original assemblies and completed member traversal
for 109. It scanned 901696 of 964336 expected original MemberRef rows and resolved
901695. The failure occurred in the next assembly; complete original-member
acceptance remains rejected. No strict PE roundtrip ran.

All 2506 wrapper before/final checks, 2019 prior inputs and 193 old generated
files were preserved. Inner checks preserved 431 files/four inventories;
six report-artifact hashes also matched, with no preservation or cleanup failure.
The child had no timeout or kill; root exit and drained streams were confirmed,
while independent full-process-tree termination remains unproven.

Root read both ZIP hashes from live Windows and authenticated the returned
archive: 11 entries, zero CRC errors, matching observation/summary/stdout and
preservation records. Both temporary transfer listeners and tunnels closed with
exit 0. This demonstrates downloading a new package, running it natively and
returning its result within an active Codex session, with an awake Mac and
reachable Shadow; it establishes no scheduler or persistent remote agent.

| authenticated count-limit handoff artifact | SHA256 |
| --- | --- |
| source package ZIP | `1258b4cb792a6adc2e5477f1f2eb597ac55bf014e9d8c8fd57027971f4047c88` |
| complete result ZIP | `41616de75e18fb82f066624b1b6d9b099099c0f5b678da451c4f71176ab3111c` |

Full original-member validation, strict PE roundtrips, sidecars, loader/menu/
exit/rollback and full two-client MW2/skate acceptance remain pending. The
preserved partial audit approves no candidate, installation, game launch or
owner playtest.

## Current Windows capture and client build

On 9 October 2026 (Europe/Brussels), the current Windows preservation capture
completed with normal child and outer exit 0, drained streams and no timeout,
kill, cleanup or preservation failure. Root verified 10953 before/final file
pins, 28 inventories and three early rejection controls against the independently
hashed returned archive. The complete input union fits the reviewed 16 MiB bound.
The earlier deadline failure remains rejected; complete historical 1517-hash
equality remains unproven.

A subsequent bounded export preserved its 11 controls and returned the exact two
compile-reference DLLs with matching source/destination hashes. Both child and
outer exited normally 0. Root verified the ten-member archive and retained the
accepted capture documents unchanged; this export did not rehash all 10953 prior
Windows inputs. Both temporary transfer routes closed with no cleanup failure.

The current original generation-only consumer compiled on macOS with SDK 8.0.425
against the exact 33 references: normal exit 0, 8 warnings/0 errors and all 5134
inputs preserved. The first stricter consumer build failed at two ambiguous null
comparisons. A separately reviewed two-expression correction then compiled with
12 warnings/0 errors and all 5139 inputs preserved. Both resulting consumers retain
net6.0, CLR 6.0.7 and disabled runtime roll-forward. These are Mac compiler results;
the subsequent native preparation, observed generation completion and failed
authored-core run are recorded below. Complete original 191-member validation,
strict PE roundtrips and sidecars remain pending. Earlier accepted or rejected
generation/audit results above retain their stated scope. No generated CLR
execution, loader approval, installation, game launch or owner playtest follows
from these builds.

## Current Windows preparation and authored-core correction

The current copy-only preparation completed normally on Shadow Windows with
child and supervisor exit 0, confirmed root exit, drained streams and no timeout,
kill, cleanup or preservation error. All 229 copied files, 231 measured outputs,
10993 input checks and 28 inventories were preserved. Root independently read the
Windows report and archive hashes, returned the actual reports to the Mac and
authenticated their exact preparation bindings. This accepts preparation only.

The subsequent generation supervisor also visibly completed on Windows with
exit 0, drained streams and no reported preservation or cleanup error. Its report
and archive hashes were read independently on Windows. The report ZIP remains on
Shadow: automatic approval review blocked its temporary return transfer pending
specific owner consent. Returned-report authentication is therefore pending;
observed completion does not accept the generated candidate.

The first current authored-core run stopped with native exit 1 before any of its
27 cases completed. Its unchanged strict consumer was not reached by those cases:
the independent PE fixture writer failed at the `MethodBodyStreamEncoder`
four-byte alignment precondition. The actual failure reports and original source
remain preserved. A separately reviewed one-statement writer correction aligns
the stream before constructing that encoder; fixture definitions, signatures,
oracles, strict consumer, runtime guards and case expectations remain unchanged.
The corrected harness compiled separately on macOS with SDK 8.0.425: all three
build processes exited 0, 2 warnings/0 errors, 5098 inputs and 5559 SDK inventory
entries preserved. This is build evidence. The next native run is recorded below.

On 9 October 2026, the aligned harness actually executed all 27 authored cases on
Shadow Windows using the pinned PowerShell 7.4.20 launcher and x64 CLR 6.0.7.
The child and outer reports returned exit 1 without timeout, kill or cleanup
failure. A complete visual filter of the saved report identifies 26 passing cases
and exactly one failure: `generic-calli`, `actual-negative-must-reject:generic-calli`.
The outer report rechecked all 331 control inputs and retained its two measured
streams. Its overall inventory-acceptance flag remains false; do not infer a
successful complete output/inventory gate from these narrower preservation flags.
The report and stream hashes were independently read on Windows, but the raw
reports have not been returned and authenticated on the Mac. This checkpoint is
visual native failure evidence, not accepted authored-core validation.

The pinned reader wraps a `calli` operand in `StandAloneSignature`; the strict
consumer checked only a bare `MethodSignature`. A separate reviewed correction
adds 12 lines to require the proper opcode and payload, retain the existing
generic-signature rejection, and check ordinary payloads with the actual caller
slots. All 27 case definitions and oracles, the aligned writer and the independent
physical oracle remain unchanged. The first correction compile failed on an
ambiguous type pattern; a separate syntax correction adds an explicit typed
discard and retains that failure evidence. The corrected strict consumer and
rebound harness built separately on macOS with SDK 8.0.425: normal exit 0,
12 and 2 warnings respectively, no errors, and 5153 and 5110 enrolled inputs with
5559 SDK inventory entries preserved. These builds do not prove a native pass.

The fresh package contains 47 app files, 12 proof files and 62 archive members.
Root staged, rendered and checked those local bytes; a separate review found no
material binding or guard issue. Transfer has not occurred: automatic approval
review rejected the temporary download tunnel pending consent for this specific
new package. The next native run must use distinct fresh directories and preserve
all prior sources and failure reports. No generated retail assembly is CLR-loaded
and no loader, installation or game-start approval follows from this correction.

Earlier local checks parsed the separately reviewed strict/sidecar admission
wrappers and exercised their isolated admission against the prior strict28 Mac
build receipt and rejection controls. They do not admit the new strict44 build;
its strict/sidecar binding validation remains pending. These checks do not execute
the full Windows wrappers. Complete original 191-member validation, authored native core
acceptance, strict roundtrips, sidecars, native ABI, loader startup/rollback and
full two-client MW2/skate acceptance remain open. No owner playtest is ready.

## Remaining acceptance

No compatible full mod is verified. A marker or passive metadata audit does not
prove weapon/camera/input adapters, a visible board and rider, movement,
lifecycle or two-client play.

Shadow remains client-only. The owner-selected DigitalOcean Linux host has
verified Rust/Oxide startup, authenticated private control, save/reload and bounded
ShortcutLoadouts checks. During a previous restricted test window, the owner
actually joined from Shadow and spawned. The intended loadout, cooldown,
full-inventory, permission and save/restart/rejoin checks were not completed;
join/spawn alone is not a loadout playtest. That window was closed and the server
was saved/stopped with external game/query ports blocked. A new player session
and a second EAC-disabled client for PvP damage remain separate checks. See
[native server evidence](../mods/rust/server/LINUX.md#native-startup-and-control-evidence).

The owner playtests after source completion and verification of the complete
MW2/skate flow. Any further rental or paid hosting change requires the owner's
choice and concrete cost approval.
