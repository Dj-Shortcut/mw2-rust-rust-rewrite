use crate::Terrain;
use crate::gathering::height_at;
use serde::{Deserialize, Deserializer, Serialize};

pub const COOK_SECONDS: f32 = 15.;
pub const COOK_WOOD: u32 = 5;
pub const RAIN_COOK_SPEED: f32 = 0.5;
pub const CAMPFIRE_REACH: f32 = 100.;
pub const CAMPFIRE_WARMTH: f32 = 10.;
pub const RAIN_CAMPFIRE_WARMTH: f32 = 5.;
pub const CAMPFIRE_WARMTH_RADIUS: f32 = 150.;
pub const TEA_WARMTH: f32 = 8.;
pub const TEA_SECONDS: f32 = 300.;
pub const COMFORT_SECONDS_PER_HP: f32 = 5.;
pub const WOOD_BURN_SECONDS: f32 = 10.;
pub const MAX_FIRE_WOOD: u32 = 1000;
pub const START_FIRE_WOOD: u32 = 500;
const _: () = assert!(COOK_WOOD as f32 * WOOD_BURN_SECONDS >= COOK_SECONDS / RAIN_COOK_SPEED);
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

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct SavedFuel {
    lit: bool,
    wood: u32,
    burnt: f32,
}

const FRESH_FUEL: SavedFuel = SavedFuel {
    lit: true,
    wood: START_FIRE_WOOD,
    burnt: 0.,
};

impl<'de> Deserialize<'de> for SavedFuel {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Saved {
            lit: bool,
            wood: u32,
            burnt: f32,
        }
        let saved = Saved::deserialize(deserializer)?;
        if saved.wood > MAX_FIRE_WOOD {
            return Err(serde::de::Error::custom(
                "Saved campfire holds too much wood",
            ));
        }
        if !saved.burnt.is_finite() || !(0. ..WOOD_BURN_SECONDS).contains(&saved.burnt) {
            return Err(serde::de::Error::custom(
                "Saved campfire burn time is out of range",
            ));
        }
        if saved.wood == 0 && (saved.lit || saved.burnt > 0.) {
            return Err(serde::de::Error::custom(
                "Saved campfire burns without wood",
            ));
        }
        Ok(Self {
            lit: saved.lit,
            wood: saved.wood,
            burnt: saved.burnt,
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Campfire {
    id: u32,
    position: [f32; 3],
    state: FireState,
    fuel: SavedFuel,
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

    pub fn is_lit(&self) -> bool {
        self.fuel.lit
    }

    pub fn wood(&self) -> u32 {
        self.fuel.wood
    }

    pub fn fuel_seconds(&self) -> f32 {
        self.fuel.wood as f32 * WOOD_BURN_SECONDS - self.fuel.burnt
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

/// Saved fire fuel in layout order; empty in saves from before fires burned wood.
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
#[serde(transparent)]
pub struct SavedFuels {
    fuels: Vec<SavedFuel>,
}

impl<'de> Deserialize<'de> for SavedFuels {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let fuels = Vec::<SavedFuel>::deserialize(deserializer)?;
        if !fuels.is_empty() && fuels.len() != LAYOUT.len() {
            return Err(serde::de::Error::custom(
                "Saved campfire fuel does not match the world",
            ));
        }
        Ok(Self { fuels })
    }
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
                    fuel: FRESH_FUEL,
                })
            })
            .collect::<Result<_, String>>()?;
        Ok(Self { fires })
    }

    pub fn all(&self) -> &[Campfire] {
        &self.fires
    }

    pub fn in_reach(&self, origin: [f32; 3]) -> Option<&Campfire> {
        self.fires
            .iter()
            .filter(|f| f.distance(origin) <= CAMPFIRE_REACH)
            .min_by(|a, b| a.distance(origin).total_cmp(&b.distance(origin)))
    }

    pub fn warms(&self, origin: [f32; 3]) -> bool {
        self.fires
            .iter()
            .any(|f| f.fuel.lit && f.distance(origin) <= CAMPFIRE_WARMTH_RADIUS)
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

    pub(crate) fn add_wood(&mut self, id: u32, amount: u32) -> Result<(), String> {
        let fire = self.fire_mut(id)?;
        if amount > MAX_FIRE_WOOD - fire.fuel.wood {
            return Err("The campfire cannot hold more wood".into());
        }
        fire.fuel.wood += amount;
        Ok(())
    }

    pub(crate) fn light(&mut self, id: u32) -> Result<(), String> {
        let fire = self.fire_mut(id)?;
        if fire.fuel.lit {
            return Err("The campfire is already burning".into());
        }
        if fire.fuel.wood == 0 {
            return Err("Add wood to the campfire first".into());
        }
        fire.fuel.lit = true;
        Ok(())
    }

    pub(crate) fn put_out(&mut self, id: u32) -> Result<(), String> {
        let fire = self.fire_mut(id)?;
        if !fire.fuel.lit {
            return Err("The campfire is not burning".into());
        }
        fire.fuel.lit = false;
        Ok(())
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

    pub(crate) fn advance(&mut self, dt_seconds: f32, raining: bool) -> Result<bool, String> {
        if !dt_seconds.is_finite() || dt_seconds < 0. {
            return Err("Invalid cooking time step".into());
        }
        let step = if raining {
            dt_seconds * RAIN_COOK_SPEED
        } else {
            dt_seconds
        };
        let mut cooked = false;
        for fire in &mut self.fires {
            if !fire.fuel.lit {
                continue;
            }
            if let FireState::Cooking { remaining } = fire.state {
                let left = remaining - step;
                fire.state = if left <= 0. {
                    cooked = true;
                    FireState::Ready
                } else {
                    FireState::Cooking { remaining: left }
                };
            }
            fire.fuel.burnt += dt_seconds;
            while fire.fuel.burnt >= WOOD_BURN_SECONDS {
                fire.fuel.burnt -= WOOD_BURN_SECONDS;
                fire.fuel.wood -= 1;
                if fire.fuel.wood == 0 {
                    fire.fuel.lit = false;
                    fire.fuel.burnt = 0.;
                    break;
                }
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

    pub(crate) fn saved_fuel(&self) -> SavedFuels {
        SavedFuels {
            fuels: self.fires.iter().map(|f| f.fuel).collect(),
        }
    }

    pub(crate) fn restore_fuel(&mut self, saved: &SavedFuels) {
        for (index, fire) in self.fires.iter_mut().enumerate() {
            fire.fuel = saved.fuels.get(index).copied().unwrap_or(FRESH_FUEL);
        }
    }
}
