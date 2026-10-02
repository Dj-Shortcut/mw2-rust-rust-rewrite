use playerstate_iw4::UserCmd;
use rust_building::{BuildingWorld, CELL, Kind, Resources, Socket, WALL_HEIGHT};
use sim::{ClientId, SimBrush, SimContentBuilder, SimWorld, Tick, TickInput};
use std::io::Read;
use std::path::Path;
mod editor;
mod terrain;
pub use editor::{EditorState, Geometry, PlacedObject, PropKind};
pub use terrain::Terrain;

pub const UNITS_TO_METERS: f32 = 0.0254;
pub const WORLD_HALF: f32 = 4000.;
pub const LOCAL: ClientId = ClientId(0);

pub struct Session {
    pub world: SimWorld,
    pub tick: u32,
    pub message: String,
    pub terrain: Terrain,
    pub editor: EditorState,
}

impl Session {
    pub fn ammo(&self) -> (i32, i32) {
        let Some(player) = self.world.player(LOCAL) else {
            return (0, 0);
        };
        let Some(facts) = self.world.weapon_combat_facts(player.weapon) else {
            return (0, 0);
        };
        (
            weapon_iw4::get_clip_for_hand(&player.ammoclip, facts.clip_index, 0),
            weapon_iw4::get_ammo_not_in_clip(&player.ammo, facts.ammo_index),
        )
    }

    pub fn place_prop_from_view(&mut self, kind: PropKind, yaw: f32) -> Result<u32, String> {
        let (start, end) = self.view_ray()?;
        let hit = self.world.trace_world(start, end, [0.; 3], [0.; 3], 1);
        if hit.startsolid != 0 || hit.fraction >= 1. || hit.normal[2] < 0.7 {
            return Err("Aim at a flat surface within reach".into());
        }
        let mut editor = self.editor.clone();
        let id = editor.place(kind, hit.endpos, yaw)?;
        self.install_editor(editor)?;
        Ok(id)
    }

    pub fn remove_prop_from_view(&mut self) -> Result<(), String> {
        let (start, end) = self.view_ray()?;
        let world_hit = self.world.trace_world(start, end, [0.; 3], [0.; 3], 1);
        let id = self
            .editor
            .objects()
            .filter_map(|o| {
                o.brushes()
                    .iter()
                    .filter_map(|b| intersect(b, start, end))
                    .min_by(f32::total_cmp)
                    .map(|distance| (o.id, distance))
            })
            .filter(|(_, t)| *t <= world_hit.fraction + 0.002)
            .min_by(|a, b| a.1.total_cmp(&b.1))
            .map(|(id, _)| id)
            .ok_or("Aim at an editor object within reach")?;
        let mut editor = self.editor.clone();
        editor.remove(id)?;
        self.install_editor(editor)
    }

    pub fn undo_props(&mut self) -> Result<(), String> {
        let mut editor = self.editor.clone();
        editor.undo()?;
        self.install_editor(editor)
    }
    pub fn redo_props(&mut self) -> Result<(), String> {
        let mut editor = self.editor.clone();
        editor.redo()?;
        self.install_editor(editor)
    }
    fn install_editor(&mut self, editor: EditorState) -> Result<(), String> {
        if props_overlap_players(&self.world, &editor) {
            return Err("Editor object overlaps a player".into());
        }
        self.world
            .install_content(authored_content(&self.terrain, &editor));
        self.editor = editor;
        Ok(())
    }
    pub fn new() -> Result<Self, String> {
        let mut world = SimWorld::new();
        let terrain = Terrain::new(terrain::TERRAIN_SEED);
        let editor = EditorState::default();
        world.install_content(authored_content(&terrain, &editor));
        world
            .bootstrap(sim::MatchBootstrap {
                seed: 1,
                ..Default::default()
            })
            .map_err(str::to_owned)?;
        world.spawn_authored_player(LOCAL, [0., 0., 1.], [0.; 3], 1)?;
        world.spawn_authored_player(ClientId(1), [550., 0., 1.], [0., 180., 0.], 0)?;
        world
            .buildings_mut()
            .grant(
                LOCAL.0,
                Resources {
                    wood: 10_000,
                    stone: 5_000,
                    metal: 2_000,
                },
            )
            .map_err(|e| e.to_string())?;
        let mut session = Self {
            world,
            terrain,
            editor,
            tick: 0,
            message: "Development world: authored content, survival systems in progress".into(),
        };
        session.advance(UserCmd {
            weapon: 1,
            ..Default::default()
        })?;
        Ok(session)
    }

