use bevy::asset::RenderAssetUsages;
use bevy::asset::{LoadState, RecursiveDependencyLoadState};
use bevy::input::gamepad::{Gamepad, GamepadButton};
use bevy::input::mouse::AccumulatedMouseMotion;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;
use bevy::window::{CursorGrabMode, CursorOptions, PrimaryWindow};
use playerstate_iw4::{UserCmd, buttons};
use rust_building::{Grade, Kind, Piece};
use std::path::{Path, PathBuf};
use survival::{
    Item, LOCAL, PlacedObject, PropKind, Recipe, ResourceNode, Session, UNITS_TO_METERS,
};

mod building_actions;
mod campfires;
mod clothing;
mod connected;
mod fishing;
mod garden;
mod inventory;
mod loot;
mod object_actions;
mod queued_crafting;
mod research;
mod tool_actions;
mod water;
pub use connected::run as run_connected;
use inventory::{Action as InventoryAction, InventoryUi};

#[derive(Resource)]
struct GameSession(Session);

#[derive(Resource, Default)]
struct Controls {
    pitch: f32,
    yaw: f32,
    paused: bool,
    focused: bool,
    building: bool,
    editor: bool,
    selection: usize,
    prop_selection: usize,
    prop_yaw: f32,
    axis: u8,
    command: UserCmd,
    error: Option<String>,
    pad: Option<Entity>,
    inventory_open: bool,
    recipe: usize,
    inventory: InventoryUi,
    capture_frames: u8,
    captured: bool,
    help: bool,
    actions: Vec<WorldAction>,
}

enum WorldAction {
    Gather,
    PickUpLoot,
    OpenCrate,
    OpenSupplyDrop,
    Fishing,
    Campfire,
    Water(water::Action),
    Garden,
    PlaceProp(PropKind, f32),
    RemoveProp,
    EditProp(object_actions::Action),
    Undo,
    Redo,
    PlaceBuilding(Kind, u8),
    MaintainBuilding(building_actions::Action),
    Door,
    Save,
    Load,
    Respawn,
}

impl Controls {
    fn queue(&mut self, action: WorldAction) {
        if self.actions.len() < 16 {
            self.actions.push(action);
        }
    }

    fn neutral(&mut self) {
        self.command.buttons = 0;
        self.command.forwardmove = 0;
        self.command.rightmove = 0;
    }
}

#[derive(Resource, Default)]
struct WeaponFeedback {
    pitch: f32,
    yaw: f32,
    back: f32,
    hit: f32,
}

#[derive(Resource)]
struct NativeSounds {
    shot: Handle<AudioSource>,
    reload: Handle<AudioSource>,
    footstep: Handle<AudioSource>,
    ollie: Handle<AudioSource>,
    land: Handle<AudioSource>,
    gather: Handle<AudioSource>,
    ui: Handle<AudioSource>,
    step_distance: f32,
}

fn sound(commands: &mut Commands, handle: &Handle<AudioSource>) {
    commands.spawn((AudioPlayer::new(handle.clone()), PlaybackSettings::DESPAWN));
}

#[derive(Resource)]
struct NativeModels {
    scenes: Vec<(&'static str, Handle<WorldAsset>)>,
    status: String,
}

#[derive(Resource)]
struct BuildVisuals {
    pieces: Vec<Piece>,
    anchor: Option<[f32; 3]>,
    entities: Vec<Entity>,
    cube: Handle<Mesh>,
    materials: [Handle<StandardMaterial>; 3],
}

#[derive(Resource)]
struct EditorVisuals {
    objects: Vec<PlacedObject>,
    entities: Vec<Entity>,
    templates: Vec<(PropKind, Handle<Mesh>, Handle<StandardMaterial>)>,
}

#[derive(Clone, PartialEq)]
enum PlacementGhost {
    Prop(PlacedObject, bool),
    Building(Vec<([f32; 3], [f32; 3])>, bool),
}

#[derive(Resource)]
struct PlacementVisuals {
    ghost: Option<PlacementGhost>,
    entities: Vec<Entity>,
    cube: Handle<Mesh>,
    valid: Handle<StandardMaterial>,
    invalid: Handle<StandardMaterial>,
    status: String,
}

#[derive(Component)]
struct PlayerCamera;

#[derive(Component)]
struct Operator;

#[derive(Component)]
struct ViewWeapon;

#[derive(Component)]
struct StatusText;

#[derive(Component)]
struct Crosshair;

#[derive(Component)]
struct InventoryText;

#[derive(Component)]
struct LocalSkater;

#[derive(Component)]
struct RideBoard;

#[derive(Resource)]
struct GatheringVisuals {
    nodes: Vec<ResourceNode>,
    entities: Vec<Entity>,
    cube: Handle<Mesh>,
    sphere: Handle<Mesh>,
    materials: Vec<(
        survival::ResourceKind,
        Handle<StandardMaterial>,
        Option<Handle<StandardMaterial>>,
    )>,
}

const KINDS: [Kind; 4] = [Kind::Foundation, Kind::Floor, Kind::Wall, Kind::Doorway];
const SAVE_PATH: &str = "iw4l-artifacts/survival/base.json";

pub fn run() -> Result<(), String> {
    let assets = asset_directory()?;
    let mut session = Session::new()?;
    session.message = "Gather resources, build, or create a skate park.".into();
    let mut app = App::new();
    app.insert_resource(GameSession(session))
        .init_resource::<Controls>()
        .init_resource::<WeaponFeedback>()
        .insert_resource(Time::<Fixed>::from_seconds(0.017))
        .insert_resource(ClearColor(Color::srgb(0.53, 0.68, 0.79)))
        .add_plugins(
            DefaultPlugins
                .set(bevy::asset::AssetPlugin {
                    file_path: assets.to_string_lossy().into_owned(),
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Survival / FPS / Skate".into(),
                        resolution: (1280, 720).into(),
                        ..default()
                    }),
                    ..default()
                }),
        )
        .insert_resource(GlobalAmbientLight {
            color: Color::srgb(0.72, 0.80, 0.90),
            brightness: 180.,
            ..default()
        })
        .add_systems(Startup, setup)
        .add_systems(PreUpdate, input.after(bevy::input::InputSystems))
        .add_systems(FixedUpdate, advance)
        .add_systems(
            Update,
            (
                models_status,
                present,
                refresh_buildings,
                refresh_props,
                refresh_gathering,
                loot::refresh,
                campfires::refresh,
                water::refresh,
                garden::refresh,
                present_skate,
                hit_feedback,
                refresh_placement,
                update_hud,
                update_inventory,
            )
                .chain(),
        );
    app.run();
    Ok(())
}

fn asset_directory() -> Result<PathBuf, String> {
    let mut candidates = Vec::new();
    if let Some(root) = std::env::var_os("SURVIVAL_ASSETS") {
        let root = PathBuf::from(root);
        if ["masked_operator", "carbine", "skateboard"]
            .iter()
            .all(|name| root.join("authored").join(format!("{name}.glb")).is_file())
        {
            return root.canonicalize().map_err(|e| e.to_string());
        }
        return Err(format!(
            "SURVIVAL_ASSETS={} does not contain all required authored/*.glb models",
            root.display()
        ));
    }
    if let Ok(current) = std::env::current_dir() {
        candidates.push(current.join("assets"));
    }
    if let Ok(exe) = std::env::current_exe()
        && let Some(parent) = exe.parent()
    {
        candidates.push(parent.join("assets"));
    }
    candidates.push(Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets"));
    for root in candidates {
        if ["masked_operator", "carbine", "skateboard"]
            .iter()
            .all(|name| root.join("authored").join(format!("{name}.glb")).is_file())
        {
            return root.canonicalize().map_err(|e| e.to_string());
        }
    }
    Err("Game content is missing: place assets/authored/{masked_operator,carbine,skateboard}.glb next to the executable, or set SURVIVAL_ASSETS.".into())
}

fn point(position: [f32; 3]) -> Vec3 {
    Vec3::new(position[0], position[2], -position[1]) * UNITS_TO_METERS
}

fn surface_mesh(
    positions: &[[f32; 3]],
    normals: &[[f32; 3]],
    indices: &[u32],
    colors: Option<&[[f32; 4]]>,
) -> Mesh {
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );
    mesh.insert_attribute(
        Mesh::ATTRIBUTE_POSITION,
        positions
            .iter()
            .map(|p| point(*p).to_array())
            .collect::<Vec<_>>(),
    );
    mesh.insert_attribute(
        Mesh::ATTRIBUTE_NORMAL,
        normals
            .iter()
            .map(|n| [n[0], n[2], -n[1]])
            .collect::<Vec<_>>(),
    );
    if let Some(colors) = colors {
        mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors.to_vec());
    }
    mesh.insert_indices(Indices::U32(indices.to_vec()));
    mesh
}

