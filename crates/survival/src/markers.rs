use crate::WORLD_HALF;
use serde::{Deserialize, Deserializer, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MarkerKind {
    Campfire,
    GardenPlot,
    TradingPost,
    LootCrate,
    LockedCrate,
    SupplyDrop,
    LootBag,
    Waypoint,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Marker {
    pub kind: MarkerKind,
    pub position: [f32; 3],
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(transparent)]
pub struct Waypoint {
    position: [f32; 2],
}

impl Waypoint {
    pub fn new(position: [f32; 2]) -> Result<Self, String> {
        if position
            .iter()
            .any(|v| !v.is_finite() || v.abs() > WORLD_HALF)
        {
            return Err("Waypoint is outside the world".into());
        }
        Ok(Self { position })
    }

    pub fn position(&self) -> [f32; 2] {
        self.position
    }
}

impl<'de> Deserialize<'de> for Waypoint {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Self::new(<[f32; 2]>::deserialize(deserializer)?).map_err(serde::de::Error::custom)
    }
}

// rem_euclid can round a tiny negative angle up to exactly 360.
fn wrap_degrees(angle: f32) -> f32 {
    let wrapped = angle.rem_euclid(360.);
    if wrapped >= 360. { 0. } else { wrapped }
}

pub fn compass_heading(yaw_degrees: f32) -> f32 {
    wrap_degrees(90. - yaw_degrees)
}

pub fn bearing(from: [f32; 3], to: [f32; 2]) -> (f32, f32) {
    let (dx, dy) = (to[0] - from[0], to[1] - from[1]);
    (wrap_degrees(dx.atan2(dy).to_degrees()), dx.hypot(dy))
}
