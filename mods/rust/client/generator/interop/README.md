# Pointer byref generation repair

The source correction for [issue #289](https://github.com/Dj-Shortcut/mw2-rust-rust-rewrite/issues/289)
preserves unmanaged pointer arguments during Il2CppInterop by-reference marshalling.
It supplies a separate generator build; the [Cpp2IL recipe](../README.md) supplies
source models and does not apply this patch. No playable mod or accepted native
interop output is supplied here.

## Behavior

The pinned upstream generator treats direct pointers as value-like, but two byref
predicates use the narrower value-type classification. A pointer passed by ref or
out can consequently enter object conversion and copyback, emitting a constructor
whose declaring type is an unmanaged pointer. The patch changes exactly those two
predicates to the existing value-like classification.

Pointer arguments retain their parameter signatures and out flags and use the
existing argument-address route. Ordinary reference, string, generic and value-type
paths remain unchanged. The shared helper also serves the unstrip invocation path.
Function-pointer and modifier-wrapped signatures are outside this correction.
Member-resolution failures remain errors; failed generated DLLs are never patched
or accepted by this recipe.

An actual Windows passive metadata read confirmed an invalid pointer constructor
in rejected output; all four pinned inputs were preserved. Exact native caller
attribution and fresh repaired generation remain pending. Private game files,
metadata, generated assemblies and diagnostics are not published.

## Build

Review `Build-InteropGenerator.ps1`, `build-inputs.json` and `pointer-byref.patch`.
Use PowerShell 7.4 or later and an existing parent directory outside any game.
Every invocation and retry requires a fresh PowerShell process; the metadata helper
must not be reused in an interactive session.
`OfficialDependencyPath` must contain the seven exact original DLLs listed in the
manifest, obtained from the official BepInEx be.788 Windows x64 archive. The recipe
checks their bytes and measured metadata before building; it does not download or
replace those dependencies.

From this directory, for example:

```powershell
pwsh -NoProfile -File ./Build-InteropGenerator.ps1 -Destination C:\RustModBuilds\interop-generator -OfficialDependencyPath C:\RustModSources\bepinex\BepInEx\core
```

The destination must not exist. On Windows, omitted tool paths download the pinned
official portable SDK 9.0.318 and MinGit into the new destination. Other platforms
require explicit `-DotNetPath` and `-GitPath`. `-SourceArchive` accepts a local ZIP
whose digest matches the pinned upstream archive. No global tool installation or
game installation occurs.

The recipe pins [upstream source dbda1cb](https://github.com/BepInEx/Il2CppInterop/tree/dbda1cb353b0f4253345dc45136d170b9e50a5a0),
the two resulting source files, patch, package configuration and both lockfiles.
It restores the original Generator net472/netstandard2.1 and Common netstandard2.0
graph in locked mode, retaining upstream project imports. A recorded SDK profile
selects 9.0.318; assembly versions remain 1.5.3.0.

Before staging, it compares Generator/Common identities, target frameworks,
definition flags, full assembly references and raw reference flags against the
original official DLLs. Public-key tokens are measured values, not a cryptographic
strong-name verification claim. Only the built Generator DLL, licences, modification
notice and build manifest enter `stage/`; Common and third-party DLLs remain
unchanged inputs. Source, input and staged Generator hashes are rechecked after work,
including failures. Failed fresh directories are retained; retries use new paths.
Embedded source paths and build metadata may vary; byte-identical builds are not
claimed.

## Verification and remaining gates

A root-executed clean macOS source build completed with four upstream warnings and
zero errors. The full recipe passed with 20/20 input rechecks, unchanged prepared
source, an unchanged staged DLL and no cleanup failures. It staged no files in Rust.

Actual built original and patched generators passed 86 authored temporary checks
per run. These exercise static/instance pointer ref/out and nested-pointer cases,
ordinary reference/string/generic/value controls, the shared helper, its unstrip
consumer and strict authored PE method/signature/local/IL round trips. All 48 control
fingerprints matched across original and patched runs; all 9 pinned fixture inputs
were preserved. The exact DLL from the full recipe also passed these checks.
Those fixture processes ran on macOS CLR 9.0.20, not Windows CLR 6.0.7. Fixtures remain
ignored and are not permanent tests or part of this source package.

Native Windows recipe execution, caller attribution, fresh generation and complete
191-assembly member/signature/reference/roundtrip plus sidecar acceptance remain
open. No generated retail assembly has been CLR-loaded by these checks. Native
calls, empty-loader menu/normal-exit/rollback, gameplay adapters and the complete
[MW2/skate mod acceptance](../../../../../docs/RUST-MW2-SKATE.md) remain separate
unfinished gates. The owner tests after the complete flow is verified.

## Licences

`UPSTREAM-LICENSE` retains upstream's complete LGPLv3 text and incorporated GPLv3
terms. `pointer-byref.patch` modifies that library under LGPL-3.0-only; keep
`MODIFICATIONS-LICENSE` and the dated `MODIFICATION-NOTICE.md` with corresponding
source and any eventual binary distribution. This LGPL boundary is separate from
the parent Cpp2IL/MIT correction. The project-authored build script follows the
repository Apache-2.0 licence. No third-party binary or proprietary game content
is distributed in this source package.
