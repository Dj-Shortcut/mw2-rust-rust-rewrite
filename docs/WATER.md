# Standalone water

Proposed native PC water flow for [#212](https://github.com/Dj-Shortcut/mw2-rust-rust-rewrite/issues/212).
The new controls and barrel visual are not implemented or graphically verified.

On foot, aim at a reachable water node to sip without carrying an item.
A sip restores 10 thirst, up to 100, without consuming items or node stock.
Existing gathering collects up to 10 Water items from finite stock per harvest.
Using one carried Water restores 35 thirst, up to 100; it leaves no empty bottle.
There are no bottle-refill, water-quality or purification mechanics.

| Action in FPS/survival | Keyboard | Xbox controller on PC |
|---|---|---|
| Sip from aimed water | P | RS click |
| Collect all nearby barrel water | O | D-pad Up |
| Gather carried Water | F | Y |
| Use carried Water | K or inventory I | Inventory A |

New actions require focused, living, unpaused on-foot input, inventory closed,
no game error, and neither build nor editor mode; execution rechecks all gates.
Fresh presses act once; equivalent requests deduplicate and different water
requests in one input frame cancel. Existing F/Y loot and gathering priority remains.
Inventory O/P clothing and RS/Up stack controls retain their existing meanings.

Sipping requires visible, nondepleted water bounds within 120 view-ray units.
Session rejects dead players, a missing/blocked target, frozen water, then full
thirst. Target errors include "Cannot gather through a solid object" and "Aim
at water within reach", followed by "The water is frozen" or "You are not thirsty".
Successful feedback is "You drink some water".

One fixed rain barrel near spawn holds up to 5 Water. Within 100 world units,
collection takes all its contents or none. Session rejects dead, out-of-reach,
empty or insufficient-space requests in that order without losing stored water.
Collection uses proximity, without aiming or a line-of-sight requirement.
Stored water remains collectible while it is dry or freezing.

Rain fills one Water per 60 simulation seconds when world temperature is at least 0 C.
Dry/frozen weather pauses progress; a full barrel resets its fill timer.
It fills while the player is dead. Inventory, pause and focus loss pause the world.
English hints distinguish collecting, paused and full; a proposed original
primitive barrel visual reflects stored count and adds no new collision rule.

F5/F9 already preserve thirst, carried Water, node stock and barrel state.
Close inventory before saving/loading. No schema, packaged asset files or dependency change.
Acceptance: actual Session/native-input checks and Linux/Mesa keyboard flows
for sip, gather/use, barrel collection, atomic errors, gates, old controls,
English hints/visuals and save/load. Session advances before queued actions;
account for ordinary thirst/timer progress. Exactly-full thirst refusal is an
authority check, not a promised GUI state.
Physical controllers, audible audio, Windows/macOS gameplay and release remain open.
