use bevy::asset::RenderAssetUsages;
use bevy::asset::{LoadState, RecursiveDependencyLoadState};
use bevy::input::mouse::AccumulatedMouseMotion;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;
use bevy::window::{CursorGrabMode, CursorOptions, PrimaryWindow};
use playerstate_iw4::{UserCmd, buttons};
use rust_building::{Grade, Kind, Piece};
use std::path::{Path, PathBuf};
use survival::{LOCAL, PlacedObject, PropKind, Session, UNITS_TO_METERS};

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

#[derive(Component)]
struct PlayerCamera;

#[derive(Component)]
struct Operator;

#[derive(Component)]
struct ViewWeapon;

#[derive(Component)]
struct StatusText;

const KINDS: [Kind; 4] = [Kind::Foundation, Kind::Floor, Kind::Wall, Kind::Doorway];
const SAVE_PATH: &str = "iw4l-artifacts/survival/base.json";

pub fn run() -> Result<(), String> {
    let assets = asset_directory()?;
    let session = Session::new()?;
    let mut app = App::new();
    app.insert_resource(GameSession(session))
        .init_resource::<Controls>()
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
                update_hud,
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
            "SURVIVAL_ASSETS={} bevat niet alle eigen authored/*.glb modellen",
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
    Err("Eigen gamecontent ontbreekt: plaats assets/authored/{masked_operator,carbine,skateboard}.glb naast het programma, of stel SURVIVAL_ASSETS in.".into())
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
    let operator: Handle<WorldAsset> = server.load("authored/masked_operator.glb#Scene0");
    let rifle: Handle<WorldAsset> = server.load("authored/carbine.glb#Scene0");
    let board: Handle<WorldAsset> = server.load("authored/skateboard.glb#Scene0");
    commands.insert_resource(NativeModels {
        scenes: vec![
            ("Operator", operator.clone()),
            ("Carbine", rifle.clone()),
            ("Skateboard", board.clone()),
        ],
        status: "Eigen modellen laden…".into(),
    });
    commands
        .spawn((
            PlayerCamera,
            Camera3d::default(),
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
        WorldAssetRoot(operator),
        Transform::from_translation(point([550., 0., 1.]))
            .with_rotation(Quat::from_rotation_y(-std::f32::consts::FRAC_PI_2)),
    ));
    commands.spawn((
        WorldAssetRoot(board),
        Transform::from_translation(point([180., -180., 6.])),
    ));
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
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    motion: Res<AccumulatedMouseMotion>,
    mut windows: Query<(&Window, &mut CursorOptions), With<PrimaryWindow>>,
    mut controls: ResMut<Controls>,
    mut game: ResMut<GameSession>,
) {
    let Ok((window, mut cursor)) = windows.single_mut() else {
        return;
    };
    if keys.just_pressed(KeyCode::Escape) {
        controls.paused = !controls.paused;
    }
    let returned = window.focused && !controls.focused;
    controls.focused = window.focused;
    let active = window.focused && !controls.paused;
    cursor.visible = !active;
    cursor.grab_mode = if active && !returned {
        CursorGrabMode::Locked
    } else {
        CursorGrabMode::None
    };
    if !active || returned {
        controls.command.buttons = 0;
        controls.command.forwardmove = 0;
        controls.command.rightmove = 0;
        return;
    }
    controls.yaw = (controls.yaw - motion.delta.x * 0.1).rem_euclid(360.);
    controls.pitch = (controls.pitch + motion.delta.y * 0.1).clamp(-89., 89.);
    if keys.just_pressed(KeyCode::KeyB) {
        controls.building = !controls.building;
        controls.editor = false;
    }
    if keys.just_pressed(KeyCode::KeyE) {
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
    if controls.building && keys.just_pressed(KeyCode::KeyR) {
        controls.axis ^= 1;
    }
    if controls.editor {
        if keys.just_pressed(KeyCode::KeyQ) {
            controls.prop_yaw = (controls.prop_yaw - 15.).rem_euclid(360.);
        }
        if keys.just_pressed(KeyCode::KeyR) {
            controls.prop_yaw = (controls.prop_yaw + 15.).rem_euclid(360.);
        }
        let control = keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight);
        if control && keys.just_pressed(KeyCode::KeyZ) {
            game.0.message = game
                .0
                .undo_props()
                .map(|()| "Objectwijziging ongedaan gemaakt".into())
                .unwrap_or_else(|e| e);
        }
        if control && keys.just_pressed(KeyCode::KeyY) {
            game.0.message = game
                .0
                .redo_props()
                .map(|()| "Objectwijziging opnieuw toegepast".into())
                .unwrap_or_else(|e| e);
        }
        if mouse.just_pressed(MouseButton::Left) {
            game.0.message = game
                .0
                .place_prop_from_view(PropKind::ALL[controls.prop_selection], controls.prop_yaw)
                .map(|id| format!("Object {id} geplaatst"))
                .unwrap_or_else(|e| e);
        }
        if mouse.just_pressed(MouseButton::Right) {
            game.0.message = game
                .0
                .remove_prop_from_view()
                .map(|()| "Object verwijderd".into())
                .unwrap_or_else(|e| e);
        }
    }
    if keys.just_pressed(KeyCode::F5) {
        game.0.message = game
            .0
            .save(Path::new(SAVE_PATH))
            .map(|()| "Basis opgeslagen".into())
            .unwrap_or_else(|e| e);
    }
    if keys.just_pressed(KeyCode::F9) {
        game.0.message = game
            .0
            .load(Path::new(SAVE_PATH))
            .map(|()| "Basis geladen".into())
            .unwrap_or_else(|e| e);
    }
    if controls.building && mouse.just_pressed(MouseButton::Left) {
        game.0.message = game
            .0
            .place_from_view(KINDS[controls.selection], controls.axis)
            .map(|id| format!("Bouwstuk {id} geplaatst"))
            .unwrap_or_else(|e| e);
    }
    if controls.building && mouse.just_pressed(MouseButton::Right) {
        game.0.message = game
            .0
            .toggle_door_from_view()
            .map(|()| "Deur bediend".into())
            .unwrap_or_else(|e| e);
    }
    let held = |key| i8::from(keys.pressed(key));
    let mut cmd = UserCmd {
        forwardmove: (held(KeyCode::KeyW) - held(KeyCode::KeyS)) * 127,
        rightmove: (held(KeyCode::KeyD) - held(KeyCode::KeyA)) * 127,
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
    if !controls.building && !controls.editor {
        if keys.pressed(KeyCode::KeyR) {
            cmd.buttons |= buttons::RELOAD;
        }
        if mouse.pressed(MouseButton::Left) {
            cmd.buttons |= buttons::ATTACK;
        }
        if mouse.pressed(MouseButton::Right) {
            cmd.buttons |= buttons::ADS;
        }
    }
    controls.command = cmd;
}

fn advance(mut game: ResMut<GameSession>, mut controls: ResMut<Controls>) {
    if controls.paused || !controls.focused || controls.error.is_some() {
        return;
    }
    if let Err(error) = game.0.advance(controls.command) {
        controls.error = Some(format!("Simulatie gestopt: {error}"));
    }
}

fn present(
    game: Res<GameSession>,
    mut camera: Query<
        (&mut Transform, &mut Projection),
        (With<PlayerCamera>, Without<Operator>, Without<ViewWeapon>),
    >,
    mut operators: Query<
        (&mut Transform, &mut Visibility),
        (With<Operator>, Without<PlayerCamera>, Without<ViewWeapon>),
    >,
    mut weapons: Query<
        &mut Transform,
        (With<ViewWeapon>, Without<PlayerCamera>, Without<Operator>),
    >,
) {
    if let Some(player) = game.0.world.player(LOCAL) {
        let pitch = player.viewangles[0].to_radians();
        let yaw = player.viewangles[1].to_radians();
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
            transform.look_to(direction, Vec3::Y);
            if let Projection::Perspective(p) = &mut *projection {
                p.fov = (75. - 20. * player.f_weapon_pos_frac).to_radians();
            }
        }
        for mut transform in &mut weapons {
            transform.translation = Vec3::new(0.24, -0.24, -0.48).lerp(
                Vec3::new(0., -0.15, -0.42),
                player.f_weapon_pos_frac.clamp(0., 1.),
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

fn models_status(server: Res<AssetServer>, mut models: ResMut<NativeModels>) {
    let mut loaded = 0;
    let mut failure = None;
    for (name, scene) in &models.scenes {
        match server.get_load_states(scene.id()) {
            Some((LoadState::Failed(error), _, _))
            | Some((_, _, RecursiveDependencyLoadState::Failed(error))) => {
                failure = Some(format!("{name} laden mislukt: {error}"));
                break;
            }
            Some((LoadState::Loaded, _, RecursiveDependencyLoadState::Loaded)) => loaded += 1,
            _ => {}
        }
    }
    models.status = if let Some(error) = failure {
        error
    } else if loaded == models.scenes.len() {
        "Eigen modellen geladen".into()
    } else {
        format!("Eigen modellen laden: {loaded}/{}", models.scenes.len())
    };
}

fn update_hud(
    game: Res<GameSession>,
    controls: Res<Controls>,
    models: Res<NativeModels>,
    mut hud: Query<&mut Text, With<StatusText>>,
) {
    let player = game.0.world.player(LOCAL);
    let health = player.map_or(0, |p| p.health);
    let resources = game.0.world.buildings().inventory(LOCAL.0);
    let (clip, reserve) = game.0.ammo();
    let target = game
        .0
        .world
        .player(sim::ClientId(1))
        .map_or(0, |p| p.health);
    let mode = if controls.paused {
        "PAUZE — Esc hervatten".into()
    } else if controls.building {
        format!(
            "BOUW / {:?} / as {}",
            KINDS[controls.selection], controls.axis
        )
    } else if controls.editor {
        format!(
            "OBJECTEDITOR / {} / {:.0}° / {} objecten",
            PropKind::ALL[controls.prop_selection].name(),
            controls.prop_yaw,
            game.0.editor.objects().count()
        )
    } else {
        "FPS".into()
    };
    for mut text in &mut hud {
        **text = format!(
            "SURVIVAL / FPS / SKATE\n{mode}\nGezondheid {health}  Munitie {clip}/{reserve}  Doel {target}\nHout {}  Steen {}  Metaal {}\nWASD lopen · Shift sprint · Spatie springen · Ctrl hurken\nLMB schieten · RMB ADS · R herladen · B bouwen · E editor\nBouw: 1–4 kiezen · R draaien · LMB plaatsen · RMB deur\nEditor: 1–6 kiezen · Q/R draaien · LMB plaatsen · RMB verwijderen\nEditor: Ctrl-Z ongedaan · Ctrl-Y opnieuw · F5 opslaan/F9 laden\nEsc muis vrijgeven · Skateboard: model; skatebediening volgt\n{}\n{}\n{}",
            resources.wood,
            resources.stone,
            resources.metal,
            models.status,
            game.0.message,
            controls.error.as_deref().unwrap_or("")
        );
    }
}
