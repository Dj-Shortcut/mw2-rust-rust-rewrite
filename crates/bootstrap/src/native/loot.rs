use super::{GameSession, point};
use bevy::mesh::MeshBuilder;
use bevy::prelude::*;
use std::collections::BTreeMap;
use std::f32::consts::{FRAC_PI_2, PI};
use survival::MAX_LOOT_BAGS;

#[derive(Resource)]
pub(super) struct LootVisuals {
    snapshot: Vec<(u32, [f32; 3])>,
    roots: BTreeMap<u32, Entity>,
    body: Handle<Mesh>,
    strap: Handle<Mesh>,
    handle: Handle<Mesh>,
    canvas: Handle<StandardMaterial>,
    webbing: Handle<StandardMaterial>,
}

pub(super) fn setup(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    commands.insert_resource(LootVisuals {
        snapshot: Vec::new(),
        roots: BTreeMap::new(),
        body: meshes.add(Sphere::new(1.).mesh().uv(16, 12)),
        strap: meshes.add(
            Torus::new(0.23, 0.25)
                .mesh()
                .major_resolution(24)
                .minor_resolution(4)
                .build(),
        ),
        handle: meshes.add(
            Torus::new(0.06, 0.08)
                .mesh()
                .major_resolution(12)
                .minor_resolution(4)
                .angle_range(0. ..=PI)
                .build(),
        ),
        canvas: materials.add(StandardMaterial {
            base_color: Color::srgb(0.68, 0.50, 0.29),
            perceptual_roughness: 0.95,
            ..default()
        }),
        webbing: materials.add(StandardMaterial {
            base_color: Color::srgb(0.13, 0.11, 0.085),
            perceptual_roughness: 0.98,
            ..default()
        }),
    });
}

pub(super) fn refresh(
    mut commands: Commands,
    game: Res<GameSession>,
    mut visuals: ResMut<LootVisuals>,
) {
    let snapshot: Vec<_> = game
        .0
        .loot_bags()
        .iter()
        .take(MAX_LOOT_BAGS)
        .map(|bag| (bag.id(), bag.position()))
        .collect();
    if snapshot == visuals.snapshot {
        return;
    }
    visuals.roots.retain(|id, entity| {
        if snapshot.iter().any(|(current, _)| current == id) {
            true
        } else {
            commands.entity(*entity).despawn();
            false
        }
    });
    for &(id, position) in &snapshot {
        if let Some(&entity) = visuals.roots.get(&id) {
            let previous = visuals.snapshot.iter().find(|(current, _)| *current == id);
            if previous.is_none_or(|(_, old_position)| *old_position != position) {
                commands
                    .entity(entity)
                    .insert(Transform::from_translation(point(position)));
            }
            continue;
        }
        let root = commands
            .spawn((
                Transform::from_translation(point(position)),
                Visibility::Inherited,
            ))
            .with_children(|bag| {
                bag.spawn((
                    Mesh3d(visuals.body.clone()),
                    MeshMaterial3d(visuals.canvas.clone()),
                    Transform::from_xyz(0., 0.17, 0.).with_scale(Vec3::new(0.36, 0.17, 0.24)),
                ));
                for x in [-0.16, 0.16] {
                    bag.spawn((
                        Mesh3d(visuals.strap.clone()),
                        MeshMaterial3d(visuals.webbing.clone()),
                        Transform::from_xyz(x, 0.17, 0.)
                            .with_rotation(Quat::from_rotation_z(FRAC_PI_2))
                            .with_scale(Vec3::new(0.64, 2., 0.90)),
                    ));
                }
                bag.spawn((
                    Mesh3d(visuals.handle.clone()),
                    MeshMaterial3d(visuals.webbing.clone()),
                    Transform::from_xyz(0., 0.325, 0.)
                        .with_rotation(Quat::from_rotation_x(-FRAC_PI_2))
                        .with_scale(Vec3::new(1.4, 1., 0.8)),
                ));
            })
            .id();
        visuals.roots.insert(id, root);
    }
    visuals.snapshot = snapshot;
}
