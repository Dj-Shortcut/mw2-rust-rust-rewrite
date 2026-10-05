use super::super::{point, surface_mesh};
use super::{Connection, Controls, inventory, network};
use bevy::asset::{LoadState, RecursiveDependencyLoadState};
use bevy::prelude::*;
use rust_building::{Grade, Kind, Piece};
use sim::{ClientId, Tick};
use std::collections::{BTreeMap, BTreeSet};
use survival::{ResourceKind, ResourceNode, Terrain, UNITS_TO_METERS};

type Bounds = ([f32; 3], [f32; 3]);

#[derive(Component)]
pub(super) struct AssignedCamera;

#[derive(Component)]
pub(super) struct Hud;

#[derive(Component)]
pub(super) struct Reticle;

struct NodeVisual {
    node: ResourceNode,
    entities: Vec<Entity>,
}

struct PieceVisual {
    piece: Piece,
    anchor: [f32; 3],
    entities: Vec<Entity>,
}

#[derive(Resource)]
pub(super) struct Visuals {
    operator: Handle<WorldAsset>,
    model_status: String,
    cube: Handle<Mesh>,
    sphere: Handle<Mesh>,
    terrain_material: Handle<StandardMaterial>,
    node_materials: Vec<(
        ResourceKind,
        Handle<StandardMaterial>,
        Option<Handle<StandardMaterial>>,
    )>,
    building_materials: [Handle<StandardMaterial>; 3],
    valid_material: Handle<StandardMaterial>,
    invalid_material: Handle<StandardMaterial>,
    terrain: Option<Entity>,
    tick: Option<Tick>,
    players: BTreeMap<ClientId, Entity>,
    nodes: BTreeMap<u32, NodeVisual>,
    pieces: BTreeMap<u32, PieceVisual>,
    ghost: Option<(Vec<Bounds>, bool)>,
    ghost_entities: Vec<Entity>,
}

