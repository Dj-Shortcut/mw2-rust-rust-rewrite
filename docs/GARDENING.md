# Standalone berry gardening

Proposed PC flow for [#239](https://github.com/Dj-Shortcut/mw2-rust-rust-rewrite/issues/239); new controls/hints/visuals are unimplemented/unverified.

Gather berries with F / Xbox Y for Food and one seed when seed storage has room;
gather Water the same way. Eat Food with J or inventory I / Xbox A.
Three fixed beds have IDs 1–3 at XY [180,160], [260,160], [340,160]; terrain supplies Z.
The living player's closest bed within 100 world units in 3D is the target,
regardless of its state. Aiming and line of sight are not required.

| Proposed on-foot action | Keyboard | Xbox controller on PC |
|---|---|---|
| Plant or harvest the nearest bed | T | Hold LB, then press RB |

A fresh RB press while LB is held requests once; T/controller requests deduplicate.
RB alone does nothing; holding the chord does not repeat; a consumed Y+shoulder skate chord suppresses
the controller request. Existing inventory/build/editor/skate T/shoulder controls remain.
Require focus, alive, unpaused, inventory closed, no error and no build/editor/skate,
after capture/mode transitions; recheck those gates at dispatch.
At execution, reselect the actual closest bed: Empty/no target calls plant_seeds;
Growing/Ripe calls harvest_plot. Do not queue a stale state-specific decision.
F/Y keeps loot > crate > supply-drop > gather priority and runs before T.
Session advances first; F+T may gather then plant. Save on a later frame;
a successful F9 load ends action dispatch. No new inventory confirmation is added.

Planting costs one berry seed and one Water; success starts 600 s of growth.
Plant API checks liveness/reach/state/seeds/Water in order: "Player is not alive",
"No garden plot within reach", "Something is already growing here" or "Harvest
the ripe berries first", "You need berry seeds to plant", "You need water to plant".
Success: "Planted berry seeds: ripe in 600 s". Failed planting consumes neither item.
Harvesting gives five Food and two berry seeds together, then empties the bed.
Harvest API checks liveness/reach/state/space in order; state errors are "The berries
are not ripe yet" or "Nothing is planted here"; space error is "Not enough inventory
space for the harvest". If only Food fits, nothing is delivered and the bed stays Ripe.
Success: "Harvested 5 food and 2 berry seeds". Seeds stack to 20; Water to 10.

Growth takes 600 eligible simulation seconds, 1.5x in rain OR dawn (05:00–07:00),
without stacking; world temperature below 0 C pauses it. Carrying Water permits
planting during frost. Growth continues away/dead; inventory/pause/focus stop the world.
Keep "Your berries are ripe" feedback; normal ticks/messages may change first.
Proposed English hints show bed ID, Empty cost, Growing time/weather or Ripe output.
Original primitive beds/plants distinguish the three states, without new collisions.
Cache IDs/positions/coarse states; reuse handles and refresh at startup/load/ripening.
Existing v3 saves keep beds, items and world state; missing garden loads Empty beds.
No backend, schema, dependency or asset-file change; fertilizer/bed placement stays out.

Planned checks: actual gather→plant→advance→harvest→eat→replant and F5/F9;
atomic errors, nearest/reach/death, old saves, rain/dawn/frost and complete output space;
actual native chord/gate/old-control/ECS checks and keyboard/Mesa F1/hints/queue at 720p.
Physical controllers, audible audio, Windows/macOS gameplay and release remain open.
