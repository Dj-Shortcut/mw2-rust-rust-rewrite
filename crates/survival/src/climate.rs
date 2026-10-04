use serde::{Deserialize, Deserializer, Serialize};

pub const DAY_SECONDS: f64 = 1800.;
pub const COLD_CELSIUS: f32 = 5.;
pub const FREEZING_CELSIUS: f32 = 0.;
const START_HOUR: f64 = 9.;
const WARMEST_HOUR: f64 = 14.;
const MEAN_CELSIUS: f64 = 9.;
const SWING_CELSIUS: f64 = 13.;

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(transparent)]
pub struct WorldClock {
    seconds: f64,
}

impl Default for WorldClock {
    fn default() -> Self {
        Self {
            seconds: START_HOUR / 24. * DAY_SECONDS,
        }
    }
}

impl WorldClock {
    pub fn hour(&self) -> f32 {
        (self.seconds / DAY_SECONDS * 24.) as f32
    }

    pub fn is_night(&self) -> bool {
        let hour = self.hour();
        !(6. ..20.).contains(&hour)
    }

    pub fn is_twilight(&self) -> bool {
        let hour = self.hour();
        (5. ..7.).contains(&hour) || (19. ..21.).contains(&hour)
    }

    pub fn temperature(&self) -> f32 {
        let hour = self.seconds / DAY_SECONDS * 24.;
        let phase = (hour - WARMEST_HOUR) / 24. * std::f64::consts::TAU;
        (MEAN_CELSIUS + SWING_CELSIUS * phase.cos()) as f32
    }

    pub(crate) fn advance(&mut self, dt_seconds: f32) -> Result<bool, String> {
        if !dt_seconds.is_finite() || dt_seconds < 0. {
            return Err("Invalid clock time step".into());
        }
        let night = self.is_night();
        self.seconds = (self.seconds + f64::from(dt_seconds)).rem_euclid(DAY_SECONDS);
        Ok(night != self.is_night())
    }
}

impl<'de> Deserialize<'de> for WorldClock {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let seconds = f64::deserialize(deserializer)?;
        if !seconds.is_finite() || !(0. ..DAY_SECONDS).contains(&seconds) {
            return Err(serde::de::Error::custom("World clock is out of range"));
        }
        Ok(Self { seconds })
    }
}
