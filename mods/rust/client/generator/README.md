# Property-signature and interop source generator

Source for [issue #289](https://github.com/Dj-Shortcut/mw2-rust-rust-rewrite/issues/289).
The .3 correction builds on Mac and actual Shadow Windows and passes 182 authored
temporary checks. Actual .3 raw-property scanning and Core initialization pass;
first managed assembly generation fails with two encountered interface-getter
value-type errors. This count is not a complete incompatible-type census.
[Actual evidence and next diagnosis](../../../../docs/RUST-CLIENT-LOADER.md#interface-diagnosis-before-another-correction)
retain the strict rejection. The .4 [explicit source-model API](#explicit-interop-source-api)
builds on Mac and actual Shadow Windows and passes 230 authored cases. A separate
private consumer wrote 191 interop DLLs and two sidecars; full output validation
failed after only three complete PE round trips. [Actual generation and rejected
partial validation](../../../../docs/RUST-CLIENT-LOADER.md#actual-windows-generation-and-output-validation)
record the current continuation point. Complete validation and installation
remain unfinished.
A separate [Il2CppInterop pointer-byref correction](interop/README.md) now supplies
a pinned LGPL patch and locked stage-only recipe. Actual Linux source/recipe builds
and authored original-versus-patched generator checks pass; this does not repair
or approve the preserved native candidate. Fresh complete native output validation
remains required before any installation.
No playable mod is available. This supports the [offline loader gate](../../../../docs/RUST-CLIENT-LOADER.md).

## Observed failure and behavior
Official BepInEx be.788 includes LibCpp2IL from
[Cpp2IL 558ddd9](https://github.com/SamboyCoding/Cpp2IL/tree/558ddd98642010897d54316b51fbaa7889fda093).
Its actual Windows read-only LoadFromFile returned true for installed Rust
build25681799/metadata39. RawPropertyType on a real getterless property whose
setter has zero parameters threw IndexOutOfRangeException in the official DLL.
No injected parameter cache was used. Original files, rows and logs stay private.
The bounded scan found 175 such setters and 938 nonempty getterless setters;
checked owner/accessor/parameter relationships passed; full byte geometry is pending.
Metadata supplies no value type when both accessors are absent or a getterless
setter has no parameters. The patch tags these contexts and emits no guessed type.
Underlying methods and original per-type property context order stay intact.
Valid setter-only/indexer properties use the last parameter as the value type.

## Correction and limits
The patch validates record-array bounds/strides/counts, unique property owners,
accessor ownership/local indices and parameter spans. This is not a complete
validator for every metadata section, binary registration or CLR relationship.
Invalid present accessors, missing parser state and unresolved eligible types fail.
Tagged wrappers are omitted from stable naming, type providers and managed/diffable
output. Original context indices still identify named attribute members.
Managed getter/setter types, staticness, value parameters and indexer signatures
must agree; eligible accessors and property output must exist.
Omitted wrapper reflection, managed property semantics and wrapper attributes
are unavailable. Accessor methods remain; explicit named-attribute references
and interface overrides depending on an omitted wrapper fail instead of guessing.
[Upstream #548](https://github.com/SamboyCoding/Cpp2IL/pull/548) closed without merge;
it does not establish approval of this independent correction.

## Pinned source build
Use PowerShell 7.4 or later. Review these source files before execution.
From a checkout, run with an absolute, new build directory outside any game:

```powershell
./Build-Generator.ps1 -Destination (Join-Path $HOME 'rust-generator-build')
```

On Windows, omitted tool paths download digest-checked official portable SDK9.0.318
and MinGit into that directory. Existing Git is used when available.
Other platforms require an explicit `-DotNetPath` to that SDK and a working Git;
`-GitPath` selects Git explicitly. `-SourceArchive` accepts a digest-matching local
copy of the pinned upstream ZIP. No installer, game copy or global profile is used.
If fetching only this folder, `fetch-generator.ps1 -Commit <40-character repository
commit> -Destination <new absolute folder>` downloads immutable reviewable source,
records the commit, and performs no build or installation. Keep that fetch script
and commit tied to the reviewed repository revision. A failed fetch preserves its
partial directory for diagnosis and writes no success manifest. Retry with a
different new absolute destination; partial files are never reused or deleted.
Patch application pins LF output per invocation, independent of user/global Git
line-ending settings. The first native Windows attempt stopped at the source-hash
gate because Git converted patched source to CRLF; the failed directory is preserved.
For the previous correction (.1), a private clean-archive reproduction matched the observed CRLF hash under adverse
Git settings (0/8 source hashes matched); the corrected invocation matched all eight
pinned source hashes while preserving the caller settings. Recipe syntax passed.
The previous correction (.1) Windows build completed with 41 upstream/archive warnings and
zero errors; its stage manifest has the expected Core/Lib assembly identities.
Actual corrected LoadFromFile returned true with unchanged retail hashes. That revision
blocked on paired empty setters; complete loader generation is unverified.
The build verifies the source archive, patch, resulting ten source files, package
source configuration and four separate locks. It restores the original framework
graph in locked mode, then builds net6 with SDK9/C#13. Keep upstream assembly-version
settings: Core/Lib/Wasm2022.1.0.0 and Stable0.1.0.0. Do not replace the other official
be.788 dependencies with the full build output. The stage contains only Core/Lib
DLLs, licences and an identity/hash manifest. Framework/informational-version fields
in that manifest are requested build settings; the recipe checks assembly identities.
Embedded PDB/source paths differ across builds; identical bytes are not claimed.

Upstream patch context retains Sam Byass's MIT licence in `UPSTREAM-LICENSE`.
Project-authored corrections and build/fetch recipes follow Apache-2.0 in
`MODIFICATIONS-LICENSE`. Retain both notices when distributing modified libraries.
No upstream binary, checkout, private fixture, generated reference or retail file ships.

## Verification and offline retry gate
The previous correction built on Mac and actual Shadow Windows with 38 upstream
nullable and three archive SourceLink warnings, zero errors. Its Windows parser
read unchanged retail binary/metadata and handled the original getterless empty
setter case without cache injection. All 71,422 RawPropertyType metadata getters
were invoked: 44,622 resolved, 175 getterless empty setters were tagged, and
26,625 getter-plus-empty-setter cases threw the explicit no-value guard.
A separate raw getter-type scan over those 26,625 rows resolved every type:
26,618 returned VOID, seven returned non-VOID, and none returned null or threw.
These counts describe metadata APIs only; no game getter was executed.
Two original-DLL fixture cases reproduced the empty-setter exception and wrong
first-parameter indexer value. Original rows, files and logs stay private.

### Bounded accessor correction
After relationship checks, a getter plus empty setter with a resolved VOID getter
has no inferable property type: tag and omit its wrapper, preserving both methods
and the original property slot. A resolved non-VOID getter retains getter-only
output and suppresses only the setter association. Original metadata setters and
exact method contexts remain available for reference checks. Null type resolution
and malformed spans remain errors. Named writes and interface setter references
to a suppressed setter fail explicitly, including an implementing suppressed
setter targeting an ordinary interface setter. Stable method semantics skip
omitted wrappers. Core type resolution and managed signature checks still apply.

The previous .2 clean Mac build passed with 41 warnings and zero errors. Its actual
Core/Lib DLLs decode as neutral unsigned net6 assemblies2022.1.0.0 with embedded
informational version2022.1.0-rust-property.2; Core's Lib/Stable reference versions
match official dependency identities. All 106 temporary authored checks against
those DLLs passed: serialized v39 rows, widths/ownership/spans, preserved slots
and methods, managed getter-only/indexer output, invalid resolution, VOID versus
pointer-to-VOID, stable semantics, named-write rejection, both interface directions
and an extra unassociated getter pairing. Sixteen serialized V29 payload cases
exercise local/base property indices and generic constructor substitution for
base declarations. Generic local-index resolution remains an explicit unsupported
boundary. Binary/image maps are controlled scaffolding, not retail registration;
parameter caches are not injected. These checks do not prove a full managed PE
round trip, full interop generation or gameplay. No permanent test was added.

Actual Windows verification of .2 passed: clean pinned build: 41 warnings, 0 errors,
loaded Core/Lib net6/.2 identities, unchanged retail inputs, and all 71,422 raw
property calls (44,629 resolved, 175 getterless empty setters tagged, 26,618
VOID-getter/empty-setter rows tagged,0 exceptions). Full Core initialization
completed with 162 assemblies; AttributeInjector PreProcess/Process returned without
exception once. First BuildAssemblies failed with four type failures, each caused by
`Property type cannot be void`. The resolved count includes 22,588 additional raw
VOID getters: 22,574 without setters, 14 with one-parameter setters. None is getterless.
Four type failures are not a count of all invalid properties. No generated library
was written, loader installed or retry attempted on the mutated context.

### Remaining VOID-getter correction
Resolve every present getter after both accessor-owner and parameter-span checks.
For exact raw VOID, tag and omit the entire wrapper regardless of setter presence
or count; retain both original method contexts and property slots. Preserve the
existing VOID/empty-setter reason; use `getter returns void` for newly covered
shapes. Do not derive a replacement type from the setter. A managed explicit-interface
getter must have a non-VOID return signature before it is queued for wrapper creation
or added as a MethodImplementation, including concrete generic targets. This checks
the implementing method itself and does not discard independently usable shared methods.
Pointer-to-VOID remains eligible; unresolved eligible types and malformed relationships remain errors.
Keep managed VOID rejection and named-attribute/interface reference guards.
Getterless setter-value VOID and class/value-type aliases are outside this change.

The .3 source implements this correction. A fresh pinned Mac recipe build passed
with 41 upstream/archive warnings and zero errors; all nine patched source hashes
and four unchanged package locks match. Actual neutral unsigned net6 Core/Lib DLLs
have assembly version 2022.1.0.0 and informational version 2022.1.0-rust-property.3.
All 182 authored checks against those DLLs pass; their fixture build has zero warnings
and errors. The 76 new checks were also run against the genuine .2 DLLs: 53 fail,
including six interface cases that emitted the offending association before rejection.
The .3 checks retain valid generic getters, ordinary VOID interface methods, shared
setters and pointer-to-VOID, and reject malformed accessors, unresolved getter types,
missing managed getter signatures and model-only interface getter associations.
No parameter cache or retail bytes were supplied to these authored fixtures.

Actual Windows .3 build passed with 41 warnings and zero errors. The full raw scan
measured 22,041 typed, 175 getterless-empty omissions, 26,618 VOID/empty omissions
and 22,588 additional VOID omissions, with zero exceptions across 71,422 rows.
Core initialized with 162 assemblies and AttributeInjector ran once. First managed
assembly generation failed with two encountered interface-getter value-type errors;
this is not a complete incompatible-type census. No output libraries or interop
were produced, and the mutated failed context was not retried. Seven independent
serialized method/type-bit comparisons pass; their registration/PE mapping still
comes from Cpp2IL. Exact vtable/offset/slot diagnosis identifies one candidate and one matching target definition per failure; neither target
is a concrete generic method context and interface reflection reports IsGenericType=false. Both candidate table
rows also match directly decoded metadata bytes. Upstream #567 leaves these pairs
because both targets have empty interface-offset arrays. Registration provenance
and native validation of the explicit source-model contract remain open; no
signature or override was changed by this diagnostic work.
Full native generation must pass before the corrected empty loader can meet its
offline menu/exit/rollback gate.

57 temporary recipe checks passed for actual SDK availability/caller-policy
distinction and mocked download failure preservation/retry; both scripts parse.

Review the exact patch/licences before a staged Windows correction is tried.
A changed DLL's bytes do not alone invalidate BepInEx's cache: preserve/retire the
whole newly added cache before retry; preserve every original same-name path.
Every bootstrap attempt uses direct EAC-disabled RustClient.exe offline only;
no server connection or Steam/EAC launch with the folder bootstrap present.
Only complete empty-loader generation, responsive menus, normal exit and verified
rollback permit a later log-only Load probe; that proves no native adapters.

## Explicit interop-source API

The .4 source implements
[the separate source-model contract](../../../../docs/RUST-CLIENT-LOADER.md#explicit-interop-only-source-models).
It adds an explicitly selected in-memory interop input API while normal assembly
generation and output remain strict. It omits all synthetic explicit-interface
MethodImpl/property construction, preserving original metadata signatures and
eligible original property semantics. Synthetic property APIs and semantic-based
names can differ. It is not a return-type correction or a normal dummy-DLL export.
The existing BepInEx default caller remains unchanged; a deliberate isolated
offline consumer, fresh native verification and output validation are required
before installation. Interface-specific guards in the omitted synthetic phase
are also absent in alternate mode; consumed original-property/type/attribute
validation remains active. Strict generation still runs that phase.

A build-attempt marker rejects reused contexts after success or failure, across
modes and formatter instances, before output-directory creation. This excludes
same-context reuse only: global caches still require one builder and a fresh
process after failure. These metadata models must not be serialized as repaired
normal dummy assemblies; the offline caller must prohibit that export.

The pinned Mac .4 recipe passed with 41 upstream/archive warnings and 0 errors,
ten changed-source hashes and four lock hashes checked. Its two staged neutral
unsigned net6 libraries retain assembly version 2022.1.0.0, with informational
version 2022.1.0-rust-property.4. All 230 authored checks pass: prior 182, plus
26 complete public builder cases and 22 selected official-consumer cases. The
fixture build has two nullable scaffold warnings and 0 errors. Raw VOID Core
resolution in the authored builder fixture uses an authored corlib binding;
raw fields remain independently asserted. Selected consumer passes and two
Pass20 CIL inspections establish bounded metadata behavior, not a complete
runner, generated PE validity, native resolution/calls or gameplay. Generic
argument and constraint-target identity coverage is incomplete.

The actual Shadow Windows .4 recipe built with 41 warnings and 0 errors; its
separate offline consumer built with 7 warnings and 0 errors and ran under the
staged exact CoreCLR 6.0.7 host. After an official instruction-set/binary-reader
bootstrap correction, fresh source-model guards passed and the official consumer
wrote 191 DLLs and two sidecars. Original strict output reread rejected the
generator's intentional unsigned neutral Runtime zero-version reference. A
passive audit found no other unresolved assembly identity in 1,172 references.
A narrowly scoped path/hash/identity-pinned alias diagnostic then ran, completed
only three PE round trips, and rejected an unresolved imported member during the
fourth assembly's original traversal. Both failed processes exited normally with
code 1 and preserved checked input hashes/inventories. The candidate stays private
and rejected; do not repeat generation or install it as a validated result.

Next, diagnose the exact member/signature rejection without weakening the
resolver guards or rewriting candidate files. Complete 191-library metadata/PE
validation, passive sidecar validity and runtime/native closure remain open.
The public recipe and stock BepInEx caller remain unchanged. No candidate
installation or Rust retry has occurred; responsive empty offline menus, normal
exit and verified rollback remain later gates, followed by full mod acceptance.
See the [full actual evidence and limits](../../../../docs/RUST-CLIENT-LOADER.md#actual-windows-generation-and-output-validation).