fn setup(
    mut commands: Commands,
    server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    game: Res<GameSession>,
) {
    commands.insert_resource(NativeSounds {
        shot: server.load("authored/audio/carbine_shot.wav"),
        reload: server.load("authored/audio/reload.wav"),
        footstep: server.load("authored/audio/footstep.wav"),
        ollie: server.load("authored/audio/ollie.wav"),
        land: server.load("authored/audio/land.wav"),
        gather: server.load("authored/audio/gather.wav"),
        ui: server.load("authored/audio/ui_click.wav"),
        step_distance: 0.,
    });
    let operator: Handle<WorldAsset> = server.load("authored/masked_operator.glb#Scene0");
    let rifle: Handle<WorldAsset> = server.load("authored/carbine.glb#Scene0");
    let board: Handle<WorldAsset> = server.load("authored/skateboard.glb#Scene0");
    commands.insert_resource(NativeModels {
        scenes: vec![
            ("Operator", operator.clone()),
            ("Carbine", rifle.clone()),
            ("Skateboard", board.clone()),
        ],
        status: "Loading models...".into(),
    });
    commands
        .spawn((
            PlayerCamera,
            Camera3d::default(),
            bevy::core_pipeline::tonemapping::Tonemapping::AcesFitted,
            Projection::Perspective(PerspectiveProjection {
                fov: 75f32.to_radians(),
                near: 0.03,
                far: 1000.,
                ..default()
            }),
            Transform::from_xyz(0., 1.6, 0.).looking_to(Vec3::X, Vec3::Y),
        ))
        .with_children(|camera| {
            camera.spawn((
                ViewWeapon,
                WorldAssetRoot(rifle),
                Transform::from_xyz(0.24, -0.24, -0.48)
                    .with_rotation(Quat::from_rotation_y(std::f32::consts::PI))
                    .with_scale(Vec3::splat(0.65)),
            ));
        });
    commands.spawn((
        Operator,
        WorldAssetRoot(operator.clone()),
        Transform::from_translation(point([550., 0., 1.]))
            .with_rotation(Quat::from_rotation_y(-std::f32::consts::FRAC_PI_2)),
    ));
    commands.spawn((
        WorldAssetRoot(board.clone()),
        Transform::from_translation(point([180., -180., 6.])),
    ));
    commands.spawn((
        LocalSkater,
        WorldAssetRoot(operator),
        Transform::default(),
        Visibility::Hidden,
    ));
    commands.spawn((
        RideBoard,
        WorldAssetRoot(board),
        Transform::default(),
        Visibility::Hidden,
    ));
    let mut node_materials = Vec::new();
    for kind in [
        survival::ResourceKind::Tree,
        survival::ResourceKind::Stone,
        survival::ResourceKind::Metal,
        survival::ResourceKind::Berry,
        survival::ResourceKind::Water,
        survival::ResourceKind::Hemp,
    ] {
        let visual = kind.visual();
        let material = materials.add(StandardMaterial {
            base_color: Color::srgba(
                visual.color[0],
                visual.color[1],
                visual.color[2],
                visual.color[3],
            ),
            perceptual_roughness: 0.9,
            ..default()
        });
        let canopy = visual.canopy.map(|c| {
            materials.add(StandardMaterial {
                base_color: Color::srgba(c.color[0], c.color[1], c.color[2], c.color[3]),
                perceptual_roughness: 0.95,
                ..default()
            })
        });
        node_materials.push((kind, material, canopy));
    }
    commands.insert_resource(GatheringVisuals {
        nodes: Vec::new(),
        entities: Vec::new(),
        cube: meshes.add(Cuboid::new(1., 1., 1.)),
        sphere: meshes.add(Sphere::new(1.).mesh().uv(16, 12)),
        materials: node_materials,
    });
    loot::setup(&mut commands, &mut meshes, &mut materials);
    campfires::setup(&mut commands, &mut meshes, &mut materials);
    water::setup(&mut commands, &mut meshes, &mut materials);
    garden::setup(&mut commands, &mut meshes, &mut materials);
    let terrain = &game.0.terrain;
    commands.spawn((
        Mesh3d(meshes.add(surface_mesh(
            &terrain.positions,
            &terrain.normals,
            &terrain.indices,
            Some(&terrain.colors),
        ))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::WHITE,
            perceptual_roughness: 0.95,
            ..default()
        })),
        Transform::default(),
    ));
    let mut templates = Vec::new();
    for kind in PropKind::ALL {
        let geometry = kind.geometry();
        let material = if kind == PropKind::Rail {
            StandardMaterial {
                base_color: Color::srgb(0.37, 0.42, 0.44),
                metallic: 0.9,
                perceptual_roughness: 0.3,
                ..default()
            }
        } else {
            StandardMaterial {
                base_color: Color::srgb(0.46, 0.25, 0.10),
                perceptual_roughness: 0.9,
                ..default()
            }
        };
        templates.push((
            kind,
            meshes.add(surface_mesh(
                &geometry.positions,
                &geometry.normals,
                &geometry.indices,
                None,
            )),
            materials.add(material),
        ));
    }
    commands.insert_resource(EditorVisuals {
        objects: Vec::new(),
        entities: Vec::new(),
        templates,
    });
    let ghost_material = |color| StandardMaterial {
        base_color: color,
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        cull_mode: None,
        ..default()
    };
    commands.insert_resource(PlacementVisuals {
        ghost: None,
        entities: Vec::new(),
        cube: meshes.add(Cuboid::new(1., 1., 1.)),
        valid: materials.add(ghost_material(Color::srgba(0.15, 1., 0.45, 0.42))),
        invalid: materials.add(ghost_material(Color::srgba(1., 0.15, 0.12, 0.42))),
        status: String::new(),
    });
    commands.spawn((
        DirectionalLight {
            illuminance: 12000.,
            shadow_maps_enabled: true,
            ..default()
        },
        Transform::from_xyz(20., 30., 10.).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    commands.insert_resource(BuildVisuals {
        pieces: Vec::new(),
        anchor: None,
        entities: Vec::new(),
        cube: meshes.add(Cuboid::new(1., 1., 1.)),
        materials: [
            materials.add(StandardMaterial {
                base_color: Color::srgb(0.42, 0.23, 0.10),
                perceptual_roughness: 0.9,
                ..default()
            }),
            materials.add(StandardMaterial {
                base_color: Color::srgb(0.42, 0.44, 0.43),
                perceptual_roughness: 0.95,
                ..default()
            }),
            materials.add(StandardMaterial {
                base_color: Color::srgb(0.30, 0.35, 0.39),
                metallic: 0.85,
                perceptual_roughness: 0.4,
                ..default()
            }),
        ],
    });
    commands.spawn((
        StatusText,
        Text::new(""),
        TextFont {
            font_size: FontSize::Px(18.),
            ..default()
        },
        TextColor(Color::WHITE),
        BackgroundColor(Color::srgba(0.015, 0.022, 0.03, 0.85)),
        Node {
            position_type: PositionType::Absolute,
            left: px(16),
            top: px(16),
            padding: UiRect::all(px(12)),
            ..default()
        },
    ));
    commands.spawn((
        InventoryText,
        Text::new(""),
        TextFont {
            font_size: FontSize::Px(16.),
            ..default()
        },
        TextColor(Color::WHITE),
        BackgroundColor(Color::srgba(0.015, 0.022, 0.03, 0.97)),
        Node {
            position_type: PositionType::Absolute,
            right: px(20),
            top: px(20),
            padding: UiRect::all(px(20)),
            display: Display::None,
            ..default()
        },
    ));
    commands.spawn((
        Crosshair,
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
            ..default()
        },
    ));
}

