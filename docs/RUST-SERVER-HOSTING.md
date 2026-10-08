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
Game remains loopback; query binds all IPv4 interfaces but is blocked externally
by the host firewall. Rust+ reports disabled. The quit process ended with
SIGKILL; no normal-exit guarantee is claimed. See [exact native scope](../mods/rust/server/LINUX.md#native-startup-and-control-evidence).
Do not publish addresses, keys, console credentials or operational logs.
Shadow join, player-state persistence, two-client PvP and full mod acceptance
remain open.

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
