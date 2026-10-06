# Self-managed Rust PC test server

Preparation for [issue #266](https://github.com/Dj-Shortcut/mw2-rust-rust-rewrite/issues/266).
A suitable host has not been selected; no server or playable mod is verified.

## Use an existing machine or choose a permitted rental

We can install the official dedicated server ourselves through SteamCMD app
258550 and add matching Oxide. Stock Rust supplies the world and game rules.
Use Windows or Linux for the published [Oxide packages](https://docs.oxidemod.com/guides/owners/install-oxide).
Use [Windows preparation/startup](../mods/rust/server/README.md) or [Linux installation preparation](../mods/rust/server/LINUX.md).
Each needs its corresponding native host; Linux startup/control and all real-host acceptance remain pending.
[Facepunch](https://wiki.facepunch.com/rust/Creating-a-server) lists 12 GB free RAM
and 15 GB free disk; reserve 20 GiB disk for our preparation, plus OS resources.
An existing suitable PC avoids a server rental; power and connectivity still matter.
Mac source access alone does not establish a usable Windows/Linux server host.

For a rental, record the exact plan, total price/currency, billing period,
setup charges and renewal/cancellation terms. The owner chooses the host and
approves concrete costs before any order. Shadow Gaming is excluded because
[its rules prohibit hosting](https://support.shadow.tech/hc/en-us/articles/32731830348305-Rules-and-Restrictions-on-Shadow).
Host Havoc declined the required startup flag in the owner's support exchange.

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
