use crate::{ConsoleCommand, ConsoleLine, ConsoleRegistry, ConsoleSettings, ConsoleState};
use bevy::prelude::*;
use rust_building::{BuildingWorld, CELL, Grade, Kind, Resources, Socket};

const USAGE: &str = "build status | anchor | grant <wood> <stone> <metal> | place <foundation|floor|wall|doorway> <x> <y> <level> <axis> | upgrade <id> <stone|metal> | door <id> | demolish <id> | save <slot> | load <slot>";

#[derive(Resource, Default)]
pub(crate) struct BuildingMode {
    pub active: bool,
    selection: usize,
    axis: u8,
    message: String,
}

#[derive(Component)]
pub(crate) struct BuildingHud;

pub(crate) fn spawn_hud(commands: &mut Commands, font: Handle<Font>) {
    commands.spawn((
        BuildingHud,
        ui::UiLayer::Overlay,
        Visibility::Hidden,
        Node {
            position_type: PositionType::Absolute,
            left: px(24),
            top: px(80),
            padding: UiRect::all(px(12)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.02, 0.03, 0.04, 0.8)),
        GlobalZIndex(18000),
        Text::new(""),
        TextFont {
            font: font.into(),
            font_size: FontSize::Px(17.),
            ..default()
        },
        TextColor(Color::WHITE),
    ));
}

pub(crate) fn consumes(mode: &BuildingMode, button: crate::BindButton) -> bool {
    use crate::{BindButton as B, PadButton as P};
    mode.active
        && matches!(
            button,
            B::Mouse(MouseButton::Left | MouseButton::Right)
                | B::Pad(
                    P::LeftTrigger
                        | P::RightTrigger
                        | P::DpadUp
                        | P::DpadDown
                        | P::DpadLeft
                        | P::DpadRight
                        | P::West
                        | P::North
                        | P::East
                        | P::Select
                )
                | B::Key(
                    KeyCode::KeyB | KeyCode::KeyQ | KeyCode::KeyE | KeyCode::KeyR | KeyCode::KeyF
                )
        )
}

