use super::{GameSession, point};
use bevy::prelude::*;
use std::collections::BTreeMap;
use survival::{FREEZING_CELSIUS, HARVEST_FOOD, HARVEST_SEEDS, PlotState, RAIN_GROWTH, Session};

pub(super) fn apply(session: &mut Session) -> Result<String, String> {
    match session.plot_in_reach().map(|plot| plot.state()) {
        Some(PlotState::Growing { .. } | PlotState::Ripe) => session.harvest_plot()?,
        Some(PlotState::Empty) | None => session.plant_seeds()?,
    }
    Ok(session.message.clone())
}

pub(super) fn hint(session: &Session) -> Option<String> {
    let plot = session.plot_in_reach()?;
    let controls = "T / Xbox hold LB, press RB";
    Some(match plot.state() {
        PlotState::Empty => format!(
            "Garden bed {} | Empty\n{controls} plant (1 berry seed + 1 Water)",
            plot.id()
        ),
        PlotState::Growing { remaining } => {
            let clock = session.clock();
            let temperature = clock.temperature();
            let raining = session.weather().is_raining();
            let dawn = clock.is_dawn();
            let growth = if temperature < FREEZING_CELSIUS {
                "Frozen: growth paused".to_string()
            } else if raining && dawn {
                format!("Rain + dawn: {RAIN_GROWTH:.1}x")
            } else if raining {
                format!("Rain: {RAIN_GROWTH:.1}x")
            } else if dawn {
                format!("Dawn: {RAIN_GROWTH:.1}x")
            } else {
                "Dry: 1.0x".to_string()
            };
            format!(
                "Garden bed {} | Growing: {remaining:.1} simulation s | {growth}\nWorld {temperature:.1} C | {controls} harvest when ripe",
                plot.id()
            )
        }
        PlotState::Ripe => format!(
            "Garden bed {} | Ripe\n{controls} harvest ({HARVEST_FOOD} Food + {HARVEST_SEEDS} berry seeds)",
            plot.id()
        ),
    })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum GardenVisualState {
    Empty,
    Growing,
    Ripe,
}

impl From<PlotState> for GardenVisualState {
    fn from(state: PlotState) -> Self {
        match state {
            PlotState::Empty => Self::Empty,
            PlotState::Growing { .. } => Self::Growing,
            PlotState::Ripe => Self::Ripe,
        }
    }
}

#[derive(Clone, Copy)]
struct PlotSnapshot {
    id: u32,
    position: [f32; 3],
    state: GardenVisualState,
}

#[derive(Resource)]
pub(super) struct GardenVisuals {
    snapshot: Vec<PlotSnapshot>,
    roots: BTreeMap<u32, Entity>,
    cube: Handle<Mesh>,
    sphere: Handle<Mesh>,
    soil: Handle<StandardMaterial>,
    furrow: Handle<StandardMaterial>,
    wood: Handle<StandardMaterial>,
    stem: Handle<StandardMaterial>,
    leaf: Handle<StandardMaterial>,
    berry: Handle<StandardMaterial>,
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
    commands.insert_resource(GardenVisuals {
        snapshot: Vec::new(),
        roots: BTreeMap::new(),
        cube: meshes.add(Cuboid::new(1., 1., 1.)),
        sphere: meshes.add(Sphere::new(1.).mesh().uv(12, 8)),
        soil: materials.add(surface(Color::srgb(0.22, 0.13, 0.07))),
        furrow: materials.add(surface(Color::srgb(0.10, 0.055, 0.025))),
        wood: materials.add(surface(Color::srgb(0.40, 0.22, 0.10))),
        stem: materials.add(surface(Color::srgb(0.16, 0.36, 0.065))),
        leaf: materials.add(surface(Color::srgb(0.26, 0.62, 0.12))),
        berry: materials.add(StandardMaterial {
            base_color: Color::srgb(0.64, 0.12, 0.78),
            unlit: true,
            ..default()
        }),
    });
}

pub(super) fn refresh(
    game: Res<GameSession>,
    mut commands: Commands,
    mut visuals: ResMut<GardenVisuals>,
) {
    let plots = game.0.garden();
    if plots.len() == visuals.snapshot.len()
        && plots.iter().zip(&visuals.snapshot).all(|(plot, previous)| {
            plot.id() == previous.id
                && plot.position() == previous.position
                && GardenVisualState::from(plot.state()) == previous.state
        })
    {
        return;
    }
    visuals.roots.retain(|id, root| {
        if plots.iter().any(|plot| plot.id() == *id) {
            true
        } else {
            commands.entity(*root).despawn();
            false
        }
    });
    let snapshot: Vec<_> = plots
        .iter()
        .map(|plot| PlotSnapshot {
            id: plot.id(),
            position: plot.position(),
            state: plot.state().into(),
        })
        .collect();
    for plot in &snapshot {
        if let Some(&root) = visuals.roots.get(&plot.id) {
            if let Some(previous) = visuals.snapshot.iter().find(|old| old.id == plot.id)
                && previous.state == plot.state
            {
                if previous.position != plot.position {
                    commands
                        .entity(root)
                        .insert(Transform::from_translation(point(plot.position)));
                }
                continue;
            }
            commands.entity(root).despawn();
        }
        let root = spawn(&mut commands, &visuals, plot);
        visuals.roots.insert(plot.id, root);
    }
    visuals.snapshot = snapshot;
}

fn spawn(commands: &mut Commands, visuals: &GardenVisuals, plot: &PlotSnapshot) -> Entity {
    commands
        .spawn((
            Name::new(format!("Garden bed {}", plot.id)),
            Transform::from_translation(point(plot.position)),
            Visibility::Inherited,
        ))
        .with_children(|bed| {
            bed.spawn((
                Mesh3d(visuals.cube.clone()),
                MeshMaterial3d(visuals.soil.clone()),
                Transform::from_xyz(0., 0.05, 0.).with_scale(Vec3::new(1.60, 0.10, 1.10)),
            ));
            for z in [-0.60, 0.60] {
                bed.spawn((
                    Mesh3d(visuals.cube.clone()),
                    MeshMaterial3d(visuals.wood.clone()),
                    Transform::from_xyz(0., 0.10, z).with_scale(Vec3::new(1.80, 0.20, 0.10)),
                ));
            }
            for x in [-0.85, 0.85] {
                bed.spawn((
                    Mesh3d(visuals.cube.clone()),
                    MeshMaterial3d(visuals.wood.clone()),
                    Transform::from_xyz(x, 0.10, 0.).with_scale(Vec3::new(0.10, 0.20, 1.10)),
                ));
            }
            for x in [-0.50, 0., 0.50] {
                bed.spawn((
                    Mesh3d(visuals.cube.clone()),
                    MeshMaterial3d(visuals.furrow.clone()),
                    Transform::from_xyz(x, 0.106, 0.).with_scale(Vec3::new(0.035, 0.012, 0.90)),
                ));
            }
            if plot.state == GardenVisualState::Empty {
                return;
            }
            let ripe = plot.state == GardenVisualState::Ripe;
            for x in [-0.50, 0., 0.50] {
                bed.spawn((Transform::from_xyz(x, 0.10, 0.), Visibility::Inherited))
                    .with_children(|plant| {
                        let height = if ripe { 0.48 } else { 0.28 };
                        plant.spawn((
                            Mesh3d(visuals.cube.clone()),
                            MeshMaterial3d(visuals.stem.clone()),
                            Transform::from_xyz(0., height * 0.5, 0.)
                                .with_scale(Vec3::new(0.035, height, 0.035)),
                        ));
                        let width = if ripe { 0.23 } else { 0.15 };
                        for (side, y) in [(-1., 0.65), (1., 0.85)] {
                            plant.spawn((
                                Mesh3d(visuals.sphere.clone()),
                                MeshMaterial3d(visuals.leaf.clone()),
                                Transform::from_xyz(side * width * 0.55, height * y, 0.)
                                    .with_rotation(Quat::from_rotation_z(side * 0.32))
                                    .with_scale(Vec3::new(width, 0.065, width * 0.45)),
                            ));
                        }
                        if ripe {
                            for (x, y, z) in
                                [(-0.13, 0.36, 0.11), (0.13, 0.40, 0.11), (0., 0.49, -0.08)]
                            {
                                plant.spawn((
                                    Mesh3d(visuals.sphere.clone()),
                                    MeshMaterial3d(visuals.berry.clone()),
                                    Transform::from_xyz(x, y, z).with_scale(Vec3::splat(0.065)),
                                ));
                            }
                        }
                    });
            }
        })
        .id()
}
