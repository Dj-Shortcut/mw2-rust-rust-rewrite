//! Player grab and aerial trick-control intents.
//!
//! This is the controller half of the TU3 `ActionGraphInputListener::Fill`
//! path.  The listener publishes both grab variants every tick; the authored
//! ActionGraph/MotionGraph decides which one is legal for the current stance.

use super::controller::DerivedControllerInput;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TrickIntent {
    pub name: &'static str,
    pub value: f32,
}

/// Recovered from `ActionGraphInputListener::Fill` (`825fb4c8`/`825999f0`):
/// ground grabs use exact trigger values, while air grabs use positive trigger
/// values.  Dark-catch requests come from the packed action flags, not from
/// gesture timing.
pub fn produce(controller: &DerivedControllerInput) -> Vec<TrickIntent> {
    let words = controller.words();
    let left_trigger = f32::from_bits(words[11]);
    let right_trigger = f32::from_bits(words[12]);
    let right_x = f32::from_bits(words[9]);
    let right_y = f32::from_bits(words[10]);

    // Fill825992D8 advances the grab timers only when the trigger action is
    // exactly 1.0; partial analog values are not a held grab in Skate 3.
    let left_ground = left_trigger == 1.0;
    let right_ground = right_trigger == 1.0;
    let left_air = left_trigger > 0.0;
    let right_air = right_trigger > 0.0;

    let mut output = Vec::with_capacity(12);
    let mut emit = |name, value| output.push(TrickIntent { name, value });
    if left_ground {
        emit("LeftGroundGrab", 1.0);
    }
    if left_air {
        emit("LeftAirGrab", 1.0);
    }
    if right_ground {
        emit("RightGroundGrab", 1.0);
    }
    if right_air {
        emit("RightAirGrab", 1.0);
    }

    let previous_flags = words[6];
    let current_flags = words[13];
    // TU3 Fill8259A9E4..AA54: B publishes Dismount while held and
    // NewDismount only on its rising edge. No grab/state/actor gate here;
    // the authored graphs decide whether this means dismount or no-foot air.
    // Names: 82F84B20 -> 830BE378, 82F84B08 -> 830C07AC.
    let dismount_button = 1 << 20;
    if current_flags & dismount_button != 0 {
        emit("Dismount", 1.0);
        if previous_flags & dismount_button == 0 {
            emit("NewDismount", 1.0);
        }
    }
    let dark_flags = (1 << 20) | (1 << 28);
    if current_flags & dark_flags != 0 {
        emit("DarkCatch", 1.0);
    }
    if current_flags & dark_flags != 0 && previous_flags & dark_flags == 0 {
        emit("NewDarkCatch", 1.0);
    }
    // Fill825999F0 publishes the board-adjust pair from the same normalized
    // stick vector used by the authored air board-adjust states.  The native
    // 82599B08..BB4: v125 = -RStickY is the denominator and v126 =
    // RStickX is the numerator. The quadrant correction also tests -RStickY.
    // This is atan2(RStickX, -RStickY), in that argument order.
    let board_adjust_magnitude = (right_x.mul_add(right_x, right_y * right_y)).sqrt();
    if board_adjust_magnitude > 0.0 {
        emit("BoardAdjustAngle", right_x.atan2(-right_y));
        emit("BoardAdjustMag", board_adjust_magnitude);
    }
    if right_x != 0.0 {
        emit("TweakX", right_x);
    }
    if right_y != 0.0 {
        emit("TweakY", right_y);
    }
    //Fill8259A554/8259AF60: current packed bit28 (RB), without an actor gate.
    //The stock ground graph attaches GrabWorld; physics selects valid coping.
    if current_flags & (1 << 28) != 0 {
        emit("GrabWorld", 1.0);
    }
    //Fill calls8259BB18 with the CURRENT RawControllerInput (words7..13).
    //Descriptor constructors82F84F28..82F84F98 establish these five names.
    if right_x != 0.0 {
        emit("HandPlantTweakX", right_x);
    }
    if right_y != 0.0 {
        emit("HandPlantTweakY", right_y);
    }
    for (bit, name) in [
        (20, "HandPlantDismount"),
        (21, "HandPlantOneFootRight"),
        (23, "HandPlantOneFootLeft"),
    ] {
        if current_flags & (1 << bit) != 0 {
            emit(name, 1.0);
        }
    }
    output
}
