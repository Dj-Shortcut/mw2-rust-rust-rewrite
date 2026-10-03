use crate::gathering::height_at;
use crate::{Inventory, Item, Terrain};
use serde::{Deserialize, Deserializer, Serialize};

pub const CRATE_REACH: f32 = 100.;
pub const HACK_SECONDS: f32 = 120.;
const LOCKED_POSITION: [f32; 2] = [1500., 1000.];

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CrateTier {
    Barrel,
    Military,
    Locked,
}

struct LootEntry {
    item: Item,
    min: u32,
    max: u32,
    weight: u32,
}

const BARREL_TABLE: [LootEntry; 4] = [
    LootEntry {
        item: Item::Food,
        min: 1,
        max: 3,
        weight: 4,
    },
    LootEntry {
        item: Item::Water,
        min: 1,
        max: 3,
        weight: 4,
    },
    LootEntry {
        item: Item::Bandage,
        min: 1,
        max: 2,
        weight: 3,
    },
    LootEntry {
        item: Item::Ammo,
        min: 10,
        max: 30,
        weight: 2,
    },
];

const MILITARY_TABLE: [LootEntry; 5] = [
    LootEntry {
        item: Item::Ammo,
        min: 30,
        max: 60,
        weight: 4,
    },
    LootEntry {
        item: Item::Syringe,
        min: 1,
        max: 2,
        weight: 3,
    },
    LootEntry {
        item: Item::Bandage,
        min: 2,
        max: 4,
        weight: 3,
    },
    LootEntry {
        item: Item::Food,
        min: 2,
        max: 4,
        weight: 2,
    },
    LootEntry {
        item: Item::Jacket,
        min: 1,
        max: 1,
        weight: 1,
    },
];

const LOCKED_TABLE: [LootEntry; 5] = [
    LootEntry {
        item: Item::Ammo,
        min: 40,
        max: 60,
        weight: 3,
    },
    LootEntry {
        item: Item::Syringe,
        min: 2,
        max: 3,
        weight: 3,
    },
    LootEntry {
        item: Item::AntiRadPills,
        min: 2,
        max: 4,
        weight: 3,
    },
    LootEntry {
        item: Item::Bandage,
        min: 3,
        max: 5,
        weight: 2,
    },
    LootEntry {
        item: Item::HazmatSuit,
        min: 1,
        max: 1,
        weight: 1,
    },
];

const LAYOUT: [(CrateTier, [f32; 2]); 6] = [
    (CrateTier::Barrel, [120., -300.]),
    (CrateTier::Barrel, [-140., 320.]),
    (CrateTier::Barrel, [600., -520.]),
    (CrateTier::Barrel, [-620., 560.]),
    (CrateTier::Military, [1700., 820.]),
    (CrateTier::Military, [-1620., -900.]),
];

