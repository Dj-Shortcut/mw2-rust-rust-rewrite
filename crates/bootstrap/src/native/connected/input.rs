use super::{Connection, Controls, filtered_stick};
use bevy::input::gamepad::{Gamepad, GamepadButton};
use bevy::input::mouse::AccumulatedMouseMotion;
use bevy::prelude::*;
use bevy::window::{CursorGrabMode, CursorOptions, PrimaryWindow};
use playerstate_iw4::{UserCmd, buttons};
use survival::{ResourceKind, SharedAction};

pub(super) fn sample(
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    motion: Res<AccumulatedMouseMotion>,
    time: Res<Time>,
    pads: Query<(Entity, &Gamepad)>,
    mut windows: Query<(&Window, &mut CursorOptions), With<PrimaryWindow>>,
    connection: Res<Connection>,
    mut controls: ResMut<Controls>,
) {
    controls.neutral();
    controls.notice = None;
    for (entity, pad) in &pads {
        if pad.get_just_pressed().next().is_some()
            || pad.left_stick().length_squared() > 0.25
            || pad.right_stick().length_squared() > 0.25
        {
            controls.pad = Some(entity);
        }
    }
    if controls.pad.is_none_or(|entity| pads.get(entity).is_err()) {
        controls.pad = pads.iter().next().map(|(entity, _)| entity);
    }
    let pad = controls
        .pad
        .and_then(|entity| pads.get(entity).ok())
        .map(|(_, pad)| pad);
    let pad_pressed = |button| pad.is_some_and(|pad| pad.pressed(button));
    let pad_just_pressed = |button| pad.is_some_and(|pad| pad.just_pressed(button));
    let Ok((window, mut cursor)) = windows.single_mut() else {
        controls.focused = false;
        controls.captured = false;
        controls.capture_frames = 2;
        return;
    };
    controls.focused = window.focused;
    if window.focused {
        if keys.just_pressed(KeyCode::Escape) || pad_just_pressed(GamepadButton::Start) {
            controls.paused = !controls.paused;
        }
        if keys.just_pressed(KeyCode::F1) {
            controls.help = !controls.help;
        }
    }
    let connected = connection.status.connected();
    if !connected {
        controls.initialized = None;
    }
    let assigned = connection.world.as_ref().and_then(|world| {
        world
            .replica
            .player(world.snapshot.recipient)
            .map(|player| (world.snapshot.recipient, player))
    });
    if connected
        && let Some((client, player)) = assigned
        && controls.initialized != Some(client)
    {
        controls.pitch = player.viewangles[0];
        controls.yaw = player.viewangles[1].rem_euclid(360.);
        controls.command.angles = std::array::from_fn(|axis| {
            packed_angle(player.viewangles[axis] - player.delta_angles[axis])
        });
        controls.initialized = Some(client);
        controls.captured = false;
        controls.capture_frames = 2;
    }
    let active = window.focused
        && !controls.paused
        && connected
        && controls.content_ready
        && controls.content_error.is_none()
        && assigned.is_some_and(|(_, player)| player.health > 0);
    let capture = active && !controls.inventory;
    cursor.visible = !capture;
    cursor.grab_mode = if capture {
        CursorGrabMode::Locked
    } else {
        CursorGrabMode::None
    };
    if capture && !controls.captured {
        controls.capture_frames = 2;
    }
    controls.captured = capture;
    if !active {
        controls.capture_frames = 2;
        return;
    }
    if controls.capture_frames > 0 {
        controls.capture_frames -= 1;
        return;
    }
    let Some((_, player)) = assigned else {
        return;
    };
    controls.command = UserCmd {
        angles: [
            packed_angle(controls.pitch - player.delta_angles[0]),
            packed_angle(controls.yaw - player.delta_angles[1]),
            packed_angle(player.viewangles[2] - player.delta_angles[2]),
        ],
        ..default()
    };
    if keys.just_pressed(KeyCode::KeyI) || pad_just_pressed(GamepadButton::DPadUp) {
        controls.inventory = !controls.inventory;
        controls.captured = !controls.inventory;
        cursor.visible = controls.inventory;
        cursor.grab_mode = if controls.inventory {
            CursorGrabMode::None
        } else {
            CursorGrabMode::Locked
        };
        controls.capture_frames = u8::from(!controls.inventory);
        // An inactive toggle cancels only unsent work in the network mailbox.
        return;
    }
    if controls.inventory {
        controls.active = true;
        if keys.just_pressed(KeyCode::KeyC) || pad_just_pressed(GamepadButton::West) {
            if connection.pending {
                controls.notice = Some("The previous action is still pending".into());
            } else {
                controls.action = Some(SharedAction::CraftBandage);
            }
        }
        return;
    }
    let movement = pad.map_or(Vec2::ZERO, |pad| filtered_stick(pad.left_stick()));
    let look = pad.map_or(Vec2::ZERO, |pad| filtered_stick(pad.right_stick()));
    let mouse_delta = if motion.delta.is_finite() {
        motion.delta
    } else {
        Vec2::ZERO
    };
    let dt = time.delta_secs();
    let dt = if dt.is_finite() {
        dt.clamp(0., 0.1)
    } else {
        0.
    };
    controls.yaw = (controls.yaw - mouse_delta.x * 0.1 - look.x * 220. * dt).rem_euclid(360.);
    controls.pitch = (controls.pitch + mouse_delta.y * 0.1 - look.y * 150. * dt).clamp(-85., 85.);
    let held = |key| i8::from(keys.pressed(key));
    let mut command = UserCmd {
        forwardmove: ((f32::from(held(KeyCode::KeyW) - held(KeyCode::KeyS)) + movement.y)
            .clamp(-1., 1.)
            * 127.)
            .round() as i8,
        rightmove: ((f32::from(held(KeyCode::KeyD) - held(KeyCode::KeyA)) + movement.x)
            .clamp(-1., 1.)
            * 127.)
            .round() as i8,
        angles: [
            packed_angle(controls.pitch - player.delta_angles[0]),
            packed_angle(controls.yaw - player.delta_angles[1]),
            packed_angle(player.viewangles[2] - player.delta_angles[2]),
        ],
        ..default()
    };
    for (pressed, flag) in [
        (
            keys.pressed(KeyCode::ShiftLeft)
                || keys.pressed(KeyCode::ShiftRight)
                || pad_pressed(GamepadButton::LeftThumb),
            buttons::SPRINT,
        ),
        (
            keys.pressed(KeyCode::Space) || pad_pressed(GamepadButton::South),
            buttons::JUMP,
        ),
        (
            keys.pressed(KeyCode::ControlLeft)
                || keys.pressed(KeyCode::ControlRight)
                || pad_pressed(GamepadButton::East),
            buttons::CROUCH,
        ),
    ] {
        if pressed {
            command.buttons |= flag;
        }
    }
    controls.command = command;
    controls.active = true;
    if keys.just_pressed(KeyCode::KeyB) || pad_just_pressed(GamepadButton::Select) {
        controls.building = !controls.building;
        controls.notice = Some(if controls.building {
            "Wood foundation mode enabled".into()
        } else {
            "Wood foundation mode disabled".into()
        });
    }
    let gather = keys.just_pressed(KeyCode::KeyF) || pad_just_pressed(GamepadButton::North);
    let place = controls.building
        && (mouse.just_pressed(MouseButton::Left)
            || pad_just_pressed(GamepadButton::RightTrigger2));
    if gather && place {
        controls.notice = Some("Choose one action: gather or place".into());
    } else if gather || place {
        if connection.pending {
            controls.notice = Some("The previous action is still pending".into());
        } else if gather {
            let target = connection
                .world
                .as_ref()
                .ok_or_else(|| "The shared world is unavailable".to_string())
                .and_then(|world| world.snapshot.gather_target(&world.replica));
            match target {
                Ok(Some(node)) => {
                    controls.action = match node.kind {
                        ResourceKind::Tree => Some(SharedAction::GatherTree),
                        ResourceKind::Hemp => Some(SharedAction::GatherCloth),
                        _ => None,
                    };
                }
                Ok(None) => {
                    controls.notice = Some("Aim at a tree or hemp plant".into());
                }
                Err(error) => controls.notice = Some(error),
            }
        } else {
            controls.action = Some(SharedAction::PlaceWoodFoundation);
        }
    }
}

fn packed_angle(degrees: f32) -> i32 {
    ((degrees.rem_euclid(360.) * (65536. / 360.)).round() as i32) & 0xffff
}
