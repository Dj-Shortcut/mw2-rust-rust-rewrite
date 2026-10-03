use super::{GameSession, point};
use bevy::mesh::MeshBuilder;
use bevy::prelude::*;
use std::collections::BTreeMap;
use std::f32::consts::{FRAC_PI_2, FRAC_PI_4, TAU};
use survival::{COOK_WOOD, FireState, Session};

#[derive(Clone, Copy, PartialEq)]
enum FireVisualState {
    Idle,
    Cooking,
    Ready,
}

impl From<FireState> for FireVisualState {
    fn from(state: FireState) -> Self {
        match state {
            FireState::Idle => Self::Idle,
            FireState::Cooking { .. } => Self::Cooking,
            FireState::Ready => Self::Ready,
        }
    }
}

#[derive(Clone, Copy)]
struct FireVisual {
    id: u32,
    position: [f32; 3],
    state: FireVisualState,
}

#[derive(Resource)]
pub(super) struct CampfireVisuals {
    snapshot: Vec<FireVisual>,
    roots: BTreeMap<u32, Entity>,
    sphere: Handle<Mesh>,
    log: Handle<Mesh>,
    flame: Handle<Mesh>,
    cube: Handle<Mesh>,
    ready_ring: Handle<Mesh>,
    stone: Handle<StandardMaterial>,
    bark: Handle<StandardMaterial>,
    steel: Handle<StandardMaterial>,
    orange: Handle<StandardMaterial>,
    yellow: Handle<StandardMaterial>,
    raw_fish: Handle<StandardMaterial>,
    cooked_fish: Handle<StandardMaterial>,
    ready: Handle<StandardMaterial>,
}

pub(super) fn setup(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    let surface = |color| StandardMaterial {
        base_color: color,
        perceptual_roughness: 0.95,
        ..default()
    };
    let glow = |color| StandardMaterial {
        base_color: color,
        unlit: true,
        ..default()
    };
    commands.insert_resource(CampfireVisuals {
        snapshot: Vec::new(),
        roots: BTreeMap::new(),
        sphere: meshes.add(Sphere::new(1.).mesh().uv(12, 8)),
        log: meshes.add(Cylinder::new(0.075, 0.92).mesh().resolution(12).build()),
        flame: meshes.add(Cone::new(0.24, 0.60).mesh().resolution(12).build()),
        cube: meshes.add(Cuboid::new(1., 1., 1.)),
        ready_ring: meshes.add(
            Torus::new(0.26, 0.30)
                .mesh()
                .major_resolution(24)
                .minor_resolution(6)
                .build(),
        ),
        stone: materials.add(surface(Color::srgb(0.38, 0.40, 0.42))),
        bark: materials.add(surface(Color::srgb(0.25, 0.13, 0.055))),
        steel: materials.add(surface(Color::srgb(0.12, 0.13, 0.14))),
        orange: materials.add(glow(Color::srgb(1., 0.24, 0.025))),
        yellow: materials.add(glow(Color::srgb(1., 0.80, 0.14))),
        raw_fish: materials.add(surface(Color::srgb(0.25, 0.48, 0.50))),
        cooked_fish: materials.add(surface(Color::srgb(0.72, 0.36, 0.12))),
        ready: materials.add(glow(Color::srgb(0.22, 1., 0.32))),
    });
}

pub(super) fn refresh(
    game: Res<GameSession>,
    mut commands: Commands,
    mut visuals: ResMut<CampfireVisuals>,
) {
    let fires = game.0.campfires();
    if fires.len() == visuals.snapshot.len()
        && fires.iter().zip(&visuals.snapshot).all(|(fire, previous)| {
            fire.id() == previous.id
                && fire.position() == previous.position
                && FireVisualState::from(fire.state()) == previous.state
        })
    {
        return;
    }
    visuals.roots.retain(|id, entity| {
        if fires.iter().any(|fire| fire.id() == *id) {
            true
        } else {
            commands.entity(*entity).despawn();
            false
        }
    });
    let snapshot: Vec<_> = fires
        .iter()
        .map(|fire| FireVisual {
            id: fire.id(),
            position: fire.position(),
            state: fire.state().into(),
        })
        .collect();
    for fire in &snapshot {
        if let Some(&entity) = visuals.roots.get(&fire.id) {
            let previous = visuals.snapshot.iter().find(|old| old.id == fire.id);
            if previous.is_some_and(|old| old.state == fire.state) {
                if previous.is_some_and(|old| old.position != fire.position) {
                    commands
                        .entity(entity)
                        .insert(Transform::from_translation(point(fire.position)));
                }
                continue;
            }
            commands.entity(entity).despawn();
        }
        let entity = spawn(&mut commands, &visuals, fire);
        visuals.roots.insert(fire.id, entity);
    }
    visuals.snapshot = snapshot;
}