fn input(
    mut commands: Commands,
    sounds: Res<NativeSounds>,
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    motion: Res<AccumulatedMouseMotion>,
    time: Res<Time>,
    pads: Query<(Entity, &Gamepad)>,
    mut windows: Query<(&Window, &mut CursorOptions), With<PrimaryWindow>>,
    mut controls: ResMut<Controls>,
    mut game: ResMut<GameSession>,
) {
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
        return;
    };
    let mut inventory_changed = false;
    if keys.just_pressed(KeyCode::Escape) || pad_just_pressed(GamepadButton::Start) {
        if controls.inventory_open {
            if controls.inventory.pending() {
                game.0.message = "Stack action canceled".into();
            }
            controls.inventory.reset();
            controls.inventory_open = false;
            inventory_changed = true;
        } else {
            controls.paused = !controls.paused;
        }
    }
    if window.focused
        && (keys.just_pressed(KeyCode::Tab)
            || (!controls.inventory_open
                && !controls.building
                && !controls.editor
                && pad_just_pressed(GamepadButton::DPadDown)))
    {
        controls.inventory_open = !controls.inventory_open;
        inventory_changed = true;
        sound(&mut commands, &sounds.ui);
    }
    if window.focused
        && controls.inventory_open
        && (keys.just_pressed(KeyCode::Backspace) || pad_just_pressed(GamepadButton::East))
    {
        if controls.inventory.pending() {
            controls
                .inventory
                .apply(InventoryAction::Cancel, &mut game.0);
        } else {
            controls.inventory_open = false;
        }
        inventory_changed = true;
    }
    if window.focused && keys.just_pressed(KeyCode::F1) {
        controls.help = !controls.help;
    }
    controls.focused = window.focused;
    if !controls.inventory_open || !window.focused || controls.paused || controls.error.is_some() {
        if controls.inventory.pending() {
            game.0.message = "Stack action canceled".into();
        }
        controls.inventory.reset();
    } else {
        controls.inventory.sync(&game.0);
    }
    let active =
        window.focused && !controls.paused && !controls.inventory_open && controls.error.is_none();
    if window.focused
        && controls.inventory_open
        && !controls.paused
        && !inventory_changed
        && controls.error.is_none()
    {
        inventory_input(&keys, pad, &mut controls, &mut game.0);
    }
    cursor.visible = !active;
    let reacquired = active && !controls.captured;
    controls.captured = active;
    if reacquired {
        controls.capture_frames = 2;
    }
    cursor.grab_mode = if active {
        CursorGrabMode::Locked
    } else {
        CursorGrabMode::None
    };
    if !active || controls.capture_frames > 0 {
        controls.capture_frames = controls.capture_frames.saturating_sub(1);
        controls.neutral();
        controls.actions.clear();
        game.0.skate_input = survival::SkateInput::default();
        return;
    }
    if keys.just_pressed(KeyCode::F5) {
        controls.queue(WorldAction::Save);
    }
    if keys.just_pressed(KeyCode::F9) {
        controls.queue(WorldAction::Load);
    }
    if game.0.world.player(LOCAL).is_none_or(|p| p.health <= 0) {
        controls.neutral();
        if keys.just_pressed(KeyCode::Enter) || pad_just_pressed(GamepadButton::South) {
            controls.queue(WorldAction::Respawn);
        }
        return;
    }
    let skate_chord = pad_just_pressed(GamepadButton::North)
        && (pad_pressed(GamepadButton::LeftTrigger) || pad_pressed(GamepadButton::RightTrigger));
    if keys.just_pressed(KeyCode::KeyV) || skate_chord {
        let result = game.0.toggle_skate();
        if let Err(error) = result {
            game.0.message = error;
        }
        if game.0.skate.is_some() {
            controls.building = false;
            controls.editor = false;
        }
    }
    survival_shortcuts(&keys, &mut game.0, controls.recipe);
    let look = pad.map_or(Vec2::ZERO, |pad| filtered_stick(pad.right_stick()));
    let ads = game
        .0
        .world
        .player(LOCAL)
        .map_or(0., |player| player.f_weapon_pos_frac.clamp(0., 1.));
    let sensitivity = 1. - 0.35 * ads;
    controls.yaw =
        (controls.yaw - motion.delta.x * 0.1 - look.x * 220. * time.delta_secs() * sensitivity)
            .rem_euclid(360.);
    let pitch_delta = game.0.world.player(LOCAL).map_or(0., |p| p.delta_angles[0]);
    controls.pitch = (controls.pitch + pitch_delta + motion.delta.y * 0.1
        - look.y * 150. * time.delta_secs() * sensitivity)
        .clamp(-89., 89.)
        - pitch_delta;
    if pad_just_pressed(GamepadButton::Select) {
        if game.0.skate.is_some() {
            let _ = game.0.toggle_skate();
        }
        if controls.building {
            controls.building = false;
            controls.editor = true;
        } else if controls.editor {
            controls.editor = false;
        } else {
            controls.building = true;
        }
    }
    if keys.just_pressed(KeyCode::KeyB) {
        if game.0.skate.is_some() {
            let _ = game.0.toggle_skate();
        }
        controls.building = !controls.building;
        controls.editor = false;
    }
    if keys.just_pressed(KeyCode::KeyE) && game.0.skate.is_none() {
        controls.editor = !controls.editor;
        controls.building = false;
    }
    for (index, key) in [
        KeyCode::Digit1,
        KeyCode::Digit2,
        KeyCode::Digit3,
        KeyCode::Digit4,
        KeyCode::Digit5,
        KeyCode::Digit6,
    ]
    .into_iter()
    .enumerate()
    {
        if controls.building && index < 4 && keys.just_pressed(key) {
            controls.selection = index;
        }
        if controls.editor && keys.just_pressed(key) {
            controls.prop_selection = index;
        }
    }
    let previous = pad_just_pressed(GamepadButton::DPadLeft)
        || (!controls.building && pad_just_pressed(GamepadButton::DPadUp));
    let next = pad_just_pressed(GamepadButton::DPadRight)
        || (!controls.building && pad_just_pressed(GamepadButton::DPadDown));
    if previous != next {
        if controls.building {
            controls.selection = (controls.selection + if next { 1 } else { 3 }) % 4;
        }
        if controls.editor {
            controls.prop_selection = (controls.prop_selection + if next { 1 } else { 5 }) % 6;
        }
    }
    let rotate_left = pad_just_pressed(GamepadButton::LeftTrigger);
    let rotate_right = pad_just_pressed(GamepadButton::RightTrigger);
    if controls.building && (keys.just_pressed(KeyCode::KeyR) || rotate_left != rotate_right) {
        controls.axis ^= 1;
    }
    if controls.editor {
        let object_rotation = pad_pressed(GamepadButton::LeftTrigger2);
        if keys.just_pressed(KeyCode::KeyQ) || (rotate_left && !object_rotation) {
            controls.prop_yaw = (controls.prop_yaw - 15.).rem_euclid(360.);
        }
        if keys.just_pressed(KeyCode::KeyR) || (rotate_right && !object_rotation) {
            controls.prop_yaw = (controls.prop_yaw + 15.).rem_euclid(360.);
        }
        let control = keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight);
        if control && keys.just_pressed(KeyCode::KeyZ) {
            controls.queue(WorldAction::Undo);
        }
        if control && keys.just_pressed(KeyCode::KeyY) {
            controls.queue(WorldAction::Redo);
        }
        if mouse.just_pressed(MouseButton::Left) || pad_just_pressed(GamepadButton::RightTrigger2) {
            let action =
                WorldAction::PlaceProp(PropKind::ALL[controls.prop_selection], controls.prop_yaw);
            controls.queue(action);
        }
        if mouse.just_pressed(MouseButton::Right) || pad_just_pressed(GamepadButton::West) {
            controls.queue(WorldAction::RemoveProp);
        }
    }
    if can_edit(&controls, &game.0) {
        let object_rotation = pad_pressed(GamepadButton::LeftTrigger2) && !skate_chord;
        let mut requests = [
            (
                keys.just_pressed(KeyCode::KeyM)
                    || (pad_just_pressed(GamepadButton::North) && !skate_chord),
                object_actions::Action::Move,
            ),
            (
                keys.just_pressed(KeyCode::Comma) || (object_rotation && rotate_left),
                object_actions::Action::Rotate(-15.),
            ),
            (
                keys.just_pressed(KeyCode::Period) || (object_rotation && rotate_right),
                object_actions::Action::Rotate(15.),
            ),
        ]
        .into_iter()
        .filter_map(|(pressed, action)| pressed.then_some(action));
        if let Some(action) = requests.next()
            && requests.next().is_none()
        {
            controls.queue(WorldAction::EditProp(action));
        }
    }
    if controls.building
        && (mouse.just_pressed(MouseButton::Left) || pad_just_pressed(GamepadButton::RightTrigger2))
    {
        let action = WorldAction::PlaceBuilding(KINDS[controls.selection], controls.axis);
        controls.queue(action);
    }
    if controls.building
        && (mouse.just_pressed(MouseButton::Right) || pad_just_pressed(GamepadButton::West))
    {
        controls.queue(WorldAction::Door);
    }
    if can_maintain(&controls, &game.0) {
        let mut requests = [
            (
                keys.just_pressed(KeyCode::KeyT)
                    || (pad_just_pressed(GamepadButton::North) && !skate_chord),
                building_actions::Action::Repair,
            ),
            (
                keys.just_pressed(KeyCode::KeyZ) || pad_just_pressed(GamepadButton::DPadUp),
                building_actions::Action::Upgrade(Grade::Stone),
            ),
            (
                keys.just_pressed(KeyCode::KeyX) || pad_just_pressed(GamepadButton::DPadDown),
                building_actions::Action::Upgrade(Grade::Metal),
            ),
        ]
        .into_iter()
        .filter_map(|(pressed, action)| pressed.then_some(action));
        if let Some(action) = requests.next()
            && requests.next().is_none()
        {
            controls.queue(WorldAction::MaintainBuilding(action));
        }
    }
    if can_interact(&controls, &game.0)
        && (keys.just_pressed(KeyCode::KeyF)
            || (pad_just_pressed(GamepadButton::North) && !skate_chord))
    {
        controls.queue(interaction_action(&game.0));
    }
    if can_interact(&controls, &game.0) {
        if keys.just_pressed(KeyCode::KeyT)
            || (!skate_chord
                && pad_pressed(GamepadButton::LeftTrigger)
                && pad_just_pressed(GamepadButton::RightTrigger))
        {
            controls.queue(WorldAction::Garden);
        }
        let sip = keys.just_pressed(KeyCode::KeyP) || pad_just_pressed(GamepadButton::RightThumb);
        let collect = keys.just_pressed(KeyCode::KeyO) || pad_just_pressed(GamepadButton::DPadUp);
        if sip && collect {
            game.0.message = "Choose one water action (P or O)".into();
        } else if sip || collect {
            controls.queue(WorldAction::Water(if sip {
                water::Action::Sip
            } else {
                water::Action::Collect
            }));
        }
        let fish = keys.just_pressed(KeyCode::KeyL) || pad_just_pressed(GamepadButton::DPadRight);
        let cook = keys.just_pressed(KeyCode::KeyG) || pad_just_pressed(GamepadButton::DPadLeft);
        if fish != cook {
            controls.queue(if fish {
                WorldAction::Fishing
            } else {
                WorldAction::Campfire
            });
        }
    }
    let held = |key| i8::from(keys.pressed(key));
    let movement = pad.map_or(Vec2::ZERO, |pad| filtered_stick(pad.left_stick()));
    let mut cmd = UserCmd {
        forwardmove: ((f32::from(held(KeyCode::KeyW) - held(KeyCode::KeyS)) + movement.y)
            .clamp(-1., 1.)
            * 127.)
            .round() as i8,
        rightmove: ((f32::from(held(KeyCode::KeyD) - held(KeyCode::KeyA)) + movement.x)
            .clamp(-1., 1.)
            * 127.)
            .round() as i8,
        angles: [controls.pitch, controls.yaw, 0.].map(|a| (a * 65536. / 360.).round() as i32),
        weapon: game.0.world.player(LOCAL).map_or(0, |p| p.weapon as u16),
        ..default()
    };
    for (key, flag) in [
        (KeyCode::ShiftLeft, buttons::SPRINT),
        (KeyCode::Space, buttons::JUMP),
        (KeyCode::ControlLeft, buttons::CROUCH),
    ] {
        let undo_shortcut =
            controls.editor && (keys.pressed(KeyCode::KeyZ) || keys.pressed(KeyCode::KeyY));
        if keys.pressed(key) && !(flag == buttons::CROUCH && undo_shortcut) {
            cmd.buttons |= flag;
        }
    }
    for (button, flag) in [
        (GamepadButton::South, buttons::JUMP),
        (GamepadButton::LeftThumb, buttons::SPRINT),
        (GamepadButton::East, buttons::CROUCH),
    ] {
        if pad_pressed(button) {
            cmd.buttons |= flag;
        }
    }
    if !controls.building && !controls.editor && game.0.skate.is_none() {
        if keys.pressed(KeyCode::KeyR) || pad_pressed(GamepadButton::West) {
            cmd.buttons |= buttons::RELOAD;
        }
        if mouse.pressed(MouseButton::Left) || pad_pressed(GamepadButton::RightTrigger2) {
            cmd.buttons |= buttons::ATTACK;
        }
        if mouse.pressed(MouseButton::Right) || pad_pressed(GamepadButton::LeftTrigger2) {
            cmd.buttons |= buttons::ADS;
        }
    }
    if game.0.skate.is_some() {
        game.0.skate_input.push = (f32::from(held(KeyCode::KeyW))
            + movement.y.max(0.)
            + f32::from(u8::from(pad_pressed(GamepadButton::RightTrigger2))))
        .clamp(0., 1.);
        game.0.skate_input.brake = (f32::from(held(KeyCode::KeyS))
            + (-movement.y).max(0.)
            + f32::from(u8::from(pad_pressed(GamepadButton::LeftTrigger2))))
        .clamp(0., 1.);
        game.0.skate_input.steer =
            (f32::from(held(KeyCode::KeyA) - held(KeyCode::KeyD)) - movement.x).clamp(-1., 1.);
        game.0.skate_input.spin = (f32::from(held(KeyCode::KeyQ) - held(KeyCode::KeyE))
            + f32::from(u8::from(pad_pressed(GamepadButton::LeftTrigger)))
            - f32::from(u8::from(pad_pressed(GamepadButton::RightTrigger))))
        .clamp(-1., 1.);
        game.0.skate_input.ollie |=
            keys.just_pressed(KeyCode::Space) || pad_just_pressed(GamepadButton::South);
        game.0.skate_input.flip |=
            keys.just_pressed(KeyCode::KeyR) || pad_just_pressed(GamepadButton::West);
        cmd.forwardmove = 0;
        cmd.rightmove = 0;
        cmd.buttons = 0;
    }
    controls.command = cmd;
}

