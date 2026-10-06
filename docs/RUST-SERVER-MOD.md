# Small existing-Rust server mod

Active direction since 5 October 2026: [issue #266](https://github.com/Dj-Shortcut/mw2-rust-rust-rewrite/issues/266).
The standalone Rust/Bevy rewrite and its automatic loop are parked.
Setup and [plugin sources](../mods/rust/plugins/README.md) exist; no playable mod is verified.

## Player flow and boundaries

Use the existing Steam PC Rust client and an Oxide/uMod Rust PC server.
Stock Rust supplies terrain, survival, building, inventory and TC rules.
Source `/loadout <name>` gives only the requesting player a configured kit,
with explicit permission, cooldown and a connected/alive/awake/non-wounded check.
Preserve inventory: reserve sufficient empty slots, prevent stacking/swapping,
and remove only newly created kit items if delivery fails. Hooks limit atomicity.
Source firearm PvP damage uses a finite positive bounded Bullet factor, default 1,
and a separate attacker permission. Use the shot's weapon, not a later held item.
NPCs, buildings/TCs, raids and non-Bullet damage retain stock behavior.
No new character/weapon assets, MW2 animations or Skate physics in this scope.
All commands, configuration feedback and player messages must be English.

## First setup milestone

Scripts: [`mods/rust/server/README.md`](../mods/rust/server/README.md), published after design.
Install official SteamCMD app 258550, then matching official Oxide.
Use a new `%LOCALAPPDATA%\CodexRustServer` root; reject an existing destination
or one inside a Git checkout. Never reset, delete or overwrite an old world.
Keep binaries, configs, credentials and saves outside the repository.
[Facepunch prerequisites](https://wiki.facepunch.com/rust/Creating-a-server):
12 GB free RAM and 15 GB free disk; reserve 20 GiB disk for preparation.
Client RAM is additional. [Shadow Gaming prohibits server hosting](https://support.shadow.tech/hc/en-us/articles/32731830348305-Rules-and-Restrictions-on-Shadow); use another host.
Start separately with a fixed `mod-demo` identity, small map, loopback game bind,
distinct game/query ports, disabled Rust+ and no configured RCON/password.
Provide argument preview; allow scalar saved settings, reject protected overrides.
Do not open firewall rules, forward ports or rent infrastructure automatically.
Actual game/query/RCON/Rust+ listeners must be checked before privacy is claimed.
## Client and acceptance

An EAC-disabled `RustClient.exe` can join only an insecure server. Use explicit
`-insecure` at server startup; `server.secure` was removed in [September 2025](https://rust.facepunch.com/news/maintenance).
Normal EAC clients/servers remain the default. Server mods do not require EAC off.
Shadow forwards the Xbox controller; Steam Input maps it to keyboard/mouse.
Rust PC has different controls/UI from Rust Console Edition; Xbox server rental
does not host this PC plugin. Controller settings are outside the server mod.
Accept setup only after real server startup, `oxide.version`, listener evidence
and intended-client join with recorded EAC/startup mode. Source checks are narrower.
Accept the later plugin after permission/cooldown/config/inventory failure flows,
two-client firearm damage, unchanged excluded targets, reload and removal pass
on recorded real Rust/Oxide versions. Carbon needs separate verification.
Mac source access works; [Windows startup](../mods/rust/server/README.md) and
[Linux installation](../mods/rust/server/LINUX.md) need native hosts; none is selected. See the [hosting gate](RUST-SERVER-HOSTING.md).
