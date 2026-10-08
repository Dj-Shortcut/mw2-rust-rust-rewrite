# Rust loadout and PvP plugin

Design for [issue #266](https://github.com/Dj-Shortcut/mw2-rust-rust-rewrite/issues/266).
Source target: `mods/rust/plugins/ShortcutLoadouts.cs`; stock Rust remains the base.
Design published before implementation; source now exists and compiles locally.
See [installation/configuration and exact source evidence](../mods/rust/plugins/README.md).
Native compile/load and bounded configuration/control checks are verified; real player behavior and full mod gameplay remain unverified.

## Player and configuration contract

`/loadout <name>` grants the named configured kit only to its caller.
Require `shortcutloadouts.use`, connected/alive/awake/non-wounded human state,
and a successful-grant cooldown of 1–3600 seconds, default 60. No target-player
argument, admin bypass, inventory wipe, arbitrary commands or automatic grant.
Kits contain bounded firearm/ammunition shortnames and amounts; resolve real
item definitions before enabling. Reject unknown names, invalid amounts,
excessive kits/items and nonfinite config without rewriting the file.
Reserve enough empty main/belt slots, precreate the kit and recheck each slot;
insert with stacking/swapping disabled. Confirm every new item in its reserved
slot before committing cooldown. Block reentrant requests through cleanup.
Failure removes only items this attempt created, wherever another hook moved
or dropped them; original inventory is never cleared/restored. Other plugins'
independent side effects cannot be made globally atomic by this hook.

`shortcutloadouts.damage` is a separate attacker permission. A finite factor
0.1–4, default 1, scales only positive Bullet damage from a recorded firearm
weapon prefab between distinct connected human players. Use shot provenance,
not the attacker's current held item. Default 1/no permission passes through.
NPCs, self-hits, structures, TCs, raids and other damage types keep stock rules.
Invalid config disables both features; unloading changes no items or damage.
Reload clears in-memory cooldowns; no player data or world-save schema is added.
All command, config and player feedback is English.

## Native server evidence

On 8 October 2026, unchanged ShortcutLoadouts 0.1.0 compiled and loaded on
Ubuntu 24.04.5 x86_64, [recorded Steam server build](../mods/rust/server/LINUX.md#native-installation-evidence)/protocol 2634.289.1
and official Oxide 2.0.7815. Authenticated private console and real server logs
confirmed compile success and `oxide.plugins` loaded state.

The actual generated config matched Version 1/cooldown 60/factor 1 and
`carbine` = rifle.ak 1 plus ammo.rifle 120. Both permission names were registered,
with no user/group grants; grant/revoke and in-game denial were not tested.
An added unknown field was rejected with a both-features-disabled log message,
and its file SHA256 was unchanged after reload. Original config bytes were
restored exactly, reloaded, and preserved across temporary unload/load and
server restart. These observations do not directly inspect the private ready
flag or prove live player behavior while disabled.

The server saved 3394 entities, quit and later loaded 3394 entities/navmesh under
the same identity/build, with this plugin loaded and its config unchanged.
The quit process ended with SIGKILL; normal exit 0 is not claimed.
No clients joined. Permission/life/cooldown, real inventory delivery and cleanup,
damage scaling, excluded-target behavior, permanent removal, player-state
persistence and full MW2/skate acceptance remain open.

## Verification and delivery gate

Before asking for rental: implement and independently review the source,
compile against recorded genuine Rust/Oxide dependencies where available,
and exercise actual pure production helpers with ignored temporary probes.
A mock API or compilation alone proves no real plugin load or player behavior.
Later require actual Oxide load/commands, permissions/life/cooldown/config refusal,
full inventory, creation/move failure cleanup, reentrancy, weapon switching,
two-client Bullet damage with unchanged exclusions, reload/removal and restart.
Record exact server/framework/client modes. No playable/ready claim before that.
See [server preparation](../mods/rust/server/README.md) and
[hosting gate](RUST-SERVER-HOSTING.md). Do not order a server before the owner's
requested source-work milestone and approval of concrete costs.
