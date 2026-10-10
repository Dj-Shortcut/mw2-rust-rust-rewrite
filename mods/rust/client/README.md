# Rust PC skate client

A plugin for the existing Rust PC client that puts the local player on a skateboard: pushing,
braking and carving, rolling on slopes, ollies, kickflips and heelflips, half spins that end in
switch stance, grabs, grinding along edges, bails, a score with combos, a rider pose on the game's
own player model, a view from behind, sounds and text on screen. It belongs to
[#337](https://github.com/Dj-Shortcut/mw2-rust-rust-rewrite/issues/337) and the
[complete flow](../../../docs/RUST-MW2-SKATE.md); the calculations it shares with the server
plugin are in [`../shared`](../shared/README.md).

The plugin is loaded by BepInEx (IL2CPP). On the client that runs it, it changes the velocity of
the local player's movement body, the pose in which the local player's model is drawn and the
position of the main camera, and it adds a board, sounds and text of its own. It sends nothing to
the server: to the server the rider is a player who moves.

## Rules

- It is used only on the owner's own server, which runs without anti-cheat. Rust is started for it
  through `RustClient.exe`, the executable that does not start the anti-cheat.
- The mod loader is in the Rust folder only while one session runs. Every way of starting a
  session here puts it there and takes it out again; see [the tools](tools/README.md#a-session).
  Rust must not be started normally while the loader is in its folder.
- The server address, game files, built binaries and logs stay out of this repository.

## Controls

A controller reaches the client as these keys, through the Steam Input layout of the owner's
`RustClient` shortcut.

| Key | On the ground | In the air |
| --- | --- | --- |
| `K`, or `Space` twice quickly | Get on. `K` again gets off. | |
| `W` | Push | Tap: kickflip |
| `S` | Brake | Tap: heelflip |
| `A` / `D` | Carve | Frontside / backside 180 |
| Mouse | The board turns toward the view | |
| `Space` | Ollie; on a grind, ollie off | |
| `Ctrl` or `C` | Held for a moment: get off | Held: grab |
| `L` | View from behind or the game's first person | |

Coming down over an edge while moving along it starts a grind. A trick only starts when the jump
lasts long enough to finish it. Tricks score when landed; landed tricks in a row multiply, and the
combo is banked after a moment of plain rolling. Coming down faster than 12 m/s, or with a
rotation unfinished, is a bail: the combo is lost and the rider steps off. Water, death, a seat or
a vehicle also end the ride. Nothing is read from the keyboard while the cursor is free (console,
chat, inventory).

## Sources

[`sources.txt`](sources.txt) lists what is compiled into `ShortcutSkateClient.dll`. A bare path is
taken from the commit that is built. A line with a commit and a SHA-256 takes shared code from
that commit and must match the hash, so that a build is exactly what was checked even while the
shared modules move on.

| File | Part |
| --- | --- |
| `Plugin.cs` | Entry point, steps, log. |
| `SkateRig.cs` | Finds the local player and its movement body, getting on and off, the per-frame order. |
| `SkateKeys.cs` | Keys. |
| `SkateRide.cs` | Ground, air, grind and bail movement; calls the shared trick module. |
| `SkateGrind.cs` | Samples the ground around the rider and asks the shared edge module for an edge. |
| `SkateBoard.cs` | The board and where it sits. |
| `RiderRig.cs` | Writes the shared rider pose onto the player model's skeleton; what is drawn in which view. |
| `SkateCamera.cs` | View from behind. |
| `SkateHud.cs`, `SkateSfx.cs` | Text and sounds. |
| `Scenarios.cs` | Scripted checks in a world. |

### Steps

The plugin reads a comma-separated list from `BepInEx\plugins\steps.txt` when it loads; without the
file it runs `skate`.

| Step | Effect |
| --- | --- |
| `skate` | The mod as the owner plays it. |
| `ride` | Scripted: ride out, brake, turn round, ride back, get off. |
| `pose` | Scripted: rider and board in each state, standing still, from fixed cameras; `K` steps on. |
| `trick` | Scripted: ollie, kickflip, heelflip, both spins, a grab and two presses that come too late. |
| `mute` | Silences the whole client. |
| `nogrind` | No grinding. |

A scripted check writes what happened to `BepInEx\plugins\skate.log` and ends the session.

## What it relies on in the game

The client's own code is obfuscated and renamed with every build. The plugin uses only Unity's
engine API, the type names of game components (`BasePlayer`, `PlayerWalkMovement`, `PlayerModel`)
and Unity's message names. Each of the following was established in the running game and can
change with a Rust update:

- The local player is moved by a separate object with the walk component, a dynamic Rigidbody and
  a capsule. Velocity written after the game's own fixed step moves the player; the walk component
  has to stay enabled.
- The player model is a humanoid; its bones are taken from the Animator. A pose written after the
  game's animation in `LateUpdate` is what gets drawn.
- The local player's full body is set to cast shadows only, and a second set of meshes whose names
  start with `leg-` is what first person shows. The view from behind swaps the two.
- Only the main camera carries the game's image effects, so the view from behind moves it.
- A controller is not visible to the plugin as a gamepad; keys are read from the Input System.
- Engine methods the game never calls can be missing from the client and throw when called.

## Playing it: the owner's launcher

[`tools/launcher.ps1`](tools/launcher.ps1) builds the commit named in
[`tools/release.txt`](tools/release.txt) and puts two files on the desktop of the PC:

- **Rust Skate** puts the loader and the plugin into the Rust folder, starts Rust through the
  owner's Steam shortcut, and takes everything out again when Rust is closed. Its window has to
  stay open while playing. A hidden helper does the same clean-up half a minute after Rust closes
  if that window was closed first.
- **Rust Skate - remove mod files** takes the loader out of the Rust folder by hand, for example
  after a power cut.

The launcher downloads nothing and builds nothing; it plays the plugin that was built when it was
installed. After a Rust update it refuses to start and says so, until the mod has been
[prepared for the new build](tools/README.md#after-a-rust-update). To remove everything, delete
the two desktop files and the folder `Downloads\claude-loader-probe`.

On the server, [`ShortcutSkate`](../plugins/SKATE.md) accepts the rider's speed and supplies a
practice area. Without it the server puts a rider back who goes faster than a player on foot; the
client then holds a lower top speed.