pub(super) fn setup(
    mut commands: Commands,
    server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mut node_materials = Vec::new();
    for kind in [
        ResourceKind::Tree,
        ResourceKind::Stone,
        ResourceKind::Metal,
        ResourceKind::Berry,
        ResourceKind::Water,
        ResourceKind::Hemp,
    ] {
        let visual = kind.visual();
        let material = |color: [f32; 4]| StandardMaterial {
            base_color: Color::srgba(color[0], color[1], color[2], color[3]),
            perceptual_roughness: 0.95,
            ..default()
        };
        let canopy = visual
            .canopy
            .map(|shape| materials.add(material(shape.color)));
        node_materials.push((kind, materials.add(material(visual.color)), canopy));
    }
    let ghost_material = |color| StandardMaterial {
        base_color: color,
        alpha_mode: AlphaMode::Blend,
        depth_bias: 8.,
        unlit: true,
        cull_mode: None,
        ..default()
    };
    commands.insert_resource(Visuals {
        operator: server.load("authored/masked_operator.glb#Scene0"),
        model_status: "Loading operator...".into(),
        cube: meshes.add(Cuboid::new(1., 1., 1.)),
        sphere: meshes.add(Sphere::new(1.).mesh().uv(16, 12)),
        terrain_material: materials.add(StandardMaterial {
            base_color: Color::WHITE,
            perceptual_roughness: 0.95,
            ..default()
        }),
        node_materials,
        building_materials: [
            materials.add(StandardMaterial {
                base_color: Color::srgb(0.42, 0.23, 0.10),
                depth_bias: 4.,
                perceptual_roughness: 0.9,
                ..default()
            }),
            materials.add(StandardMaterial {
                base_color: Color::srgb(0.42, 0.44, 0.43),
                depth_bias: 4.,
                perceptual_roughness: 0.95,
                ..default()
            }),
            materials.add(StandardMaterial {
                base_color: Color::srgb(0.30, 0.35, 0.39),
                depth_bias: 4.,
                metallic: 0.85,
                perceptual_roughness: 0.4,
                ..default()
            }),
        ],
        valid_material: materials.add(ghost_material(Color::srgba(0.15, 1., 0.45, 0.42))),
        invalid_material: materials.add(ghost_material(Color::srgba(1., 0.15, 0.12, 0.42))),
        terrain: None,
        tick: None,
        players: BTreeMap::new(),
        nodes: BTreeMap::new(),
        pieces: BTreeMap::new(),
        ghost: None,
        ghost_entities: Vec::new(),
    });
    commands.spawn((
        AssignedCamera,
        Camera3d::default(),
        bevy::core_pipeline::tonemapping::Tonemapping::AcesFitted,
        Projection::Perspective(PerspectiveProjection {
            fov: 75f32.to_radians(),
            near: 0.03,
            far: 1000.,
            ..default()
        }),
        Transform::default(),
    ));
    commands.spawn((
        DirectionalLight {
            illuminance: 12000.,
            shadow_maps_enabled: true,
            ..default()
        },
        Transform::from_xyz(20., 30., 10.).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    commands.spawn((
        Hud,
        Text::new("Joining shared world..."),
        TextFont {
            font_size: FontSize::Px(18.),
            ..default()
        },
        TextColor(Color::WHITE),
        BackgroundColor(Color::srgba(0.015, 0.022, 0.03, 0.88)),
        Node {
            position_type: PositionType::Absolute,
            left: px(16),
            top: px(16),
            max_width: percent(97.5),
            padding: UiRect::all(px(12)),
            ..default()
        },
    ));
    commands.spawn((
        Reticle,
        Text::new("+"),
        TextFont {
            font_size: FontSize::Px(24.),
            ..default()
        },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            left: percent(50),
            top: percent(50),
            display: Display::None,
            ..default()
        },
    ));
}

pub(super) fn present(
    mut commands: Commands,
    server: Res<AssetServer>,
    connection: Res<Connection>,
    mut controls: ResMut<Controls>,
    mut visuals: ResMut<Visuals>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut cameras: Query<(&mut Camera, &mut Transform), With<AssignedCamera>>,
    mut hud: Query<(&mut Text, &mut TextFont), With<Hud>>,
    mut reticles: Query<&mut Node, With<Reticle>>,
) {
    match server.get_load_states(visuals.operator.id()) {
        Some((LoadState::Failed(error), _, _))
        | Some((_, _, RecursiveDependencyLoadState::Failed(error))) => {
            let error = format!("Failed to load operator: {error}");
            controls.content_ready = false;
            controls.content_error = Some(error.clone());
            visuals.model_status = error;
        }
        Some((LoadState::Loaded, _, RecursiveDependencyLoadState::Loaded)) => {
            controls.content_ready = true;
            controls.content_error = None;
            visuals.model_status = "Operator loaded".into();
        }
        _ => {
            controls.content_ready = false;
            controls.content_error = None;
            visuals.model_status = "Loading operator...".into();
        }
    }
    if !controls.content_ready {
        controls.neutral();
    }

    if let Some(world) = &connection.world {
        if visuals.terrain.is_none() {
            let terrain = Terrain::new(world.snapshot.terrain_seed);
            visuals.terrain = Some(
                commands
                    .spawn((
                        Mesh3d(meshes.add(surface_mesh(
                            &terrain.positions,
                            &terrain.normals,
                            &terrain.indices,
                            Some(&terrain.colors),
                        ))),
                        MeshMaterial3d(visuals.terrain_material.clone()),
                        Transform::default(),
                    ))
                    .id(),
            );
        }
        if visuals.tick != Some(world.snapshot.tick) {
            refresh_players(&mut commands, world, &mut visuals);
            refresh_nodes(&mut commands, &world.snapshot.nodes, &mut visuals);
            refresh_buildings(&mut commands, world, &mut visuals);
            visuals.tick = Some(world.snapshot.tick);
        }
        if let Some(player) = world.replica.player(world.snapshot.recipient) {
            let pitch = player.viewangles[0].to_radians();
            let yaw = player.viewangles[1].to_radians();
            let direction = Vec3::new(
                pitch.cos() * yaw.cos(),
                -pitch.sin(),
                -pitch.cos() * yaw.sin(),
            );
            for (mut camera, mut transform) in &mut cameras {
                camera.is_active = true;
                transform.translation = point([
                    player.origin[0],
                    player.origin[1],
                    player.origin[2] + player.view_height_current,
                ]);
                transform.look_to(direction, Vec3::Y);
            }
        }
    }

    let (ghost, hint) = interaction(&connection, &controls);
    refresh_ghost(&mut commands, &mut visuals, ghost);
    let content = status(&connection, &controls, &visuals.model_status, &hint);
    for (mut text, mut font) in &mut hud {
        if **text != content {
            **text = content.clone();
        }
        let font_size = FontSize::Px(if controls.inventory { 14. } else { 18. });
        if font.font_size != font_size {
            font.font_size = font_size;
        }
    }
    for mut node in &mut reticles {
        node.display = if controls.active && !controls.inventory {
            Display::Flex
        } else {
            Display::None
        };
    }
}

fn refresh_players(commands: &mut Commands, world: &network::ReceivedWorld, visuals: &mut Visuals) {
    let mut current = BTreeSet::new();
    world.replica.visit_players(|id, player| {
        current.insert(id);
        let transform = Transform::from_translation(point(player.origin))
            .with_rotation(Quat::from_rotation_y(
                std::f32::consts::FRAC_PI_2 + player.viewangles[1].to_radians(),
            ))
            .with_scale(Vec3::new(
                1.,
                (player.view_height_current / 60.).clamp(0.45, 1.),
                1.,
            ));
        let visibility = if id == world.snapshot.recipient || player.health <= 0 {
            Visibility::Hidden
        } else {
            Visibility::Inherited
        };
        if let Some(entity) = visuals.players.get(&id) {
            commands.entity(*entity).insert((transform, visibility));
        } else {
            let entity = commands
                .spawn((
                    WorldAssetRoot(visuals.operator.clone()),
                    transform,
                    visibility,
                ))
                .id();
            visuals.players.insert(id, entity);
        }
    });
    visuals.players.retain(|id, entity| {
        if current.contains(id) {
            true
        } else {
            commands.entity(*entity).despawn();
            false
        }
    });
}

fn refresh_nodes(commands: &mut Commands, nodes: &[ResourceNode], visuals: &mut Visuals) {
    let mut current = BTreeSet::new();
    for node in nodes.iter().filter(|node| node.remaining > 0) {
        current.insert(node.id);
        if let Some(existing) = visuals.nodes.get_mut(&node.id)
            && existing.node.kind == node.kind
            && existing.node.position == node.position
        {
            existing.node = node.clone();
            continue;
        }
        if let Some(existing) = visuals.nodes.remove(&node.id) {
            despawn(commands, existing.entities);
        }
        let Some((_, material, canopy_material)) = visuals
            .node_materials
            .iter()
            .find(|(kind, _, _)| *kind == node.kind)
        else {
            continue;
        };
        let shape = node.kind.visual();
        let center = std::array::from_fn(|axis| node.position[axis] + shape.center[axis]);
        let mut size = Vec3::new(
            shape.half_extents[0],
            shape.half_extents[2],
            shape.half_extents[1],
        ) * (2. * UNITS_TO_METERS);
        let mesh = if node.kind == ResourceKind::Berry {
            size *= 0.5;
            visuals.sphere.clone()
        } else {
            visuals.cube.clone()
        };
        let mut entities = vec![
            commands
                .spawn((
                    Mesh3d(mesh),
                    MeshMaterial3d(material.clone()),
                    Transform::from_translation(point(center)).with_scale(size),
                ))
                .id(),
        ];
        if let (Some(canopy), Some(material)) = (shape.canopy, canopy_material) {
            let center = std::array::from_fn(|axis| node.position[axis] + canopy.center[axis]);
            entities.push(
                commands
                    .spawn((
                        Mesh3d(visuals.sphere.clone()),
                        MeshMaterial3d(material.clone()),
                        Transform::from_translation(point(center))
                            .with_scale(Vec3::splat(canopy.radius * UNITS_TO_METERS)),
                    ))
                    .id(),
            );
        }
        visuals.nodes.insert(
            node.id,
            NodeVisual {
                node: node.clone(),
                entities,
            },
        );
    }
    visuals.nodes.retain(|id, visual| {
        if current.contains(id) {
            true
        } else {
            despawn(commands, visual.entities.drain(..));
            false
        }
    });
}

fn refresh_buildings(
    commands: &mut Commands,
    world: &network::ReceivedWorld,
    visuals: &mut Visuals,
) {
    let buildings = world.replica.buildings();
    let mut current = BTreeSet::new();
    for piece in buildings.pieces() {
        current.insert(piece.id);
        if visuals
            .pieces
            .get(&piece.id)
            .is_some_and(|visual| visual.piece == *piece && visual.anchor == buildings.anchor)
        {
            continue;
        }
        if let Some(existing) = visuals.pieces.remove(&piece.id) {
            despawn(commands, existing.entities);
        }
        let material = visuals.building_materials[match piece.grade {
            Grade::Wood => 0,
            Grade::Stone => 1,
            Grade::Metal => 2,
        }]
        .clone();
        let entities = buildings
            .bounds(piece)
            .into_iter()
            .map(|bounds| {
                commands
                    .spawn((
                        Mesh3d(visuals.cube.clone()),
                        MeshMaterial3d(material.clone()),
                        box_transform(bounds),
                    ))
                    .id()
            })
            .collect();
        visuals.pieces.insert(
            piece.id,
            PieceVisual {
                piece: piece.clone(),
                anchor: buildings.anchor,
                entities,
            },
        );
    }
    visuals.pieces.retain(|id, visual| {
        if current.contains(id) {
            true
        } else {
            despawn(commands, visual.entities.drain(..));
            false
        }
    });
}

fn interaction(
    connection: &Connection,
    controls: &Controls,
) -> (Option<(Vec<Bounds>, bool)>, String) {
    let Some(world) = &connection.world else {
        return (None, String::new());
    };
    if !controls.active || controls.inventory {
        return (None, String::new());
    }
    if controls.building {
        let preview = if controls.building_wall {
            world
                .snapshot
                .wall_preview(&world.replica, controls.wall_axis)
        } else {
            world.snapshot.foundation_preview(&world.replica)
        };
        match preview {
            Ok(preview) => {
                let valid = preview.valid();
                let hint = preview.error.unwrap_or_else(|| {
                    if connection.pending {
                        "Waiting for the shared world...".into()
                    } else if controls.building_wall {
                        "Place wall: Left click / Controller RT".into()
                    } else {
                        "Place foundation: Left click / Controller RT".into()
                    }
                });
                let ghost = (!preview.bounds.is_empty()).then_some((preview.bounds, valid));
                (ghost, hint)
            }
            Err(error) => (None, error),
        }
    } else {
        match world.snapshot.gather_target(&world.replica) {
            Ok(Some(node)) => {
                let resource = match node.kind {
                    ResourceKind::Tree => "Wood",
                    ResourceKind::Hemp => "Cloth",
                    _ => return (None, String::new()),
                };
                (
                    None,
                    format!(
                        "{}: {} {resource} remaining | F / Controller Y to gather",
                        node.kind.name(),
                        node.remaining
                    ),
                )
            }
            Ok(None) => (None, String::new()),
            Err(error) => (None, error),
        }
    }
}

fn refresh_ghost(
    commands: &mut Commands,
    visuals: &mut Visuals,
    ghost: Option<(Vec<Bounds>, bool)>,
) {
    if visuals.ghost == ghost {
        return;
    }
    let Some((bounds, valid)) = &ghost else {
        despawn(commands, visuals.ghost_entities.drain(..));
        visuals.ghost = None;
        return;
    };
    let material = if *valid {
        visuals.valid_material.clone()
    } else {
        visuals.invalid_material.clone()
    };
    while visuals.ghost_entities.len() > bounds.len() {
        if let Some(entity) = visuals.ghost_entities.pop() {
            commands.entity(entity).despawn();
        }
    }
    for (index, &bounds) in bounds.iter().enumerate() {
        if let Some(entity) = visuals.ghost_entities.get(index) {
            commands
                .entity(*entity)
                .insert((box_transform(bounds), MeshMaterial3d(material.clone())));
        } else {
            visuals.ghost_entities.push(
                commands
                    .spawn((
                        Mesh3d(visuals.cube.clone()),
                        MeshMaterial3d(material.clone()),
                        box_transform(bounds),
                        bevy::light::NotShadowCaster,
                        bevy::light::NotShadowReceiver,
                    ))
                    .id(),
            );
        }
    }
    visuals.ghost = ghost;
}

fn box_transform((lo, hi): Bounds) -> Transform {
    let center = std::array::from_fn(|axis| (lo[axis] + hi[axis]) * 0.5);
    let size = Vec3::new(hi[0] - lo[0], hi[2] - lo[2], hi[1] - lo[1]) * UNITS_TO_METERS;
    Transform::from_translation(point(center)).with_scale(size)
}

fn despawn(commands: &mut Commands, entities: impl IntoIterator<Item = Entity>) {
    for entity in entities {
        commands.entity(entity).despawn();
    }
}

fn status(connection: &Connection, controls: &Controls, model_status: &str, hint: &str) -> String {
    let mut content = match &connection.status {
        network::NetworkStatus::Connecting => "Joining shared world...".into(),
        network::NetworkStatus::ApplyingBootstrap => "Loading shared world...".into(),
        network::NetworkStatus::Connected => "SHARED WORLD".into(),
        network::NetworkStatus::Stalled => {
            "Connection interrupted. Waiting for the shared world...".into()
        }
        network::NetworkStatus::Failed(error) => format!("Disconnected: {error}"),
    };
    if let Some(world) = &connection.world {
        let owner = world
            .snapshot
            .owners
            .iter()
            .find(|(client, _)| *client == world.snapshot.recipient)
            .map(|(_, owner)| *owner);
        if let Some(owner) = owner {
            let wood = world.replica.buildings().inventory(owner).wood;
            if controls.building && controls.building_wall && !controls.inventory {
                let cost = Grade::Wood.cost(Kind::Wall).wood;
                content.push_str(&format!("\nWood: {wood} | Wall cost: {cost} Wood"));
            } else {
                content.push_str(&format!("\nWood: {wood} | Foundation cost: 200 Wood"));
            }
        }
        if controls.building && !controls.inventory {
            if controls.building_wall {
                let orientation = match controls.wall_axis {
                    0 => "X-axis",
                    1 => "Y-axis",
                    _ => "invalid",
                };
                content.push_str(&format!(
                    "\nWOOD WALL | Orientation: {orientation} | B / Controller Back to close\nQ / Controller LB rotate Wall"
                ));
            } else {
                content.push_str("\nWOOD FOUNDATION | B / Controller Back to close");
            }
            content
                .push_str("\nLeft / Controller Left: Foundation | Right / Controller Right: Wall");
        }
    }
    if !controls.content_ready {
        content.push_str(&format!("\n{model_status}"));
    }
    if controls.paused {
        content.push_str("\nInput paused.\nEsc / Controller Start to resume");
    } else if !controls.focused {
        content.push_str("\nFocus this window to control your player.");
    }
    if controls.inventory {
        if let Some(world) = &connection.world {
            content.push_str(&format!(
                "\n{}",
                inventory::panel(world, connection.pending, &connection.message)
            ));
        } else {
            content.push_str("\nInventory is not available yet.");
            if !connection.message.is_empty() {
                content.push_str(&format!("\n{}", connection.message));
            }
        }
        if controls.help {
            content.push_str("\nCrafting needs 4 Cloth and room for the output.\nPausing stops your input; it does not cancel a posted offer.\nF1 hide help");
        } else {
            content.push_str("\nEsc pause | F1 help");
        }
    } else {
        if connection.pending {
            content.push_str("\nAction pending...");
        }
        if !hint.is_empty() {
            content.push_str(&format!("\n{hint}"));
        }
        if !connection.message.is_empty() {
            if controls.building {
                content.push_str(&format!("\nLast own action: {}", connection.message));
            } else {
                content.push_str(&format!("\n{}", connection.message));
            }
        }
        if controls.help {
            content.push_str("\nWASD move | Mouse look | Shift sprint | Space jump | Ctrl crouch\nF gather Tree/Hemp | I inventory | B building mode\nBuild: Left Foundation | Right Wall | Q rotate Wall | Left click place\nEsc pause | F1 hide help\nController: LS move | RS look | LS click sprint | A jump | B crouch\nY gather Tree/Hemp | Up inventory | Back building mode | Start pause\nBuild: Left Foundation | Right Wall | LB rotate Wall | RT place");
        } else {
            content.push_str(
                "\nF gather Tree/Hemp | I inventory | B building mode | Esc pause | F1 help",
            );
        }
    }
    content
}
