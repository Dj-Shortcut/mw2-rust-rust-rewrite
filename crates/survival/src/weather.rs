use crate::crates::splitmix;
use serde::{Deserialize, Deserializer, Serialize};

pub const RAIN_CHILL_CELSIUS: f32 = 5.;
pub const MIN_SPELL_SECONDS: f32 = 120.;
pub const MAX_SPELL_SECONDS: f32 = 480.;
const RAIN_PERCENT: u64 = 30;
const FIRST_SPELL_SECONDS: f32 = 300.;

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct Weather {
    raining: bool,
    remaining: f32,
    spells: u32,
}

impl Default for Weather {
    fn default() -> Self {
        Self {
            raining: false,
            remaining: FIRST_SPELL_SECONDS,
            spells: 0,
        }
    }
}

impl Weather {
    pub fn is_raining(&self) -> bool {
        self.raining
    }

    pub fn remaining(&self) -> f32 {
        self.remaining
    }

    pub fn spells(&self) -> u32 {
        self.spells
    }

    pub fn chill(&self) -> f32 {
        if self.raining { RAIN_CHILL_CELSIUS } else { 0. }
    }

    pub fn forecast(&self, seed: u32) -> bool {
        spell(seed, self.spells.wrapping_add(1)).0
    }

    pub(crate) fn advance(&mut self, dt_seconds: f32, seed: u32) -> Result<bool, String> {
        if !dt_seconds.is_finite() || dt_seconds < 0. {
            return Err("Invalid weather time step".into());
        }
        let before = self.raining;
        let mut left = dt_seconds;
        while left >= self.remaining {
            left -= self.remaining;
            self.spells = self.spells.wrapping_add(1);
            (self.raining, self.remaining) = spell(seed, self.spells);
        }
        self.remaining -= left;
        Ok(self.raining != before)
    }
}

fn spell(seed: u32, spells: u32) -> (bool, f32) {
    let mut state = (u64::from(seed) << 32) ^ u64::from(spells) ^ 0x5EA7_0000;
    let roll = splitmix(&mut state);
    let span = (MAX_SPELL_SECONDS - MIN_SPELL_SECONDS) as u64 + 1;
    (
        roll % 100 < RAIN_PERCENT,
        MIN_SPELL_SECONDS + ((roll >> 8) % span) as f32,
    )
}

impl<'de> Deserialize<'de> for Weather {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Saved {
            raining: bool,
            remaining: f32,
            spells: u32,
        }
        let saved = Saved::deserialize(deserializer)?;
        if !saved.remaining.is_finite()
            || !(saved.remaining > 0. && saved.remaining <= MAX_SPELL_SECONDS)
        {
            return Err(serde::de::Error::custom("Weather spell is out of range"));
        }
        Ok(Self {
            raining: saved.raining,
            remaining: saved.remaining,
            spells: saved.spells,
        })
    }
}
