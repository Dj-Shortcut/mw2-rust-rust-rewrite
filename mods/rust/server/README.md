# Rust PC server preparation

Development bootstrap for [issue #266](https://github.com/Dj-Shortcut/mw2-rust-rust-rewrite/issues/266). [Loadout/damage source](../plugins/README.md) is included; real loading is unverified. The PowerShell scripts have not installed or started a real Windows Rust server. Do not treat them as a verified playable demo.

For Linux installation preparation and read-only preview, see [LINUX.md](LINUX.md). Native Linux installation/startup remains unverified.

## Choose a permitted host first

An existing suitable PC or a self-managed rental can run the official dedicated server. See the [hosting capability and acceptance checklist](../../../docs/RUST-SERVER-HOSTING.md) before choosing. The PowerShell scripts execute only on Windows; Linux/Oxide is a separate setup path. A rental needs the owner's choice and approval of concrete costs. Local Mac source access does not prove server startup or remote-client reachability.

Use a Windows 64-bit machine where server hosting is permitted, with at least **12 GiB available RAM** and **20 GiB free local disk**. The Rust client needs additional resources; use a separate server when cohosting does not fit. Shadow Gaming prohibits server hosting, and Neo has 16 GB total RAM: leave Shadow as the owner's reported insecure-client setup, not the server. A Rust Console Edition/Xbox server cannot load this PC plugin framework. No paid infrastructure is provisioned by these scripts.

## Install a new server

On the permitted Windows host, open PowerShell 5.1 or 7. Review the scripts before running them from your checkout:

```powershell
.\mods\rust\server\Install-Server.ps1
```

The default root is `%LOCALAPPDATA%\CodexRustServer`. Use `-RootPath 'D:\RustModServer'` for another **new** local directory whose parent already exists. Existing destinations, Git checkouts, junction parents and mapped network drives are refused. The installer never updates an existing installation or deletes a world. A failed attempt retains partial files plus its failure manifest: inspect those files before choosing another empty destination.

SteamCMD installs public app **258550** anonymously, before official Windows Oxide files are overlaid at the server root. The selected stable Oxide release must publish a SHA-256; missing/mismatching digests stop installation. SteamCMD's measured archive hash has no independently published comparison and is labelled accordingly. `install-manifest.json` records the selected release, asset identity/digests and Steam build; `installed-unverified` means files are present, **not** that the versions work together. Matching server/Oxide version and `oxide.version` remain real-host checks. `-OxideRelease 2.0.7801` pins a release tag; it does not pin the Steam server build or prove compatibility.

## Review and start separately

```powershell
.\mods\rust\server\Start-Server.ps1 -Plan
.\mods\rust\server\Start-Server.ps1 -Insecure
```

The second command is specifically for an EAC-disabled `RustClient.exe`; omit `-Insecure` for normal EAC. Preview supports an explicit Windows root from other platforms, for example `-ServerRoot 'C:\RustModServer' -Plan`. Normal start requires Windows and a completed installer manifest. The server uses identity `mod-demo`, seed 12345, map size 1500, maxplayers 4 and game/query ports **28015/28016**. Game IP is `127.0.0.1`, Rust+ is disabled, and RCON is not configured. No firewall rules or forwarding are changed. **Game bind is not proof of every listener's isolation.**

Existing `server.cfg`/`serverauto.cfg` files are preserved. Ordinary scalar settings and protected settings matching the launcher are accepted; conflicting bind, port, identity, map or player-limit values, enabled Rust+/RCON and a custom level URL are refused. Supported syntax is one dotted-name assignment per line, with a single unquoted value or a quoted value (spaces and empty values allowed), plus blank lines and `//` comments. Known command indirection, semicolons, escaped quotes and control characters are refused. This bounded check is not the full Rust console grammar; actual server-generated configs and restart still need real-host verification.

Before connecting, an integrator must see `Server startup complete`, check `oxide.version` and actual UDP/TCP listeners (game/query loopback; no unwanted RCON/Rust+ listener). Only then connect from the same Windows machine in Rust's F1 console: `connect 127.0.0.1:28015`. On a Mac keyboard, Fn+F1 may be needed in Shadow. A separate host needs separately reviewed access/bind settings; this local launcher does not expose a public server. No EAC-secure-server compatibility is claimed for Shadow.

Stop the server through its interactive console using `quit` and wait for exit. For updates, stop first and follow the publishers' SteamCMD-then-matching-Oxide order; this first installer intentionally refuses update mode. Use the [plugin removal instructions](../plugins/README.md) for plugin source. Keep installation files, saves, configs, screenshots with account details and credentials outside GitHub.

Sources: [Facepunch server setup](https://wiki.facepunch.com/rust/Creating-a-server), [Oxide installation](https://docs.oxidemod.com/guides/owners/install-oxide), [Rust+ disable syntax](https://wiki.facepunch.com/rust/rust-companion-server), [EAC-disabled client](https://support.facepunchstudios.com/hc/en-us/articles/15041503601437-Launching-Rust-with-EAC-disabled-RustClient-exe), [current insecure-server flag](https://rust.facepunch.com/news/maintenance), [Shadow hosting restriction](https://support.shadow.tech/hc/en-us/articles/32731830348305-Rules-and-Restrictions-on-Shadow).
