# Pointer byref and nested method generation repair

Source for the separate Il2CppInterop generator correction defined in merged
[PR #295](https://github.com/Dj-Shortcut/mw2-rust-rust-rewrite/pull/295), tracked in
[issue #289](https://github.com/Dj-Shortcut/mw2-rust-rust-rewrite/issues/289).
The existing [Cpp2IL recipe](../README.md) addresses source-model generation;
it does not apply this correction. This directory supplies a pinned patch and
source-build recipe; no DLL or client installation is published here.

The nested-method traversal correction in [issue #303](https://github.com/Dj-Shortcut/mw2-rust-rust-rewrite/issues/303)
/ [merged design PR #304](https://github.com/Dj-Shortcut/mw2-rust-rust-rewrite/pull/304) is implemented and verified on authored models.
Actual SDK8.0.425 source builds and isolated baseline/candidate generator checks pass on macOS.
Fresh Windows generation, full output acceptance and the loader remain unverified.

## Observed failure

An actual native Windows audit rejected a generated imported member after only
3 of 191 complete assembly round trips. An independent CLR8 passive metadata read
confirmed an instance constructor taking `IntPtr` whose declaring type is an
unmanaged pointer. All four pinned inputs remained unchanged before and after
that read. This is rejected generated metadata, not accepted output or gameplay.
A later passive native scan found one constructor use in a static caller ending
in an out pointer-byref parameter. Its signature/local pattern matches the authored
reproduction, while precise emitting-pass attribution remains unconfirmed. The
authored generator reproduction below is a separate check.
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

## Nested method traversal correction

A fresh native candidate was rejected on an unresolved imported member after
107 complete assembly round trips. A separate passive original-only diagnosis
found an empty nested owner; the exact bound Unity library contains an eligible
managed callee and nine source-only callers. These source observations do not
establish the generated caller's emitter or the earlier failure's reread phase.

Pinned Pass79 restores nested types, but Pass80 methods only visited top-level
types and Pass81 consumes that method queue. The additional Pass80 change keeps
the complete original global top-level phase as the queue prefix, then walks the
source nested tree in stable order using an explicit stack. It reuses the old
per-type method block unchanged, preserving eligibility, signatures, attributes,
properties, InternalCall handling and managed queue behavior.

Each descendant requires its exact mapped parent and an already-registered target
context. Prefer the original full-name registration and verify target ownership.
For a newly cloned child registered under a renamed output parent, use the unique
source-name child within that parent and its existing target-definition context.
Missing mappings are reported and skipped; ambiguous/mismatched registrations or
cyclic ownership fail. No orphan promotion, global simple-name search, new type
context or traversal of generated delegate types is added.

Type and field restoration remain unchanged. Pass81 still replaces failed body
translation with its NotSupportedException fallback. A restored method definition
or resolved MemberRef does not prove successful body restoration; field/getter,
local/signature and unsupported-operation dependencies require separate checks.
The prior pointer-byref hunks are unchanged. No native fix or output acceptance
is claimed by this source correction.

## Build and verification contract

The recipe pins the source archive, three-file patch, SDK 8.0.425 and complete
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

For nested restoration, use actual baseline/candidate passes in isolated processes
with authored nested-call, deep-owner, same-name, renamed-parent and existing-method
controls. Preserve the old top-level queue prefix and exclusion/parameter/generic/
property/InternalCall outcomes. Check translated field/getter routes and negative
body dependencies, distinguishing usable bodies from fallback throws. Require
strict authored PE round trips and imported-member closure before native repetition.

Then generate a fresh native candidate outside the game and require the complete
191-assembly reference/member/signature/roundtrip and sidecar checks. Retain all
original pins and failed evidence. Later loader/menu/normal-exit/rollback checks and
native gameplay adapters remain separate gates. Full MW2/skate mod acceptance and
two-client server flows are still unfinished; the owner tests after those gates.


## Authored nested-method verification

Both actual Generator builds use the original SDK8.0.425, projects, global policy
and complete restore locks. The macOS source builds pass with four upstream
warnings and zero errors. An explicit local feed of69 archived packages matches
both full locked dependency graphs through the SDK's NuGet content-hash reader.
The complete PowerShell build/staging recipe did not finish on macOS because its
NuGet service-index request failed; these results are source compilation only.
The fresh Windows recipe and corresponding-source staging remain a separate gate.

One authored fixture is compiled against the baseline and run in two fresh CLR9.0.20
processes. Only Generator changes; the fixture, Common, vendor dependencies and
metadata-only official Runtime/framework references have identical pinned bytes.
Both modes pass32/32 checks and three strict authored PE round trips. The twelve
original top-level queued methods remain the exact global prefix. The candidate's
live queue assertion appends fourteen eligible nested methods in the explicit
expected owner/sibling/depth order; twelve bodies translate successfully and two
negative cases retain the existing exception fallback. The tail is asserted by
source-object identity, while the prefix and28 unchanged native-sentinel/getter/
top-level control fingerprints are exported and independently compared.

Checks cover deeper calls, same names in distinct owners/assemblies, renamed
parents, existing-method skips, constructor/abstract/no-body/array exclusions,
In/Out parameter metadata and a generic identity. The field-shaped marshaller
uses the actual Pass40-created parent IntPtr getter: its three-instruction body
and exact owner/signature resolve again after strict serialization. Missing-field
and unsupported-operation fallback bodies are checked separately. The existing
native method body is an authored sentinel, not a Pass50/native-call execution.

The baseline reproduces exactly nine missing imported methods. Both first written
and strict rebuilt/reread PE boundaries require that exact baseline set; the
candidate requires zero misses at both boundaries. Ordinary in-memory modules
do not enumerate serialized import rows, so the fixture also checks CIL operands
before serialization. Unexpected unresolved references remain fatal. All fourteen
baseline and fifteen candidate file obligations pass before/final checks;52 root
input checks also remain unchanged. No inspected/generated retail DLL is CLR-loaded
and no native Unity call is executed. These results do not accept the rejected
native output, sidecars, installed loader or full mod.

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

## Verified pointer-only scope (prior version)

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

A subsequent actual Shadow Windows source build completed with normal exit 0,
confirmed root exit and complete streams. All 1,347 input and 15 build/staging
output hashes were preserved, the output inventory was unchanged, and no
recheck/cleanup failures were reported. The actual staged Generator SHA256 is
`364e6baa9c6ad50456a45aa056d97879ec759c9404ec3a98d2864bbea33c591e`,
with product version 1.5.3. The result and observation digest were read from the
live Windows console; the full report has not been copied to the Mac. See the
[native build result and exact limits](../../../../../docs/RUST-CLIENT-LOADER.md#actual-windows-pointer-generator-build).
This proves source-build/staging completion only; no fresh generation, candidate
acceptance, installation or game start followed from this build.

A separate observed Windows passive caller diagnostic decoded all 513,677 expected
method bodies in the rejected assembly and found one use of the invalid constructor,
with no scan error or truncation. All four original pins and 193 wrapper file checks
remained unchanged; the child exited normally with complete streams. This read-only
result concerns the rejected assembly, not the patched Generator or a fresh full
candidate. It does not supply native regeneration, output, sidecar or loader approval.
