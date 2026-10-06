# Bounded property-signature generator correction

Source for [issue #289](https://github.com/Dj-Shortcut/mw2-rust-rust-rewrite/issues/289).
The .3 correction builds on Mac and actual Shadow Windows and passes 182 authored
temporary checks. Actual .3 raw-property scanning and Core initialization pass;
first managed assembly generation fails with two encountered interface-getter
value-type errors. This count is not a complete incompatible-type census.
[Actual evidence and next diagnosis](../../../../docs/RUST-CLIENT-LOADER.md#interface-diagnosis-before-another-correction)
retain the rejection pending vtable/slot/signature verification.
Complete interop generation and installation remain unfinished.
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
The build verifies the source archive, patch, resulting nine source files, package
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
and a separately reviewed interop-source contract remain open; no signature or
override was changed by this diagnostic work.
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

## Proposed explicit interop-source API

Design only. [The separate source-model contract](../../../../docs/RUST-CLIENT-LOADER.md#proposed-interop-only-source-models)
adds an explicitly selected in-memory interop input API while normal assembly
generation and output remain strict. It omits all synthetic explicit-interface
MethodImpl/property construction, preserving original metadata signatures and
eligible original property semantics. Synthetic property APIs and semantic-based
names can differ. It is not a return-type correction or a normal dummy-DLL export.
The existing BepInEx default caller remains unchanged; a deliberate isolated
offline consumer, fresh fixture/interop/native verification and output validation
are required before installation. No such source mode is implemented yet.
