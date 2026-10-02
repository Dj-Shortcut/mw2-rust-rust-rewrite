# Project brief and current handoff

Build a standalone PC game in Rust and Bevy combining Rust-inspired survival,
building and an island world, MW2-inspired FPS gunplay and original operators,
and Skate 3-inspired skating and a park editor. The user explicitly clarified
that this is inspiration, with original assets made from scratch. They want
the breadth of Rust's systems. This is a substantial unfinished game project.

Do the implementation and verification yourself. Do not ask the user to test
incremental builds. Maintain TODO.md with honest progress. The user has
explicitly authorized committing and pushing the current development source
to GitHub. This is a source handoff, not a finished or playable release.
Do not revive the canceled Xbox 360–PC crossplay project.

## Repository and access

Cloud working directory: `/workspace/mw2-rust-rust-rewrite`.
Remote: https://github.com/Dj-Shortcut/mw2-rust-rust-rewrite .
The development tree includes the source imported from the public mashup
engine and the subsequent standalone-game implementation. Use the current
repository revision for the source handoff; TODO.md describes its unfinished
state and verification limits.

The user's intended Mac directory is
`/Users/Mac/Documents/Codex/2026-10-02-kan-je-een-nieuw-project-beginnen`.
This cloud cannot access it or control the Claude app on that Mac. For local
development, clone or update the GitHub repository at the intended directory.
Ignored `context/artifacts/` evidence is not included in the source handoff;
transfer it separately if developer context is needed. This document has not
been sent to Claude through an app connection.

Read AGENT.md, CONTEXT.md, docs/INDEX.md and TODO.md before editing.

The imported engine is public upstream source, not newly authored project
code. The Apache licence and NOTICE are retained. Separate licence scope for
the in-tree SK8/MinecraftOSS modules and provenance of legacy numerical data
remain part of the pending review in TODO.md. Do not claim independently
verified clean-room provenance or that every imported datum is from scratch.

## Reference requested by the user

https://github.com/rehan-remade/universal-modder

Its MIT repository provides AI-agent skills, a Python `um` CLI, modding and
reverse-engineering playbooks, asset workflows and game automation. It supports
Claude Code and Codex. It is not a ready-made game engine or this game's source.
The fal asset-generation route requires an API key and may incur charges.
Review useful workflows; do not assume universal compatibility, install tools
blindly, upload project data or publish external contributions automatically.

## Implemented code and evidence

- `crates/rust_building`: bounded building placement, materials, costs, owner,
  doors, stability/collapse, JSON storage and collision.
- Shared simulation/network integration and authoritative bullet/melee damage
  to exact building IDs. Five damage probes passed; prediction does not damage
  buildings. Network protocol version is now 95.
- `crates/rust_maps`: bounded Facepunch SDK-v9/LZ4/protobuf inspection tool.
  It is an optional reader, not a generator or runtime replacement terrain.
- Original procedural building materials in WGSL; Naga validation passed.
  Downloaded textures were removed from the product asset path.
- `scripts/generate_authored_content.py`: Blender generator for original masked
  operator, carbine, skateboard and timber pieces. Generated GLB and editable
  blend files under assets/authored. Operator includes a rig and three clips.
  Props have no animations. Asset preview renders are not game screenshots.
- `crates/survival`: authored session, weapon data, island mesh/triangle-prism
  collision, six editor object geometries, edit history and scene save/load.
- `crates/bootstrap/src/native.rs`: Bevy native frontend, first-person controls,
  authored GLB scenes, terrain and editor rendering, HUD, pause and save input.
  `cargo check -p bootstrap` passed after Bevy 0.19 API corrections.
- Launcher default/`game` route now calls `bootstrap::run_native()` before the
  optional legacy game-import path. Latest launcher/mapreader compiler check passed.

## Immediate blocking issue

The ignored native-world probe currently fails in Session::new with:
`<runtime>:0:0 in step: no loaded and started GSC program`.
The shared simulation requires an installed and started authority script.
Implement our own meaningful session rules/bootstrap or a well-designed native
simulation mode. Do not require original MW2 scripts or silently bypass all
authority systems. APIs include `SimWorld::install_gsc_program`, `start_gsc`,
`Program::load` and `SourceResolver`. Bootstrap resets script runtime, so order
matters. Verify movement, ammo/fire, player damage and building/editor behavior
after fixing it. The new terrain/editor probe has NOT passed yet.

The initial NPC spawn overlapped interpolated terrain; the central plateau was
expanded to address that. Probe now passes both spawn checks but fails on the
script preflight above. No successful graphical game run has been verified.

## Development controls in the new frontend

WASD move, mouse look, Shift sprint, Space jump, Ctrl crouch; LMB shoot,
RMB aim, R reload; Escape pause. B enables building; 1–4 choose pieces,
R rotates, LMB places and RMB toggles a door. E enables park editing;
1–6 choose ramp, quarterpipe, rail, stairs, platform or funbox; Q/R rotate
15 degrees, LMB places, RMB deletes and Ctrl-Z/Y undo/redo.
F5 saves and F9 loads the authored scene. These bindings compile but the native
end-to-end flow is unverified. Native controller integration remains pending.

## Remaining work

TODO.md is the full roadmap. Major missing systems include actual skating,
grinding/tricks/bails, editor ghosts and moving existing props, gathering,
inventory/crafting, needs/food/water, animals/NPC AI, monuments, electrical and
fluid systems, farming, vehicles, full weapons/audio/animation, authoritative
multiplayer and complete world persistence, menus/settings/respawn and releases.
The small development island is not a full Rust-scale world. Starting building
resources are development grants, not a gathering implementation.

No executable release exists. Do not label this finished, fully rewritten,
Rust-equivalent or playable based on compiler checks.

## Cloud toolchain and evidence

Rust executable: `/workspace/toolchains/cargo/bin/cargo`.
Environment: CARGO_HOME=/workspace/toolchains/cargo,
RUSTUP_HOME=/workspace/toolchains/rustup,
CARGO_TARGET_DIR=/workspace/rust-mw2-skate/target,
PKG_CONFIG_PATH=/workspace/native/pkgconfig.
Bevy 0.19 PBR/glTF/scene/animation dependencies have been fetched.

Checks: `cargo check --offline -p launcher -p rust_maps`.
Native probe manifest:
`context/artifacts/2026-10-02-native-world/1-TERRAIN-EDITOR-PART/probe/Cargo.toml`.
Keep temporary probe code/evidence under ignored context; only approved named
test scenarios belong in crates/approved_tests. Format only touched files.

Graphical validation is currently unavailable: no Xvfb and no Vulkan ICD.
A workspace-only apt download attempt could not locate those packages.
Do not misreport this as an automatic approval rejection or ask the user to
perform incremental tests. Arrange suitable development graphics and verify.
