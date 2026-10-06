# Bounded property-signature generator correction

Source for [issue #289](https://github.com/Dj-Shortcut/mw2-rust-rust-rewrite/issues/289).
The correction and a clean Mac source build are verified; corrected Windows
metadata parsing, complete interop generation and installation remain pending.
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
./Build-Generator.ps1 -Destination C:\Users\Shadow\Downloads\rust-generator-build
```

On Windows, omitted tool paths download digest-checked official portable SDK9.0.318
and MinGit into that directory. Existing Git is used when available.
Other platforms require an explicit `-DotNetPath` to that SDK and a working Git;
`-GitPath` selects Git explicitly. `-SourceArchive` accepts a digest-matching local
copy of the pinned upstream ZIP. No installer, game copy or global profile is used.
If fetching only this folder, `fetch-generator.ps1 -Commit <40-character repository
commit> -Destination <new absolute folder>` downloads immutable reviewable source,
records the commit, and performs no build or installation. Keep that fetch script
and commit tied to the reviewed repository revision.
Patch application pins LF output per invocation, independent of user/global Git
line-ending settings. The first native Windows attempt stopped at the source-hash
gate because Git converted patched source to CRLF; the failed directory is preserved.
A private clean-archive reproduction matched the observed CRLF hash under adverse
Git settings (0/8 source hashes matched); the corrected invocation matched all eight
pinned source hashes while preserving the caller settings. Recipe syntax passed.
A fresh corrected Windows build remains required.
The build verifies the source archive, patch, resulting eight source files, package
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
A clean source build using the published recipe passed on macOS/arm64: 38 upstream nullable and 3 archive source-link warnings, zero errors. Both resulting DLLs decode as neutral,
unsigned net6 assemblies2022.1.0.0 with informational version2022.1.0-rust-property.1;
Core's Lib/Stable reference versions match the official dependency identities.
54 temporary authored assertions against those actual DLLs passed, covering v39
widths/ownership/spans, omission/order, controlled lazy type resolution, explicit
attribute/interface failures, six incoherent managed signatures and three valid
paired/indexer/setter-only emissions. Two original-DLL fixture cases separately
reproduced the empty-setter exception and wrong first-parameter indexer value.
Primitive binary and image maps are controlled scaffolding, not retail registration.
Attribute payloads and truly parameterized generic-interface round trips remain unproved.
No permanent test was added. These results prove no Windows parser or gameplay flow.

Review the exact patch/licences before a staged Windows correction is tried.
A changed DLL's bytes do not alone invalidate BepInEx's cache: preserve/retire the
whole newly added cache before retry; preserve every original same-name path.
Every bootstrap attempt uses direct EAC-disabled RustClient.exe offline only;
no server connection or Steam/EAC launch with the folder bootstrap present.
Only complete empty-loader generation, responsive menus, normal exit and verified
rollback permit a later log-only Load probe; that proves no native adapters.
