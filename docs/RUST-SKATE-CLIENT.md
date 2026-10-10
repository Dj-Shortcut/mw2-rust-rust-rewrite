# Rust PC skate client architecture

Scope: existing Rust PC game; standalone rewrite parked. Claude/the owner reported these observations in [#337, run R](https://github.com/Dj-Shortcut/mw2-rust-rust-rewrite/issues/337#issuecomment-6094262459), [integrated runs R2/R3/T/P](https://github.com/Dj-Shortcut/mw2-rust-rust-rewrite/issues/337#issuecomment-6094749999) and, on the source of [PR #340](https://github.com/Dj-Shortcut/mw2-rust-rust-rewrite/pull/340), in further runs on 10 October 2026.
Codex did not rerun them. Client Steam build25824447. The owner's launcher plays client commit `c5098aef57967d45e8ed5d21be08eb184b03ca6b` (plugin 0.14.0) with the shared modules of that commit.
Source, controls and tools: [client](../mods/rust/client/README.md), [tools](../mods/rust/client/tools/README.md).

## Reported client route

BepInEx IL2CPP loads through the owner's Steam shortcut. InputSystem Keyboard works;
the owner reported controller keys through Steam Input. Direct executable launch found no Gamepad.
A press that begins and ends between two frames shows only as `wasPressedThisFrame`.
The kinematic BasePlayer follows a separate dynamic movement body; PlayerWalkMovement stays
enabled. The client uses its own **SkateRide**, allowing downhill reversal, rather than SkateDrive.
While mounted, it owns full velocity, including ollie 6 m/s and air gravity 16 m/s², writing
after the game's 32 Hz fixed step. Preserve this exclusive ownership when adapting shared APIs.
The measured movement capsule is 1.8 m high/0.5 m radius; those are not skeleton dimensions.
Rig metres: thigh .422, shin .399, ankle-to-toe .162, upper arm .290, forearm .253, hip span .200, hips .048 below pelvis bone, pelvis-to-neck .491, neck-to-head .139;
standing hips about .87 above soles, ankle joints about .095. Measure missing dimensions.
Rider BoardPosition is the ankle plane: deck top + ankle height − FootLift (.01 m); the
adapter lowers toes and owns the separate .055 m heel-edge shift. DeckOffset is passed as
FootLift−ankleHeight (−.085 m for that rig); it does not shift ankle anchors.

## What the runs showed

The local model has 27 full-body skinned meshes normally shadows-only and eight `leg-` meshes
normally visible in first person. For the view from behind, full-body drawing is switched on and the leg set off.
Late bone writes rendered stance/push/air-flip/grab/bail/switch/grind/manual **poses** across 19 keyed states,
stepped with key presses sent from a distance. With the rider module of items 8–10 TryCreate refused nothing
and every joint met its target (miss .000 m): the adapter tips the pelvis about the hip line so that the
spine root meets the folded chest, and lifts neck and head when the chest folds more than 20°.
In a manual the board tips 15° about its rear axle and the footing follows. Bones changed before the next frame, after the rendered picture.
Moving the **main camera** late gives a lit/skinned view; a second CopyFrom camera rendered white.
Scripted tricks: .72 s air, about 1 m peak, 5 m/s impact; Ollie, Kickflip, Frontside 180, Heelflip,
Backside 180 Grab, switch changes, a held manual (+159) and a banked combo (+418) gave score 1268, the offline total.
Two late presses were refused as plain airs; native bail gameplay remains unverified.
Keys: K mount, W push, Space ollie, L camera, Shift manual, held Ctrl/C dismount; K, L and Space were pressed as real keys.
Sound: the engine's output per source, read back at a muted listener and scaled to full volume, was
.044–.132 for the rolling loop (walking pace to 8 m/s), .082–.205 for the grind loop and .278–.550 for the one-shots.
At a listener of .001 the rolling loop read .000, so a muted check uses .004. Nobody has listened yet.

## The server's limit without ShortcutSkate

Two kinds of pull-back were seen on the owner's server, which runs without the skate plugin.
**After waking:** five to seven during the first 2–2.5 s of the first ride, about .7 m back every .37 s, at
walking pace too and also when the ride began 8 s after waking. None when the first movement came 40 s or
150 s after waking, none on a second ride. Unexplained; the scripted runs touch neither mouse nor keyboard,
and the owner has not mentioned it from his own play.
**Speed:** rolling downhill at 4.4 m/s was pulled back four times in consecutive ticks after a few seconds, and a
ride pushed to 8 m/s twenty times on its way out, the after-waking ones among them. Rides at 2.1–2.8 m/s were not.
That fits the pace the server allows a walking player, with slack after a rest; the game does not report a rider as sprinting.
The client lowers its cap on two pull-backs in a row (to 2.1 m/s in that run), shows `server limit`,
rode back at the cap without a pull-back, and tries a little more later. ShortcutSkate is what lifts this.

## Integration and remaining acceptance

[Shared APIs](../mods/rust/shared/README.md) supply Tricks/Rider/Edges/Audio/BoardMesh and an alternative
Drive model; the client takes them from the commit it is built from.
Native grinding has not run: the stand-in adapter captured a .4 m ledge at 5 m/s, ground 11.8 m
in 2.9 s and exited, but the game beach had no ledge. A grind pose is not grind gameplay.
Reported server: Oxide 2.0.7820, Rust 126/2634.289.1; only ShortcutLoadouts 0.1.0 was installed.
[ShortcutSkate](../mods/rust/plugins/SKATE.md) installation/practice/patch/load/reload/cleanup remain open;
console-field submit/end-edit calls did nothing, so client command submission is not established.
The launcher was installed from `c5098ae` and tested: Rust started through the Steam shortcut, and with the
launcher killed the helper removed all six loader roots 35 s after Rust closed. Complete
update/crash/lifecycle recovery remains open. Death/respawn/reconnect, a lost late driver, clothing changes and
sound failure are covered only offline, against a stand-in engine outside this repository.
Interop regeneration after Rust updates and [loader gates](RUST-CLIENT-LOADER.md) remain separate.
One client/account was available; second-client observation, the owner's control feel and
[full acceptance](RUST-MW2-SKATE.md) remain open. [TODO](../TODO.md) tracks progress;
[Codex item 12](RUST-SKATE-CLIENT-REVIEW.md) reviewed the client source.
