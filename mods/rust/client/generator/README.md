# Bounded property-signature generator correction

Source for [issue #289](https://github.com/Dj-Shortcut/mw2-rust-rust-rewrite/issues/289).
The current accessor correction builds on Mac; the previous revision also built on actual Windows.
Actual Windows metadata loaded with the previous revision; the current Windows build, full property coverage, complete interop generation and installation remain unfinished.
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
Actual corrected LoadFromFile returned true with unchanged retail hashes. The full
property scan remains blocked; complete loader generation is unverified.
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

The current clean Mac build passed with 41 warnings and zero errors. The actual
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

A fresh current Windows build and parser scan are pending. Expected raw
classification is 44,629 resolved, 175 tagged getterless empty setters and 26,618
tagged VOID-getter/empty-setter rows, with zero exceptions; this is an acceptance
target, not a measured result. Full native generation must also pass before the
corrected empty loader can meet its offline menu/exit/rollback gate.

57 temporary recipe checks passed for actual SDK availability/caller-policy
distinction and mocked download failure preservation/retry; both scripts parse.

Review the exact patch/licences before a staged Windows correction is tried.
A changed DLL's bytes do not alone invalidate BepInEx's cache: preserve/retire the
whole newly added cache before retry; preserve every original same-name path.
Every bootstrap attempt uses direct EAC-disabled RustClient.exe offline only;
no server connection or Steam/EAC launch with the folder bootstrap present.
Only complete empty-loader generation, responsive menus, normal exit and verified
rollback permit a later log-only Load probe; that proves no native adapters.
