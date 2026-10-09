# Skate motion through the native collision scene

Part of [the full mod](RUST-MW2-SKATE.md); [task #327](https://github.com/Dj-Shortcut/mw2-rust-rust-rewrite/issues/327).
Status: design; shared geometry extraction and actual core/scene flow pending.

## Production boundary

Keep the exact connected/alive rider, identity, owned board, board lease, main
thread, reentry and publication guards in RustSkateWorld. Never invent a player
or relax those guards for a geometry test. After its real binding succeeds, the
wrapper delegates to one internal RustSkateCollisionScene with its exact ignore
entity and fresh rider/board roots for each query.

The collision scene implements ISkateWorld over the existing actual Unity queries,
owned probe factory and exact rail catalog/lease. Its actor-free geometry entry
has no actor to ignore; it grants no player or native movement authority. Fixed
movement mask, bounded buffers and hierarchy traversal, Rust Verify, concave
refusal, valid hull/float/sweep limits, penetration restoration and freshness
checks remain shared. There is no second test-only collision implementation.
Neither entry changes native entity transforms, persistence or global physics
settings. Temporary enabling is confined to the owned parked probes.

## Native integration gate

Root freezes source/projects and the current genuine server references before
building. Compose that same production collision source and unchanged SkateMotion,
SkateRails and SkatePose into a temporary ignored Oxide probe. On the existing
externally closed server with zero connected players, create only exact owned
static fixtures and synchronize them before querying.

Observe mount/support, push/brake/steering, obstacle refusal, jump/landing/bail,
rail capture/release and safe dismount against actual colliders. Record each
actual failure; source review or direct Physics calls do not prove this core flow.
Reject stale rails/probes and invalid actor bindings without partial motion
publication. Observe actual deferred fixture/probe cleanup and unchanged global
settings. Never acknowledge Destroy submission alone as cleanup complete.

These results describe calculated skate states and poses against real geometry,
not movement of a connected player, a replicated board or a complete mod. The
native rider/board/input/effect host, accepted exact client/model route and the
full two-client #275 flow remain separate unfinished gates. No stock bike/sled,
paid change, permanent test or ShortcutLoadouts behavior change is introduced.
