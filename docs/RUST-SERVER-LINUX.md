# Linux Rust server installation preparation

Design for [issue #266](https://github.com/Dj-Shortcut/mw2-rust-rust-rewrite/issues/266): `mods/rust/server/Install-LinuxServer.py`.
Design published first; installer source and bounded Mac checks now exist.
See [commands and evidence](../mods/rust/server/LINUX.md); no Linux installation or playable server is verified.

## Command and failure contract

Python3 standard-library CLI: `--root /absolute/new/path --plan` prints preparation
steps on any platform without network, downloads, file writes or execution.
Actual installation requires permitted Linux x86_64/glibc, a nonroot user,
12 GiB available RAM after host/cgroup bounds, 20 GiB free disk and an existing writable parent.
Refuse existing destinations, Git roots, symlink ancestors, traversal,
control characters and paths that SteamCMD cannot safely consume.
Check path/resources before creating a fresh installation root.
Read active v1/v2 cgroup limits/usage and all ancestors; retain the host bound.
Refuse unknown/clipped namespace views; this RAM snapshot is not a reservation.
No automatic package install, sudo, service, server start, update or deletion.

Install official Linux SteamCMD, public app258550 anonymously, then matching
stable official Oxide Linux release overlay at server root. Validate release
identity, asset URL/name/size and publisher SHA256 before extraction.
SteamCMD measured hash lacks an independent publisher digest; label it as such.
Reject archive traversal, links/special entries, duplicates and unreasonable sizes.
Retain executable SteamCMD permissions; safely overwrite only installed regular
files during the intended Oxide overlay. Never overwrite old worlds/configs.
SteamCMD timeout: `--steam-timeout-seconds` 60–86400, default 14400 (4 hours).
Guard fresh-root/staging initialization; failure recording needs only the owned root.
Record release/hashes/build/stages/timeout with unique atomic temporaries; preserve failures.
Clean tracked process groups on every exit before overlay; bounded uncertainty refuses.
`installed-unverified` proves
files/recording only, not dependency/version compatibility or startup.
SteamCMD may create its normal per-user Steam cache/logs outside the new root.

## Linux control and acceptance

Linux server console input is noninteractive; screen/tmux are output surfaces.
Real commands/permissions/save/quit need authenticated RCON or separately
verified in-game administrator control on a chosen host. No public RCON needed.
Host selection, credentials, private access and startup/bind design are separate.
The owner's Shadow direct join later needs the current explicit `-insecure`
flag and a reviewed UDP game-port route. Query access is not a direct-join gate.
No VPS order before owner choice/concrete costs approval and source milestone.

Verify genuine production planning and helper refusal/IO cases in ignored
Mac probes; no mock game API or claim of native Linux installation.
Review/source/publish/asset gates precede source publication. Actual Linux
dependencies, full installation/start, matching `oxide.version`, listeners,
save/restart, intended-client join and plugin/two-client acceptance remain open.
See [hosting gate](RUST-SERVER-HOSTING.md) and [mod scope](RUST-SERVER-MOD.md).