fn filtered_stick(value: Vec2) -> Vec2 {
    let length = value.length();
    if !length.is_finite() || length <= 0.18 {
        return Vec2::ZERO;
    }
    value / length * ((length - 0.18) / 0.82).clamp(0., 1.)
}

fn survival_shortcuts(keys: &ButtonInput<KeyCode>, game: &mut Session, recipe: usize) {
    if keys.just_pressed(KeyCode::KeyC) {
        game.message = game
            .craft(Recipe::ALL[recipe])
            .map(|()| "Crafted and added to inventory".into())
            .unwrap_or_else(|e| e);
    }
    for (key, item) in [
        (KeyCode::KeyH, Item::Bandage),
        (KeyCode::KeyJ, Item::Food),
        (KeyCode::KeyK, Item::Water),
        (KeyCode::KeyU, Item::Ammo),
    ] {
        if keys.just_pressed(key) {
            game.message = game
                .use_item(item)
                .map(|()| format!("{} used", item.name()))
                .unwrap_or_else(|e| e);
        }
    }
}

fn can_interact(controls: &Controls, session: &Session) -> bool {
    controls.focused
        && !controls.paused
        && !controls.inventory_open
        && controls.error.is_none()
        && !controls.building
        && !controls.editor
        && session.skate.is_none()
        && session.world.player(LOCAL).is_some_and(|p| p.health > 0)
}

fn can_maintain(controls: &Controls, session: &Session) -> bool {
    controls.focused
        && !controls.paused
        && !controls.inventory_open
        && controls.error.is_none()
        && controls.building
        && !controls.editor
        && session.skate.is_none()
        && session.world.player(LOCAL).is_some_and(|p| p.health > 0)
}

fn can_edit(controls: &Controls, session: &Session) -> bool {
    controls.focused
        && !controls.paused
        && !controls.inventory_open
        && controls.error.is_none()
        && controls.editor
        && !controls.building
        && session.skate.is_none()
        && session.world.player(LOCAL).is_some_and(|p| p.health > 0)
}

fn interaction_action(session: &Session) -> WorldAction {
    if session.loot_bag_in_reach().is_some() {
        WorldAction::PickUpLoot
    } else if session.crate_in_reach().is_some() {
        WorldAction::OpenCrate
    } else if session.supply_drop_in_reach() {
        WorldAction::OpenSupplyDrop
    } else {
        WorldAction::Gather
    }
}

fn loot_hint(session: &Session) -> Option<String> {
    let bag = session.loot_bag_in_reach()?;
    let count: u32 = bag.inventory().stacks().iter().map(|s| s.quantity).sum();
    Some(format!(
        "Loot bag #{} | {count} items | F / Xbox Y to recover",
        bag.id()
    ))
}

fn crate_hint(session: &Session) -> Option<String> {
    let lootable = session.crate_in_reach()?;
    Some(format!(
        "{} #{} | F / Xbox Y to open",
        lootable.tier().name(),
        lootable.id()
    ))
}

fn supply_hint(session: &Session) -> Option<String> {
    if !session.supply_drop_in_reach() {
        return None;
    }
    Some("Supply drop | F / Xbox Y to open".into())
}

fn gather_feedback(harvest: survival::Harvest) -> String {
    let mut message = format!(
        "{} +{} | {} remaining",
        if harvest.kind == survival::ResourceKind::Hemp {
            "Cloth"
        } else {
            harvest.kind.name()
        },
        harvest.amount,
        harvest.remaining
    );
    if harvest.tool_broke
        && let Some(tool) = harvest.kind.tool()
    {
        message.push_str(&format!(" | {} broke", tool.name()));
    } else if harvest.tool_almost_broken
        && let Some(tool) = harvest.kind.tool()
    {
        message.push_str(&format!(" | {} almost broken", tool.name()));
    }
    message
}

fn open_crate(session: &mut Session) -> Result<String, String> {
    session.open_crate()?;
    Ok(session.message.clone())
}

fn open_supply_drop(session: &mut Session) -> Result<String, String> {
    session.open_supply_drop()?;
    Ok(session.message.clone())
}

fn recover_loot(session: &mut Session) -> Result<String, String> {
    let id = session
        .loot_bag_in_reach()
        .ok_or("No loot bag within reach")?
        .id();
    let taken = session.pick_up_loot()?;
    let remaining: u32 = session
        .loot_bags()
        .iter()
        .find(|bag| bag.id() == id)
        .map_or(0, |bag| {
            bag.inventory().stacks().iter().map(|s| s.quantity).sum()
        });
    let noun = if taken == 1 { "item" } else { "items" };
    Ok(if remaining == 0 {
        format!("Recovered {taken} {noun}; loot bag emptied")
    } else {
        format!("Recovered {taken} {noun} | {remaining} left in bag")
    })
}

