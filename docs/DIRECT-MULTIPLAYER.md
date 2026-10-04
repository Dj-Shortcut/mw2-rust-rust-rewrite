# Direct authored-world multiplayer

Design for #248; part of [MULTIPLAYER.md](MULTIPLAYER.md).
The current `game` command remains offline. This slice adds a headless server
and reusable client adapter; graphical host/join is a later stage.

## Runtime and launch
`cargo run -p launcher --profile play --locked -- server --bind 127.0.0.1:28980`
starts one SharedSession with original procedural content, without a window,
original game files, imported-data setup, master service or certificates.
Default bind is loopback; explicit LAN addresses are supported. No authentication
or encryption is provided. TCP is a development baseline; internet FPS transport,
discovery and production-scale hosting remain unfinished. Stop with Ctrl+C.
No server saves or authenticated reconnect restoration are added.

## Transport and admission
Use nonblocking std TCP with a 4-byte LE length and maximum 256 KiB body.
Reject zero/oversized lengths before allocation, unknown magic/version/tags,
trailing data and invalid bounded payloads. Preserve inherited protocol95 codecs.
Allow two actors and two pending admissions; handshake/application deadline 5s.
Bound input to one frame, service to four messages per peer/iteration, queued
output to two frames, and writes to a 1s progress deadline. Slow peers cannot
block authority ticks. Never drop a partially written frame.
Freeze pristine authored content identity; check every fingerprint field.
The server assigns and binds socket identity, connection/epoch and actor handle.
Require same-stream acknowledgement of the complete bootstrap before gameplay.
Ignore claimed client IDs; reject foreign/stale headers and raw debug actions.
Disconnect retires the actor; later joins get fresh building ownership.

## Commands and state
One server clock advances SharedSession once per 50ms, at most four catch-up
ticks per loop. Clients never run an offline Session or mutate authority state.
One newest movement command and one typed gather/build request per actor/tick;
request IDs preserve cached outcomes and reject conflicting/expired requests.
Use full baseline-zero snapshots and complete Frame metadata, including buildings
and player lifecycle. Validate nodes, inventory and building data before apply.
Rebuild client static collision from original terrain and validated live nodes.
Only recipient carried inventory is sent; legacy building balances remain public.

## Acceptance and limits
Execute one actual server process plus two independent clients over sockets.
Verify mutual movement, finite gather → actor credit → shared Wood foundation,
consistent replicas, refusal/replay, disconnect/late join and malformed traffic.
Record commands/platform/source and distinguish this headless check from native
rendering, two-machine Windows/GPU play and release readiness. Full crafting,
combat, skating/editor, private populated inventories, persistence and latency/
loss verification remain open. No permanent test scenario is introduced.
