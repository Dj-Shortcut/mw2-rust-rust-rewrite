use crate::Terrain;
use crate::gathering::height_at;
use serde::{Deserialize, Deserializer, Serialize};

pub const BARREL_REACH: f32 = 100.;
pub const BARREL_WATER: u32 = 5;
pub const BARREL_FILL_SECONDS: f32 = 60.;
const POSITION: [f32; 2] = [-140., -260.];

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct SavedBarrel {
    water: u32,
    remaining: f32,
}

impl Default for SavedBarrel {
    fn default() -> Self {
        Self {
            water: 0,
            remaining: BARREL_FILL_SECONDS,
        }
    }
}

impl<'de> Deserialize<'de> for SavedBarrel {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Saved {
            water: u32,
            remaining: f32,
        }
        let saved = Saved::deserialize(deserializer)?;
        if saved.water > BARREL_WATER {
            return Err(serde::de::Error::custom("Saved rain barrel holds too much"));
        }
        if !saved.remaining.is_finite()
            || saved.remaining <= 0.
            || saved.remaining > BARREL_FILL_SECONDS
        {
            return Err(serde::de::Error::custom(
                "Saved rain barrel timer is out of range",
            ));
        }
        Ok(Self {
            water: saved.water,
            remaining: saved.remaining,
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct RainBarrel {
    position: [f32; 3],
    state: SavedBarrel,
}

impl RainBarrel {
    pub(crate) fn new(terrain: &Terrain) -> Result<Self, String> {
        let z = height_at(terrain, POSITION).ok_or("Rain barrel lies outside terrain mesh")?;
        Ok(Self {
            position: [POSITION[0], POSITION[1], z],
            state: SavedBarrel::default(),
        })
    }

    pub fn position(&self) -> [f32; 3] {
        self.position
    }

    pub fn water(&self) -> u32 {
        self.state.water
    }

    pub fn remaining(&self) -> f32 {
        self.state.remaining
    }

    pub fn in_reach(&self, origin: [f32; 3]) -> bool {
        (0..3)
            .map(|k| (self.position[k] - origin[k]).powi(2))
            .sum::<f32>()
            .sqrt()
            <= BARREL_REACH
    }

    pub(crate) fn advance(&mut self, dt_seconds: f32, raining: bool) -> Result<(), String> {
        if !dt_seconds.is_finite() || dt_seconds < 0. {
            return Err("Rain barrel time step must be finite and non-negative".into());
        }
        let state = &mut self.state;
        if state.water >= BARREL_WATER {
            state.remaining = BARREL_FILL_SECONDS;
            return Ok(());
        }
        if !raining {
            return Ok(());
        }
        state.remaining -= dt_seconds;
        if state.remaining <= 0. {
            state.water += 1;
            state.remaining = BARREL_FILL_SECONDS;
        }
        Ok(())
    }

    pub(crate) fn take_water(&mut self) -> Result<u32, String> {
        if self.state.water == 0 {
            return Err("The rain barrel is empty".into());
        }
        Ok(std::mem::take(&mut self.state.water))
    }

    pub(crate) fn saved(&self) -> SavedBarrel {
        self.state
    }

    pub(crate) fn restore(&mut self, saved: SavedBarrel) {
        self.state = saved;
    }
}
