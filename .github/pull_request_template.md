## What changed

<!-- The effect, not the investigation. -->

## What actually works now

<!--
Be literal. Separate what was run and observed from what only compiles.
"cargo check passed" is not "playable"; an unverified control binding is
unverified. Update TODO.md in the same PR when progress changes.
-->

- Verified (how):
- Compiles but unverified:
- Still missing:

## How to build and try it

<!-- Commands, map or scene, controls. Note anything that needs game data. -->

```bash

```

## Checklist

- [ ] `make publish-check` passes
- [ ] `python3 scripts/check_assets.py` passes
- [ ] Touched `.rs` files are formatted (`rustfmt --edition 2024 <files>`)
- [ ] `cargo clippy --workspace --all-targets` is clean for the touched crates
- [ ] No original game files, extracted assets, `.env` or keys are included
- [ ] New models are generated into `assets/authored/` and listed in its `manifest.json`
- [ ] New sounds are `.wav` under `assets/authored/audio/` and listed in its `manifest.json`
- [ ] Any new test is an owner-approved scenario in `crates/approved_tests`
- [ ] Architectural change (new crate or subsystem, sim/render/net data flow) was discussed in an issue first
- [ ] `TODO.md` reflects the honest state after this change
