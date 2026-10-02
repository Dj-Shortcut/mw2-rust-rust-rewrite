# From-scratch content

These models are authored by this project's Blender generator. No original
game files, imported meshes, texture packs, sound packs or model libraries
are inputs. Source code is Apache-2.0; the generated models are
[CC0-1.0](CC0-1.0.txt).

Generate with Blender 4.3 or later:

```bash
blender -b --python scripts/generate_authored_content.py -- --output assets/authored
```

The output includes editable `.blend` sources, self-contained `.glb` models
and a manifest of bounds, vertex/triangle counts and SHA-256 hashes.
Coordinates in Blender are Z-up, front toward -Y; glTF export converts to
its standard Y-up coordinates. Units are meters.

- `masked_operator`: articulated tactical character, 17-bone skeleton and
  authored idle, walk and aim animation clips.
- `carbine`: authored receiver, barrel, furniture, magazine, rail and sight.
- `skateboard`: curved deck, trucks, bearings and four wheels.
- `timber_construction`: foundation, planked floor and wall, using 120 game
  units at 0.0254 meters per unit.

These are early assets for the developing standalone game. The native frontend
loads the operator, carbine and skateboard scenes; animation playback and the
complete gameplay integration remain unfinished.
