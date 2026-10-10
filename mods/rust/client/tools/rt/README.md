# Il2CppInterop.Runtime changes

Two files from [BepInEx/Il2CppInterop](https://github.com/BepInEx/Il2CppInterop) at commit
`dbda1cb353b0f4253345dc45136d170b9e50a5a0` (version 1.5.3, the runtime bundled with BepInEx be.788),
modified for the Rust client on Unity 6000.3. They are LGPL-3.0-only, like upstream; the licence text
is in [`mods/rust/client/generator/interop/UPSTREAM-LICENSE`](../../generator/interop/UPSTREAM-LICENSE).

| File | Upstream path | Change |
| --- | --- | --- |
| `InjectorHelpers.cs` | `Il2CppInterop.Runtime/Injection/InjectorHelpers.cs` | Upstream PR #286: resolve `Class::Init` through il2cpp API exports when no signature matches. |
| `Class_GetFieldDefaultValue_Hook.cs` | `Il2CppInterop.Runtime/Injection/Hooks/Class_GetFieldDefaultValue_Hook.cs` | Upstream PR #280: hook only a 16-byte-aligned function entry, otherwise skip the hook. Narrowed here so no other jump target is tried. Adds the `SUB RSP, imm32` signature from upstream issue #284. |

`runtime.ps1` downloads the pinned upstream source on the Windows machine, overlays these two files and builds
`Il2CppInterop.Runtime.dll`. Nothing here is installed into the game by this repository.
