# Pointer byref generation repair

Design for a separate Il2CppInterop generator correction, tracked in
[issue #289](https://github.com/Dj-Shortcut/mw2-rust-rust-rewrite/issues/289).
The existing [Cpp2IL recipe](../README.md) addresses source-model generation;
it does not apply this correction. No repaired generator is supplied yet.

## Observed failure

An actual native Windows audit rejected a generated imported member after only
3 of 191 complete assembly round trips. An independent CLR8 passive metadata read
confirmed an instance constructor taking `IntPtr` whose declaring type is an
unmanaged pointer. All four pinned inputs remained unchanged before and after
that read. This is rejected generated metadata, not accepted output or gameplay.
The exact caller is being traced privately before attributing the emitting path.
Game files, candidate assemblies, raw metadata and diagnostic logs stay outside
GitHub.

## Proposed correction

Pin [Il2CppInterop source dbda1cb](https://github.com/BepInEx/Il2CppInterop/tree/dbda1cb353b0f4253345dc45136d170b9e50a5a0).
Its existing value-like helper includes direct pointer signatures. Apply the same
classification to the byref argument marshalling predicate and the special out
argument temporary predicate. Pointer arguments should use the existing argument
address path, without an object conversion, pointer constructor or object-reference
copyback. Keep original parameter signatures and out flags.

Ordinary references, strings, generics and value types must retain their existing
behavior. Inspect the unstrip consumer of the shared helper as well as ordinary
method generation. Function-pointer and modifier-wrapped signatures are outside
this proposed correction. Do not loosen member resolution, suppress errors or
rewrite the failed candidate DLLs.

## Build and verification contract

The implementation will provide a source-hash-pinned patch and explicit SDK/dependency
recipe with locked restore. Retain upstream project properties, assembly identities
and dependency boundaries. Stage the intended Generator replacement separately;
building it must not install files in Rust. Preserve upstream LGPL-3.0-only attribution
and corresponding source/modification terms separately from the MIT Cpp2IL material.

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
