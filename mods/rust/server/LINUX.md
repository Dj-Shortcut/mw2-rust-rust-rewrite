# Linux Rust/Oxide installation preparation

Source for [issue #266](https://github.com/Dj-Shortcut/mw2-rust-rust-rewrite/issues/266). No native Linux installation/startup or playable server has been verified. The standalone rewrite remains parked.

## Preview without installing

Python3.9+ standard library; the plan uses a platform-independent Linux path and makes no network/file changes:

```sh
python3 mods/rust/server/Install-LinuxServer.py --root /srv/rust-new --plan
```

Use an absolute Linux destination without spaces, traversal or shell punctuation. The plan prints intentions only; it does not establish that the target path, resources or packages exist on the chosen host.
No provider is chosen or paid for. The owner requires source work before rental and approves the chosen host and concrete costs separately; see the [hosting gate](../../../docs/RUST-SERVER-HOSTING.md).

## Install later on a permitted Linux host

Actual installation requires **Linux x86_64/glibc**, a dedicated **nonroot user**, at least **12 GiB available RAM**, **20 GiB free disk**, and an existing writable/trusted parent. Keep that parent exclusive during installation. Existing destinations, Git checkouts and symlink components are refused. This is not protection against arbitrary concurrent changes by the same account.
RAM is bounded by host `MemAvailable` and active v1/v2 cgroup limits minus usage across the full ancestor view, including finite v1 memory/swap bounds. Unreadable/ambiguous controls or clipped/namespaced views with unknown ancestors are refused before writes/downloads. This snapshot reserves no future RAM.
SteamCMD needs its 32-bit runtime dependencies. Exact distro/package preparation is a chosen-host step; this source never runs sudo, a package manager or services. [Ubuntu24.04 SteamCMD](https://packages.ubuntu.com/noble/steamcmd) and [32-bit GCC runtime](https://packages.ubuntu.com/noble/lib32gcc-s1) describe relevant packages, not complete Rust compatibility.

```sh
python3 mods/rust/server/Install-LinuxServer.py --root /srv/rust-new --oxide-release 2.0.7801
```

SteamCMD install/validation allows 14400 seconds (4 hours) by default; `--steam-timeout-seconds` accepts 60–86400 for the chosen host. Exceeding it stops the process group and retains partial files; existing-root refusal still applies, with no resume mode.
Use a **new** directory whose parent is writable by that user. The installer does not update, delete or reset an old installation/world. Failure retains partial files; inspect `install-manifest.json` and `.mod-bootstrap/steamcmd-install.log` before choosing another empty root. Do not publish game files, logs with account details, saves or credentials.
Fresh-root/staging creation is guarded. Catchable interruptions record failure in the owned root even without staging; racing/existing destinations remain untouched. Root-local unique atomic temporaries permit retry. Recording is best effort if storage becomes unwritable; SIGKILL/power loss cannot be recovered automatically.
It downloads official SteamCMD, installs app258550/public anonymously with failed-command/password-prompt guards, checks installed manifest/executable, then overlays official stable Linux Oxide **at server root**. SteamCMD may also write its ordinary per-user Steam cache/logs outside that root.
The selected Oxide asset must have the expected release/name/URL/size and publisher SHA256. Download limits, bounded TAR decompression/metadata, unsafe-path/link/special/duplicate/ancestor-collision refusal, ZIP layout checks and protected data namespaces bound extraction. SteamCMD's recorded measured hash has no independently published comparison. Every launcher exit, caught interruption and timeout cleans the tracked installer group before overlay; an unreaped leader protects group identity during signaling. Cleanup is bounded and refuses continuation if group disappearance cannot be established, including un-reaped orphan zombies. Partial evidence remains.
`installed-unverified` records files, Steam build and Oxide identity only. The pinned Oxide tag does not pin Steam's evolving public build or prove compatibility. No server, world, plugin, port, RCON credential or service is started/configured by this installer.

## Control and actual acceptance

Linux server console input is noninteractive: screen/tmux show output. Real `oxide.version`, permissions, save and `quit` need authenticated RCON/private access or separately verified in-game administrator control. That access/credential/startup design is a separate selected-host task; no public RCON is required.
For Shadow's EAC-disabled client, later startup needs explicit current `-insecure` plus a reviewed **UDP game-port** route for F1 direct join. Query exposure is only for separately required discovery. The Windows loopback launcher is not a Linux/remote-client launcher.
After a real install, record actual server startup, matching framework, listeners, intended-client join, save/restart and the [plugin acceptance](../../../docs/RUST-SERVER-PLUGIN.md). No playable/ready claim before those results.

## Source evidence

On macOS arm64/Python3.14.6: source compiles; actual CLI plan and Mac install refusal pass. 82 regression and 93 cgroup/timeout/manifest assertions pass, plus 59 initialization/signal/manifest/group checks using owned IO/processes. Background helpers stop after success/nonzero/signal/timeout exits and unrelated groups survive. Darwin zombie-only signaling denial is a real rejected path; one observation-failure check uses an explicit syscall seam. These checks do not prove native Linux cleanup. These are narrower than native Linux/SteamCMD execution.
Actual current official SteamCMD archive and publisher-hash-verified Oxide2.0.7801 archive pass production inspection/unpacking in ignored local folders. No SteamCMD, Rust or plugin process was run. Dependencies, binaries, own fixtures and probes stay ignored; no permanent tests added.
See [design and remaining gates](../../../docs/RUST-SERVER-LINUX.md).

Sources: [Facepunch server setup and Linux control](https://wiki.facepunch.com/rust/Creating-a-server), [Oxide installation](https://docs.oxidemod.com/guides/owners/install-oxide), [official pinned Oxide release](https://github.com/OxideMod/Oxide.Rust/releases/tag/2.0.7801), [kernel cgroup v2](https://docs.kernel.org/admin-guide/cgroup-v2.html), [kernel memory v1](https://docs.kernel.org/admin-guide/cgroup-v1/memory.html).
