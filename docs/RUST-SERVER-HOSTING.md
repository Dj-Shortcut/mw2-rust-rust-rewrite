# Self-managed Rust PC test server

Preparation for [issue #266](https://github.com/Dj-Shortcut/mw2-rust-rust-rewrite/issues/266).
The owner selected and created a DigitalOcean Linux test host on 8 October 2026. Native installation is verified; remote-client and playable-mod acceptance remain incomplete.

## Use an existing machine or choose a permitted rental

We can install the official dedicated server ourselves through SteamCMD app
258550 and add matching Oxide. Stock Rust supplies the world and game rules.
Use Windows or Linux for the published [Oxide packages](https://docs.oxidemod.com/guides/owners/install-oxide).
Use [Windows preparation/startup](../mods/rust/server/README.md) or [Linux installation preparation](../mods/rust/server/LINUX.md).
Each needs its corresponding native host. Linux installation has completed on the selected host; startup/control and client acceptance are separate gates.
[Facepunch](https://wiki.facepunch.com/rust/Creating-a-server) lists 12 GB free RAM
and 15 GB free disk; reserve 20 GiB disk for our preparation, plus OS resources.
An existing suitable PC avoids a server rental; power and connectivity still matter.
Mac source access alone does not establish a usable Windows/Linux server host.

For a rental, record the exact plan, total price/currency, billing period,
setup charges and renewal/cancellation terms. The owner chooses the host and
approves concrete costs before any order. Shadow Gaming is excluded because
[its rules prohibit hosting](https://support.shadow.tech/hc/en-us/articles/32731830348305-Rules-and-Restrictions-on-Shadow).
Host Havoc declined the required startup flag in the owner's support exchange.

## Selected native test host

The owner created a Premium Intel London Droplet: four shared vCPUs,
16 GB RAM, 320 GB disk, Ubuntu 24.04 LTS x64. The dashboard showed $96 USD/month
or approximately $0.143/hour before applicable taxes. This is a private
development host, not a capacity/performance guarantee or a playable release.
[DigitalOcean billing](https://docs.digitalocean.com/products/droplets/details/pricing/)
explains that powered-off Droplets still accrue charges until destroyed.

Actual SSH access, available RAM/disk and nonroot installation are confirmed.
The recorded Steam public build/Oxide 2.0.7815 reached startup complete,
authenticated loopback console, bounded plugin checks and save/reload.
At the 8 October checkpoint, game was loopback; query bound all IPv4 interfaces
but was blocked externally by the host firewall. Rust+ reports disabled. The quit process ended with
SIGKILL; no normal-exit guarantee is claimed. See [exact native scope](../mods/rust/server/LINUX.md#native-startup-and-control-evidence).
Do not publish addresses, keys, console credentials or operational logs.
At that checkpoint, Shadow join, player-state persistence, two-client PvP and
full mod acceptance remained open. The later player check is recorded below.

## Shadow player check — 9 October 2026

The owner authorized a first real loadout component check on the existing host.
After a save and restart under the same identity, actual startup completed with
unchanged ShortcutLoadouts 0.1.0 and default configuration. Measured game UDP
28015 and query UDP 28017 bind the assigned public IPv4; RCON TCP 28016 remains
on 127.0.0.1. Rust+ is disabled and launch uses `-insecure`. The existing private
loopback TCP listener changes its ephemeral port after restart; no additional
public TCP or IPv6 listener appeared. Launch arguments contain `-insecure`,
but authenticated `status` still reports `secure`. This discrepancy remains
unresolved in meaning. The later owner spawn and authenticated active-account
check establish that this Shadow connection worked; they do not explain the
security-status string.

Only game UDP 28015 was opened, from the actually observed Shadow public IPv4
as a single `/32` to the assigned server IPv4. Address stability is unproved,
so this is a bounded test window, rather than permanent access. Active UFW
retains default inbound denial, SSH and IPv6 rules; query and RCON have no
external allow rule. A persistent calendar timer was scheduled for closure at
03:24:21 UTC on 9 October, with catch-up after boot. Closure removes the owned
rule, attempts a save and stops only the explicitly tracked test units;
it must verify configured-rule absence and disappearance of the game listener.
After stopping the player checks, actual manual closure was verified at
03:15:11 UTC: the owned rule was removed, the save succeeded and fresh reads
showed game, query and RCON listeners gone with default inbound denial active.
The owned timer was then disabled to prevent a second closure attempt.

Only the owner's account received `shortcutloadouts.use` and
`shortcutloadouts.damage`; no other users or groups held either permission.
Both grants survived this preparation restart. The owner used F1 direct join
and reported spawning; authenticated status independently confirmed that account
as the sole active player, with no joining players. The later private recording
also visually confirms the in-world spawn. It corrects the initially reported
loadout failure: T chat actually contains `/loudaout carbinew`, followed by
`Unknown command: loudaout`. That mistyped command did not exercise the requested
`/loadout carbine`; all five loadout/persistence checks remain unrun. Severe
Shadow lag was reported and live screen control timed out, but the recording
was inspected locally. No plugin registration failure is established and no
plugin behavior changed. See the [ordered results](RUST-SERVER-PLUGIN.md#first-shadow-player-check).
A join alone is setup evidence, not a completed loadout playtest.
No new rental, paid backup or paid change was made. The existing Droplet remains
billable while it exists. Addresses, account identifiers and raw reports stay
private.

## Return session — 9 October 2026

The owner requested another in-game check. The unchanged host/world and
ShortcutLoadouts source/config were restarted with external access initially
closed. The supplied source address did not match two actual inbound game UDP
attempts; the initially allowed address had zero game packets. The owned rule
was replaced with the sole observed source `/32`, removing the wrong rule first.
This is observed routing evidence, not a permanent source-stability guarantee.

An attempted transient runtime-limit extension returned nonzero and its
fail-closed handler stopped the first return instance. Actual closure at
14:26:56 UTC confirmed the save acknowledgement, rule removal and game-listener
absence. The old timer fired again at 14:28:05 UTC after the instance had stopped;
that later save returned exit 1, with rule/listener closure still confirmed.
No second successful save or new-kit persistence is inferred. This interrupted
the test setup; it
does not establish the cause of the separately reported Rust client crash.
The owner explicitly chose to continue the playtest. The same world was then
restarted under a fresh nonroot instance with a 4,500-second runtime backstop.

The new one-hour source-restricted window was opened only after startup and
current plugin/permissions checks. Independent readback at 14:35:56 UTC confirmed
public game UDP 28015, loopback-only RCON, blocked query/RCON, no unintended Rust
listener, the actual `-insecure` process and active persistent closure. Current
`app.port=-1` confirmed disabled Rust+. Its checked deadline is 15:34:57 UTC on
9 October. Independent readback at 15:35:34 UTC confirmed the completed
automatic closure: save acknowledged with exit 0, the owned game rule removed,
the instance inactive, all Rust listeners absent and the default-deny firewall
active. The old return timer is disabled and inactive. Addresses, account
identifiers, credentials and raw evidence remain private. This save does not
verify persistence of the player and kit after a restart/rejoin.

Authenticated status at 14:45:44 UTC confirmed the owner as the sole active
player, with the observed source matching the window rule. The owner confirmed
one Assault Rifle and 120 Rifle Ammo after `/loadout carbine`. This is
owner-reported inventory evidence, not an independent client capture. The
[remaining ordered checks](RUST-SERVER-PLUGIN.md#return-session--9-october-2026)
are still open. No plugin behavior, rental or paid service changed.

Root leaves Shadow input to the owner. Concurrent input was suggested as a lag
hypothesis, not verified as its cause. An empty-host snapshot showed 226 server
FPS and ample available memory; it did not measure client FPS or streaming
latency. Neither lag resolution nor a Rust client-crash cause is established.

## Require these capabilities before setup

- Hosting is permitted, with sufficient available RAM and disk.
- We control startup arguments, including the current
  [`-insecure` flag](https://rust.facepunch.com/news/maintenance) for the owner's
  EAC-disabled client; an obsolete `server.secure` setting is insufficient.
- We can stop/start the server, read logs and use an authenticated server console.
- We can install matching Oxide and our C# plugin, edit its config and permissions,
  reload/remove it and retain the installed Rust build/Oxide version evidence.
- The remote Shadow client has a reviewed route to the UDP game port for F1 direct join.
  Query access is only needed for separately required discovery, not this direct join.
  The loopback launcher provides no remote route; review host-specific bindings
  and access rules before changes. No public RCON is required.
- Existing installations/worlds are preserved; game files and credentials stay
  outside the repository. Access secrets are never put in issue/PR comments.

## Prove the host before calling it usable

Record actual startup completion, `oxide.version`, game/query/RCON/Rust+
listeners and a successful join from the owner's intended client. Verify save
and restart on that host. This proves setup only; plugin acceptance still
requires permissions/cooldown, inventory
failure/recovery, two-client PvP, unchanged excluded targets and reload/removal.
Local source checks and historical Linux PowerShell probes cannot replace these
real-host results. Ignored cloud evidence is not transferred by a Git checkout.
