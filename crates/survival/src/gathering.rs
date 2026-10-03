use crate::{Terrain, WORLD_HALF};
use rust_building::Resources;
use serde::{Deserialize, Serialize};
use sim::SimBrush;
use std::collections::BTreeMap;

pub const HARVEST_REACH: f32 = 120.;
pub const MAX_RESOURCE_NODES: usize = 64;
pub const REGROW_SECONDS: f32 = 300.;
pub const REGROW_RETRY_SECONDS: f32 = 5.;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResourceKind {
    Tree,
    Stone,
    Metal,
    Berry,
    Water,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ResourceCanopy {
    pub center: [f32; 3],
    pub radius: f32,
    pub color: [f32; 4],
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ResourceVisual {
    pub center: [f32; 3],
    pub half_extents: [f32; 3],
    pub color: [f32; 4],
    pub canopy: Option<ResourceCanopy>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResourceNode {
    pub id: u32,
    pub kind: ResourceKind,
    pub position: [f32; 3],
    pub remaining: u32,
    #[serde(default)]
    pub regrow_in: f32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "GatheringSave", into = "GatheringSave")]
pub struct GatheringWorld {
    seed: u32,
    nodes: BTreeMap<u32, ResourceNode>,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct GatheringSave {
    seed: u32,
    nodes: Vec<ResourceNode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Harvest {
    pub node_id: u32,
    pub kind: ResourceKind,
    pub amount: u32,
    pub remaining: u32,
}

impl ResourceKind {
    pub fn name(self) -> &'static str {
        match self {
            Self::Tree => "Tree",
            Self::Stone => "Stone",
            Self::Metal => "Metal ore",
            Self::Berry => "Berries",
            Self::Water => "Water",
        }
    }

    pub fn capacity(self) -> u32 {
        match self {
            Self::Tree | Self::Stone => 300,
            Self::Metal => 150,
            Self::Berry => 20,
            Self::Water => 300,
        }
    }

    pub fn harvest_amount(self) -> u32 {
        match self {
            Self::Tree | Self::Stone => 25,
            Self::Metal => 15,
            Self::Berry => 2,
            Self::Water => 10,
        }
    }

    pub fn is_solid(self) -> bool {
        matches!(self, Self::Tree | Self::Stone | Self::Metal)
    }

    pub fn visual(self) -> ResourceVisual {
        match self {
            Self::Tree => ResourceVisual {
                center: [0., 0., 80.],
                half_extents: [12., 12., 80.],
                color: [0.32, 0.17, 0.07, 1.],
                canopy: Some(ResourceCanopy {
                    center: [0., 0., 160.],
                    radius: 65.,
                    color: [0.10, 0.27, 0.08, 1.],
                }),
            },
            Self::Stone => ResourceVisual {
                center: [0., 0., 25.],
                half_extents: [36., 28., 25.],
                color: [0.39, 0.41, 0.40, 1.],
                canopy: None,
            },
            Self::Metal => ResourceVisual {
                center: [0., 0., 24.],
                half_extents: [30., 27., 24.],
                color: [0.48, 0.35, 0.23, 1.],
                canopy: None,
            },
            Self::Berry => ResourceVisual {
                center: [0., 0., 17.5],
                half_extents: [25., 25., 17.5],
                color: [0.34, 0.12, 0.35, 1.],
                canopy: None,
            },
            Self::Water => ResourceVisual {
                center: [0., 0., 4.],
                half_extents: [30., 30., 4.],
                color: [0.13, 0.43, 0.66, 1.],
                canopy: None,
            },
        }
    }

    fn surface_flags(self) -> u32 {
        match self {
            Self::Tree => 21 << 20,
            Self::Metal => 13 << 20,
            _ => 17 << 20,
        }
    }
}

impl ResourceNode {
    pub fn bounds(&self) -> ([f32; 3], [f32; 3]) {
        let visual = self.kind.visual();
        (
            std::array::from_fn(|k| self.position[k] + visual.center[k] - visual.half_extents[k]),
            std::array::from_fn(|k| self.position[k] + visual.center[k] + visual.half_extents[k]),
        )
    }

    pub fn brush(&self) -> Option<SimBrush> {
        if self.id == 0
            || self.remaining == 0
            || self.remaining > self.kind.capacity()
            || !self.kind.is_solid()
        {
            return None;
        }
        let (lo, hi) = self.bounds();
        if lo
            .iter()
            .chain(hi.iter())
            .any(|v| !v.is_finite() || v.abs() > WORLD_HALF)
        {
            return None;
        }
        Some(SimBrush {
            planes: vec![
                [1., 0., 0., hi[0]],
                [-1., 0., 0., -lo[0]],
                [0., 1., 0., hi[1]],
                [0., -1., 0., -lo[1]],
                [0., 0., 1., hi[2]],
                [0., 0., -1., -lo[2]],
            ],
            contents: 1,
            plane_surface_flags: vec![self.kind.surface_flags(); 6],
            glass_encoded: 0,
        })
    }
}

impl Harvest {
    pub fn resources(self) -> Resources {
        let mut resources = Resources::default();
        match self.kind {
            ResourceKind::Tree => resources.wood = self.amount,
            ResourceKind::Stone => resources.stone = self.amount,
            ResourceKind::Metal => resources.metal = self.amount,
            ResourceKind::Berry | ResourceKind::Water => {}
        }
        resources
    }
}

impl GatheringWorld {
    pub fn new(terrain: &Terrain) -> Result<Self, String> {
        Ok(Self {
            seed: terrain.seed,
            nodes: initial_nodes(terrain)?,
        })
    }

    pub fn seed(&self) -> u32 {
        self.seed
    }

    pub fn nodes(&self) -> impl Iterator<Item = &ResourceNode> {
        self.nodes.values()
    }

    pub fn node(&self, id: u32) -> Option<&ResourceNode> {
        self.nodes.get(&id)
    }

    pub fn brushes(&self) -> Vec<SimBrush> {
        self.nodes
            .values()
            .filter_map(ResourceNode::brush)
            .collect()
    }

    pub fn from_nodes(seed: u32, nodes: Vec<ResourceNode>) -> Result<Self, String> {
        if nodes.len() > MAX_RESOURCE_NODES {
            return Err("Too many resource nodes".into());
        }
        let terrain = Terrain::new(seed);
        let mut world = Self::new(&terrain)?;
        if nodes.len() != world.nodes.len() {
            return Err("Resource node layout is incomplete".into());
        }
        let mut loaded = BTreeMap::new();
        for mut node in nodes {
            let expected = world
                .nodes
                .get(&node.id)
                .ok_or("Unknown resource node ID")?;
            if node.kind != expected.kind
                || node.remaining > node.kind.capacity()
                || node
                    .position
                    .iter()
                    .zip(expected.position)
                    .any(|(a, b)| !a.is_finite() || a.abs() > WORLD_HALF || (*a - b).abs() > 0.01)
                || !(0. ..=REGROW_SECONDS).contains(&node.regrow_in)
                || node.remaining > 0 && node.regrow_in != 0.
            {
                return Err("Invalid resource node state".into());
            }
            if node.remaining == 0 && node.regrow_in == 0. {
                node.regrow_in = REGROW_SECONDS;
            }
            let id = node.id;
            if loaded.insert(id, node).is_some() {
                return Err("Duplicate resource node ID".into());
            }
        }
        world.nodes = loaded;
        Ok(world)
    }

    pub fn target_from_ray(
        &self,
        start: [f32; 3],
        end: [f32; 3],
        world_obstacle_fraction: f32,
    ) -> Result<Option<&ResourceNode>, String> {
        if !world_obstacle_fraction.is_finite()
            || !(0. ..=1.).contains(&world_obstacle_fraction)
            || start
                .iter()
                .chain(end.iter())
                .any(|v| !v.is_finite() || v.abs() > WORLD_HALF + 1000.)
        {
            return Err("Invalid gathering ray".into());
        }
        let delta = std::array::from_fn::<_, 3, _>(|k| end[k] - start[k]);
        let length = delta.iter().map(|v| v * v).sum::<f32>().sqrt();
        if length <= 0.001 {
            return Err("Gathering ray is too short".into());
        }
        let reach_fraction = (HARVEST_REACH / length).min(1.);
        Ok(self
            .nodes
            .values()
            .filter(|node| node.remaining > 0)
            .filter_map(|node| {
                ray_box(start, delta, node.bounds())
                    .filter(|(entry, clipped)| {
                        let visible_entry = if node.kind.is_solid() {
                            *clipped
                        } else {
                            *entry
                        };
                        *entry <= reach_fraction
                            && visible_entry <= world_obstacle_fraction + 0.000001
                    })
                    .map(|(entry, _)| (entry, node))
            })
            .min_by(|a, b| a.0.total_cmp(&b.0).then(a.1.id.cmp(&b.1.id)))
            .map(|(_, node)| node))
    }

    pub fn harvest_from_ray(
        &mut self,
        start: [f32; 3],
        end: [f32; 3],
        world_obstacle_fraction: f32,
    ) -> Result<Harvest, String> {
        let id = self
            .target_from_ray(start, end, world_obstacle_fraction)?
            .map(|node| node.id)
            .ok_or("Aim at a resource within reach")?;
        let node = self.nodes.get_mut(&id).ok_or("Resource no longer exists")?;
        let amount = node.remaining.min(node.kind.harvest_amount());
        node.remaining -= amount;
        if node.remaining == 0 {
            node.regrow_in = REGROW_SECONDS;
        }
        Ok(Harvest {
            node_id: node.id,
            kind: node.kind,
            amount,
            remaining: node.remaining,
        })
    }
}

impl GatheringWorld {
    pub(crate) fn advance(
        &mut self,
        dt_seconds: f32,
        mut blocked: impl FnMut(&ResourceNode) -> bool,
    ) -> Vec<u32> {
        let mut regrown = Vec::new();
        for node in self.nodes.values_mut().filter(|n| n.remaining == 0) {
            node.regrow_in = (node.regrow_in - dt_seconds).max(0.);
            if node.regrow_in > 0. {
                continue;
            }
            let mut full = node.clone();
            full.remaining = full.kind.capacity();
            if full.kind.is_solid() && blocked(&full) {
                node.regrow_in = REGROW_RETRY_SECONDS;
            } else {
                full.regrow_in = 0.;
                *node = full;
                regrown.push(node.id);
            }
        }
        regrown
    }
}

impl TryFrom<GatheringSave> for GatheringWorld {
    type Error = String;

    fn try_from(save: GatheringSave) -> Result<Self, Self::Error> {
        Self::from_nodes(save.seed, save.nodes)
    }
}

impl From<GatheringWorld> for GatheringSave {
    fn from(world: GatheringWorld) -> Self {
        Self {
            seed: world.seed,
            nodes: world.nodes.into_values().collect(),
        }
    }
}

fn initial_nodes(terrain: &Terrain) -> Result<BTreeMap<u32, ResourceNode>, String> {
    let mut layout = vec![
        (ResourceKind::Tree, [220., 220.]),
        (ResourceKind::Stone, [-210., 160.]),
        (ResourceKind::Metal, [290., -220.]),
        (ResourceKind::Berry, [-230., -210.]),
        (ResourceKind::Water, [-360., -100.]),
        (ResourceKind::Tree, [710., 230.]),
    ];
    let kinds = [
        ResourceKind::Tree,
        ResourceKind::Stone,
        ResourceKind::Metal,
        ResourceKind::Berry,
        ResourceKind::Water,
    ];
    let mut random = terrain.seed;
    for ring in 0..3 {
        for sector in 0..8 {
            random = random.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            let jitter = ((random >> 16) % 181) as f32 - 90.;
            let radius = [1350., 2200., 3150.][ring] + jitter;
            let angle =
                (sector as f32 * 45. + ring as f32 * 13. + (terrain.seed % 31) as f32).to_radians();
            let (sin, cos) = angle.sin_cos();
            layout.push((
                kinds[(sector + ring * 2) % kinds.len()],
                [cos * radius, sin * radius],
            ));
        }
    }
    let mut nodes = BTreeMap::new();
    for (index, (kind, xy)) in layout.into_iter().enumerate() {
        let z = height_at(terrain, xy).ok_or("Resource node lies outside terrain mesh")?;
        let id = index as u32 + 1;
        let node = ResourceNode {
            id,
            kind,
            position: [xy[0], xy[1], z],
            remaining: kind.capacity(),
            regrow_in: 0.,
        };
        let (lo, hi) = node.bounds();
        if lo
            .iter()
            .chain(hi.iter())
            .any(|v| !v.is_finite() || v.abs() > WORLD_HALF)
        {
            return Err("Invalid resource node bounds".into());
        }
        nodes.insert(id, node);
    }
    Ok(nodes)
}

fn height_at(terrain: &Terrain, xy: [f32; 2]) -> Option<f32> {
    for indices in terrain.indices.chunks_exact(3) {
        let a = *terrain.positions.get(indices[0] as usize)?;
        let b = *terrain.positions.get(indices[1] as usize)?;
        let c = *terrain.positions.get(indices[2] as usize)?;
        let denominator = (b[1] - c[1]) * (a[0] - c[0]) + (c[0] - b[0]) * (a[1] - c[1]);
        if !denominator.is_finite() || denominator.abs() < 0.001 {
            continue;
        }
        let wa = ((b[1] - c[1]) * (xy[0] - c[0]) + (c[0] - b[0]) * (xy[1] - c[1])) / denominator;
        let wb = ((c[1] - a[1]) * (xy[0] - c[0]) + (a[0] - c[0]) * (xy[1] - c[1])) / denominator;
        let wc = 1. - wa - wb;
        if wa >= -0.00001 && wb >= -0.00001 && wc >= -0.00001 {
            let z = wa * a[2] + wb * b[2] + wc * c[2];
            return z.is_finite().then_some(z);
        }
    }
    None
}

fn ray_box(start: [f32; 3], delta: [f32; 3], (lo, hi): ([f32; 3], [f32; 3])) -> Option<(f32, f32)> {
    let mut enter: f32 = 0.;
    let mut clipped_enter: f32 = 0.;
    let mut leave: f32 = 1.;
    for k in 0..3 {
        if delta[k].abs() <= 0.000001 {
            if start[k] < lo[k] || start[k] > hi[k] {
                return None;
            }
        } else {
            let a = (lo[k] - start[k]) / delta[k];
            let b = (hi[k] - start[k]) / delta[k];
            enter = enter.max(a.min(b));
            clipped_enter = clipped_enter.max(a.min(b) - 0.125 / delta[k].abs());
            leave = leave.min(a.max(b));
            if enter > leave {
                return None;
            }
        }
    }
    Some((enter, clipped_enter))
}
