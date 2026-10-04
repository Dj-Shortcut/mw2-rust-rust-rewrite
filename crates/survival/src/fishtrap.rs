use crate::Terrain;
use crate::gathering::height_at;
use serde::{Deserialize, Deserializer, Serialize};

pub const TRAP_REACH: f32 = 100.;
pub const TRAP_BAIT: u32 = 5;
pub const TRAP_FISH: u32 = 5;
pub const TRAP_CATCH_SECONDS: f32 = 120.;
pub const RAIN_TRAP_SPEED: f32 = 1.5;
const POSITION: [f32; 2] = [-320., -100.];

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct SavedTrap {
    bait: u32,
    fish: u32,
    remaining: f32,
}

impl Default for SavedTrap {
    fn default() -> Self {
        Self {
            bait: 0,
            fish: 0,
            remaining: TRAP_CATCH_SECONDS,
        }
    }
}

impl<'de> Deserialize<'de> for SavedTrap {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Saved {
            bait: u32,
            fish: u32,
            remaining: f32,
        }
        let saved = Saved::deserialize(deserializer)?;
        if saved.bait > TRAP_BAIT || saved.fish > TRAP_FISH {
            return Err(serde::de::Error::custom("Saved fish trap holds too much"));
        }
        if !saved.remaining.is_finite()
            || saved.remaining <= 0.
            || saved.remaining > TRAP_CATCH_SECONDS
        {
            return Err(serde::de::Error::custom(
                "Saved fish trap timer is out of range",
            ));
        }
        Ok(Self {
            bait: saved.bait,
            fish: saved.fish,
            remaining: saved.remaining,
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct FishTrap {
    position: [f32; 3],
    state: SavedTrap,
}

impl FishTrap {
    pub(crate) fn new(terrain: &Terrain) -> Result<Self, String> {
        let z = height_at(terrain, POSITION).ok_or("Fish trap lies outside terrain mesh")?;
        Ok(Self {
            position: [POSITION[0], POSITION[1], z],
            state: SavedTrap::default(),
        })
    }

    pub fn position(&self) -> [f32; 3] {
        self.position
    }

    pub fn bait(&self) -> u32 {
        self.state.bait
    }

    pub fn fish(&self) -> u32 {
        self.state.fish
    }

    pub fn remaining(&self) -> f32 {
        self.state.remaining
    }

    pub fn in_reach(&self, origin: [f32; 3]) -> bool {
        (0..3)
            .map(|k| (self.position[k] - origin[k]).powi(2))
            .sum::<f32>()
            .sqrt()
            <= TRAP_REACH
    }

    pub(crate) fn advance(
        &mut self,
        dt_seconds: f32,
        lively: bool,
        frost: bool,
    ) -> Result<bool, String> {
        if !dt_seconds.is_finite() || dt_seconds < 0. {
            return Err("Fish trap time step must be finite and non-negative".into());
        }
        let state = &mut self.state;
        if state.bait == 0 || state.fish >= TRAP_FISH {
            state.remaining = TRAP_CATCH_SECONDS;
            return Ok(false);
        }
        if frost {
            return Ok(false);
        }
        state.remaining -= if lively {
            dt_seconds * RAIN_TRAP_SPEED
        } else {
            dt_seconds
        };
        if state.remaining > 0. {
            return Ok(false);
        }
        state.bait -= 1;
        state.fish += 1;
        state.remaining = TRAP_CATCH_SECONDS;
        Ok(true)
    }

    pub(crate) fn stock(&mut self, quantity: u32) -> Result<(), String> {
        if quantity == 0 {
            return Err("Stock at least one bait".into());
        }
        if self.state.bait + quantity > TRAP_BAIT {
            return Err(format!("The fish trap holds at most {TRAP_BAIT} bait"));
        }
        self.state.bait += quantity;
        Ok(())
    }

    pub(crate) fn take_fish(&mut self) -> Result<u32, String> {
        if self.state.fish == 0 {
            return Err("The fish trap is empty".into());
        }
        Ok(std::mem::take(&mut self.state.fish))
    }

    pub(crate) fn saved(&self) -> SavedTrap {
        self.state
    }

    pub(crate) fn restore(&mut self, saved: SavedTrap) {
        self.state = saved;
    }
}
