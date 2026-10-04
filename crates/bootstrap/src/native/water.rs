use super::{GameSession, point};
use bevy::mesh::MeshBuilder;
use bevy::prelude::*;
use std::f32::consts::{FRAC_PI_2, TAU};
use survival::{BARREL_WATER, FREEZING_CELSIUS, ResourceKind, Session};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Action {
    Sip,
    Collect,
}

pub(super) fn apply(session: &mut Session, action: Action) -> Result<String, String> {
    match action {
        Action::Sip => session.drink_from_water()?,
        Action::Collect => session.empty_rain_barrel()?,
    }
    Ok(session.message.clone())
}

pub(super) fn hint(session: &Session) -> Option<String> {
    let frozen = session.clock().temperature() < FREEZING_CELSIUS;
    let mut hints = Vec::new();
    if let Some(node) = session
        .gather_target_from_view()
        .ok()
        .flatten()
        .filter(|node| node.kind == ResourceKind::Water)
    {
        let action = if frozen {
            "Frozen: cannot sip"
        } else {
            "P / Xbox RS click sip (+10 thirst)"
        };
        hints.push(format!("Water #{} | {action}", node.id));
    }
    if session.rain_barrel_in_reach() {
        let barrel = session.rain_barrel();
        let status = if barrel.water() == BARREL_WATER {
            "Full".to_string()
        } else if session.weather().is_raining() && !frozen {
            format!("Collecting: {:.1} s", barrel.remaining())
        } else {
            "Paused".to_string()
        };
        let action = if barrel.water() == 0 {
            "Empty"
        } else {
            "O / Xbox D-pad Up collect all"
        };
        hints.push(format!(
            "Rain barrel {}/{BARREL_WATER} | {status} | {action}",
            barrel.water()
        ));
    }
    (!hints.is_empty()).then(|| hints.join("\n"))
}

#[derive(Component)]
pub(super) struct BarrelRoot;

#[derive(Component)]
pub(super) struct BarrelWaterSurface {
    pub(super) count: u32,
}

#[derive(Component)]
pub(super) struct BarrelGaugeMark {
    pub(super) index: u32,
    pub(super) filled: bool,
}

#[derive(Resource)]
pub(super) struct BarrelVisuals {
    snapshot: Option<([f32; 3], u32)>,
    root: Option<Entity>,
    stave: Handle<Mesh>,
    base: Handle<Mesh>,
    water: Handle<Mesh>,
    band: Handle<Mesh>,
    gauge: Handle<Mesh>,
    wood: Handle<StandardMaterial>,
    metal: Handle<StandardMaterial>,
    blue: Handle<StandardMaterial>,
    empty: Handle<StandardMaterial>,
}

pub(super) fn setup(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    let surface = |color| StandardMaterial {
        base_color: color,
        perceptual_roughness: 0.9,
        ..default()
    };
    commands.insert_resource(BarrelVisuals {
        snapshot: None,
        root: None,
        stave: meshes.add(Cuboid::new(0.122, 0.78, 0.045)),
        base: meshes.add(Cylinder::new(0.345, 0.06).mesh().resolution(24).build()),
        water: meshes.add(Cylinder::new(0.295, 0.018).mesh().resolution(24).build()),
        band: meshes.add(
            Torus::new(0.315, 0.36)
                .mesh()
                .major_resolution(32)
                .minor_resolution(6)
                .build(),
        ),
        gauge: meshes.add(Cuboid::new(0.055, 0.055, 0.015)),
        wood: materials.add(surface(Color::srgb(0.34, 0.19, 0.075))),
        metal: materials.add(surface(Color::srgb(0.16, 0.18, 0.19))),
        blue: materials.add(StandardMaterial {
            base_color: Color::srgb(0.10, 0.64, 0.94),
            unlit: true,
            ..default()
        }),
        empty: materials.add(surface(Color::srgb(0.08, 0.09, 0.10))),
    });
}

pub(super) fn refresh(
    game: Res<GameSession>,
    mut commands: Commands,
    mut visuals: ResMut<BarrelVisuals>,
) {
    let barrel = game.0.rain_barrel();
    let snapshot = (barrel.position(), barrel.water());
    if visuals.snapshot == Some(snapshot) {
        return;
    }
    if let Some(root) = visuals.root.take() {
        commands.entity(root).despawn();
    }
    let root = commands
        .spawn((
            BarrelRoot,
            Transform::from_translation(point(snapshot.0)),
            Visibility::Inherited,
        ))
        .with_children(|barrel| {
            barrel.spawn((
                Mesh3d(visuals.base.clone()),
                MeshMaterial3d(visuals.wood.clone()),
                Transform::from_xyz(0., 0.03, 0.),
            ));
            for index in 0..16 {
                let angle = index as f32 * TAU / 16.;
                barrel.spawn((
                    Mesh3d(visuals.stave.clone()),
                    MeshMaterial3d(visuals.wood.clone()),
                    Transform::from_xyz(angle.cos() * 0.325, 0.43, angle.sin() * 0.325)
                        .with_rotation(Quat::from_rotation_y(FRAC_PI_2 - angle)),
                ));
            }
            for height in [0.12, 0.67, 0.82] {
                barrel.spawn((
                    Mesh3d(visuals.band.clone()),
                    MeshMaterial3d(visuals.metal.clone()),
                    Transform::from_xyz(0., height, 0.),
                ));
            }
            if snapshot.1 > 0 {
                let surface = BarrelWaterSurface { count: snapshot.1 };
                let height = 0.10 + 0.60 * surface.count as f32 / BARREL_WATER as f32;
                barrel.spawn((
                    surface,
                    Mesh3d(visuals.water.clone()),
                    MeshMaterial3d(visuals.blue.clone()),
                    Transform::from_xyz(0., height, 0.),
                ));
            }
            for side in 0..4 {
                let angle = side as f32 * TAU / 4.;
                for index in 0..BARREL_WATER {
                    let mark = BarrelGaugeMark {
                        index,
                        filled: index < snapshot.1,
                    };
                    let height = 0.20 + mark.index as f32 * 0.11;
                    let material = if mark.filled {
                        &visuals.blue
                    } else {
                        &visuals.empty
                    };
                    barrel.spawn((
                        mark,
                        Mesh3d(visuals.gauge.clone()),
                        MeshMaterial3d(material.clone()),
                        Transform::from_xyz(angle.cos() * 0.36, height, angle.sin() * 0.36)
                            .with_rotation(Quat::from_rotation_y(FRAC_PI_2 - angle)),
                    ));
                }
            }
        })
        .id();
    visuals.root = Some(root);
    visuals.snapshot = Some(snapshot);
}
