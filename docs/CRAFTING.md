# Standalone crafting queue

Native inventory flow for [#196](https://github.com/Dj-Shortcut/mw2-rust-rust-rewrite/issues/196).
87 actual native-input/Session checks passed. Linux/Mesa keyboard flows and
inventory layout were checked; this is development source, not a release.

Open inventory with Tab and select one of nine recipes with 1–9 or PgUp/PgDn.
Xbox LB/RB selects recipes. Known blueprints and sufficient materials are
required; research with R / Xbox Back retains its existing rules and costs.

| Action in inventory | Keyboard | Xbox controller on PC |
|---|---|---|
| Craft immediately | C | X |
| Add selected recipe to queue | V | Unbound |
| Cancel first queued job | F | Unbound |

Adding pays the full recipe cost immediately, without needing inventory space.
At most eight jobs fit. Only the first progresses; the next starts on a later
simulation tick after delivery. Times are simulation seconds, not wall time.
The inventory pauses the world: **close inventory to progress crafting**.
Pause and focus loss also stop progress. A ready job waits until its complete
output fits inventory. The summary reports the first job, job count and
remaining time, or "Ready; waiting for delivery" when its time is zero.
It also appears in the ordinary HUD while jobs exist.

F cancels the current first job and refunds its full prepaid material cost.
If any refund exceeds resource storage capacity, cancellation fails atomically.
Empty/full queue, locked blueprint, missing materials and dead-player requests
report English errors without changing queue, resources or inventory items.
V/F requires focused, unpaused inventory input without a blocking game error.
V/F acts on a fresh key press; holding a key does not repeat the action.

Recipe selection runs first. Any V/F request cancels a pending stack move,
discard or recycling action and consumes other inventory actions that frame,
including Enter/Y confirmation and C/X immediate crafting. This also happens
if the queue request fails. V+F reports "Choose one queue action (V or F)"
without changing queue/resources/items; a dead-player error takes precedence.
Existing controller inventory mappings and world-mode V/F controls remain.

Existing format-3 saves preserve queue order, recipes and remaining time.
Close inventory before F5/F9; normal simulation ticks can progress the queue
before a save. Death clears jobs and refunds prepaid costs within resource
storage limits before the existing carried-item loot drop. No save change.

Verified: actual Session/native-input/panel checks cover same-frame conflicts,
focus/pause/death gates and software-pad legacy actions. Linux/Mesa keyboard
flows at 1280×720 cover enqueue, sequential delivery, refund, atomic errors,
ready output/freeing space, pending actions, pause, C and F5/F9. Original
images check English normal/pending/dead panels and the nonempty world HUD.
Physical controller, audio, other-OS gameplay and release remain unverified.
