# Weapon UID gunplay state

Source slice of [the full mod](RUST-MW2-SKATE.md), [#312](https://github.com/Dj-Shortcut/mw2-rust-rust-rewrite/issues/312), part of [#275](https://github.com/Dj-Shortcut/mw2-rust-rust-rewrite/issues/275).
New C#7.3 source uses the public [gunplay](RUST-GUNPLAY-CORE.md) API.
This is a proposed contract; implementation and verification are not complete.

## Fixed roster

Create once per host-issued life lease from 0–64 unique positive weapon UIDs.
Copy immutable GunStates at one host tick; retain their profiles and state.
No enrollment, removal, transfer, profile replacement, restart or packet reset.
Reject malformed/excess bindings without eviction. Lease IDs are host fences.

## Tick and selection

Only the exact next host tick is accepted; supply complete trusted ammo observations.
Selected UID is an already confirmed physical fact; zero means no weapon.
Reconcile all UIDs through Gunplay; begin equip on a confirmed selection change.
From that common original-tick baseline, stage separate action and fallback branches.
Tick every UID once. Only the selected UID receives already-gated input and pose.
All other UIDs, and all fallback UIDs, receive no input and Unavailable mode.
Fallback never starts from an already advanced candidate.
Both branches retain confirmed selection, including its equip/release requirements.

## Publication

One opaque candidate binds owner, epoch, original snapshot and tick.
Applied publishes action; confirmed zero effects publishes fallback at the same tick.
Foreign, stale, repeated and overlapping work publishes nothing.
Unknown/partial effects close permanently; there is no restart/reconciliation API.
Historical states and candidate outputs authorize no native effect.

## Adapter and evidence limits

The adapter fences ownership/stock, [Rider](RUST-RIDER-CONTROLS.md), effects and settlement.
It closes both coordinators when effects or settlement become ambiguous.
Fresh stock/selection changes after preparation require closure and native read-back.
Reserve is an observed count: weapons can share it; never sum or allocate it.
Only the selected weapon may request ammo changes; native stock must be revalidated.
Source locks do not prove native/Rider atomicity, authentication or inventory ownership.
Genuine-reference builds and ignored actual-DLL probes verify only this source boundary.
No permanent tests, probes or binaries ship. Native ammo/firing, inventory lifecycle,
client presentation, loader, server integration and complete two-client play remain open.
