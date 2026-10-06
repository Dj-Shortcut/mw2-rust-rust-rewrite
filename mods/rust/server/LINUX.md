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
SteamCMD needs its 32-bit runtime dependencies. Exact distro/package preparation is a chosen-host step; this source never runs sudo, a package manager or services. [Ubuntu24.04 SteamCMD](https://packages.ubuntu.com/noble/steamcmd) and [32-bit GCC runtime](https://packages.ubuntu.com/noble/lib32gcc-s1) describe relevant packages, not complete Rust compatibility.

```sh
python3 mods/rust/server/Install-LinuxServer.py --root /srv/rust-new --oxide-release 2.0.7801
```

Use a **new** directory whose parent is writable by that user. The installer does not update, delete or reset an old installation/world. Failure retains partial files; inspect `install-manifest.json` and `.mod-bootstrap/steamcmd-install.log` before choosing another empty root. Do not publish game files, logs with account details, saves or credentials.
It downloads official SteamCMD, installs app258550/public anonymously with failed-command/password-prompt guards, checks installed manifest/executable, then overlays official stable Linux Oxide **at server root**. SteamCMD may also write its ordinary per-user Steam cache/logs outside that root.
The selected Oxide asset must have the expected release/name/URL/size and publisher SHA256. Download limits, bounded TAR decompression/metadata, unsafe-path/link/special/duplicate/ancestor-collision refusal, ZIP layout checks and protected data namespaces bound extraction. SteamCMD's recorded measured hash has no independently published comparison. Caught interrupts/timeouts stop the installer process group and retain partial evidence.
`installed-unverified` records files, Steam build and Oxide identity only. The pinned Oxide tag does not pin Steam's evolving public build or prove compatibility. No server, world, plugin, port, RCON credential or service is started/configured by this installer.

## Control and actual acceptance

Linux server console input is noninteractive: screen/tmux show output. Real `oxide.version`, permissions, save and `quit` need authenticated RCON/private access or separately verified in-game administrator control. That access/credential/startup design is a separate selected-host task; no public RCON is required.
For Shadow's EAC-disabled client, later startup needs explicit current `-insecure` plus a reviewed **UDP game-port** route for F1 direct join. Query exposure is only for separately required discovery. The Windows loopback launcher is not a Linux/remote-client launcher.
After a real install, record actual server startup, matching framework, listeners, intended-client join, save/restart and the [plugin acceptance](../../../docs/RUST-SERVER-PLUGIN.md). No playable/ready claim before those results.

## Source evidence

On macOS arm64/Python3.14.6: source compiles; actual CLI plan and Mac install refusal pass. 82 temporary actual-production helper/CLI/own-file IO/POSIX-process assertions cover paths, metadata, archives, partial-state manifests and timeout-group cleanup; these are narrower than native Linux/SteamCMD execution.
Actual current official SteamCMD archive and publisher-hash-verified Oxide2.0.7801 archive pass production inspection/unpacking in ignored local folders. No SteamCMD, Rust or plugin process was run. Dependencies, binaries, own fixtures and probes stay ignored; no permanent tests added.
See [design and remaining gates](../../../docs/RUST-SERVER-LINUX.md).

Sources: [Facepunch server setup and Linux control](https://wiki.facepunch.com/rust/Creating-a-server), [Oxide installation](https://docs.oxidemod.com/guides/owners/install-oxide), [official pinned Oxide release](https://github.com/OxideMod/Oxide.Rust/releases/tag/2.0.7801).
