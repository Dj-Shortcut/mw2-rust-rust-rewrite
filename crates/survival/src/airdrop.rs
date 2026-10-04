use crate::crates::{CRATE_REACH, roll_supply, splitmix};
use crate::gathering::height_at;
use crate::{Inventory, Item, Terrain};
use serde::{Deserialize, Deserializer, Serialize};

pub const FIRST_DROP_SECONDS: f32 = 600.;
pub const DROP_INTERVAL_SECONDS: f32 = 900.;
pub const DROP_LIFETIME_SECONDS: f32 = 600.;
pub const FLARE_SECONDS: f32 = 30.;
const SITES: [[f32; 2]; 4] = [
    [-900., 1200.],
    [1100., -1100.],
    [300., 1500.],
    [-1400., -200.],
];

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct SupplyDrop {
    pub site: u32,
    pub remaining: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct SavedAirdrops {
    pub next_in: f32,
    pub drops: u32,
    pub active: Option<SupplyDrop>,
}

impl Default for SavedAirdrops {
    fn default() -> Self {
        Self {
            next_in: FIRST_DROP_SECONDS,
            drops: 0,
            active: None,
        }
    }
}

impl<'de> Deserialize<'de> for SavedAirdrops {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Drop {
            site: u32,
            remaining: f32,
        }
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Saved {
            next_in: f32,
            drops: u32,
            active: Option<Drop>,
        }
        let saved = Saved::deserialize(deserializer)?;
        let within = |t: f32, limit: f32| t.is_finite() && t > 0. && t <= limit;
        let longest = if saved.drops == 0 {
            FIRST_DROP_SECONDS
        } else {
            DROP_INTERVAL_SECONDS
        };
        if !within(saved.next_in, longest) {
            return Err(serde::de::Error::custom(
                "Supply drop timer is out of range",
            ));
        }
        let active = match saved.active {
            None => None,
            Some(drop) => {
                // A drop is always gone before the next one lands.
                if drop.site as usize >= SITES.len()
                    || !within(drop.remaining, DROP_LIFETIME_SECONDS)
                    || drop.remaining >= saved.next_in
                    || saved.drops == 0
                {
                    return Err(serde::de::Error::custom("Invalid supply drop"));
                }
                Some(SupplyDrop {
                    site: drop.site,
                    remaining: drop.remaining,
                })
            }
        };
        Ok(Self {
            next_in: saved.next_in,
            drops: saved.drops,
            active,
        })
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct DropEvents {
    pub landed: bool,
    pub lost: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Airdrops {
    seed: u32,
    sites: Vec<[f32; 3]>,
    state: SavedAirdrops,
}

impl Airdrops {
    pub(crate) fn new(terrain: &Terrain) -> Result<Self, String> {
        let sites = SITES
            .iter()
            .map(|&xy| {
                let z =
                    height_at(terrain, xy).ok_or("Supply drop site lies outside terrain mesh")?;
                Ok([xy[0], xy[1], z])
            })
            .collect::<Result<_, String>>()?;
        Ok(Self {
            seed: terrain.seed,
            sites,
            state: SavedAirdrops::default(),
        })
    }

    pub fn sites(&self) -> &[[f32; 3]] {
        &self.sites
    }

    pub fn next_in(&self) -> f32 {
        self.state.next_in
    }

    pub fn drops(&self) -> u32 {
        self.state.drops
    }

    pub fn active(&self) -> Option<SupplyDrop> {
        self.state.active
    }

    pub(crate) fn call_in(&mut self) -> Result<(), String> {
        if self.state.active.is_some() {
            return Err("A supply drop is already down".into());
        }
        if self.state.next_in <= FLARE_SECONDS {
            return Err("A supply drop is already on its way".into());
        }
        self.state.next_in = FLARE_SECONDS;
        Ok(())
    }

    pub fn position(&self) -> Option<[f32; 3]> {
        self.state.active.map(|d| self.sites[d.site as usize])
    }

    pub fn in_reach(&self, origin: [f32; 3]) -> bool {
        self.position().is_some_and(|p| {
            (0..3)
                .map(|k| (p[k] - origin[k]).powi(2))
                .sum::<f32>()
                .sqrt()
                <= CRATE_REACH
        })
    }

    fn pick_site(&self, drops: u32) -> u32 {
        let mut state = (u64::from(self.seed) << 32) ^ u64::from(drops) ^ 0xA1D0_0000;
        (splitmix(&mut state) % SITES.len() as u64) as u32
    }

    pub(crate) fn advance(&mut self, dt_seconds: f32) -> Result<DropEvents, String> {
        if !dt_seconds.is_finite() || dt_seconds < 0. {
            return Err("Invalid supply drop time step".into());
        }
        let mut events = DropEvents::default();
        if let Some(drop) = &mut self.state.active {
            drop.remaining -= dt_seconds;
            if drop.remaining <= 0. {
                self.state.active = None;
                events.lost = true;
            }
        }
        self.state.next_in -= dt_seconds;
        while self.state.next_in <= 0. {
            self.state.drops = self.state.drops.wrapping_add(1);
            self.state.active = Some(SupplyDrop {
                site: self.pick_site(self.state.drops),
                remaining: DROP_LIFETIME_SECONDS,
            });
            self.state.next_in += DROP_INTERVAL_SECONDS;
            events.landed = true;
            events.lost = false;
        }
        Ok(events)
    }

    pub(crate) fn open(&mut self, inventory: &mut Inventory) -> Result<Vec<(Item, u32)>, String> {
        if self.state.active.is_none() {
            return Err("There is no supply drop".into());
        }
        let loot = roll_supply(self.seed, self.state.drops);
        let mut filled = inventory.clone();
        for &(item, quantity) in &loot {
            filled
                .add(item, quantity)
                .map_err(|_| "Not enough inventory space for the supply drop")?;
        }
        *inventory = filled;
        self.state.active = None;
        Ok(loot)
    }

    pub(crate) fn saved(&self) -> SavedAirdrops {
        self.state
    }

    pub(crate) fn restore(&mut self, saved: &SavedAirdrops) {
        self.state = *saved;
    }
}