pub(crate) fn tools(
    mut mode: ResMut<BuildingMode>,
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    pads: Query<&bevy::input::gamepad::Gamepad>,
    active: Res<frame::ActivePad>,
    screen: Res<frame::AppScreen>,
    role: Res<frame::RuntimeRole>,
    console: Res<ConsoleState>,
    skate: Res<frame::SkateMode>,
    devices: Res<frame::InputDevices>,
    mut authority: Option<ResMut<net::AuthorityWorld>>,
    local: Res<net::LocalPresentClient>,
    script_menus: Option<Res<hud::ScriptMenus>>,
) {
    use bevy::input::gamepad::GamepadButton as G;
    let pad = active.0.and_then(|id| pads.get(id).ok());
    let pressed = |b| pad.is_some_and(|p| p.just_pressed(b));
    if *screen != frame::AppScreen::InGame
        || *role != frame::RuntimeRole::Listen
        || skate.active
        || skate.entering
    {
        mode.active = false;
        return;
    }
    if console.open
        || script_menus.is_some_and(|m| m.captures_input())
        || !devices.focused
        || pressed(G::Start)
        || keys.just_pressed(KeyCode::Escape)
    {
        return;
    }
    let Some(authority) = authority.as_deref_mut() else {
        return;
    };
    if keys.just_pressed(KeyCode::KeyB)
        || (pad.is_some_and(|p| p.pressed(G::Select)) && pressed(G::DPadUp))
    {
        mode.active = !mode.active;
        if mode.active && authority.0.buildings().pieces().count() == 0 {
            let _ =
                execute(&["anchor".into()], &mut authority.0, local.0).map(|s| mode.message = s);
        }
    }
    if !mode.active {
        return;
    }
    if keys.just_pressed(KeyCode::KeyQ) || pressed(G::DPadLeft) {
        mode.selection = (mode.selection + 3) % 4;
    }
    if keys.just_pressed(KeyCode::KeyE) || pressed(G::DPadRight) {
        mode.selection = (mode.selection + 1) % 4;
    }
    if keys.just_pressed(KeyCode::KeyR) || pressed(G::LeftTrigger2) {
        mode.axis ^= 1;
    }
    if keys.just_pressed(KeyCode::KeyF) || pressed(G::North) {
        mode.message = execute(
            &["save".into(), "autosave".into()],
            &mut authority.0,
            local.0,
        )
        .unwrap_or_else(|e| e);
    }
    let Some(player) = authority.0.player(local.0) else {
        return;
    };
    let (pitch, yaw) = (
        player.viewangles[0].to_radians(),
        player.viewangles[1].to_radians(),
    );
    let start = [player.origin[0], player.origin[1], player.origin[2] + 60.];
    let direction = [
        pitch.cos() * yaw.cos(),
        pitch.cos() * yaw.sin(),
        -pitch.sin(),
    ];
    let end = std::array::from_fn(|k| start[k] + direction[k] * 360.);
    let hit = authority.0.trace_world(start, end, [0.; 3], [0.; 3], 1);
    let anchor = authority.0.buildings().anchor;
    let position = hit.endpos;
    let names = ["foundation", "floor", "wall", "doorway"];
    let selection = mode.selection;
    let socket = Socket {
        x: if selection >= 2 && mode.axis == 1 {
            ((position[0] - anchor[0]) / CELL).round() as i32
        } else {
            ((position[0] - anchor[0]) / CELL).floor() as i32
        },
        y: if selection >= 2 && mode.axis == 0 {
            ((position[1] - anchor[1]) / CELL).round() as i32
        } else {
            ((position[1] - anchor[1]) / CELL).floor() as i32
        },
        level: if selection == 0 {
            0
        } else if selection == 1 {
            (((position[2] - anchor[2]) / CELL).round() as i32).max(1)
        } else {
            (((position[2] - anchor[2]) / CELL + 0.001).floor() as i32).max(0)
        },
        axis: if selection >= 2 { mode.axis } else { 0 },
    };
    if mouse.just_pressed(MouseButton::Left) || pressed(G::RightTrigger2) {
        if hit.fraction >= 1. {
            mode.message = "Aim at a surface within building reach".into();
            return;
        }
        let args = vec![
            "place".into(),
            names[selection].into(),
            socket.x.to_string(),
            socket.y.to_string(),
            socket.level.to_string(),
            socket.axis.to_string(),
        ];
        mode.message = execute(&args, &mut authority.0, local.0).unwrap_or_else(|e| e);
    }
    if mouse.just_pressed(MouseButton::Right) || pressed(G::West) {
        let candidates: Vec<_> = authority
            .0
            .buildings()
            .pieces()
            .filter(|p| p.kind == Kind::Doorway)
            .filter(|p| {
                authority.0.buildings().bounds(p).iter().any(|(lo, hi)| {
                    (0..3).all(|k| position[k] >= lo[k] - 5. && position[k] <= hi[k] + 5.)
                })
            })
            .map(|p| p.id)
            .collect();
        mode.message = if let Some(id) = candidates.first() {
            execute(&["door".into(), id.to_string()], &mut authority.0, local.0)
                .unwrap_or_else(|e| e)
        } else {
            "Aim at the door frame to open or close it".into()
        };
    }
}