fn spawn(commands: &mut Commands, visuals: &CampfireVisuals, fire: &FireVisual) -> Entity {
    commands
        .spawn((
            Transform::from_translation(point(fire.position)),
            Visibility::Inherited,
        ))
        .with_children(|campfire| {
            for index in 0..10 {
                let angle = index as f32 * TAU / 10.;
                campfire.spawn((
                    Mesh3d(visuals.sphere.clone()),
                    MeshMaterial3d(visuals.stone.clone()),
                    Transform::from_xyz(angle.cos() * 0.52, 0.10, angle.sin() * 0.52)
                        .with_scale(Vec3::new(0.14, 0.10, 0.12)),
                ));
            }
            for yaw in [-FRAC_PI_4, FRAC_PI_4] {
                campfire.spawn((
                    Mesh3d(visuals.log.clone()),
                    MeshMaterial3d(visuals.bark.clone()),
                    Transform::from_xyz(0., 0.09, 0.).with_rotation(
                        Quat::from_rotation_y(yaw) * Quat::from_rotation_z(FRAC_PI_2),
                    ),
                ));
            }
            campfire.spawn((
                Mesh3d(visuals.flame.clone()),
                MeshMaterial3d(visuals.orange.clone()),
                Transform::from_xyz(0., 0.42, 0.),
            ));
            for x in [-0.10, 0.10] {
                campfire.spawn((
                    Mesh3d(visuals.flame.clone()),
                    MeshMaterial3d(visuals.yellow.clone()),
                    Transform::from_xyz(x, 0.33, 0.04).with_scale(Vec3::new(0.40, 0.70, 0.40)),
                ));
            }
            campfire.spawn((
                PointLight {
                    color: Color::srgb(1., 0.48, 0.14),
                    intensity: 45000.,
                    range: 5.,
                    ..default()
                },
                Transform::from_xyz(0., 0.65, 0.),
            ));
            if fire.state != FireVisualState::Idle {
                for x in [-0.40, 0.40] {
                    campfire.spawn((
                        Mesh3d(visuals.cube.clone()),
                        MeshMaterial3d(visuals.steel.clone()),
                        Transform::from_xyz(x, 0.43, 0.).with_scale(Vec3::new(0.025, 0.86, 0.025)),
                    ));
                }
                campfire.spawn((
                    Mesh3d(visuals.cube.clone()),
                    MeshMaterial3d(visuals.steel.clone()),
                    Transform::from_xyz(0., 0.86, 0.).with_scale(Vec3::new(0.84, 0.025, 0.025)),
                ));
                let fish = if fire.state == FireVisualState::Ready {
                    &visuals.cooked_fish
                } else {
                    &visuals.raw_fish
                };
                campfire.spawn((
                    Mesh3d(visuals.sphere.clone()),
                    MeshMaterial3d(fish.clone()),
                    Transform::from_xyz(0., 0.87, 0.).with_scale(Vec3::new(0.22, 0.07, 0.09)),
                ));
                campfire.spawn((
                    Mesh3d(visuals.flame.clone()),
                    MeshMaterial3d(fish.clone()),
                    Transform::from_xyz(0.25, 0.87, 0.)
                        .with_rotation(Quat::from_rotation_z(FRAC_PI_2))
                        .with_scale(Vec3::new(0.28, 0.30, 0.28)),
                ));
                campfire.spawn((
                    Mesh3d(visuals.sphere.clone()),
                    MeshMaterial3d(visuals.steel.clone()),
                    Transform::from_xyz(-0.14, 0.90, -0.065).with_scale(Vec3::splat(0.012)),
                ));
            }
            if fire.state == FireVisualState::Ready {
                campfire.spawn((
                    Mesh3d(visuals.ready_ring.clone()),
                    MeshMaterial3d(visuals.ready.clone()),
                    Transform::from_xyz(0., 1.06, 0.),
                ));
            }
        })
        .id()
}

pub(super) fn apply(session: &mut Session) -> Result<String, String> {
    if session
        .campfire_in_reach()
        .is_some_and(|fire| matches!(fire.state(), FireState::Ready))
    {
        session.take_cooked_fish()?;
    } else {
        session.cook_fish()?;
    }
    Ok(session.message.clone())
}

pub(super) fn hint(session: &Session) -> Option<String> {
    let fire = session.campfire_in_reach()?;
    Some(match fire.state() {
        FireState::Idle => format!(
            "Campfire {} | G / Xbox D-pad Left cook 1 raw fish + {COOK_WOOD} wood",
            fire.id()
        ),
        FireState::Cooking { remaining } => format!(
            "Campfire {} | Cooking: {remaining:.1} s | Collect when ready",
            fire.id()
        ),
        FireState::Ready => format!(
            "Campfire {} | READY | G / Xbox D-pad Left take cooked fish",
            fire.id()
        ),
    })
}