fn inventory_input(
    keys: &ButtonInput<KeyCode>,
    pad: Option<&Gamepad>,
    controls: &mut Controls,
    game: &mut Session,
) {
    if !controls.focused || !controls.inventory_open || controls.paused || controls.error.is_some()
    {
        return;
    }
    let pressed = |button| pad.is_some_and(|pad| pad.just_pressed(button));
    for (index, (key, _)) in [
        KeyCode::Digit1,
        KeyCode::Digit2,
        KeyCode::Digit3,
        KeyCode::Digit4,
        KeyCode::Digit5,
        KeyCode::Digit6,
        KeyCode::Digit7,
        KeyCode::Digit8,
        KeyCode::Digit9,
    ]
    .into_iter()
    .zip(Recipe::ALL)
    .enumerate()
    {
        if keys.just_pressed(key) {
            controls.recipe = index;
        }
    }
    let previous = keys.just_pressed(KeyCode::PageUp) || pressed(GamepadButton::LeftTrigger);
    let next = keys.just_pressed(KeyCode::PageDown) || pressed(GamepadButton::RightTrigger);
    if previous != next {
        let count = Recipe::ALL.len();
        controls.recipe = if previous {
            (controls.recipe + count - 1) % count
        } else {
            (controls.recipe + 1) % count
        };
    }
    let enqueue = keys.just_pressed(KeyCode::KeyV);
    let cancel = keys.just_pressed(KeyCode::KeyF);
    if enqueue || cancel {
        controls.inventory.apply(InventoryAction::Cancel, game);
        let result = if !game
            .world
            .player(LOCAL)
            .is_some_and(|player| player.health > 0)
        {
            Err("Player is not alive".into())
        } else if enqueue && cancel {
            Err("Choose one queue action (V or F)".into())
        } else {
            queued_crafting::apply(
                game,
                if enqueue {
                    queued_crafting::Action::Queue(Recipe::ALL[controls.recipe])
                } else {
                    queued_crafting::Action::CancelFirst
                },
            )
        };
        game.message = result.unwrap_or_else(|error| error);
        controls.inventory.sync(game);
        return;
    }
    if keys.just_pressed(KeyCode::ArrowLeft) || pressed(GamepadButton::DPadLeft) {
        controls
            .inventory
            .apply(InventoryAction::Navigate(-1), game);
    }
    if keys.just_pressed(KeyCode::ArrowRight) || pressed(GamepadButton::DPadRight) {
        controls.inventory.apply(InventoryAction::Navigate(1), game);
    }
    if keys.just_pressed(KeyCode::ArrowUp) || pressed(GamepadButton::DPadUp) {
        controls
            .inventory
            .apply(InventoryAction::Navigate(-3), game);
    }
    if keys.just_pressed(KeyCode::ArrowDown) || pressed(GamepadButton::DPadDown) {
        controls.inventory.apply(InventoryAction::Navigate(3), game);
    }
    if keys.just_pressed(KeyCode::KeyQ) || pressed(GamepadButton::LeftTrigger2) {
        controls.inventory.apply(InventoryAction::Adjust(-1), game);
    }
    if keys.just_pressed(KeyCode::KeyE) || pressed(GamepadButton::RightTrigger2) {
        controls.inventory.apply(InventoryAction::Adjust(1), game);
    }
    if keys.just_pressed(KeyCode::Delete) || pressed(GamepadButton::LeftThumb) {
        controls.inventory.apply(InventoryAction::Discard, game);
        return;
    }
    if keys.just_pressed(KeyCode::KeyS) || pressed(GamepadButton::RightThumb) {
        controls.inventory.apply(InventoryAction::Split, game);
        return;
    }
    if keys.just_pressed(KeyCode::KeyN) {
        controls.inventory.apply(InventoryAction::Recycle, game);
        return;
    }
    let research = keys.just_pressed(KeyCode::KeyR) || pressed(GamepadButton::Select);
    let wear = keys.just_pressed(KeyCode::KeyO);
    let take_off = keys.just_pressed(KeyCode::KeyP);
    let repair = keys.just_pressed(KeyCode::KeyT);
    if research || wear || take_off || repair {
        controls.inventory.apply(InventoryAction::Cancel, game);
        let result = if research {
            research::apply(game, Recipe::ALL[controls.recipe])
        } else if wear {
            clothing::wear(game, controls.inventory.slot)
        } else if take_off {
            clothing::take_off(game)
        } else {
            tool_actions::repair(game, controls.inventory.slot)
        };
        game.message = result.unwrap_or_else(|error| error);
        controls.inventory.sync(game);
        return;
    }
    if keys.just_pressed(KeyCode::Enter) || pressed(GamepadButton::North) {
        controls.inventory.apply(InventoryAction::Activate, game);
        return;
    }
    if [
        KeyCode::KeyC,
        KeyCode::KeyH,
        KeyCode::KeyJ,
        KeyCode::KeyK,
        KeyCode::KeyU,
    ]
    .into_iter()
    .any(|key| keys.just_pressed(key))
    {
        controls.inventory.apply(InventoryAction::Cancel, game);
        survival_shortcuts(keys, game, controls.recipe);
        controls.inventory.sync(game);
        return;
    }
    if pressed(GamepadButton::West) {
        controls.inventory.apply(InventoryAction::Cancel, game);
        game.message = game
            .craft(Recipe::ALL[controls.recipe])
            .map(|()| "Crafted".into())
            .unwrap_or_else(|e| e);
        controls.inventory.sync(game);
        return;
    }
    if keys.just_pressed(KeyCode::KeyI) || pressed(GamepadButton::South) {
        controls.inventory.apply(InventoryAction::Cancel, game);
        if let Some(stack) = game.inventory.stacks().get(controls.inventory.slot) {
            let item = stack.item;
            game.message = game
                .use_item(item)
                .map(|()| format!("{} used", item.name()))
                .unwrap_or_else(|e| e);
        } else {
            game.message = "This slot is empty".into();
        }
        controls.inventory.sync(game);
    }
}

fn update_inventory(
    game: Res<GameSession>,
    controls: Res<Controls>,
    mut panels: Query<(&mut Text, &mut Node), With<InventoryText>>,
) {
    for (mut text, mut node) in &mut panels {
        node.display = if controls.inventory_open {
            Display::Flex
        } else {
            Display::None
        };
        if !controls.inventory_open {
            continue;
        }
        let mut content = String::from("INVENTORY | world paused\n");
        if !controls.inventory.pending() {
            content.push_str(&format!("{}\n", game.0.message));
        }
        for row in 0..8 {
            for column in 0..3 {
                let index = row * 3 + column;
                let label = game
                    .0
                    .inventory
                    .stacks()
                    .get(index)
                    .map(|stack| format!("{} x{}", stack.item.name(), stack.quantity))
                    .unwrap_or_else(|| "-".into());
                content.push_str(&format!(
                    "{} {:02} {:<15} ",
                    if index == controls.inventory.slot {
                        ">"
                    } else {
                        " "
                    },
                    index + 1,
                    label
                ));
            }
            content.push('\n');
        }
        content.push_str(&format!(
            "{}\n{}\n{}\n{}\n",
            controls.inventory.status(&game.0),
            tool_actions::status(&game.0, controls.inventory.slot),
            clothing::status(&game.0),
            queued_crafting::summary(&game.0, true),
        ));
        content.push_str("RECIPES | blueprint status: Known / Locked\n");
        for (index, recipe) in Recipe::ALL.into_iter().enumerate() {
            let cost = recipe.cost();
            let ingredients = recipe.carried_cost().map_or_else(
                || {
                    format!(
                        "wood {} / stone {} / metal {}",
                        cost.wood, cost.stone, cost.metal
                    )
                },
                |(item, amount)| format!("{amount} {}", item.name()),
            );
            let (item, amount) = recipe.output();
            content.push_str(&format!(
                "{} {}: {} x{} [{}] | {}\n",
                if index == controls.recipe { ">" } else { " " },
                index + 1,
                item.name(),
                amount,
                if game.0.blueprints().knows(recipe) {
                    "Known"
                } else {
                    "Locked"
                },
                ingredients
            ));
        }
        content.push_str(&format!(
            "{}\nArrow keys slot | Q/E quantity | S split | Enter move/confirm | I use\nT repair | N recycle (confirm) | Delete discard (destroys items)\nBackspace cancel/close | Tab close | H bandage | J food | K water | U ammo\n1-{} / PgUp/PgDn recipe | C craft | R research | V queue | F cancel first\nXbox: D-pad slot | LT/RT quantity | RS split | Y move/confirm\nLS discard | B cancel/close | LB/RB recipe | X craft | A use | Back research",
            research::status(&game.0, Recipe::ALL[controls.recipe]),
            Recipe::ALL.len().min(9)
        ));
        **text = content;
    }
}

fn refresh_gathering(
    mut commands: Commands,
    game: Res<GameSession>,
    mut visuals: ResMut<GatheringVisuals>,
) {
    let nodes: Vec<_> = game
        .0
        .gathering
        .nodes()
        .filter(|node| node.remaining > 0)
        .cloned()
        .collect();
    if nodes == visuals.nodes {
        return;
    }
    for entity in visuals.entities.drain(..) {
        commands.entity(entity).despawn();
    }
    for node in &nodes {
        let shape = node.kind.visual();
        let Some((_, material, canopy_material)) = visuals
            .materials
            .iter()
            .find(|(kind, _, _)| *kind == node.kind)
        else {
            continue;
        };
        let material = material.clone();
        let canopy_material = canopy_material.clone();
        let center = std::array::from_fn(|i| node.position[i] + shape.center[i]);
        let size = Vec3::new(
            shape.half_extents[0],
            shape.half_extents[2],
            shape.half_extents[1],
        ) * (2. * UNITS_TO_METERS);
        let mesh = if node.kind == survival::ResourceKind::Berry {
            visuals.sphere.clone()
        } else {
            visuals.cube.clone()
        };
        let size = if node.kind == survival::ResourceKind::Berry {
            size * 0.5
        } else {
            size
        };
        let entity = commands
            .spawn((
                Mesh3d(mesh),
                MeshMaterial3d(material),
                Transform::from_translation(point(center)).with_scale(size),
            ))
            .id();
        visuals.entities.push(entity);
        if let (Some(canopy), Some(material)) = (shape.canopy, canopy_material) {
            let center = std::array::from_fn(|i| node.position[i] + canopy.center[i]);
            let entity = commands
                .spawn((
                    Mesh3d(visuals.sphere.clone()),
                    MeshMaterial3d(material),
                    Transform::from_translation(point(center))
                        .with_scale(Vec3::splat(canopy.radius * UNITS_TO_METERS)),
                ))
                .id();
            visuals.entities.push(entity);
        }
    }
    visuals.nodes = nodes;
}