pub(crate) fn update_hud(
    mode: Res<BuildingMode>,
    authority: Option<Res<net::AuthorityWorld>>,
    local: Res<net::LocalPresentClient>,
    mut hud: Query<(&mut Text, &mut Visibility), With<BuildingHud>>,
) {
    for (mut text, mut visibility) in &mut hud {
        *visibility = if mode.active {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
        if mode.active {
            let resources = authority
                .as_ref()
                .map(|a| a.0.buildings().inventory(local.0.0))
                .unwrap_or_default();
            **text = format!(
                "BUILD / {} / axis {}\nWood {}  Stone {}  Metal {}\nRT: place  LT: rotate  D-pad left/right: select\nX: door  Y: save  Back + D-pad up: leave\nMouse: LMB place / RMB door | B mode / Q,E select / R rotate / F save\n{}",
                ["Foundation", "Floor", "Wall", "Doorway"][mode.selection],
                mode.axis,
                resources.wood,
                resources.stone,
                resources.metal,
                mode.message
            );
        }
    }
}

pub(crate) fn register(registry: &mut ConsoleRegistry) {
    registry.register(crate::CommandSpec::new("build").usage(USAGE));
}

pub(crate) fn route(
    mut events: MessageReader<ConsoleCommand>,
    mut authority: Option<ResMut<net::AuthorityWorld>>,
    local: Res<net::LocalPresentClient>,
    role: Res<frame::RuntimeRole>,
    mut console: ResMut<ConsoleState>,
    settings: Res<ConsoleSettings>,
    mut line: ResMut<ConsoleLine>,
) {
    for command in events.read().filter(|c| c.name == "build") {
        let result = if *role != frame::RuntimeRole::Listen {
            Err("Building commands currently require a local listen host".into())
        } else if let Some(authority) = authority.as_deref_mut() {
            execute(&command.args, &mut authority.0, local.0)
        } else {
            Err("Load a map before building".into())
        };
        let message = result.unwrap_or_else(|e| format!("build: {e}"));
        console.echo(message.clone(), settings.log_capacity);
        line.0 = message;
    }
}

fn overlaps_player(world: &sim::SimWorld, lo: [f32; 3], hi: [f32; 3]) -> bool {
    let mut overlaps = false;
    world.visit_players(|_, player| {
        if player.pm_type == 0 {
            overlaps |= (0..3).all(|k| {
                hi[k] > player.origin[k] + [-15., -15., 0.][k]
                    && lo[k] < player.origin[k] + [15., 15., 70.][k]
            });
        }
    });
    overlaps
}

fn execute(
    args: &[String],
    world: &mut sim::SimWorld,
    client: sim::ClientId,
) -> Result<String, String> {
    let verb = args.first().map(String::as_str).unwrap_or("status");
    let number = |index: usize| {
        args.get(index)
            .ok_or_else(|| USAGE.to_string())
            .and_then(|s| {
                s.parse::<u32>()
                    .map_err(|_| "Expected an unsigned integer".into())
            })
    };
    if verb == "status" && args.len() <= 1 {
        return Ok(format!(
            "Buildings: {} | anchor {:?} | resources {:?}",
            world.buildings().pieces().count(),
            world.buildings().anchor,
            world.buildings().inventory(client.0)
        ));
    }
    let player = world
        .player(client)
        .filter(|p| p.pm_type == 0)
        .ok_or("Spawn an alive player before building")?;
    let origin = player.origin;
    let yaw = player.viewangles[1].to_radians();
    let close = |world: &sim::SimWorld, id: u32| -> Result<(), String> {
        let p = world.buildings().piece(id).ok_or("Unknown building id")?;
        let (lo, hi) = world.buildings().bounds(p)[0];
        if (0..3)
            .map(|k| {
                let d = (lo[k] - origin[k]).max(origin[k] - hi[k]).max(0.);
                d * d
            })
            .sum::<f32>()
            > 600f32.powi(2)
        {
            return Err("Building is outside interaction range".into());
        }
        Ok(())
    };
    match verb {
        "anchor" if args.len() == 1 => {
            let start = [
                origin[0] + 180. * yaw.cos(),
                origin[1] + 180. * yaw.sin(),
                origin[2] + 80.,
            ];
            let end = [start[0], start[1], origin[2] - 240.];
            let hit = world.trace_static_world(start, end, [0.; 3], [0.; 3], 1);
            if hit.fraction >= 1. || hit.normal[2] < 0.7 || hit.startsolid != 0 {
                return Err("No suitable ground for the building anchor".into());
            }
            world
                .buildings_mut()
                .set_anchor(hit.endpos)
                .map_err(|e| e.to_string())?;
            Ok(format!("Building grid anchored at {:?}", hit.endpos))
        }
        "grant" if args.len() == 4 => {
            world
                .buildings_mut()
                .grant(
                    client.0,
                    Resources {
                        wood: number(1)?,
                        stone: number(2)?,
                        metal: number(3)?,
                    },
                )
                .map_err(|e| e.to_string())?;
            Ok("Development resources granted; gathering is not implemented".into())
        }
        "place" if args.len() == 6 => {
            let kind = match args[1].as_str() {
                "foundation" => Kind::Foundation,
                "floor" => Kind::Floor,
                "wall" => Kind::Wall,
                "doorway" => Kind::Doorway,
                _ => return Err(USAGE.into()),
            };
            let value = |i: usize| {
                args[i]
                    .parse::<i32>()
                    .map_err(|_| "Invalid grid coordinate".to_string())
            };
            let socket = Socket {
                x: value(2)?,
                y: value(3)?,
                level: value(4)?,
                axis: args[5].parse::<u8>().map_err(|_| "Invalid wall axis")?,
            };
            let a = world.buildings().anchor;
            let center = [
                a[0] + (socket.x as f32 + 0.5) * CELL,
                a[1] + (socket.y as f32 + 0.5) * CELL,
                a[2] + socket.level as f32 * CELL,
            ];
            if (0..3).map(|k| (center[k] - origin[k]).powi(2)).sum::<f32>() > 600f32.powi(2) {
                return Err("Placement is outside interaction range".into());
            }
            let grounded = kind != Kind::Foundation
                || [10., CELL - 10.].into_iter().all(|x| {
                    [10., CELL - 10.].into_iter().all(|y| {
                        let p = [
                            a[0] + socket.x as f32 * CELL + x,
                            a[1] + socket.y as f32 * CELL + y,
                            a[2] + 8.,
                        ];
                        let hit = world.trace_static_world(
                            p,
                            [p[0], p[1], a[2] - 16.],
                            [0.; 3],
                            [0.; 3],
                            1,
                        );
                        hit.fraction < 1. && hit.normal[2] > 0.7 && hit.startsolid == 0
                    })
                });
            world
                .buildings()
                .can_place(client.0, kind, socket, grounded)
                .map_err(|e| e.to_string())?;
            let mut preview = world.buildings().clone();
            let id = preview
                .place(client.0, kind, socket, grounded)
                .map_err(|e| e.to_string())?;
            let p = preview
                .piece(id)
                .ok_or("Placement did not produce a piece")?;
            for (lo, hi) in preview.bounds(p) {
                if overlaps_player(world, lo, hi) {
                    return Err("Cannot place through a player".into());
                }
                if kind != Kind::Foundation {
                    let c = std::array::from_fn(|k| (lo[k] + hi[k]) * 0.5);
                    let mins = std::array::from_fn(|k| lo[k] - c[k] + 0.5);
                    let maxs = std::array::from_fn(|k| hi[k] - c[k] - 0.5);
                    let hit = world.trace_static_world(c, c, mins, maxs, 1);
                    if hit.startsolid != 0 || hit.allsolid != 0 {
                        return Err("Placement overlaps map geometry".into());
                    }
                }
            }
            *world.buildings_mut() = preview;
            Ok(format!("Placed {kind:?} id={id}"))
        }
        "upgrade" if args.len() == 3 => {
            let id = number(1)?;
            close(world, id)?;
            let grade = match args[2].as_str() {
                "stone" => Grade::Stone,
                "metal" => Grade::Metal,
                _ => return Err(USAGE.into()),
            };
            world
                .buildings_mut()
                .upgrade(client.0, id, grade)
                .map_err(|e| e.to_string())?;
            Ok(format!("Upgraded {id} to {grade:?}"))
        }
        "door" if args.len() == 2 => {
            let id = number(1)?;
            close(world, id)?;
            let mut changed = world.buildings().clone();
            changed
                .toggle_door(client.0, id)
                .map_err(|e| e.to_string())?;
            let door = changed.piece(id).ok_or("Unknown building id")?;
            if !door.open
                && changed
                    .bounds(door)
                    .into_iter()
                    .any(|(lo, hi)| overlaps_player(world, lo, hi))
            {
                return Err("Cannot close a door through a player".into());
            }
            *world.buildings_mut() = changed;
            Ok(format!("Toggled door {id}"))
        }
        "demolish" if args.len() == 2 => {
            let id = number(1)?;
            close(world, id)?;
            let removed = world
                .buildings_mut()
                .demolish(client.0, id)
                .map_err(|e| e.to_string())?;
            Ok(format!("Removed buildings {removed:?}"))
        }
        "save" | "load" if args.len() == 2 => {
            let slot = &args[1];
            if slot.is_empty()
                || slot.len() > 32
                || !slot
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
            {
                return Err(
                    "Use a save name of 1–32 letters, digits, underscores or hyphens".into(),
                );
            }
            let directory = std::path::Path::new("iw4l-artifacts/buildings")
                .join(format!("{:016x}", world.content_digest()));
            let path = directory.join(format!("{slot}.json"));
            if verb == "save" {
                std::fs::create_dir_all(&directory).map_err(|e| e.to_string())?;
                let bytes = world.buildings().to_json().map_err(|e| e.to_string())?;
                let temporary = directory.join(format!("{slot}.pending"));
                std::fs::write(&temporary, bytes).map_err(|e| e.to_string())?;
                std::fs::rename(&temporary, &path).map_err(|e| e.to_string())?;
            } else {
                use std::io::Read;
                let mut bytes = Vec::new();
                std::fs::File::open(&path)
                    .map_err(|e| e.to_string())?
                    .take((rust_building::MAX_SAVE_BYTES + 1) as u64)
                    .read_to_end(&mut bytes)
                    .map_err(|e| e.to_string())?;
                let loaded = BuildingWorld::from_json(&bytes).map_err(|e| e.to_string())?;
                if loaded.pieces().any(|piece| {
                    loaded
                        .bounds(piece)
                        .into_iter()
                        .any(|(lo, hi)| overlaps_player(world, lo, hi))
                }) {
                    return Err("Saved buildings overlap a player; move before loading".into());
                }
                *world.buildings_mut() = loaded;
            }
            Ok(format!("{verb}: {}", path.display()))
        }
        _ => Err(USAGE.into()),
    }
}
