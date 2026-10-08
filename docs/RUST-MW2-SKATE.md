# Complete Rust / MW2 gunplay / skate mod

Owner clarification on 6 October 2026: [issue #275](https://github.com/Dj-Shortcut/mw2-rust-rust-rewrite/issues/275).
Target is the existing Rust PC game with MW2-inspired gunplay and skateboarding.
The standalone Rust/Bevy rewrite stays parked. This is a design, not implemented gameplay.
[Issue #266](https://github.com/Dj-Shortcut/mw2-rust-rust-rewrite/issues/266) supplies the [loadout/damage component](RUST-SERVER-MOD.md), not full product acceptance.
Rust supplies the world, survival, building and TC rules; player text stays English.

## Complete player flow

1. Join the intended server/client mode and use the documented controls.
2. Choose a personal weapon class; preserve unrelated inventory and permissions.
3. Aim, fire, reload and switch weapons with a defined MW2-inspired gunplay profile.
   Record weapon damage, firing cadence, recoil/spread, ADS and movement behavior;
   a damage multiplier alone does not verify weapon feel or client behavior.
4. Obtain a visible board with an appropriate rider pose, then mount, push,
   steer, brake, jump/trick, use a rail, land or bail safely, and dismount.
5. Switch back to walking and firing; a second client observes the same board,
   rider, movement and combat outcomes. No bike/sled substitute for this flow.
6. Keep Rust survival/building/TC behavior working throughout both modes.
7. Verify death, disconnect/rejoin, invalid input, plugin reload, server restart
   and removal: no stuck rider, orphan board or corruption of existing inventory/world.

## Server/client feasibility gate

[Oxide](https://docs.oxidemod.com/guides/developers/server-lifecycle) modifies server behavior and does not support player client modification.
[Weapon-fired](https://docs.oxidemod.com/hooks/weapon/OnWeaponFired) and [player-input](https://docs.oxidemod.com/hooks/player/OnPlayerInput) hooks provide server entry points;
they do not establish client recoil/ADS/animation replacement or predicted board physics.
[Facepunch entities](https://wiki.facepunch.com/rust/Entities) reference native game prefabs. Arbitrary authored client assets are not a proven server-plugin capability.
First prove a real visible board, rider pose, input and network movement with the
intended client. Headless physics, a renamed native vehicle or a server field
change is insufficient. Exact tuning and viable client presentation remain unresolved.
If a client extension is required, research an explicit compatible architecture
before implementing it; do not silently resume Bevy or reduce the product goal.
Use native game content or original authored content through a verified route;
keep installed game files, binaries, credentials and temporary probes out of GitHub.

## Definition of ready

Root verifies the complete flow, including two clients and failure/lifecycle cases,
on recorded real Rust/framework/client versions before asking the owner to play.
Record intended controller/Steam Input mappings and observed input; injected input
is not a physical-controller test. Source compilation and helper checks remain narrower.
A small component, a green CI run or an unverified prototype is not the finished mod.
Loadouts/Bullet scaling, [skate motion source](RUST-SKATE-CORE.md) and [gunplay source](RUST-GUNPLAY-CORE.md) exist; native MW2 gunplay/skating remain missing.
The owner-selected Linux host now runs Rust/Oxide with private control and bounded
loadout-plugin checks. Shadow join, real player flows and full MW2/skate acceptance
remain open. Keep [TODO](../TODO.md) and component claims explicit.
Select/rent nothing without the owner's choice and concrete cost approval; source
and architecture work precede rental. See the [hosting gate](RUST-SERVER-HOSTING.md).
