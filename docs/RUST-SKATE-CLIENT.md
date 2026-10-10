# Rust PC skate client architecture

Scope: existing Rust PC game; standalone rewrite parked. Claude/the owner reported these observations in [#337, run R](https://github.com/Dj-Shortcut/mw2-rust-rust-rewrite/issues/337#issuecomment-6094262459) and [integrated runs R2/R3/T/P](https://github.com/Dj-Shortcut/mw2-rust-rust-rewrite/issues/337#issuecomment-6094749999), 10 October 2026.
Codex did not rerun them. Client Steam build25824447, released client commit `d0c3a4c2b48d1f5b7f4d30bdf767eaaa879d29b0`, shared pin `5a9ed6e550a405365c8a18746dfd276a310b6848`.

## Reported client route

BepInEx IL2CPP loads through the owner's Steam shortcut. InputSystem Keyboard works;
the owner reported controller keys through Steam Input. Direct executable launch found no Gamepad.
The kinematic BasePlayer follows a separate dynamic movement body; PlayerWalkMovement stays
enabled. The client uses its own **SkateRide**, allowing downhill reversal, rather than SkateDrive.
While mounted, it owns full velocity, including ollie 6 m/s and air gravity 16 m/s², writing
after the game's 32 Hz fixed step. Preserve this exclusive ownership when adapting shared APIs.
The measured movement capsule is 1.8 m high/0.5 m radius; those are not skeleton dimensions.
Rig metres: thigh .422, shin .399, ankle-to-toe .162, upper arm .290, forearm .253, hip span .200, hips .048 below pelvis bone, pelvis-to-neck .491, neck-to-head .139;
standing hips about .87 above soles, ankle joints about .095. Measure missing dimensions.
Rider BoardPosition is the ankle plane: deck top + ankle height − FootLift (.01 m); the
adapter lowers toes and owns the separate .055 m heel-edge shift. The new DeckOffset should
be FootLift−ankleHeight (−.085 m for that reported rig); it does not shift ankle anchors.

## What the integrated runs showed

The local model has 27 full-body skinned meshes normally shadows-only and eight `leg-` meshes
normally visible in first person. In the pose runs, full-body drawing was switched on and the leg set off.
Late bone writes rendered stance/push/air-flip/grab/bail/switch/grind **poses** across 19 keyed states;
TryCreate accepted the measured rig. Target errors: stance .000 m, grab hand at most .024 m,
bail neck .015 m. Bones changed before the next frame, after the reported rendered picture.
Moving the **main camera** late gives a lit/skinned view; a second CopyFrom camera rendered white.
Scripted tricks: .72 s air, .99 m peak, 5 m/s impact; named Ollie, Kickflip, Frontside 180,
Heelflip, Backside 180 Grab and Kickflip, switch changes and banked score 900 appeared in the HUD.
Two late presses were refused as plain airs; native bail gameplay remains unverified. Real keys: K mount, W push to 8 m/s/air tap for kickflip, Space ollie, V camera, held C dismount. Five pullbacks occurred in the
first second of first push, none in a later 1.5 s at 7.4 m/s; their cause remains unexplained.
Six audio clips and loop/one-shot playback calls ran without exception. Sessions were muted; nobody listened, so audibility and sound quality remain unverified.

## Integration and remaining acceptance

[Shared APIs](../mods/rust/shared/README.md) supply Tricks/Rider/Edges/Audio/BoardMesh and an alternative
Drive model. Codex items 8–10 add configurable impact, pose polish and held manuals **after** that
native shared pin; actual-DLL net48/private Mac checks do not prove these new parts in the game.
Native grinding has not run: the stand-in adapter captured a .4 m ledge at 5 m/s, ground 11.8 m
in 2.9 s and exited, but the game beach had no ledge. A grind pose is not grind gameplay.
Reported server: Oxide 2.0.7820, Rust 126/2634.289.1; only ShortcutLoadouts 0.1.0 was installed.
[ShortcutSkate](../mods/rust/plugins/SKATE.md) installation/practice/patch/load/reload/cleanup remain open;
console-field submit/end-edit calls did nothing, so client command submission is not established.
The desktop launcher ran twice through the Steam shortcut and removed the loader afterwards;
a forced launcher death left a helper that removed six roots 35 s after Rust closed. Complete
update/crash/lifecycle recovery remains open, as do death/respawn/reconnect and owner control feel.
Interop regeneration after Rust updates and [loader gates](RUST-CLIENT-LOADER.md) remain separate.
One client/account was available; second-client observation and [full acceptance](RUST-MW2-SKATE.md)
remain open. [TODO](../TODO.md) tracks progress; Codex item 12 reviews the published client source.
