# Il2CppInterop modification notice

The project-authored changes in `pointer-byref.patch` are licensed under
LGPL-3.0-only. `MODIFICATIONS-LICENSE` contains the complete LGPLv3 text and
incorporated GPLv3 terms. Upstream notices and licence terms remain in force;
`UPSTREAM-LICENSE` is an unchanged copy of the pinned upstream `LICENSE`.

Modification date: 2026-10-07. The two changes use upstream's existing
`IsValueTypeLike()` classification for by-reference argument marshalling and
the special out-reference temporary predicate. They are changes to
Il2CppInterop source, separate from the Cpp2IL recipe and its modification terms.

Corresponding source consists of pinned upstream source commit
`dbda1cb353b0f4253345dc45136d170b9e50a5a0`, this patch, the build recipe,
its complete dependency lockfiles and recorded tool/input profiles. The upstream
source is available at
https://github.com/BepInEx/Il2CppInterop/tree/dbda1cb353b0f4253345dc45136d170b9e50a5a0 .
No DLLs, retail files, private research or diagnostic outputs are distributed
with this source recipe. The repository's authored-source licence remains
separate from the LGPL-covered upstream library and these library modifications.