    pub fn advance(&mut self, mut cmd: UserCmd) -> Result<(), String> {
        self.tick = self.tick.checked_add(1).ok_or("Session clock exhausted")?;
        cmd.server_time =
            i32::try_from(u64::from(self.tick) * 17).map_err(|_| "Session clock exhausted")?;
        cmd.weapon = 1;
        sim::try_step(
            &mut self.world,
            Tick(self.tick),
            &TickInput::from_cmds(vec![(LOCAL, cmd)]),
            17,
            sim::StepReason::AuthorityFrame,
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    fn view_ray(&self) -> Result<([f32; 3], [f32; 3]), String> {
        let player = self
            .world
            .player(LOCAL)
            .filter(|p| p.health > 0)
            .ok_or("Player is not alive")?;
        let mut start = player.origin;
        start[2] += player.view_height_current;
        let (pitch, yaw) = (
            player.viewangles[0].to_radians(),
            player.viewangles[1].to_radians(),
        );
        let direction = [
            pitch.cos() * yaw.cos(),
            pitch.cos() * yaw.sin(),
            -pitch.sin(),
        ];
        Ok((
            start,
            std::array::from_fn(|k| start[k] + direction[k] * 360.),
        ))
    }

    pub fn place_from_view(&mut self, kind: Kind, axis: u8) -> Result<u32, String> {
        let (start, end) = self.view_ray()?;
        let hit = self.world.trace_world(start, end, [0.; 3], [0.; 3], 1);
        if hit.fraction >= 1. || hit.startsolid != 0 {
            return Err("Aim at ground or a building within reach".into());
        }
        let anchor = self.world.buildings().anchor;
        let pos =
            std::array::from_fn::<_, 3, _>(|k| hit.endpos[k] + hit.normal[k] * 0.2 - anchor[k]);
        let deck = matches!(kind, Kind::Foundation | Kind::Floor);
        let axis = if deck { 0 } else { axis % 2 };
        let socket = Socket {
            x: if !deck && axis == 1 {
                (pos[0] / CELL).round() as i32
            } else {
                (pos[0] / CELL).floor() as i32
            },
            y: if !deck && axis == 0 {
                (pos[1] / CELL).round() as i32
            } else {
                (pos[1] / CELL).floor() as i32
            },
            level: if kind == Kind::Foundation {
                0
            } else {
                (pos[2] / WALL_HEIGHT).round().max(0.) as i32
            },
            axis,
        };
        let mut candidate = self.world.buildings().clone();
        let grounded = if kind == Kind::Foundation {
            [(0.1, 0.1), (0.9, 0.1), (0.1, 0.9), (0.9, 0.9)]
                .iter()
                .all(|&(x, y)| {
                    let p = [
                        anchor[0] + (socket.x as f32 + x) * CELL,
                        anchor[1] + (socket.y as f32 + y) * CELL,
                        anchor[2],
                    ];
                    let tr = self.world.trace_static_world(
                        [p[0], p[1], p[2] + 16.],
                        [p[0], p[1], p[2] - 16.],
                        [0.; 3],
                        [0.; 3],
                        1,
                    );
                    tr.fraction < 1.
                        && tr.startsolid == 0
                        && tr.normal[2] >= 0.7
                        && (tr.endpos[2] - p[2]).abs() <= 1.
                })
        } else {
            false
        };
        let id = candidate
            .place(LOCAL.0, kind, socket, grounded)
            .map_err(|e| e.to_string())?;
        if overlaps_players(&self.world, &candidate) {
            return Err("Building overlaps a player".into());
        }
        if kind != Kind::Foundation {
            let piece = candidate.piece(id).ok_or("Building was not created")?;
            for (lo, hi) in candidate.bounds(piece) {
                let center = std::array::from_fn(|k| (lo[k] + hi[k]) * 0.5);
                let mins = std::array::from_fn(|k| lo[k] - center[k] + 0.5);
                let maxs = std::array::from_fn(|k| hi[k] - center[k] - 0.5);
                if self
                    .world
                    .trace_static_world(center, center, mins, maxs, 1)
                    .startsolid
                    != 0
                {
                    return Err("Building overlaps terrain".into());
                }
            }
        }
        *self.world.buildings_mut() = candidate;
        Ok(id)
    }

    pub fn toggle_door_from_view(&mut self) -> Result<(), String> {
        let (start, end) = self.view_ray()?;
        let outcome = self.world.bullet_trace(
            sim::BulletTraceQuery {
                start,
                end,
                mask: 1,
                ignore: Some(LOCAL),
                ignore_hit: None,
                ignore_model: None,
            },
            None,
        );
        let sim::TraceOutcome::Hit {
            collider:
                sim::ColliderId::World {
                    building_id: Some(id),
                    ..
                },
            ..
        } = outcome
        else {
            return Err("Aim at a door within reach".into());
        };
        let mut candidate = self.world.buildings().clone();
        candidate
            .toggle_door(LOCAL.0, id)
            .map_err(|e| e.to_string())?;
        if overlaps_players(&self.world, &candidate) {
            return Err("Door cannot close through a player".into());
        }
        *self.world.buildings_mut() = candidate;
        Ok(())
    }

    pub fn save(&self, path: &Path) -> Result<(), String> {
        let scene = SavedScene {
            version: 1,
            seed: self.terrain.seed,
            buildings: String::from_utf8(
                self.world
                    .buildings()
                    .to_json()
                    .map_err(|e| e.to_string())?,
            )
            .map_err(|e| e.to_string())?,
            objects: self.editor.objects().cloned().collect(),
        };
        let data = serde_json::to_vec(&scene).map_err(|e| e.to_string())?;
        if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let pending = path.with_extension("pending");
        std::fs::write(&pending, data).map_err(|e| e.to_string())?;
        std::fs::rename(pending, path).map_err(|e| e.to_string())
    }

    pub fn load(&mut self, path: &Path) -> Result<(), String> {
        let mut bytes = Vec::new();
        std::fs::File::open(path)
            .map_err(|e| e.to_string())?
            .take((MAX_SCENE_BYTES + 1) as u64)
            .read_to_end(&mut bytes)
            .map_err(|e| e.to_string())?;
        if bytes.len() > MAX_SCENE_BYTES {
            return Err("Scene save too large".into());
        }
        let scene: SavedScene = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
        if scene.version != 1 || scene.seed != self.terrain.seed {
            return Err("Unsupported scene version or terrain seed".into());
        }
        let loaded =
            BuildingWorld::from_json(scene.buildings.as_bytes()).map_err(|e| e.to_string())?;
        let editor = EditorState::from_objects(scene.objects)?;
        if overlaps_players(&self.world, &loaded) {
            return Err("Saved buildings overlap a player".into());
        }
        if props_overlap_players(&self.world, &editor) {
            return Err("Saved editor objects overlap a player".into());
        }
        self.install_editor(editor)?;
        *self.world.buildings_mut() = loaded;
        Ok(())
    }
}

fn overlaps_players(world: &SimWorld, buildings: &BuildingWorld) -> bool {
    let mut overlaps = false;
    world.visit_players(|_, p| {
        if p.health > 0
            && buildings
                .trace(p.origin, p.origin, [-15., -15., 0.], [15., 15., 70.], 1)
                .startsolid
                != 0
        {
            overlaps = true;
        }
    });
    overlaps
}

const MAX_SCENE_BYTES: usize = 4 * 1024 * 1024;
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct SavedScene {
    version: u32,
    seed: u32,
    buildings: String,
    objects: Vec<PlacedObject>,
}

fn props_overlap_players(world: &SimWorld, editor: &EditorState) -> bool {
    let brushes = editor.brushes();
    let mut overlap = false;
    world.visit_players(|_, p| {
        if p.health <= 0 {
            return;
        }
        let center = [p.origin[0], p.origin[1], p.origin[2] + 35.];
        overlap |= brushes.iter().any(|brush| {
            brush.planes.iter().all(|plane| {
                let extent = plane[0].abs() * 15. + plane[1].abs() * 15. + plane[2].abs() * 35.;
                dot([plane[0], plane[1], plane[2]], center) - extent < plane[3] - 0.1
            })
        });
    });
    overlap
}

fn intersect(brush: &SimBrush, start: [f32; 3], end: [f32; 3]) -> Option<f32> {
    let mut enter: f32 = 0.;
    let mut leave: f32 = 1.;
    for plane in &brush.planes {
        let n = [plane[0], plane[1], plane[2]];
        let a = dot(n, start) - plane[3];
        let b = dot(n, end) - plane[3];
        if a > 0. && b > 0. {
            return None;
        }
        if a <= 0. && b <= 0. {
            continue;
        }
        let t = a / (a - b);
        if a > b {
            enter = enter.max(t);
        } else {
            leave = leave.min(t);
        }
        if enter > leave {
            return None;
        }
    }
    Some(enter)
}

fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}
fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    std::array::from_fn(|i| a[i] - b[i])
}
fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}
fn normalize(v: [f32; 3]) -> [f32; 3] {
    let length = dot(v, v).sqrt();
    v.map(|x| x / length)
}
fn triangle_prism(v: [[f32; 3]; 3], bottom: f32, flags: u32) -> SimBrush {
    let normal = normalize(cross(sub(v[1], v[0]), sub(v[2], v[0])));
    let mut planes = vec![
        [normal[0], normal[1], normal[2], dot(normal, v[0])],
        [0., 0., -1., -bottom],
    ];
    for i in 0..3 {
        let a = v[i];
        let b = v[(i + 1) % 3];
        let n = normalize([b[1] - a[1], a[0] - b[0], 0.]);
        planes.push([n[0], n[1], 0., dot(n, a)]);
    }
    SimBrush {
        plane_surface_flags: vec![flags; planes.len()],
        planes,
        contents: 1,
        glass_encoded: 0,
    }
}

