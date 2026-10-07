# Pointer byref generation repair

Source for the separate Il2CppInterop generator correction defined in merged
[PR #295](https://github.com/Dj-Shortcut/mw2-rust-rust-rewrite/pull/295), tracked in
[issue #289](https://github.com/Dj-Shortcut/mw2-rust-rust-rewrite/issues/289).
The existing [Cpp2IL recipe](../README.md) addresses source-model generation;
it does not apply this correction. This directory supplies a pinned patch and
source-build recipe; no DLL or client installation is published here.

## Observed failure

An actual native Windows audit rejected a generated imported member after only
3 of 191 complete assembly round trips. An independent CLR8 passive metadata read
confirmed an instance constructor taking `IntPtr` whose declaring type is an
unmanaged pointer. All four pinned inputs remained unchanged before and after
that read. This is rejected generated metadata, not accepted output or gameplay.
The exact native emitting caller remains unconfirmed; the authored generator
reproduction below is a separate check.
Game files, candidate assemblies, raw metadata and diagnostic logs stay outside
GitHub.

## Correction

Pin [Il2CppInterop source dbda1cb](https://github.com/BepInEx/Il2CppInterop/tree/dbda1cb353b0f4253345dc45136d170b9e50a5a0).
Its existing value-like helper includes direct pointer signatures. The patch
applies that classification in the byref argument marshalling predicate and the special out
argument temporary predicate. Pointer arguments should use the existing argument
address path, without an object conversion, pointer constructor or object-reference
copyback. Keep original parameter signatures and out flags.

Ordinary references, strings, generics and value types must retain their existing
behavior. Inspect the unstrip consumer of the shared helper as well as ordinary
method generation. Function-pointer and modifier-wrapped signatures are outside
this correction. Do not loosen member resolution, suppress errors or
rewrite the failed candidate DLLs.

## Build and verification contract

The recipe pins the source archive, two-file patch, SDK 8.0.425 and complete
Generator/Common restore locks. It preserves upstream projects, global SDK policy,
assembly identities and dependency boundaries; only packaging is disabled during
the build. It stages only the netstandard2.1 Generator replacement, version 1.5.3.0.
This is a replacement candidate for the pinned existing Common/dependency closure,
not a complete generator/runtime installation. Building installs nothing in Rust.
Upstream LGPL-3.0-only attribution and corresponding source/modification terms
remain separate from the MIT Cpp2IL material.

Use the actual built Generator DLL with authored ignored ref/out and nested-pointer
fixtures, static/instance argument controls, ordinary reference/string/generic/value
controls and the unstrip call site. Inspect emitted IL and serialized MemberRefs,
compare unchanged controls with the original generator, and require strict PE round
trips. Temporary fixtures remain ignored; no permanent tests are introduced here.

Then generate a fresh native candidate outside the game and require the complete
191-assembly reference/member/signature/roundtrip and sidecar checks. Retain all
original pins and failed evidence. Later loader/menu/normal-exit/rollback checks and
native gameplay adapters remain separate gates. Full MW2/skate mod acceptance and
two-client server flows are still unfinished; the owner tests after those gates.

## Build separately

Use PowerShell 7.4 or later and installed Git. On Windows the recipe can download
the SHA512-pinned portable SDK into its new build directory:

```powershell
pwsh -NoProfile -File mods/rust/client/generator/interop/Build-Generator.ps1 -Destination C:\ModBuilds\interop-pointer-byref
```

The destination must be new, absolute and outside Rust game/server directories.
Symlink/junction ancestors are refused. Existing output is never overwritten.
Other platforms require an explicit `-DotNetPath` to SDK 8.0.425; `-GitPath` selects
a specific Git executable, and `-SourceArchive` accepts the pinned upstream ZIP.
The local Git attributes preserve all hash-checked text as LF even with Windows
automatic line-ending conversion enabled.

`stage/` contains only the Generator DLL, licences, a build manifest and complete
corresponding source archive/patch/recipe/locks. Keep that source and licences with
any distributed candidate. No Common/Runtime replacement is staged; the manifest
records the built Common identity only. Do not install the candidate until the
full native generation, output and loader gates above pass.

## Verified scope

Actual Linux x64 source builds and the final PowerShell recipe completed with
SDK 8.0.425: four upstream warnings, zero errors. Locked restore covers both
original Generator target frameworks and Common; the built/staged target is
netstandard2.1. Fifteen temporary recipe scenarios passed for path/hash/SDK
rejections and environment restoration, including absent/empty/nonempty values.
An isolated `core.autocrlf=true` Git clone preserved all six pinned payload hashes.
These are Linux observations, not native Windows junction/drive/build proof.

The actual original and final staged Generator DLLs were exercised on authored
metadata models: 108 production-pass methods and 64 actual unstrip invokers.
Each caller covers 32 static/instance `ref`/`out` pointer cases with ordinary
zero-flag `ref` metadata, both argument slots, int/void/nested/struct pointers.
All 108 unaffected value/reference/string/generic/by-value-pointer controls have
identical normalized IL, locals, signatures and flags versus the original DLL.
The original output contains eight pointer-owned constructor MemberRefs across
the two reads; the patched output contains zero. Four authored PE write/reread
round trips preserve full body inventories and normalized instructions/locals.
Independent BCL decoding inspects the actual serialized TypeSpecs/MemberRefs.
Both fixture builds have zero warnings/errors; the original/patched runs pass
4,247/4,218 assertions. Temporary fixtures/logs remain ignored.

The authored minimal corlib is metadata scaffolding; no emitted IL or native call
is executed, and external Runtime import resolution is not attempted. These
structural/behavioral checks do not establish full CIL/dependency/native validity.
The cloud session has no private retail/native fixtures or Mac/Shadow control.
No fresh 191-assembly native generation, sidecar audit, loader install or gameplay
follows from these source checks. All those acceptance gates remain open.
