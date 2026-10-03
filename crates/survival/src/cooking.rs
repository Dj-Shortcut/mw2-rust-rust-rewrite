use crate::Terrain;
use crate::gathering::height_at;
use serde::{Deserialize, Deserializer, Serialize};

pub const COOK_SECONDS: f32 = 15.;
pub const COOK_WOOD: u32 = 5;
pub const CAMPFIRE_REACH: f32 = 100.;
pub const CAMPFIRE_WARMTH: f32 = 10.;
pub const CAMPFIRE_WARMTH_RADIUS: f32 = 150.;
const LAYOUT: [[f32; 2]; 2] = [[-260., -140.], [820., 260.]];

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub enum FireState {
    Idle,
    Cooking { remaining: f32 },
    Ready,
}

impl<'de> Deserialize<'de> for FireState {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        enum Saved {
            Idle,
            Cooking { remaining: f32 },
            Ready,
        }
        Ok(match Saved::deserialize(deserializer)? {
            Saved::Idle => Self::Idle,
            Saved::Ready => Self::Ready,
            Saved::Cooking { remaining } => {
                if !remaining.is_finite() || remaining <= 0. || remaining > COOK_SECONDS {
                    return Err(serde::de::Error::custom("Cooking time is out of range"));
                }
                Self::Cooking { remaining }
            }
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Campfire {
    id: u32,
    position: [f32; 3],
    state: FireState,
}

impl Campfire {
    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn position(&self) -> [f32; 3] {
        self.position
    }

    pub fn state(&self) -> FireState {
        self.state
    }

    fn distance(&self, origin: [f32; 3]) -> f32 {
        (0..3)
            .map(|k| (self.position[k] - origin[k]).powi(2))
            .sum::<f32>()
            .sqrt()
    }
}

/// Saved fire states in layout order; empty in saves from before cooking.
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
#[serde(transparent)]
pub struct SavedFires {
    states: Vec<FireState>,
}

impl<'de> Deserialize<'de> for SavedFires {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let states = Vec::<FireState>::deserialize(deserializer)?;
        if !states.is_empty() && states.len() != LAYOUT.len() {
            return Err(serde::de::Error::custom(
                "Saved campfires do not match the world",
            ));
        }
        Ok(Self { states })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Campfires {
    fires: Vec<Campfire>,
}

impl Campfires {
    pub(crate) fn new(terrain: &Terrain) -> Result<Self, String> {
        let fires = LAYOUT
            .iter()
            .enumerate()
            .map(|(index, &xy)| {
                let z = height_at(terrain, xy).ok_or("Campfire lies outside terrain mesh")?;
                Ok(Campfire {
                    id: index as u32 + 1,
                    position: [xy[0], xy[1], z],
                    state: FireState::Idle,
                })
            })
            .collect::<Result<_, String>>()?;
        Ok(Self { fires })
    }

    pub fn all(&self) -> &[Campfire] {
        &self.fires
    }

    /// The nearest campfire within reach of `origin`.
    pub fn in_reach(&self, origin: [f32; 3]) -> Option<&Campfire> {
        self.fires
            .iter()
            .filter(|f| f.distance(origin) <= CAMPFIRE_REACH)
            .min_by(|a, b| a.distance(origin).total_cmp(&b.distance(origin)))
    }

    pub fn warms(&self, origin: [f32; 3]) -> bool {
        self.fires
            .iter()
            .any(|f| f.distance(origin) <= CAMPFIRE_WARMTH_RADIUS)
    }

    fn fire_mut(&mut self, id: u32) -> Result<&mut Campfire, String> {
        self.fires
            .iter_mut()
            .find(|f| f.id == id)
            .ok_or_else(|| "No such campfire".into())
    }

    pub(crate) fn start(&mut self, id: u32) -> Result<(), String> {
        let fire = self.fire_mut(id)?;
        match fire.state {
            FireState::Idle => {
                fire.state = FireState::Cooking {
                    remaining: COOK_SECONDS,
                };
                Ok(())
            }
            FireState::Cooking { .. } => Err("Something is already cooking on this fire".into()),
            FireState::Ready => Err("Take the cooked fish off the fire first".into()),
        }
    }

    pub(crate) fn take(&mut self, id: u32) -> Result<(), String> {
        let fire = self.fire_mut(id)?;
        match fire.state {
            FireState::Ready => {
                fire.state = FireState::Idle;
                Ok(())
            }
            FireState::Cooking { .. } => Err("The fish is not cooked yet".into()),
            FireState::Idle => Err("Nothing is cooking on this fire".into()),
        }
    }

    /// Advances every fire; returns true when a fish finished cooking.
    pub(crate) fn advance(&mut self, dt_seconds: f32) -> Result<bool, String> {
        if !dt_seconds.is_finite() || dt_seconds < 0. {
            return Err("Invalid cooking time step".into());
        }
        let mut cooked = false;
        for fire in &mut self.fires {
            if let FireState::Cooking { remaining } = fire.state {
                let left = remaining - dt_seconds;
                fire.state = if left <= 0. {
                    cooked = true;
                    FireState::Ready
                } else {
                    FireState::Cooking { remaining: left }
                };
            }
        }
        Ok(cooked)
    }

    pub(crate) fn saved(&self) -> SavedFires {
        SavedFires {
            states: self.fires.iter().map(|f| f.state).collect(),
        }
    }

    pub(crate) fn restore(&mut self, saved: &SavedFires) {
        for (index, fire) in self.fires.iter_mut().enumerate() {
            fire.state = saved.states.get(index).copied().unwrap_or(FireState::Idle);
        }
    }
}