impl CrateTier {
    pub fn name(self) -> &'static str {
        match self {
            Self::Barrel => "Barrel",
            Self::Military => "Military crate",
            Self::Locked => "Locked crate",
        }
    }

    pub fn respawn_seconds(self) -> f32 {
        match self {
            Self::Barrel => 300.,
            Self::Military => 600.,
            Self::Locked => 1800.,
        }
    }

    fn rolls(self) -> u32 {
        match self {
            Self::Barrel => 2,
            Self::Military => 3,
            Self::Locked => 4,
        }
    }

    fn table(self) -> &'static [LootEntry] {
        match self {
            Self::Barrel => &BARREL_TABLE,
            Self::Military => &MILITARY_TABLE,
            Self::Locked => &LOCKED_TABLE,
        }
    }

    pub fn possible_items(self) -> Vec<Item> {
        self.table().iter().map(|e| e.item).collect()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct LootCrate {
    id: u32,
    tier: CrateTier,
    position: [f32; 3],
    respawn_in: f32,
    opened: u32,
}

impl LootCrate {
    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn tier(&self) -> CrateTier {
        self.tier
    }

    pub fn position(&self) -> [f32; 3] {
        self.position
    }

    pub fn respawn_in(&self) -> f32 {
        self.respawn_in
    }

    pub fn is_full(&self) -> bool {
        self.respawn_in == 0.
    }

    fn roll(&self, seed: u32) -> Vec<(Item, u32)> {
        let mut state =
            (u64::from(seed) << 32) ^ (u64::from(self.id) << 20) ^ u64::from(self.opened);
        let table = self.tier.table();
        let total: u32 = table.iter().map(|e| e.weight).sum();
        let mut loot = Vec::new();
        for _ in 0..self.tier.rolls() {
            let mut pick = (splitmix(&mut state) % u64::from(total)) as u32;
            let entry = table
                .iter()
                .find(|e| {
                    let hit = pick < e.weight;
                    pick = pick.saturating_sub(e.weight);
                    hit
                })
                .unwrap_or(&table[0]);
            let span = u64::from(entry.max - entry.min + 1);
            let quantity = entry.min + (splitmix(&mut state) % span) as u32;
            loot.push((entry.item, quantity));
        }
        loot
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SavedCrate {
    pub id: u32,
    pub respawn_in: f32,
    pub opened: u32,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(transparent)]
pub struct SavedCrates(Vec<SavedCrate>);

impl<'de> Deserialize<'de> for SavedCrates {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let crates = Vec::<SavedCrate>::deserialize(deserializer)?;
        let valid = crates.len() == LAYOUT.len()
            && crates.iter().enumerate().all(|(index, saved)| {
                let tier = LAYOUT[index].0;
                saved.id == index as u32 + 1
                    && saved.respawn_in.is_finite()
                    && (0. ..=tier.respawn_seconds()).contains(&saved.respawn_in)
            });
        if !valid {
            return Err(serde::de::Error::custom("Invalid loot crates"));
        }
        Ok(Self(crates))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub enum LockState {
    Locked,
    Hacking { remaining: f32 },
    Unlocked,
    Restocking { remaining: f32 },
}

impl<'de> Deserialize<'de> for LockState {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        enum Saved {
            Locked,
            Hacking { remaining: f32 },
            Unlocked,
            Restocking { remaining: f32 },
        }
        let within = |remaining: f32, limit: f32| {
            remaining.is_finite() && remaining > 0. && remaining <= limit
        };
        match Saved::deserialize(deserializer)? {
            Saved::Locked => Ok(Self::Locked),
            Saved::Unlocked => Ok(Self::Unlocked),
            Saved::Hacking { remaining } if within(remaining, HACK_SECONDS) => {
                Ok(Self::Hacking { remaining })
            }
            Saved::Restocking { remaining }
                if within(remaining, CrateTier::Locked.respawn_seconds()) =>
            {
                Ok(Self::Restocking { remaining })
            }
            _ => Err(serde::de::Error::custom("Invalid locked crate timer")),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SavedLocked {
    pub state: LockState,
    pub opened: u32,
}

impl Default for SavedLocked {
    fn default() -> Self {
        Self {
            state: LockState::Locked,
            opened: 0,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct LockedCrate {
    lootable: LootCrate,
    state: LockState,
}

impl LockedCrate {
    pub fn position(&self) -> [f32; 3] {
        self.lootable.position
    }

    pub fn state(&self) -> LockState {
        self.state
    }

    pub fn opened(&self) -> u32 {
        self.lootable.opened
    }

    pub fn in_reach(&self, origin: [f32; 3]) -> bool {
        let p = self.lootable.position;
        ((p[0] - origin[0]).powi(2) + (p[1] - origin[1]).powi(2) + (p[2] - origin[2]).powi(2))
            .sqrt()
            <= CRATE_REACH
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct LootCrates {
    seed: u32,
    crates: Vec<LootCrate>,
    locked: LockedCrate,
}

impl LootCrates {
    pub fn new(terrain: &Terrain) -> Result<Self, String> {
        let crates = LAYOUT
            .iter()
            .enumerate()
            .map(|(index, &(tier, xy))| {
                let z = height_at(terrain, xy).ok_or("Loot crate lies outside terrain mesh")?;
                Ok(LootCrate {
                    id: index as u32 + 1,
                    tier,
                    position: [xy[0], xy[1], z],
                    respawn_in: 0.,
                    opened: 0,
                })
            })
            .collect::<Result<_, String>>()?;
        let z =
            height_at(terrain, LOCKED_POSITION).ok_or("Locked crate lies outside terrain mesh")?;
        let locked = LockedCrate {
            lootable: LootCrate {
                id: LAYOUT.len() as u32 + 1,
                tier: CrateTier::Locked,
                position: [LOCKED_POSITION[0], LOCKED_POSITION[1], z],
                respawn_in: 0.,
                opened: 0,
            },
            state: LockState::Locked,
        };
        Ok(Self {
            seed: terrain.seed,
            crates,
            locked,
        })
    }

    pub fn crates(&self) -> &[LootCrate] {
        &self.crates
    }

    pub fn locked(&self) -> &LockedCrate {
        &self.locked
    }

    pub(crate) fn start_hack(&mut self) -> Result<(), String> {
        match self.locked.state {
            LockState::Locked => {
                self.locked.state = LockState::Hacking {
                    remaining: HACK_SECONDS,
                };
                Ok(())
            }
            LockState::Hacking { .. } => Err("The crate is already being hacked".into()),
            LockState::Unlocked => Err("The crate is already unlocked".into()),
            LockState::Restocking { .. } => Err("The locked crate is empty".into()),
        }
    }

    pub(crate) fn open_locked(
        &mut self,
        inventory: &mut Inventory,
    ) -> Result<Vec<(Item, u32)>, String> {
        match self.locked.state {
            LockState::Unlocked => {}
            LockState::Restocking { .. } => return Err("The locked crate is empty".into()),
            _ => return Err("The crate is still locked".into()),
        }
        let loot = self.locked.lootable.roll(self.seed);
        let mut filled = inventory.clone();
        for &(item, quantity) in &loot {
            filled
                .add(item, quantity)
                .map_err(|_| "Not enough inventory space for the crate's loot")?;
        }
        *inventory = filled;
        self.locked.state = LockState::Restocking {
            remaining: CrateTier::Locked.respawn_seconds(),
        };
        self.locked.lootable.opened = self.locked.lootable.opened.wrapping_add(1);
        Ok(loot)
    }

    pub fn nearest_full(&self, origin: [f32; 3]) -> Option<&LootCrate> {
        let distance = |c: &LootCrate| {
            let p = c.position;
            ((p[0] - origin[0]).powi(2) + (p[1] - origin[1]).powi(2) + (p[2] - origin[2]).powi(2))
                .sqrt()
        };
        self.crates
            .iter()
            .filter(|c| c.is_full() && distance(c) <= CRATE_REACH)
            .min_by(|a, b| distance(a).total_cmp(&distance(b)))
    }

    pub(crate) fn open(
        &mut self,
        id: u32,
        inventory: &mut Inventory,
    ) -> Result<(CrateTier, Vec<(Item, u32)>), String> {
        let lootable = self
            .crates
            .iter_mut()
            .find(|c| c.id == id && c.is_full())
            .ok_or("That crate is empty")?;
        let loot = lootable.roll(self.seed);
        let mut filled = inventory.clone();
        for &(item, quantity) in &loot {
            filled
                .add(item, quantity)
                .map_err(|_| "Not enough inventory space for the crate's loot")?;
        }
        *inventory = filled;
        lootable.respawn_in = lootable.tier.respawn_seconds();
        lootable.opened = lootable.opened.wrapping_add(1);
        Ok((lootable.tier, loot))
    }

    pub(crate) fn advance(&mut self, dt_seconds: f32) -> Result<bool, String> {
        if !dt_seconds.is_finite() || dt_seconds < 0. {
            return Err("Invalid crate time step".into());
        }
        for lootable in &mut self.crates {
            lootable.respawn_in = (lootable.respawn_in - dt_seconds).max(0.);
        }
        let mut unlocked = false;
        self.locked.state = match self.locked.state {
            LockState::Hacking { remaining } if remaining > dt_seconds => LockState::Hacking {
                remaining: remaining - dt_seconds,
            },
            LockState::Hacking { .. } => {
                unlocked = true;
                LockState::Unlocked
            }
            LockState::Restocking { remaining } if remaining > dt_seconds => {
                LockState::Restocking {
                    remaining: remaining - dt_seconds,
                }
            }
            LockState::Restocking { .. } => LockState::Locked,
            state => state,
        };
        Ok(unlocked)
    }

    pub(crate) fn saved_locked(&self) -> SavedLocked {
        SavedLocked {
            state: self.locked.state,
            opened: self.locked.lootable.opened,
        }
    }

    pub(crate) fn restore_locked(&mut self, saved: &SavedLocked) {
        self.locked.state = saved.state;
        self.locked.lootable.opened = saved.opened;
    }

    pub(crate) fn saved(&self) -> SavedCrates {
        SavedCrates(
            self.crates
                .iter()
                .map(|c| SavedCrate {
                    id: c.id,
                    respawn_in: c.respawn_in,
                    opened: c.opened,
                })
                .collect(),
        )
    }

    pub(crate) fn restore(&mut self, saved: Option<&SavedCrates>) {
        for (index, lootable) in self.crates.iter_mut().enumerate() {
            let state = saved.and_then(|s| s.0.get(index));
            lootable.respawn_in = state.map_or(0., |s| s.respawn_in);
            lootable.opened = state.map_or(0, |s| s.opened);
        }
    }
}

pub(crate) fn splitmix(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE5_E9B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}
