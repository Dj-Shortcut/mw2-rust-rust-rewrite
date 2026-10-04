use crate::crates::splitmix;
use serde::{Deserialize, Deserializer, Serialize};

pub const CAST_SECONDS: f32 = 6.;
pub const FISHING_REACH: f32 = 300.;
pub const CATCH_PERCENT: u64 = 60;
pub const BAITED_CATCH_PERCENT: u64 = 90;
pub const RAIN_CATCH_PERCENT: u64 = 80;
pub const RAIN_BAITED_CATCH_PERCENT: u64 = 95;
pub const BAIT_PER_FOOD: u32 = 4;

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct Cast {
    pub node: u32,
    pub remaining: f32,
    pub baited: bool,
}

impl<'de> Deserialize<'de> for Cast {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Saved {
            node: u32,
            remaining: f32,
            #[serde(default)]
            baited: bool,
        }
        let saved = Saved::deserialize(deserializer)?;
        if !saved.remaining.is_finite() || !(0. ..=CAST_SECONDS).contains(&saved.remaining) {
            return Err(serde::de::Error::custom(
                "Fishing cast time is out of range",
            ));
        }
        Ok(Self {
            node: saved.node,
            remaining: saved.remaining,
            baited: saved.baited,
        })
    }
}

pub(crate) fn bites(seed: u32, node: u32, casts: u32, baited: bool, feeding: bool) -> bool {
    let mut state = (u64::from(seed) << 32) ^ (u64::from(node) << 20) ^ u64::from(casts) ^ 0xF15F;
    let percent = match (baited, feeding) {
        (false, false) => CATCH_PERCENT,
        (true, false) => BAITED_CATCH_PERCENT,
        (false, true) => RAIN_CATCH_PERCENT,
        (true, true) => RAIN_BAITED_CATCH_PERCENT,
    };
    splitmix(&mut state) % 100 < percent
}
