# Skate motion through the native collision scene

Part of [the full mod](RUST-MW2-SKATE.md); [task #327](https://github.com/Dj-Shortcut/mw2-rust-rust-rewrite/issues/327).
Status: shared collision source and actual calculated core/scene flow observed; connected player movement pending.

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

## Observed native results

On 9 October 2026 the exact production source built as net48/C#7.3 against
262 current genuine Rust/Oxide/Unity references with zero warnings/errors.
All 274 source/project/reference inputs stayed unchanged during preparation,
deployment and execution. The temporary Oxide composition compiled and loaded
on the existing Linux server with no connected players; public game access
remained closed, RCON/game stayed loopback and Rust+ stayed disabled.
ShortcutLoadouts source and configuration were unchanged.

The first run stopped at `flat-supported`: mount succeeded (Grounded, zero
velocity, up normal), but Y301.154752836 did not match the analytic Y301.152
expectation within 0.001m. Retained result: 2 pass / 1 fail, zero steps/poses;
all 3 owned objects actually gone and settings unchanged.
A separate native observation on three owned floor sizes found the same raw
cast distance 0.14725615084171295 and backend fraction 0.49082387862691995.
That fraction reproduces the returned anchor exactly; both returned and analytic
anchors were clear and repeating the original mount was identical. All 9 owned
objects were actually removed. This establishes native early contact and correct
propagation, not the engine's internal cause or a contact-offset causal claim.

Only the temporary oracle changed: from exact analytic plane height to an
independent direct native cast using the known pre-query inputs and exact owned,
verified floor. Compare its fraction with the production backend, then require
the core support formula, clearance, normal and state. Standing dismount records
its own support cast and subsequent 0.15m feet adjustment. No tolerance widening
or production motion/pose behavior change was made; the old failure is retained.

The corrected native run completed **75 checks, zero failures**, with **978
successful core steps and 987 calculated poses**:

| Actual fixture flow | Observed result |
| --- | --- |
| Mount, solid/missing support, push, brake and steering | accepted support; refused invalid mounts; expected motion and stop |
| Slow/fast wall, held ollie and unfinished flip | block without bail; fast collision bail; one jump/landing; unfinished flip bail |
| Standing exits | safe feet position, blocked-side fallback, both sides refused and recovery |
| Rails at yaw 0 and 90 | actual descending capture, jump/endpoint release, observed expiry, irreversible stale refusal and new-revision release |
| Walkable 20-degree slope and ledge | support/ground motion; natural loss of support and next-step gravity |
| Invalid dt/axis, missing real actor, worker thread, publication revocation and buffer saturation | refused with no partial calculated motion; real actor guards retained |

All **167 owned objects** were actually absent, every owned-probe cleanup receipt
completed and an independent scene scan found no survivors. No worker timeout;
gravity, trigger/backface query settings, simulation mode and the full 32x32 layer
collision matrix stayed unchanged. Temporary probe plugins were unloaded and their
own sources removed; private evidence was retained.

Corrected temporary source SHA256:
`495158f5a045e20600b38b2e330d7e998fafafbd4699562538faf5285af1e745`.
Corrected native report SHA256:
`613024d3c76a11f308e043495720233616f8ad22553f2ef6c21187216da39d5d`.
These identify private observations, not shipped probes or game binaries.

## Remaining product gates

There were **zero connected-player checks, authenticated world queries or native
entity movement effects**. The actor-free scene entry calculates geometry only;
it neither creates/moves a board nor grants the production wrapper's authority.
Static owned box fixtures are not full terrain, concave-interior, performance,
networking or real-world lifecycle acceptance. Calculated poses are not rendered
board/rider animation. The native input/mount/effect host, accepted client/model
route, replication and the full two-client #275 flow remain unfinished.
This is component evidence, not a playable release or an owner test request.
