use super::{Connection, Controls, filtered_stick};
use bevy::input::gamepad::{Gamepad, GamepadButton};
use bevy::input::mouse::AccumulatedMouseMotion;
use bevy::prelude::*;
use bevy::window::{CursorGrabMode, CursorOptions, PrimaryWindow};
use playerstate_iw4::{UserCmd, buttons};
use survival::{ResourceKind, SharedAction, SharedTradeOffer};

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
        let craft = keys.just_pressed(KeyCode::KeyC) || pad_just_pressed(GamepadButton::West);
        let offer = keys.just_pressed(KeyCode::KeyV) || pad_just_pressed(GamepadButton::DPadRight);
        let accept = keys.just_pressed(KeyCode::Enter) || pad_just_pressed(GamepadButton::South);
        let close = keys.just_pressed(KeyCode::Backspace) || pad_just_pressed(GamepadButton::East);
        let intents = [craft, offer, accept, close]
            .into_iter()
            .filter(|pressed| *pressed)
            .count();
        if intents > 1 {
            controls.notice = Some("Choose one action: craft, offer, accept or close".into());
        } else if intents == 1 {
            if connection.pending {
                controls.notice = Some("The previous action is still pending".into());
            } else if craft {
                controls.action = Some(SharedAction::CraftBandage);
            } else if offer {
                controls.action = Some(SharedAction::OfferBandage);
            } else if let Some(trade) = confirmed_trade_offer(&connection) {
                if accept {
                    if assigned.is_some_and(|(client, _)| client == trade.buyer.client) {
                        controls.action = Some(SharedAction::AcceptTrade { offer_id: trade.id });
                    } else {
                        controls.notice =
                            Some("Only the addressed buyer can accept this offer".into());
                    }
                } else {
                    controls.action = Some(SharedAction::CloseTrade { offer_id: trade.id });
                }
            } else {
                controls.notice = Some("No trade offer is available for you".into());
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
    let mode = keys.just_pressed(KeyCode::KeyB) || pad_just_pressed(GamepadButton::Select);
    let building_controls = controls.building || mode;
    let foundation = building_controls
        && (keys.just_pressed(KeyCode::ArrowLeft) || pad_just_pressed(GamepadButton::DPadLeft));
    let wall = building_controls
        && (keys.just_pressed(KeyCode::ArrowRight) || pad_just_pressed(GamepadButton::DPadRight));
    let rotate = building_controls
        && (keys.just_pressed(KeyCode::KeyQ) || pad_just_pressed(GamepadButton::LeftTrigger));
    let intents = [mode, foundation, wall, rotate]
        .into_iter()
        .filter(|pressed| *pressed)
        .count();
    if intents > 1 {
        controls.notice = Some("Choose one building control: mode, piece or rotate".into());
        return;
    }
    if intents == 1 {
        if connection.pending {
            controls.notice = Some("The previous action is still pending".into());
        } else if mode {
            controls.building = !controls.building;
            let kind = if controls.building_wall {
                "wall"
            } else {
                "foundation"
            };
            let state = if controls.building {
                "enabled"
            } else {
                "disabled"
            };
            controls.notice = Some(format!("Wood {kind} mode {state}"));
        } else if foundation {
            controls.building_wall = false;
            controls.notice = Some("Wood foundation selected | Cost: 200 Wood".into());
        } else if wall {
            controls.building_wall = true;
            controls.notice = Some("Wood wall selected | Cost: 100 Wood".into());
        } else if controls.building_wall {
            controls.wall_axis = u8::from(controls.wall_axis == 0);
            let axis = if controls.wall_axis == 0 { "X" } else { "Y" };
            controls.notice = Some(format!("Wood wall rotated | Axis: {axis}"));
        } else {
            controls.notice = Some("Select Wall to rotate".into());
        }
        return;
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
            controls.action = Some(if controls.building_wall {
                SharedAction::PlaceWoodWall {
                    axis: controls.wall_axis,
                }
            } else {
                SharedAction::PlaceWoodFoundation
            });
        }
    }
}

fn confirmed_trade_offer(connection: &Connection) -> Option<SharedTradeOffer> {
    let world = connection.world.as_ref()?;
    let snapshot = &world.snapshot;
    let offer = snapshot.trade_offer?;
    if offer.id == 0
        || offer.created_at.0 > snapshot.tick.0
        || snapshot.tick.0 >= offer.expires_at.0
        || offer.seller.client == offer.buyer.client
        || offer.seller.owner == offer.buyer.owner
        || ![offer.seller, offer.buyer]
            .iter()
            .any(|party| party.client == snapshot.recipient)
    {
        return None;
    }
    for party in [offer.seller, offer.buyer] {
        let mut owners = snapshot
            .owners
            .iter()
            .filter(|(client, _)| *client == party.client);
        if owners.next().copied() != Some((party.client, party.owner))
            || owners.next().is_some()
            || world
                .replica
                .player(party.client)
                .is_none_or(|player| player.health <= 0)
        {
            return None;
        }
    }
    Some(offer)
}

fn packed_angle(degrees: f32) -> i32 {
    ((degrees.rem_euclid(360.) * (65536. / 360.)).round() as i32) & 0xffff
}