fn present_skate(
    game: Res<GameSession>,
    mut riders: Query<(&mut Transform, &mut Visibility), (With<LocalSkater>, Without<RideBoard>)>,
    mut boards: Query<(&mut Transform, &mut Visibility), (With<RideBoard>, Without<LocalSkater>)>,
) {
    let active = game
        .0
        .skate
        .as_ref()
        .zip(game.0.world.player(LOCAL))
        .filter(|(_, player)| player.health > 0);
    for (mut transform, mut visibility) in &mut riders {
        *visibility = if active.is_some() {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
        if let Some((skate, player)) = active {
            transform.translation = point(player.origin) + Vec3::Y * 0.08;
            transform.rotation =
                Quat::from_rotation_y(std::f32::consts::FRAC_PI_2 + skate.yaw.to_radians());
        }
    }
    for (mut transform, mut visibility) in &mut boards {
        *visibility = if active.is_some() {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
        if let Some((skate, player)) = active {
            transform.translation = point(player.origin) + Vec3::Y * 0.06;
            transform.rotation =
                Quat::from_rotation_y(std::f32::consts::FRAC_PI_2 + skate.yaw.to_radians())
                    * Quat::from_rotation_z(game.0.skate_roll.to_radians());
        }
    }
}

fn advance(
    mut commands: Commands,
    mut sounds: ResMut<NativeSounds>,
    mut game: ResMut<GameSession>,
    mut controls: ResMut<Controls>,
    mut feedback: ResMut<WeaponFeedback>,
) {
    if controls.paused || controls.inventory_open || !controls.focused || controls.error.is_some() {
        return;
    }
    let (before, before_reserve) = game.0.ammo();
    let before_origin = game.0.world.player(LOCAL).map(|p| p.origin);
    let before_target = game
        .0
        .world
        .player(sim::ClientId(1))
        .map_or(0, |p| p.health);
    if let Err(error) = game.0.advance(controls.command) {
        controls.error = Some(format!("Simulation stopped: {error}"));
        controls.actions.clear();
        return;
    }
    let fired = (before - game.0.ammo().0).max(0) as f32;
    if game
        .0
        .world
        .player(sim::ClientId(1))
        .is_some_and(|p| p.health < before_target)
    {
        feedback.hit = 0.22;
    }
    if fired > 0. {
        sound(&mut commands, &sounds.shot);
        let ads = game
            .0
            .world
            .player(LOCAL)
            .map_or(0., |player| player.f_weapon_pos_frac.clamp(0., 1.));
        let alternating = game.0.world.player(LOCAL).map_or(1., |player| {
            if player.weapon_shot_count & 1 == 0 {
                1.
            } else {
                -1.
            }
        });
        feedback.pitch = (feedback.pitch + fired * (5. - 2. * ads)).min(12.);
        feedback.yaw = (feedback.yaw + fired * alternating * 0.4).clamp(-1.5, 1.5);
        feedback.back = (feedback.back + fired * 0.025).min(0.12);
    }
    if game.0.ammo().0 > before && game.0.ammo().1 < before_reserve {
        sound(&mut commands, &sounds.reload);
    }
    match game.0.last_skate_event {
        survival::SkateEvent::Ollie => sound(&mut commands, &sounds.ollie),
        survival::SkateEvent::Landed { .. } | survival::SkateEvent::Bailed => {
            sound(&mut commands, &sounds.land)
        }
        survival::SkateEvent::None => {}
    }
    if game.0.skate.is_none() {
        if let (Some(before), Some(player)) = (before_origin, game.0.world.player(LOCAL)) {
            if player.health > 0 && player.ground_entity_num != playerstate_iw4::ENTITYNUM_NONE {
                let dx = player.origin[0] - before[0];
                let dy = player.origin[1] - before[1];
                let moved = (dx * dx + dy * dy).sqrt() * UNITS_TO_METERS;
                if moved < 1. {
                    sounds.step_distance += moved;
                }
                if sounds.step_distance >= 1.65 {
                    sounds.step_distance = 0.;
                    sound(&mut commands, &sounds.footstep);
                }
            } else {
                sounds.step_distance = 0.;
            }
        }
    }
    for action in std::mem::take(&mut controls.actions) {
        if game.0.world.player(LOCAL).is_none_or(|p| p.health <= 0)
            && !matches!(
                &action,
                WorldAction::Save | WorldAction::Load | WorldAction::Respawn
            )
        {
            continue;
        }
        if matches!(
            &action,
            WorldAction::Gather
                | WorldAction::PickUpLoot
                | WorldAction::OpenCrate
                | WorldAction::OpenSupplyDrop
                | WorldAction::Fishing
                | WorldAction::Campfire
                | WorldAction::Water(_)
                | WorldAction::Garden
        ) && !can_interact(&controls, &game.0)
        {
            continue;
        }
        if matches!(&action, WorldAction::MaintainBuilding(_)) && !can_maintain(&controls, &game.0)
        {
            continue;
        }
        if matches!(&action, WorldAction::EditProp(_)) && !can_edit(&controls, &game.0) {
            continue;
        }
        let loading = matches!(&action, WorldAction::Load);
        let result = match action {
            WorldAction::Gather => game.0.gather_from_view().map(|harvest| {
                sound(&mut commands, &sounds.gather);
                gather_feedback(harvest)
            }),
            WorldAction::PickUpLoot => recover_loot(&mut game.0).inspect(|_| {
                sound(&mut commands, &sounds.ui);
            }),
            WorldAction::OpenCrate => open_crate(&mut game.0).inspect(|_| {
                sound(&mut commands, &sounds.ui);
            }),
            WorldAction::OpenSupplyDrop => open_supply_drop(&mut game.0).inspect(|_| {
                sound(&mut commands, &sounds.ui);
            }),
            WorldAction::Fishing => fishing::toggle(&mut game.0).inspect(|_| {
                sound(&mut commands, &sounds.ui);
            }),
            WorldAction::Campfire => campfires::apply(&mut game.0).inspect(|_| {
                sound(&mut commands, &sounds.ui);
            }),
            WorldAction::Water(action) => water::apply(&mut game.0, action).inspect(|_| {
                sound(&mut commands, &sounds.ui);
            }),
            WorldAction::Garden => garden::apply(&mut game.0).inspect(|_| {
                sound(&mut commands, &sounds.ui);
            }),
            WorldAction::PlaceProp(kind, yaw) => game
                .0
                .place_prop_from_view(kind, yaw)
                .map(|id| format!("Object {id} placed")),
            WorldAction::RemoveProp => game
                .0
                .remove_prop_from_view()
                .map(|()| "Object removed".into()),
            WorldAction::EditProp(action) => object_actions::apply(&mut game.0, action)
                .inspect(|_| sound(&mut commands, &sounds.ui)),
            WorldAction::Undo => game.0.undo_props().map(|()| "Object change undone".into()),
            WorldAction::Redo => game.0.redo_props().map(|()| "Object change redone".into()),
            WorldAction::PlaceBuilding(kind, axis) => game
                .0
                .place_from_view(kind, axis)
                .map(|id| format!("Building piece {id} placed")),
            WorldAction::Door => game
                .0
                .toggle_door_from_view()
                .map(|()| "Door toggled".into()),
            WorldAction::MaintainBuilding(action) => building_actions::apply(&mut game.0, action)
                .inspect(|_| sound(&mut commands, &sounds.ui)),
            WorldAction::Save => game
                .0
                .save(Path::new(SAVE_PATH))
                .map(|()| "Session saved".into()),
            WorldAction::Load => game.0.load(Path::new(SAVE_PATH)).map(|()| {
                if let Some(player) = game.0.world.player(LOCAL) {
                    // Dead movement retains its last accepted command. Keep that
                    // history so the next scripted spawn rebases the view correctly.
                    let angles = if player.health <= 0 {
                        game.0
                            .world
                            .old_cmd_angles(LOCAL)
                            .unwrap_or(controls.command.angles)
                    } else {
                        std::array::from_fn(|k| {
                            ((player.viewangles[k] - player.delta_angles[k]) * 65536. / 360.)
                                .round() as i32
                        })
                    };
                    controls.pitch = angles[0] as f32 * (360. / 65536.);
                    controls.yaw = (angles[1] as f32 * (360. / 65536.)).rem_euclid(360.);
                    controls.command = UserCmd {
                        angles,
                        weapon: player.weapon as u16,
                        ..default()
                    };
                }
                controls.building = false;
                controls.editor = false;
                sounds.step_distance = 0.;
                *feedback = WeaponFeedback::default();
                "Session loaded".into()
            }),
            WorldAction::Respawn => game.0.respawn().map(|()| "Respawn requested".into()),
        };
        let loaded = loading && result.is_ok();
        game.0.message = result.unwrap_or_else(|error| error);
        if loaded {
            break;
        }
    }
}

fn refresh_placement(
    mut commands: Commands,
    game: Res<GameSession>,
    controls: Res<Controls>,
    editor: Res<EditorVisuals>,
    mut visuals: ResMut<PlacementVisuals>,
) {
    let active = controls.focused
        && !controls.paused
        && !controls.inventory_open
        && controls.error.is_none()
        && game.0.skate.is_none()
        && game.0.world.player(LOCAL).is_some_and(|p| p.health > 0);
    let (ghost, status) = if active && controls.editor {
        let preview = game
            .0
            .preview_prop_from_view(PropKind::ALL[controls.prop_selection], controls.prop_yaw);
        let valid = preview.valid();
        let status = preview
            .error
            .unwrap_or_else(|| "Ready to place | LMB / Xbox RT".into());
        (
            preview
                .object
                .map(|object| PlacementGhost::Prop(object, valid)),
            status,
        )
    } else if active && controls.building {
        let preview = game
            .0
            .preview_building_from_view(KINDS[controls.selection], controls.axis);
        let valid = preview.valid();
        let status = format!(
            "Place cost: wood {} / stone {} / metal {}\n{}",
            preview.cost.wood,
            preview.cost.stone,
            preview.cost.metal,
            preview
                .error
                .unwrap_or_else(|| "Ready to place | LMB / Xbox RT".into())
        );
        (
            (!preview.bounds.is_empty()).then_some(PlacementGhost::Building(preview.bounds, valid)),
            status,
        )
    } else {
        (None, String::new())
    };
    visuals.status = status;
    if ghost == visuals.ghost {
        return;
    }
    for entity in visuals.entities.drain(..) {
        commands.entity(entity).despawn();
    }
    if let Some(ghost) = &ghost {
        let valid = match ghost {
            PlacementGhost::Prop(_, valid) | PlacementGhost::Building(_, valid) => *valid,
        };
        let material = if valid {
            &visuals.valid
        } else {
            &visuals.invalid
        }
        .clone();
        let mut spawn = |mesh, transform| {
            commands
                .spawn((
                    Mesh3d(mesh),
                    MeshMaterial3d(material.clone()),
                    transform,
                    bevy::light::NotShadowCaster,
                    bevy::light::NotShadowReceiver,
                ))
                .id()
        };
        match ghost {
            PlacementGhost::Prop(object, _) => {
                if let Some((_, mesh, _)) = editor
                    .templates
                    .iter()
                    .find(|(kind, _, _)| *kind == object.kind)
                {
                    let entity = spawn(
                        mesh.clone(),
                        Transform::from_translation(point(object.position))
                            .with_rotation(Quat::from_rotation_y(object.yaw.to_radians())),
                    );
                    visuals.entities.push(entity);
                }
            }
            PlacementGhost::Building(bounds, _) => {
                for (lo, hi) in bounds {
                    let center = std::array::from_fn(|i| (lo[i] + hi[i]) * 0.5);
                    let size =
                        Vec3::new(hi[0] - lo[0], hi[2] - lo[2], hi[1] - lo[1]) * UNITS_TO_METERS;
                    let entity = spawn(
                        visuals.cube.clone(),
                        Transform::from_translation(point(center)).with_scale(size),
                    );
                    visuals.entities.push(entity);
                }
            }
        }
    }
    visuals.ghost = ghost;
}

fn present(
    game: Res<GameSession>,
    time: Res<Time>,
    controls: Res<Controls>,
    mut feedback: ResMut<WeaponFeedback>,
    mut camera: Query<
        (&mut Transform, &mut Projection),
        (With<PlayerCamera>, Without<Operator>, Without<ViewWeapon>),
    >,
    mut operators: Query<
        (&mut Transform, &mut Visibility),
        (With<Operator>, Without<PlayerCamera>, Without<ViewWeapon>),
    >,
    mut weapons: Query<
        (&mut Transform, &mut Visibility),
        (With<ViewWeapon>, Without<PlayerCamera>, Without<Operator>),
    >,
) {
    let decay = (-14. * time.delta_secs()).exp();
    feedback.pitch *= decay;
    feedback.yaw *= decay;
    feedback.back *= decay;
    feedback.hit = (feedback.hit - time.delta_secs()).max(0.);
    if let Some(player) = game.0.world.player(LOCAL) {
        let skating = game.0.skate.is_some();
        let pitch = if skating {
            (controls.command.angles[0] as f32 * (360. / 65536.) + player.delta_angles[0])
                .clamp(-89., 89.)
        } else {
            player.viewangles[0]
        }
        .to_radians();
        let yaw = if skating {
            (controls.command.angles[1] as f32 * (360. / 65536.) + player.delta_angles[1])
                .rem_euclid(360.)
        } else {
            player.viewangles[1]
        }
        .to_radians();
        let direction = Vec3::new(
            pitch.cos() * yaw.cos(),
            -pitch.sin(),
            -pitch.cos() * yaw.sin(),
        );
        for (mut transform, mut projection) in &mut camera {
            transform.translation = point([
                player.origin[0],
                player.origin[1],
                player.origin[2] + player.view_height_current.max(28.),
            ]);
            if skating {
                let focus = point(player.origin) + Vec3::Y * 1.1;
                transform.translation = focus - direction * 4.5 + Vec3::Y * 0.7;
                transform.look_at(focus, Vec3::Y);
            } else {
                transform.look_to(direction, Vec3::Y);
            }
            if let Projection::Perspective(p) = &mut *projection {
                p.fov = if skating {
                    75f32.to_radians()
                } else {
                    (75. - 20. * player.f_weapon_pos_frac.clamp(0., 1.)).to_radians()
                };
            }
        }
        for (mut transform, mut visible) in &mut weapons {
            *visible = if skating {
                Visibility::Hidden
            } else {
                Visibility::Inherited
            };
            transform.translation = Vec3::new(0.24, -0.24, -0.48).lerp(
                Vec3::new(0., -0.15, -0.42),
                player.f_weapon_pos_frac.clamp(0., 1.),
            );
            transform.translation.z += feedback.back;
            transform.rotation = Quat::from_rotation_y(std::f32::consts::PI)
                * Quat::from_euler(
                    EulerRot::XYZ,
                    -feedback.pitch.to_radians(),
                    feedback.yaw.to_radians(),
                    0.,
                );
        }
    }
    if let Some(player) = game.0.world.player(sim::ClientId(1)) {
        for (mut transform, mut visibility) in &mut operators {
            transform.translation = point(player.origin);
            transform.rotation = Quat::from_rotation_y(
                std::f32::consts::FRAC_PI_2 + player.viewangles[1].to_radians(),
            );
            *visibility = if player.health > 0 {
                Visibility::Inherited
            } else {
                Visibility::Hidden
            };
        }
    }
}

fn refresh_buildings(
    mut commands: Commands,
    game: Res<GameSession>,
    mut visuals: ResMut<BuildVisuals>,
) {
    let pieces: Vec<_> = game.0.world.buildings().pieces().cloned().collect();
    let anchor = game.0.world.buildings().anchor;
    if visuals.pieces == pieces && visuals.anchor == Some(anchor) {
        return;
    }
    for entity in visuals.entities.drain(..) {
        commands.entity(entity).despawn();
    }
    for piece in &pieces {
        let index = match piece.grade {
            Grade::Wood => 0,
            Grade::Stone => 1,
            Grade::Metal => 2,
        };
        for (lo, hi) in game.0.world.buildings().bounds(piece) {
            let center = std::array::from_fn(|k| (lo[k] + hi[k]) * 0.5);
            let size = Vec3::new(hi[0] - lo[0], hi[2] - lo[2], hi[1] - lo[1]) * UNITS_TO_METERS;
            let entity = commands
                .spawn((
                    Mesh3d(visuals.cube.clone()),
                    MeshMaterial3d(visuals.materials[index].clone()),
                    Transform::from_translation(point(center)).with_scale(size),
                ))
                .id();
            visuals.entities.push(entity);
        }
    }
    visuals.pieces = pieces;
    visuals.anchor = Some(anchor);
}

fn refresh_props(
    mut commands: Commands,
    game: Res<GameSession>,
    mut visuals: ResMut<EditorVisuals>,
) {
    let objects: Vec<_> = game.0.editor.objects().cloned().collect();
    if visuals.objects == objects {
        return;
    }
    for entity in visuals.entities.drain(..) {
        commands.entity(entity).despawn();
    }
    for object in &objects {
        if let Some((_, mesh, material)) = visuals
            .templates
            .iter()
            .find(|(kind, _, _)| *kind == object.kind)
        {
            let entity = commands
                .spawn((
                    Mesh3d(mesh.clone()),
                    MeshMaterial3d(material.clone()),
                    Transform::from_translation(point(object.position))
                        .with_rotation(Quat::from_rotation_y(object.yaw.to_radians())),
                ))
                .id();
            visuals.entities.push(entity);
        }
    }
    visuals.objects = objects;
}

fn hit_feedback(
    feedback: Res<WeaponFeedback>,
    controls: Res<Controls>,
    game: Res<GameSession>,
    mut reticles: Query<(&mut Text, &mut TextColor, &mut Node), With<Crosshair>>,
) {
    for (mut text, mut color, mut node) in &mut reticles {
        node.display = if controls.inventory_open || controls.paused || game.0.skate.is_some() {
            Display::None
        } else {
            Display::Flex
        };
        **text = if feedback.hit > 0. { "x" } else { "+" }.into();
        color.0 = if feedback.hit > 0. {
            Color::srgb(1., 0.72, 0.18)
        } else {
            Color::WHITE
        };
    }
}

fn models_status(
    server: Res<AssetServer>,
    sounds: Res<NativeSounds>,
    mut models: ResMut<NativeModels>,
) {
    let mut loaded = 0;
    let mut failure = None;
    for (name, scene) in &models.scenes {
        match server.get_load_states(scene.id()) {
            Some((LoadState::Failed(error), _, _))
            | Some((_, _, RecursiveDependencyLoadState::Failed(error))) => {
                failure = Some(format!("Failed to load {name}: {error}"));
                break;
            }
            Some((LoadState::Loaded, _, RecursiveDependencyLoadState::Loaded)) => loaded += 1,
            _ => {}
        }
    }
    models.status = if let Some(error) = failure {
        error
    } else if loaded == models.scenes.len() {
        "Models loaded".into()
    } else {
        format!("Loading models: {loaded}/{}", models.scenes.len())
    };
    for (name, handle) in [
        ("Carbine", &sounds.shot),
        ("Reload", &sounds.reload),
        ("Footstep", &sounds.footstep),
        ("Ollie", &sounds.ollie),
        ("Land", &sounds.land),
        ("Gather", &sounds.gather),
        ("UI", &sounds.ui),
    ] {
        if let Some((LoadState::Failed(error), _, _)) = server.get_load_states(handle.id()) {
            models
                .status
                .push_str(&format!("\nFailed to load sound {name}: {error}"));
        }
    }
}

fn update_hud(
    game: Res<GameSession>,
    controls: Res<Controls>,
    models: Res<NativeModels>,
    placement: Res<PlacementVisuals>,
    mut hud: Query<&mut Text, With<StatusText>>,
) {
    let player = game.0.world.player(LOCAL);
    let health = player.map_or(0, |p| p.health);
    let resources = game.0.world.buildings().inventory(LOCAL.0);
    let (clip, reserve) = game.0.ammo();
    let grinding = game
        .0
        .skate
        .as_ref()
        .is_some_and(|skate| skate.is_grinding());
    let mode = if health <= 0 && controls.inventory_open {
        "DEAD | Tab close\nXbox B cancel/close\nThen Enter / Xbox A to respawn".into()
    } else if health <= 0 {
        "DEAD | Enter / Xbox A to respawn".into()
    } else if controls.inventory_open {
        "INVENTORY | Tab close\nXbox B cancel/close".into()
    } else if controls.paused {
        "PAUSED | Esc / Start to resume".into()
    } else if controls.building {
        format!(
            "BUILD / {:?} / axis {}",
            KINDS[controls.selection], controls.axis
        )
    } else if controls.editor {
        format!(
            "OBJECT EDITOR / {} / {:.0} deg / {} objects",
            PropKind::ALL[controls.prop_selection].name(),
            controls.prop_yaw,
            game.0.editor.objects().count()
        )
    } else if let Some(skate) = &game.0.skate {
        let speed = (skate.velocity[0] * skate.velocity[0] + skate.velocity[1] * skate.velocity[1])
            .sqrt()
            * UNITS_TO_METERS
            * 3.6;
        format!(
            "{} / {:.0} km/h / banked {} / pending {} / bails {}",
            if grinding { "GRIND" } else { "SKATE" },
            speed,
            game.0.skate_score(),
            skate.pending_points(),
            skate.bails
        )
    } else {
        format!("FPS / SURVIVAL / skate score {}", game.0.skate_score())
    };
    let mode_controls = if grinding {
        "S brake | Space ollie off | V dismount\nLand safely to bank pending points; bails lose them"
    } else if game.0.skate.is_some() {
        "W push | A/D steer | S brake | Space ollie | Q/E spin | R flip | V dismount\nGrind: align with a rail, then ollie onto it to catch automatically.\nLand safely to bank pending points; bails lose them"
    } else if controls.editor {
        "1-6 object | Q/R preview rotate | LMB place | RMB remove | E close\nAim at object: M move to surface | ,/. rotate 15 deg | Ctrl-Z/Y undo/redo"
    } else if controls.building {
        "1-4 building piece | R rotate | LMB place | RMB door | B close\nAim at your building: T repair | Z upgrade Stone | X upgrade Metal"
    } else {
        "LMB shoot | RMB ADS | R reload | F gather/loot/crate\nP sip water | O collect barrel | L cast/reel | G cook/take fish\nT plant/harvest | B build | E editor | V skate"
    };
    let pad_controls = if controls.pad.is_some() {
        if grinding {
            "Xbox grind: LT brake | A ollie off | LB/RB + Y dismount\nRS camera | Start pause\n"
        } else if game.0.skate.is_some() {
            "Xbox skate: LS push/steer | RT push / LT brake | A ollie | LB/RB spin | X flip\nLB/RB + Y dismount | RS camera | Start pause\n"
        } else if controls.building {
            "Xbox build: LS move / RS look | RT place | X door | A jump\nD-pad Left/Right piece | Up upgrade Stone / Down upgrade Metal\nY repair | LB/RB rotate | Back mode | Start pause\n"
        } else if controls.editor {
            "Xbox editor: LS move / RS look | RT place | X remove | Y move\nD-pad select | LB/RB preview rotate | LT + LB/RB object rotate\nBack mode | LB/RB + Y skate | Start pause\n"
        } else {
            "Xbox: LS move / RS look | RT shoot | LT ADS | A jump\nLS click sprint | B crouch | X reload | Y gather/loot/crate\nRS click sip water | D-pad Up collect barrel / Down inventory\nD-pad Right cast/reel | D-pad Left cook/take fish\nHold LB + press RB garden | Back mode | LB/RB + Y skate | Start pause\n"
        }
    } else {
        ""
    };
    for mut text in &mut hud {
        if controls.inventory_open {
            **text = format!(
                "{mode}\nHealth {health}  Ammo {clip}/{reserve}\nWood {}\nStone {}\nMetal {}\n{}",
                resources.wood,
                resources.stone,
                resources.metal,
                controls.error.as_deref().unwrap_or("")
            );
            continue;
        }
        let mut content = format!(
            "{mode}\nHealth {health}  Ammo {clip}/{reserve}  Hunger {:.0}  Thirst {:.0}\nWood {}  Stone {}  Metal {}",
            game.0.vitals.hunger(),
            game.0.vitals.thirst(),
            resources.wood,
            resources.stone,
            resources.metal,
        );
        if !game.0.crafting_queue().is_empty() {
            content.push_str(&format!("\n{}", queued_crafting::summary(&game.0, false)));
        }
        if !placement.status.is_empty() {
            content.push_str(&format!("\n{}", placement.status));
        }
        if can_interact(&controls, &game.0) {
            if let Some(hint) = water::hint(&game.0) {
                content.push_str(&format!("\n{hint}"));
            }
            if let Some(hint) = garden::hint(&game.0) {
                content.push_str(&format!("\n{hint}"));
            }
            if let Some(hint) = fishing::hint(&game.0) {
                content.push_str(&format!("\n{hint}"));
            }
            if let Some(hint) = campfires::hint(&game.0) {
                content.push_str(&format!("\n{hint}"));
            }
            if let Some(hint) = loot_hint(&game.0) {
                content.push_str(&format!("\n{hint}"));
            } else if let Some(hint) = crate_hint(&game.0) {
                content.push_str(&format!("\n{hint}"));
            } else if let Some(hint) = supply_hint(&game.0) {
                content.push_str(&format!("\n{hint}"));
            } else if let Ok(Some(node)) = game.0.gather_target_from_view() {
                content.push_str(&format!(
                    "\n{} | {} remaining | F / Xbox Y to gather",
                    node.kind.name(),
                    node.remaining
                ));
            }
        }
        content.push_str(&format!(
            "\n{}\nF1 controls | Tab inventory | F5 save / F9 load",
            game.0.message
        ));
        if models.status != "Models loaded" {
            content.push_str(&format!("\n{}", models.status));
        }
        if let Some(error) = &controls.error {
            content.push_str(&format!("\n{error}"));
        }
        if controls.help {
            content.push_str(&format!(
                "\n\nCONTROLS\nWASD move | Shift sprint | Space jump | Ctrl crouch\n{mode_controls}\nTab inventory | C craft | H bandage | J food | K water | U ammo\nEsc pause / release mouse | F1 hide controls\n{pad_controls}"
            ));
        }
        **text = content;
    }
}
