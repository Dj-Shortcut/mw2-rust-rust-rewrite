# Contributing

Welcome! Current work is a small server mod for the existing Rust PC game:
permission-controlled weapon loadouts and configurable firearm PvP damage.
Read [the mod plan](docs/RUST-SERVER-MOD.md) and [TODO.md](TODO.md) before
contributing. The standalone Rust/Bevy game is parked; its source and historical
verification remain available. Coordinate documentation, setup and focused mod
work through issues and PRs. No mod plugin or playable release is ready yet.

## Documentation first

Before writing code, read [README.md](README.md), [TODO.md](TODO.md), and the
relevant guides in [docs/INDEX.md](docs/INDEX.md). Check the project's
[open issues](https://github.com/Dj-Shortcut/mw2-rust-rust-rewrite/issues) and
[pull requests](https://github.com/Dj-Shortcut/mw2-rust-rust-rewrite/pulls) to
see whether someone is already working on the same area.

Start in an issue. For an existing task, comment with your proposed scope,
branch and files before editing. Otherwise, open an issue describing the
problem and intended behavior. Include how you will verify the result. An
issue is a coordination record; opening one does not establish agreement
with another contributor whose work overlaps yours.

For a new feature, public API or subsystem, start a draft PR with a short
design note or an update to the relevant guide **before implementation**.
Describe the player flow, controls, rules, failure cases, save compatibility
and acceptance criteria where applicable. Agree on the direction in the
issue, then implement in the same focused PR. Architectural changes need
discussion before a large branch is built. A bug fix starts with a written
reproduction and expected behavior; a documentation fix can be its own PR.

Documentation is a useful first contribution. Improving setup instructions,
explaining an existing API, documenting controls, or clarifying an unfinished
TODO item helps the next contributor start with accurate information.

## Choose a focused task

Work on one issue at a time. State which files you own and coordinate shared
files or public API changes in the issue. Use your own branch or worktree;
do not push to another contributor's branch. Keep unrelated fixes and broad
formatting changes out of your PR.

Useful contributions include documented mod/setup bug fixes, English command
feedback, bounded loadout/damage improvements and reproducible server reports.
Standalone gameplay/art work is parked. Describe the specific behavior you
intend to deliver rather than claiming an entire roadmap category.

## Build and run the parked standalone game

Install Rust through rustup and the system dependencies in
[docs/BUILD.md](docs/BUILD.md). Fork the repository on GitHub, then replace
`YOUR_USERNAME` below with your account name:

```bash
git clone https://github.com/YOUR_USERNAME/mw2-rust-rust-rewrite.git
cd mw2-rust-rust-rewrite
git remote add upstream https://github.com/Dj-Shortcut/mw2-rust-rust-rewrite.git
git fetch upstream
git switch -c feature/short-description upstream/main
cargo check -p launcher --locked
cargo run -p launcher --profile play --locked -- game
```

Run from the repository so the authored assets can be found. F1 shows the
controls. This standalone path needs no original game files or `IW4L_GAMES`
configuration. The installation/import sections in the inherited build,
run and Skate guides describe separate optional upstream modes.

## Project rules

- All player-facing text must remain **English**: HUD, menus, item names,
  controls, feedback and errors. Check changed text in the running game,
  including failure paths and whether it fits the UI.
- Use open-source code and original content created from scratch. Never commit,
  attach or link proprietary game files or content extracted or recorded from
  other games. Downloaded asset packs are outside this project's authored-content
  workflow. Keep credentials and private dumps out of GitHub.
- Preserve licences and attribution in [LICENSE](LICENSE) and [NOTICE](NOTICE),
  including the separate terms for imported modules. New code follows the
  repository's Apache-2.0 licence; the project's generated authored assets
  use CC0-1.0. Document the origin and licence of any proposed addition.
- Follow the [authored model guide](assets/authored/README.md) and
  [audio guide](assets/authored/audio/README.md). Include editable sources or
  reproducible generators and update the appropriate manifest and hashes.
  Asset checks validate files and manifests; they do not prove authorship.
- AI-assisted contributions follow the same rules. The contributor remains
  responsible for understanding, reviewing and verifying the submitted work.

## Verify and document the result

Exercise the behavior you changed, including relevant rejection cases and
save/load. Report the commit, platform, commands, inputs and observed results.
Distinguish compiler checks, headless behavior, graphical play, physical
controller checks and release readiness. A successful build proves compilation;
a screenshot or software controller event has its own narrower scope.

Before opening a ready-for-review PR, run the relevant repository gates:

```bash
python3 scripts/check_assets.py
make publish-check
cargo check --workspace --all-targets --locked
cargo test --workspace --locked
```

For documentation-only changes, check links, instructions and scope, and run
the asset/publish gates; a new gameplay build is not needed. Format only the
Rust files your PR owns. `rustfmt --edition 2024 path/to/changed.rs` can visit
child modules, so review its diff and keep unrelated files unchanged. CI
checks the committed changed files, rather than whole-workspace formatting.

Clippy follows the pinned toolchain and gate in
[the CI workflow](.github/workflows/ci.yml). It blocks findings in the packages
listed in [scripts/clippy_clean.txt](scripts/clippy_clean.txt); other imported
code is reported. A whole-workspace warning report is not a clean Clippy gate.

Permanent tests may only be added to `crates/approved_tests` after the project
owner approves the scenario by name; see its
[test policy](crates/approved_tests/README.md). Keep temporary probes, fixtures
and development logs out of the published tree. Contributors do not need the
maintainer's artifact naming or clone workflow in [CONTEXT.md](CONTEXT.md).

Update the relevant documentation and `TODO.md` in the same PR when behavior
or verified progress changes. Keep implemented, tested and unfinished work
separate; leave broad roadmap items open until their full flow is verified.

## Open the pull request

Link the issue with `Closes #123` when its scope is complete, or `Refs #123`
for partial work. Use the existing PR template: explain what changes, what
actually works, how to run it, the verification performed and remaining limits.
Keep it a draft while implementation or verification is incomplete. Respond
to review and let maintainers handle integration after the relevant checks.

## Report a bug

Open an issue with the commit or development-build identifier, OS/GPU,
standalone or optional import mode, reproduction steps, expected behavior and
actual result. Include relevant controls, save/load steps and sanitized logs
or screenshots. For an import-mode bug, name the game/map without uploading
its files. Do not attach secrets, private dumps or game archives.
