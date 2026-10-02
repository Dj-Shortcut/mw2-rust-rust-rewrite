use crate::{Geometry, WORLD_HALF};
use sim::SimBrush;

pub const TERRAIN_SEED: u32 = 731;
const CELLS: usize = 32;

pub struct Terrain {
    pub seed: u32,
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub colors: Vec<[f32; 4]>,
    pub indices: Vec<u32>,
}

impl Terrain {
    pub fn new(seed: u32) -> Self {
        let mut geometry = Geometry::default();
        let mut colors = Vec::new();
        for y in 0..=CELLS {
            for x in 0..=CELLS {
                let px = x as f32 * 2. * WORLD_HALF / CELLS as f32 - WORLD_HALF;
                let py = y as f32 * 2. * WORLD_HALF / CELLS as f32 - WORLD_HALF;
                let z = sample(seed, px, py);
                geometry.positions.push([px, py, z]);
                let dx = (sample(seed, px + 5., py) - sample(seed, px - 5., py)) / 10.;
                let dy = (sample(seed, px, py + 5.) - sample(seed, px, py - 5.)) / 10.;
                geometry.normals.push(crate::normalize([-dx, -dy, 1.]));
                colors.push(if z < -35. {
                    [0.55, 0.48, 0.31, 1.]
                } else if z > 120. {
                    [0.32, 0.34, 0.31, 1.]
                } else {
                    [0.20, 0.35, 0.12, 1.]
                });
            }
        }
        for y in 0..CELLS {
            for x in 0..CELLS {
                let a = (y * (CELLS + 1) + x) as u32;
                let b = a + 1;
                let c = a + (CELLS + 1) as u32;
                geometry.indices.extend([a, b, c, b, c + 1, c]);
            }
        }
        Self {
            seed,
            positions: geometry.positions,
            normals: geometry.normals,
            colors,
            indices: geometry.indices,
        }
    }

    pub fn brushes(&self) -> Vec<SimBrush> {
        self.indices
            .chunks_exact(3)
            .map(|t| {
                crate::triangle_prism(
                    [
                        self.positions[t[0] as usize],
                        self.positions[t[1] as usize],
                        self.positions[t[2] as usize],
                    ],
                    -512.,
                    6 << 20,
                )
            })
            .collect()
    }
}

fn sample(seed: u32, x: f32, y: f32) -> f32 {
    let radius = x.hypot(y);
    let inland = (1. - radius / (WORLD_HALF * 1.1)).clamp(0., 1.);
    let phase = seed as f32 * 0.013;
    let hills = (x * 0.0015 + phase).sin() * (y * 0.0012 - phase).cos() * 170.
        + (x * 0.003 + y * 0.002 + phase).sin() * 35.;
    let height = hills * inland + 100. * inland - 90.;
    height * ((radius - 1000.) / 600.).clamp(0., 1.)
}
