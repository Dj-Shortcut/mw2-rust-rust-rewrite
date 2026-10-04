# Standalone crafting queue

Proposed native inventory flow for [#196](https://github.com/Dj-Shortcut/mw2-rust-rust-rewrite/issues/196).
The Session queue already exists; these controls are not implemented yet.
This design draft precedes implementation and verification.

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

Acceptance: actual Session/native-input/panel checks, then Linux/Mesa keyboard
flows at 1280×720 for enqueue, sequential completion, cancellation/refund,
atomic errors, ready output waiting/freeing space, pending/conflicting inputs,
pause/focus/death gates, old inventory actions and F5/F9 persistence.
Check readable English normal/pending panels and nonempty world HUD. Physical
controller, Windows/macOS gameplay and release readiness are separate scopes.
