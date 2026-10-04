use crate::Terrain;
use crate::gathering::height_at;
use serde::{Deserialize, Deserializer, Serialize};

pub const HIVE_REACH: f32 = 100.;
pub const HIVE_HONEY: u32 = 5;
pub const HIVE_SECONDS: f32 = 90.;
pub const BEE_STING_DAMAGE: u32 = 5;
const POSITION: [f32; 2] = [460., -220.];

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct SavedHive {
    honey: u32,
    remaining: f32,
}

impl Default for SavedHive {
    fn default() -> Self {
        Self {
            honey: 0,
            remaining: HIVE_SECONDS,
        }
    }
}

impl<'de> Deserialize<'de> for SavedHive {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Saved {
            honey: u32,
            remaining: f32,
        }
        let saved = Saved::deserialize(deserializer)?;
        if saved.honey > HIVE_HONEY {
            return Err(serde::de::Error::custom("Saved beehive holds too much"));
        }
        if !saved.remaining.is_finite() || saved.remaining <= 0. || saved.remaining > HIVE_SECONDS {
            return Err(serde::de::Error::custom(
                "Saved beehive timer is out of range",
            ));
        }
        Ok(Self {
            honey: saved.honey,
            remaining: saved.remaining,
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Beehive {
    position: [f32; 3],
    state: SavedHive,
}

impl Beehive {
    pub(crate) fn new(terrain: &Terrain) -> Result<Self, String> {
        let z = height_at(terrain, POSITION).ok_or("Beehive lies outside terrain mesh")?;
        Ok(Self {
            position: [POSITION[0], POSITION[1], z],
            state: SavedHive::default(),
        })
    }

    pub fn position(&self) -> [f32; 3] {
        self.position
    }

    pub fn honey(&self) -> u32 {
        self.state.honey
    }

    pub fn remaining(&self) -> f32 {
        self.state.remaining
    }

    pub fn in_reach(&self, origin: [f32; 3]) -> bool {
        (0..3)
            .map(|k| (self.position[k] - origin[k]).powi(2))
            .sum::<f32>()
            .sqrt()
            <= HIVE_REACH
    }

    pub(crate) fn advance(&mut self, dt_seconds: f32, resting: bool) -> Result<(), String> {
        if !dt_seconds.is_finite() || dt_seconds < 0. {
            return Err("Beehive time step must be finite and non-negative".into());
        }
        let state = &mut self.state;
        if state.honey >= HIVE_HONEY {
            state.remaining = HIVE_SECONDS;
            return Ok(());
        }
        if resting {
            return Ok(());
        }
        state.remaining -= dt_seconds;
        if state.remaining <= 0. {
            state.honey += 1;
            state.remaining = HIVE_SECONDS;
        }
        Ok(())
    }

    pub(crate) fn take_honey(&mut self) -> Result<u32, String> {
        if self.state.honey == 0 {
            return Err("The beehive is empty".into());
        }
        Ok(std::mem::take(&mut self.state.honey))
    }

    pub(crate) fn saved(&self) -> SavedHive {
        self.state
    }

    pub(crate) fn restore(&mut self, saved: SavedHive) {
        self.state = saved;
    }
}
