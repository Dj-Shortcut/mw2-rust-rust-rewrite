use serde::{Deserialize, Serialize};
use sim::SimBrush;
use std::collections::BTreeMap;

pub const MAX_PROPS: usize = 1024;

/// Local grind centerline of [`PropKind::Rail`]: the top of its beam.
const RAIL_START: [f32; 3] = [-120., 0., 50.];
const RAIL_END: [f32; 3] = [120., 0., 50.];
pub(crate) const RAIL_LENGTH: f32 = RAIL_END[0] - RAIL_START[0];

/// The grindable centerline of one authored rail, in world space. `id` is
/// the editor object ID, which stays stable across undo/redo and saves.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RailSegment {
    pub id: u32,
    pub start: [f32; 3],
    pub end: [f32; 3],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PropKind {
    Ramp,
    QuarterPipe,
    Rail,
    Stairs,
    Platform,
    Funbox,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PlacedObject {
    pub id: u32,
    pub kind: PropKind,
    pub position: [f32; 3],
    pub yaw: f32,
}

#[derive(Clone, Default)]
pub struct Geometry {
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
}

#[derive(Clone, Default)]
pub struct EditorState {
    objects: BTreeMap<u32, PlacedObject>,
    next_id: u32,
    undo: Vec<BTreeMap<u32, PlacedObject>>,
    redo: Vec<BTreeMap<u32, PlacedObject>>,
}

impl EditorState {
    pub fn objects(&self) -> impl Iterator<Item = &PlacedObject> {
        self.objects.values()
    }
    pub fn from_objects(objects: Vec<PlacedObject>) -> Result<Self, String> {
        if objects.len() > MAX_PROPS {
            return Err("Too many editor objects".into());
        }
        let mut state = Self::default();
        for object in objects {
            validate(&object)?;
            state.next_id = state.next_id.max(object.id);
            if state.objects.insert(object.id, object).is_some() {
                return Err("Duplicate editor object ID".into());
            }
        }
        Ok(state)
    }
    fn checkpoint(&mut self) {
        if self.undo.len() >= 64 {
            self.undo.remove(0);
        }
        self.undo.push(self.objects.clone());
        self.redo.clear();
    }
    pub fn placement_candidate(
        &self,
        kind: PropKind,
        position: [f32; 3],
        yaw: f32,
    ) -> Result<PlacedObject, String> {
        if self.objects.len() >= MAX_PROPS {
            return Err("Editor object limit reached".into());
        }
        let id = self.next_id.checked_add(1).ok_or("Editor IDs exhausted")?;
        let object = PlacedObject {
            id,
            kind,
            position,
            yaw: yaw.rem_euclid(360.),
        };
        validate(&object)?;
        Ok(object)
    }
    pub fn place(&mut self, kind: PropKind, position: [f32; 3], yaw: f32) -> Result<u32, String> {
        let object = self.placement_candidate(kind, position, yaw)?;
        let id = object.id;
        self.checkpoint();
        self.objects.insert(id, object);
        self.next_id = id;
        Ok(id)
    }
    pub fn remove(&mut self, id: u32) -> Result<(), String> {
        if !self.objects.contains_key(&id) {
            return Err("Object no longer exists".into());
        }
        self.checkpoint();
        self.objects.remove(&id);
        Ok(())
    }
    pub fn undo(&mut self) -> Result<(), String> {
        let previous = self.undo.pop().ok_or("Nothing to undo")?;
        self.redo
            .push(std::mem::replace(&mut self.objects, previous));
        Ok(())
    }
    pub fn redo(&mut self) -> Result<(), String> {
        let next = self.redo.pop().ok_or("Nothing to redo")?;
        self.undo.push(std::mem::replace(&mut self.objects, next));
        Ok(())
    }
    pub fn brushes(&self) -> Vec<SimBrush> {
        self.objects().flat_map(PlacedObject::brushes).collect()
    }
    pub fn rails(&self) -> Vec<RailSegment> {
        self.objects()
            .filter_map(PlacedObject::rail_segment)
            .collect()
    }
}

fn validate(object: &PlacedObject) -> Result<(), String> {
    if object.id == 0
        || !object.yaw.is_finite()
        || object.yaw.abs() > 360_000.
        || object
            .position
            .iter()
            .any(|v| !v.is_finite() || v.abs() > crate::WORLD_HALF)
    {
        return Err("Invalid editor object transform".into());
    }
    Ok(())
}

impl PropKind {
    pub const ALL: [Self; 6] = [
        Self::Ramp,
        Self::QuarterPipe,
        Self::Rail,
        Self::Stairs,
        Self::Platform,
        Self::Funbox,
    ];
    pub fn name(self) -> &'static str {
        match self {
            Self::Ramp => "Ramp",
            Self::QuarterPipe => "Quarterpipe",
            Self::Rail => "Rail",
            Self::Stairs => "Stairs",
            Self::Platform => "Platform",
            Self::Funbox => "Funbox",
        }
    }
    fn solids(self) -> Vec<([f32; 2], [f32; 2], [f32; 2])> {
        match self {
            Self::Ramp => vec![([-100., 100.], [-70., 70.], [2., 100.])],
            Self::QuarterPipe => (0..8)
                .map(|i| {
                    let height = |t: f32| 142. - (142. * 142. - t * t).max(0.).sqrt();
                    let a = i as f32 * 17.5;
                    let b = (i + 1) as f32 * 17.5;
                    (
                        [a - 70., b - 70.],
                        [-80., 80.],
                        [height(a).max(2.), height(b).max(2.)],
                    )
                })
                .collect(),
            Self::Rail => vec![
                ([-120., 120.], [-4., 4.], [50., 50.]),
                ([-90., -84.], [-6., 6.], [44., 44.]),
                ([84., 90.], [-6., 6.], [44., 44.]),
            ],
            Self::Stairs => (0..6)
                .map(|i| {
                    (
                        [-90. + i as f32 * 30., -60. + i as f32 * 30.],
                        [-65., 65.],
                        [12. * (i + 1) as f32; 2],
                    )
                })
                .collect(),
            Self::Platform => vec![([-100., 100.], [-80., 80.], [70., 70.])],
            Self::Funbox => vec![
                ([-160., -60.], [-80., 80.], [2., 70.]),
                ([-60., 60.], [-80., 80.], [70., 70.]),
                ([60., 160.], [-80., 80.], [70., 2.]),
            ],
        }
    }
    pub fn geometry(self) -> Geometry {
        let mut mesh = Geometry::default();
        for (x, y, z) in self.solids() {
            let bottom = if self == Self::Rail && z[0] == 50. {
                44.
            } else {
                0.
            };
            let v = [
                [x[0], y[0], bottom],
                [x[1], y[0], bottom],
                [x[1], y[1], bottom],
                [x[0], y[1], bottom],
                [x[0], y[0], z[0]],
                [x[1], y[0], z[1]],
                [x[1], y[1], z[1]],
                [x[0], y[1], z[0]],
            ];
            for face in [
                [0, 3, 2, 1],
                [4, 5, 6, 7],
                [0, 1, 5, 4],
                [1, 2, 6, 5],
                [2, 3, 7, 6],
                [3, 0, 4, 7],
            ] {
                let a = v[face[0]];
                let b = v[face[1]];
                let c = v[face[2]];
                let normal = crate::normalize(crate::cross(crate::sub(b, a), crate::sub(c, a)));
                let base = mesh.positions.len() as u32;
                for index in face {
                    mesh.positions.push(v[index]);
                    mesh.normals.push(normal);
                }
                mesh.indices
                    .extend([base, base + 1, base + 2, base, base + 2, base + 3]);
            }
        }
        mesh
    }
}

