# Standalone park editor

This guide describes the authored standalone PC session (`launcher game`).
Existing placement/removal and undo/redo are implemented. Existing-prop controls
below are proposed in [#178](https://github.com/Dj-Shortcut/mw2-rust-rust-rewrite/issues/178)
and are not yet implemented or graphically verified.

Enter E mode while alive and on foot. Aim at a prop within the existing editor
reach; nearer solid geometry can block the target. A move places that prop on
the first background surface hit along the same view ray, ignoring the prop
itself; that surface must face upward. There is no pickup/drag mode or snapping.
Rotation changes yaw in place. IDs and kind stay the same; a move keeps yaw.

| Action | Keyboard/mouse | Xbox controller on PC |
|---|---|---|
| Enter/leave editor | E | Back cycles FPS/build/editor |
| Select new prop | 1–6 | D-pad |
| Rotate placement preview | Q / R | LB / RB without LT |
| Place / remove aimed prop | LMB / RMB | RT / X |
| Move aimed prop | M | Y |
| Rotate aimed prop −15° / +15° | comma / period | Hold LT, press LB / RB |
| Undo / redo | Ctrl-Z / Ctrl-Y | Keyboard |
| Save / load local session | F5 / F9 | Keyboard |

Focused, living, unpaused, on-foot editor input is required. Inventory, error,
building and skate modes block editing; execution rechecks these conditions.
Consumed LB/RB+Y skate chords never also move or rotate objects. Modified
shoulders do not rotate the preview. Equivalent requests share one action;
different move/rotation requests in one frame cancel. Held keys do not repeat.

No target, no upward move destination, invalid pose or living-player overlap
fails with English feedback and preserves objects and undo/redo history.
The backend also rejects editing an active grind object; native skate mode
blocks editor input. General object overlap, snapping and
unsupported-geometry prevention are outside these operations' rules.
Successful edits are undoable and clear redo. Saves preserve poses and IDs;
loading clears edit history and exits editor mode. Save on a separate frame
after editing: the frame's queued save precedes object actions.

Acceptance: actual native-input/Session checks cover deduplication, conflicting
requests, mode/focus/death gates, consumed chords and atomic rejection. A real
native window must demonstrate keyboard move/rotate, collision refresh,
undo/redo, save/load and readable English help/feedback. Software gamepad
events do not establish physical controller verification or release readiness.