fn authored_content(terrain: &Terrain, editor: &EditorState) -> std::sync::Arc<sim::SimContent> {
    let mut content = SimContentBuilder::default();
    let mut brushes = terrain.brushes();
    brushes.extend(editor.brushes());
    content.set_clip_brushes(brushes);
    content.set_weapon_combat_table(vec![
        weapon_iw4::WeaponCombatFacts::none(),
        weapon_iw4::WeaponCombatFacts {
            damage: 30,
            min_damage: 20,
            max_damage_range: 600.,
            min_damage_range: 2000.,
            fire_time_ms: 95,
            raise_time_ms: 250,
            drop_time_ms: 200,
            reload_time_ms: 2000,
            reload_empty_time_ms: 2400,
            reload_add_time_ms: 1600,
            reload_empty_add_time_ms: 1800,
            clip_size: 30,
            start_ammo: 120,
            max_ammo: 180,
            ammo_index: 1,
            clip_index: 1,
            shots_per_fire: 1,
            ads_in_rate: 0.005,
            ads_out_rate: 0.006,
            melee_damage: 45,
            hip_spread_stand_min: 1.5,
            hip_spread_stand_max: 4.,
            ..weapon_iw4::WeaponCombatFacts::none()
        },
    ]);
    content.set_weapon_runnable_table(vec![false, true]);
    content.set_weapon_script_names(vec![String::new(), "authored_carbine".into()]);

    content.finish()
}
