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
During those 8 October checks, no clients joined. Permission/life/cooldown,
real inventory delivery and cleanup,
damage scaling, excluded-target behavior, permanent removal, player-state
persistence and full MW2/skate acceptance remain open.

## First Shadow player check

On 9 October 2026, the owner requested a narrow existing-server component check.
The server reached startup complete with public game UDP 28015, private RCON,
externally blocked query and disabled Rust+. Only the observed Shadow IPv4 was
allowed during the [bounded test window](RUST-SERVER-HOSTING.md#shadow-player-check--9-october-2026);
actual rule removal and listener closure were verified after the stopped check.
Plugin source and default config hashes remain unchanged. Authenticated console
confirmed both permissions granted only to the owner, no grants to other users
or groups, and retention across the preparation restart. Revoke/denial and
player/item persistence are separate checks below, not inferred from that restart.

| Ordered player check | Actual result |
| --- | --- |
| Owner Shadow client joins and spawns | Pass: owner reports spawn; authenticated status confirms the owner account as the sole active player and no joining players |
| `/loadout carbine`: rifle.ak x1 and ammo.rifle x120 | Fail, owner observed: command in T chat produced `unknown command`; delivery was not confirmed |
| Second request within 60 seconds: English cooldown refusal | Not run: stopped at first reported failure |
| Full main/belt inventory: English refusal, no original items lost | Not run: stopped at first reported failure |
| Revoke use permission, observe English denial, then grant again | Not run: stopped at first reported failure |
| Save, restart same world, rejoin with player and items preserved | Not run: stopped at first reported failure |

The owner also reported severe Shadow lag. Screen control returned
`timeoutReached`, so the command context and failure above are owner observations,
not an independently captured chat/inventory result. The cause has not been
established. No cooldown, inventory safety, permission denial or player/item
reload acceptance is claimed. Saving during closure does not prove rejoin
persistence. Both original grants remain; the revocation check was not reached.

Run these in order and stop at the first observed failure; any behavior fix
belongs in a separate focused PR. Record the exact failure without public
player identifiers or raw logs. Wait beyond the successful 60-second cooldown
before the full-inventory request so cooldown cannot mask inventory refusal.
Observe original items immediately before and after that request; do not clear
or replace the owner's inventory. Restore the direct use grant after the denial
check. A connection without these loadout checks is not a loadout playtest.
Two-client PvP remains open and requires a second EAC-disabled player. This
component check does not establish full MW2 gunplay/skate acceptance.

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
