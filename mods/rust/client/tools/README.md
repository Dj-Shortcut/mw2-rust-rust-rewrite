# Tools for the skate client

PowerShell scripts for the Windows PC that runs the Rust client. They generate what the mod loader
needs for the installed Rust build, compile the plugin, run a bounded session with the loader and
take it out again, show the logs, and install the owner's launcher.

They are written to be run by an agent that reaches the PC only through typed keys. Every script
is therefore fetched from this repository and run in the PowerShell session (`zz name`), names are
short, and settings are global variables that a script sets for the next one. Nothing here is
installed permanently except the launcher described below.

## What must be on the PC

All of it lives under the user's `Downloads` folder and none of it is in this repository.

| Path under `Downloads` | What it is |
| --- | --- |
| `BepInEx-Unity.IL2CPP-win-x64-6.0.0-be.788+5b766a3.zip` | The official mod loader build ([identity](../../../../docs/RUST-CLIENT-LOADER.md)). |
| `codex-bep788-probe\interop-nested-build-20261007-c5ab5f\source-build\sdk` | The .NET SDK whose compiler builds the plugin and the generator's front end. |
| `codex-bep788-probe\interop-callee-prepared-20261009-007748-v1` | The corrected interop generator built from [`../generator`](../generator/README.md), with its own .NET host. |
| `codex-bep788-probe\pwsh-diag\pwsh.exe` | PowerShell 7, used by `survey` only. |
| `claude-loader-probe\` | The staging folder of these tools. Created as needed. |

The staging folder holds `bep788\` (the unpacked loader), `gen-<build>\` (one interop set per Rust
build), `rt\` (the changed runtime library), `plugin\` (the last build), `play\` (the released
plugin), `launcher\` (the owner's launcher), `run-<tag>-<time>\` (one folder per session with its
logs and the loader files taken back out of the Rust folder) and `server.txt` (the address of the
private test server as `ip:port`; it is read by `ping` and `session` and is never committed).

## Starting

In a PowerShell window on the PC, once per window:

```powershell
iex (irm https://raw.githubusercontent.com/Dj-Shortcut/mw2-rust-rust-rewrite/main/mods/rust/client/tools/remote.ps1)
```

After that `zz name` runs `name.ps1` from this folder and `zg file` returns a file from it. Both
read the branch named in `$pref`; set it before or after loading `remote.ps1` to work from another
branch. A keyboard that cannot type `/` can build the address with `-f [char]47`.

## Scripts

| `zz` | What it does |
| --- | --- |
| `status` | Steam, the installed Rust build, whether an interop set exists for it, leftovers in the Rust folder, what is built and installed. Starts Steam when it is not running. |
| `interop` | Generates the interop set for the installed Rust build in the background. Needed once after every Rust update. |
| `interop-status` | Progress of that generation; done when the log ends with `GEN DONE`. |
| `runtime` | Builds the changed `Il2CppInterop.Runtime.dll` from the pinned upstream source and [`rt/`](rt/README.md). Needed once. |
| `build` | Compiles the plugin from [`../sources.txt`](../sources.txt) at the head of the branch, or at `$pcommit`. |
| `session` | One bounded run of the client with the loader; see below. Normally started by one of the next five. |
| `play` | Build, then a session to play in, started directly, sound on. |
| `pad` | The same through the owner's Steam shortcut, so that the controller layout applies. |
| `ride`, `pose`, `tricks` | Build, then a muted session that runs one scripted check in the world and ends by itself. |
| `log`, `log-all` | The plugin's log of the last session, without or with its once-a-second status lines. `$pinc` filters. |
| `file` | Another file of the last session, by default the game's `Player.log`. |
| `more` | The next page of whatever was shown last. |
| `clean` | Moves a loader that was left in the Rust folder out of it. |
| `survey` | Lists what the interop set of the installed build offers for given type names. |
| `launcher` | Installs or updates the owner's launcher from the commit in [`release.txt`](release.txt). |
| `launcher-check` | Starts the installed launcher, kills it while Rust runs, closes Rust and reports whether the Rust folder was cleaned all the same. |
| `ping` | Whether the private test server answers on its game port. |

## A session

`session` copies the loader, the interop set of the installed build, the changed runtime and the
plugin into the Rust folder, starts the client, waits, closes the client and moves every file it
added out again into the session's own folder. The last part runs in a `finally` block: the loader
must not stay in the Rust folder, because any later start of Rust would load it too, also on a
server with anti-cheat. It refuses to start when the loader is already there, when Rust is running
or when there is no interop set for the installed build; a set generated for another build crashes
the client.

The client is started directly (`RustClient.exe`, which does not start the anti-cheat) or through
the owner's non-Steam shortcut `RustClient` for the same executable. With `$pconnect` a directly
started client joins the server in `server.txt`; nothing has to be typed in the game.

The plugin reads its steps from `plugins\steps.txt`, which `session` writes from `$psteps`; they are
listed in the [client's README](../README.md#steps). A scripted check ends the session by writing
`plugins\skate.done`; otherwise the session ends when Rust is closed or after `$pdone` seconds.

## After a Rust update

1. `zz status` shows the new build and `interop for this build=False`.
2. `zz interop`, then `zz interop-status` until `GEN DONE` (about ten minutes).
3. `zz ride` in a world. If the plugin no longer compiles or the check fails, the game changed
   something the client relies on; `survey` shows what the new build offers.
4. `zz launcher`, so that the owner's plugin is rebuilt against the new set.

`interop.ps1` and `Gen.cs` name the Unity version of the client (6000.3.15). A Rust update that
moves to another Unity version needs both changed, and the base libraries for that version.

## Releasing to the owner

1. Verify the head of the branch in the game.
2. Put that commit in `release.txt` and push.
3. `zz launcher` builds exactly that commit, keeps the plugin in `play\`, saves `skate.ps1`,
   `session.ps1`, `watch.ps1` and `clean.ps1` of that commit in `launcher\` and writes two files on
   the desktop. The launcher uses only these local copies, so later changes in the repository do
   not reach the owner until the next `zz launcher`.
4. `zz launcher-check`.

## Settings

Set by the scripts for one another; `sv name value` sets one by hand.

| Variable | Used by | Meaning |
| --- | --- | --- |
| `$pref` | `zz`, `zg`, `build` | Branch to work from (default `main`). |
| `$pcommit` | `build` | Build this commit instead of the head of the branch. |
| `$pnext` | `build`, `runtime` | Script to run after a successful build; used once. |
| `$ptag` | `session` | Label in the session folder's name. |
| `$psteps` | `session` | Steps for the plugin. |
| `$pplug`, `$prt` | `session` | Folder with the plugin, folder with the changed runtime. |
| `$pdone` | `session` | Seconds after which a session in a world is ended. |
| `$psteam`, `$pshortcut` | `session` | Start through the Steam shortcut of that name. |
| `$pconnect` | `session` | Join the server in `server.txt` at start (direct start only). |
| `$plast`, `$pfile` | `log`, `file`, `more` | The last session's folder, the file being paged. |
| `$pinc`, `$pname` | `log`, `file` | Filter expression, file name. |
| `$pt`, `$pp`, `$pfind` | `survey` | Type names, member filter, type-name search. |
