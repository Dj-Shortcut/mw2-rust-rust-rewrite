use crate::gathering::height_at;
use crate::{Inventory, Item, Terrain};
use serde::{Deserialize, Deserializer, Serialize};

pub const CRATE_REACH: f32 = 100.;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CrateTier {
    Barrel,
    Military,
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
        }
    }

    pub fn respawn_seconds(self) -> f32 {
        match self {
            Self::Barrel => 300.,
            Self::Military => 600.,
        }
    }

    fn rolls(self) -> u32 {
        match self {
            Self::Barrel => 2,
            Self::Military => 3,
        }
    }

    fn table(self) -> &'static [LootEntry] {
        match self {
            Self::Barrel => &BARREL_TABLE,
            Self::Military => &MILITARY_TABLE,
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

#[derive(Clone, Debug, Default, PartialEq, Serialize)]
#[serde(transparent)]
pub struct SavedCrates(Vec<SavedCrate>);

impl<'de> Deserialize<'de> for SavedCrates {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let crates = Vec::<SavedCrate>::deserialize(deserializer)?;
        if !crates.is_empty() {
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
        }
        Ok(Self(crates))
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct LootCrates {
    seed: u32,
    crates: Vec<LootCrate>,
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
        Ok(Self {
            seed: terrain.seed,
            crates,
        })
    }

    pub fn crates(&self) -> &[LootCrate] {
        &self.crates
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

    pub(crate) fn advance(&mut self, dt_seconds: f32) -> Result<(), String> {
        if !dt_seconds.is_finite() || dt_seconds < 0. {
            return Err("Invalid crate time step".into());
        }
        for lootable in &mut self.crates {
            lootable.respawn_in = (lootable.respawn_in - dt_seconds).max(0.);
        }
        Ok(())
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

    pub(crate) fn restore(&mut self, saved: &SavedCrates) {
        for (lootable, state) in self.crates.iter_mut().zip(&saved.0) {
            lootable.respawn_in = state.respawn_in;
            lootable.opened = state.opened;
        }
        if saved.0.is_empty() {
            for lootable in &mut self.crates {
                lootable.respawn_in = 0.;
                lootable.opened = 0;
            }
        }
    }
}

fn splitmix(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE5_E9B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}
