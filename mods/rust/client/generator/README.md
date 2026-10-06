# Bounded property-signature generator correction

Design for [issue #289](https://github.com/Dj-Shortcut/mw2-rust-rust-rewrite/issues/289).
No correction is implemented or installed yet; no playable mod is available.
This supports the [offline loader gate](../../../../docs/RUST-CLIENT-LOADER.md).

## Reproduction and expected behavior
Official BepInEx be.788 includes LibCpp2IL from
[Cpp2IL 558ddd9](https://github.com/SamboyCoding/Cpp2IL/tree/558ddd98642010897d54316b51fbaa7889fda093).
Its actual Windows read-only LoadFromFile returned true for the installed Rust
build25681799/metadata39. RawPropertyType on a real getterless property whose
setter has zero parameters threw IndexOutOfRangeException in the official DLL.
No injected parameter cache was used. Original files, rows and logs stay private.
The bounded scan found 175 such setters and 938 nonempty getterless setters;
checked owner/accessor/parameter relationships passed, full byte geometry is pending.
Metadata alone cannot supply a property value type when both accessors are absent
or a getterless setter has no parameters. Emit no guessed object/void signature.
Preserve the underlying methods and original per-type property context order.
Valid setter-only/indexer properties derive their value type from the last parameter.

## Proposed correction
Validate ownership, accessor indices and parameter spans before eligibility.
Invalid present accessors, missing parser state and unresolved eligible types fail.
Keep contexts in place: named attribute member indices refer to their original order.
Tag only signatures that cannot be inferred, then omit their property wrappers
from stable naming, type providers and managed/diffable output.
Custom-attribute output skips only a tagged wrapper; a missing eligible output
member or a named attribute referring to an omitted property must fail explicitly.
Underlying accessor methods remain available; omitted wrapper reflection,
semantics and wrapper attributes are unavailable and cannot be claimed preserved.
Check explicit interface reconstruction and any attribute references separately.
Keep the patch focused, pinned and reproducible, with upstream MIT licence/credit.
[Upstream #548](https://github.com/SamboyCoding/Cpp2IL/pull/548) closed without merge;
its guard does not establish approval or cover the observed empty setter.

## Verification and retry gate
Build the genuine modified libraries; use temporary original fixtures outside Git.
Exercise absent/empty/normal/indexer signatures, invalid owners/ranges/spans,
original property-index retention, attribute references and missing output members.
Then check actual private metadata/binary and all output consumers on Windows.
Review the exact patch and licence before replacing any staged generator DLL.
Every bootstrap attempt launches direct EAC-disabled RustClient.exe offline only;
no server connection or Steam/EAC launch with the folder bootstrap present.
Only complete empty-loader generation, responsive menus, normal exit and verified
rollback permit a later log-only Load probe; that still proves no native adapters.
No permanent test, generated DLL/reference, retail file or probe is published.