impl PlacedObject {
    /// The rail's centerline rotated by yaw around Z and translated by the
    /// object position; `None` for every other prop.
    pub fn rail_segment(&self) -> Option<RailSegment> {
        if self.kind != PropKind::Rail {
            return None;
        }
        let (sin, cos) = self.yaw.to_radians().sin_cos();
        let place = |p: [f32; 3]| {
            [
                p[0] * cos - p[1] * sin + self.position[0],
                p[0] * sin + p[1] * cos + self.position[1],
                p[2] + self.position[2],
            ]
        };
        Some(RailSegment {
            id: self.id,
            start: place(RAIL_START),
            end: place(RAIL_END),
        })
    }
    pub fn brushes(&self) -> Vec<SimBrush> {
        let (sin, cos) = self.yaw.to_radians().sin_cos();
        let mut solids: Vec<Vec<[f32; 4]>> = self
            .kind
            .solids()
            .into_iter()
            .map(|(x, y, z)| {
                let slope = (z[1] - z[0]) / (x[1] - x[0]);
                let bottom = if self.kind == PropKind::Rail && z[0] == 50. {
                    44.
                } else {
                    0.
                };
                vec![
                    [1., 0., 0., x[1]],
                    [-1., 0., 0., -x[0]],
                    [0., 1., 0., y[1]],
                    [0., -1., 0., -y[0]],
                    [-slope, 0., 1., z[0] - slope * x[0]],
                    [0., 0., -1., -bottom],
                ]
            })
            .collect();
        if self.kind == PropKind::Funbox {
            // The funbox profile (kicker, deck, kicker) is convex, so collide
            // with it as one brush. Three touching boxes leave the deck's end
            // faces as seams that a rider coming up a kicker runs into.
            let [up, deck, down] = [&solids[0], &solids[1], &solids[2]];
            solids = vec![vec![
                down[0], up[1], deck[2], deck[3], up[4], deck[4], down[4], deck[5],
            ]];
        }
        solids
            .into_iter()
            .map(|mut planes| {
                for plane in &mut planes {
                    let length =
                        (plane[0] * plane[0] + plane[1] * plane[1] + plane[2] * plane[2]).sqrt();
                    for value in plane.iter_mut() {
                        *value /= length;
                    }
                    let (nx, ny) = (
                        plane[0] * cos - plane[1] * sin,
                        plane[0] * sin + plane[1] * cos,
                    );
                    plane[0] = nx;
                    plane[1] = ny;
                    plane[3] +=
                        nx * self.position[0] + ny * self.position[1] + plane[2] * self.position[2];
                }
                SimBrush {
                    plane_surface_flags: vec![21 << 20; planes.len()],
                    planes,
                    contents: 1,
                    glass_encoded: 0,
                }
            })
            .collect()
    }
}
