use crate::radiation::{
    RADIATION_DAMAGE_PER_SECOND, RADIATION_DECAY_PER_SECOND, RADIATION_PER_SECOND,
};
use crate::{COLD_CELSIUS, FREEZING_CELSIUS, MAX_RADIATION, RADIATION_SICK, TEA_SECONDS};
use rust_building::Resources;
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::BTreeSet;

pub const INVENTORY_SLOTS: usize = 24;
pub const MAX_VITAL_SECONDS: f32 = 3600.;
pub const BLEED_THRESHOLD: u32 = 15;
pub const MAX_BLEED: f32 = 40.;
const BLEED_PER_DAMAGE: f64 = 0.5;
const BLEED_PER_SECOND: f64 = 1.;
const FREEZE_PER_SECOND: f64 = 0.05;
pub const MAX_REGEN: f32 = 40.;
const REGEN_PER_SECOND: f64 = 2.;
pub const MAX_TOOL_WEAR: u32 = 50;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Item {
    Bandage,
    Ammo,
    Food,
    Water,
    Syringe,
    Hatchet,
    Pickaxe,
    Jacket,
    FishingRod,
    Fish,
    AntiRadPills,
    HazmatSuit,
    CookedFish,
    BerrySeeds,
    Bait,
    Fertilizer,
    BerryTea,
    Raincoat,
    FishStew,
    SignalFlare,
    Honey,
}

impl Item {
    pub const ALL: [Self; 21] = [
        Self::Bandage,
        Self::Ammo,
        Self::Food,
        Self::Water,
        Self::Syringe,
        Self::Hatchet,
        Self::Pickaxe,
        Self::Jacket,
        Self::FishingRod,
        Self::Fish,
        Self::AntiRadPills,
        Self::HazmatSuit,
        Self::CookedFish,
        Self::BerrySeeds,
        Self::Bait,
        Self::Fertilizer,
        Self::BerryTea,
        Self::Raincoat,
        Self::FishStew,
        Self::SignalFlare,
        Self::Honey,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Self::Bandage => "Bandage",
            Self::Ammo => "Carbine ammunition",
            Self::Food => "Food",
            Self::Water => "Water",
            Self::Syringe => "Medical syringe",
            Self::Hatchet => "Stone hatchet",
            Self::Pickaxe => "Stone pickaxe",
            Self::Jacket => "Padded jacket",
            Self::FishingRod => "Fishing rod",
            Self::Fish => "Raw fish",
            Self::AntiRadPills => "Anti-radiation pills",
            Self::HazmatSuit => "Hazmat suit",
            Self::CookedFish => "Cooked fish",
            Self::BerrySeeds => "Berry seeds",
            Self::Bait => "Fishing bait",
            Self::Fertilizer => "Fertilizer",
            Self::BerryTea => "Berry tea",
            Self::Raincoat => "Raincoat",
            Self::FishStew => "Fish stew",
            Self::SignalFlare => "Signal flare",
            Self::Honey => "Honey",
        }
    }

    pub fn is_clothing(self) -> bool {
        matches!(self, Self::Jacket | Self::HazmatSuit | Self::Raincoat)
    }

    pub fn rain_proof(self) -> bool {
        self == Self::Raincoat
    }

    pub fn warmth(self) -> f32 {
        match self {
            Self::Jacket => 8.,
            Self::HazmatSuit | Self::Raincoat => 2.,
            _ => 0.,
        }
    }

    pub fn radiation_protection(self) -> f32 {
        if self == Self::HazmatSuit { 0.75 } else { 0. }
    }

    pub fn is_tool(self) -> bool {
        matches!(self, Self::Hatchet | Self::Pickaxe | Self::FishingRod)
    }

    pub fn stack_limit(self) -> u32 {
        match self {
            Self::Bandage
            | Self::Water
            | Self::Fish
            | Self::CookedFish
            | Self::AntiRadPills
            | Self::BerryTea
            | Self::FishStew
            | Self::Honey => 10,
            Self::Ammo => 60,
            Self::Food | Self::BerrySeeds | Self::Bait | Self::Fertilizer => 20,
            Self::Syringe | Self::SignalFlare => 5,
            Self::Hatchet
            | Self::Pickaxe
            | Self::Jacket
            | Self::FishingRod
            | Self::HazmatSuit
            | Self::Raincoat => 1,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Recipe {
    Bandage,
    Ammo,
    Syringe,
    Hatchet,
    Pickaxe,
    Jacket,
    FishingRod,
    AntiRadPills,
    HazmatSuit,
}

impl Recipe {
    pub const ALL: [Self; 9] = [
        Self::Bandage,
        Self::Ammo,
        Self::Syringe,
        Self::Hatchet,
        Self::Pickaxe,
        Self::Jacket,
        Self::FishingRod,
        Self::AntiRadPills,
        Self::HazmatSuit,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Self::Bandage => "Bandage",
            Self::Ammo => "30 carbine rounds",
            Self::Syringe => "Medical syringe",
            Self::Hatchet => "Stone hatchet",
            Self::Pickaxe => "Stone pickaxe",
            Self::Jacket => "Padded jacket",
            Self::FishingRod => "Fishing rod",
            Self::AntiRadPills => "Anti-radiation pills",
            Self::HazmatSuit => "Hazmat suit",
        }
    }

    pub fn cost(self) -> Resources {
        match self {
            Self::Bandage => Resources {
                wood: 20,
                ..Default::default()
            },
            Self::Ammo => Resources {
                metal: 15,
                stone: 10,
                ..Default::default()
            },
            Self::Syringe => Resources {
                wood: 15,
                metal: 20,
                ..Default::default()
            },
            Self::Hatchet | Self::Pickaxe => Resources {
                wood: 100,
                stone: 50,
                ..Default::default()
            },
            Self::Jacket => Resources {
                wood: 60,
                metal: 10,
                ..Default::default()
            },
            Self::FishingRod => Resources {
                wood: 60,
                metal: 5,
                ..Default::default()
            },
            Self::AntiRadPills => Resources {
                wood: 10,
                metal: 15,
                ..Default::default()
            },
            Self::HazmatSuit => Resources {
                wood: 50,
                metal: 40,
                ..Default::default()
            },
        }
    }

    pub fn recycle_yield(item: Item, quantity: u32) -> Result<Resources, String> {
        let recipe = Self::ALL
            .into_iter()
            .find(|r| r.output().0 == item)
            .ok_or("That item cannot be recycled")?;
        let batch = recipe.output().1;
        if quantity == 0 || quantity % batch != 0 {
            return Err(format!("Recycle {} in batches of {batch}", item.name()));
        }
        let batches = quantity / batch;
        let cost = recipe.cost();
        Ok(Resources {
            wood: cost.wood * batches / 2,
            stone: cost.stone * batches / 2,
            metal: cost.metal * batches / 2,
        })
    }

    pub fn craft_seconds(self) -> f32 {
        match self {
            Self::Bandage => 3.,
            Self::Ammo => 5.,
            Self::Syringe => 10.,
            Self::Hatchet | Self::Pickaxe => 8.,
            Self::Jacket => 6.,
            Self::FishingRod => 5.,
            Self::AntiRadPills => 3.,
            Self::HazmatSuit => 8.,
        }
    }

    pub fn needs_blueprint(self) -> bool {
        matches!(self, Self::Syringe | Self::Pickaxe | Self::HazmatSuit)
    }

    pub fn research_cost(self) -> Resources {
        let cost = self.cost();
        Resources {
            wood: cost.wood * 2,
            stone: cost.stone * 2,
            metal: cost.metal * 2,
        }
    }

    pub fn repair_cost(item: Item, wear: u32) -> Result<Resources, String> {
        let recipe = Self::ALL
            .into_iter()
            .find(|r| r.output().0 == item && item.is_tool())
            .ok_or("That item cannot be repaired")?;
        if wear == 0 {
            return Err("That tool is not worn".into());
        }
        let cost = recipe.cost();
        let part = |n: u32| (n * wear).div_ceil(2 * MAX_TOOL_WEAR);
        Ok(Resources {
            wood: part(cost.wood),
            stone: part(cost.stone),
            metal: part(cost.metal),
        })
    }

    pub fn output(self) -> (Item, u32) {
        match self {
            Self::Bandage => (Item::Bandage, 1),
            Self::Ammo => (Item::Ammo, 30),
            Self::Syringe => (Item::Syringe, 1),
            Self::Hatchet => (Item::Hatchet, 1),
            Self::Pickaxe => (Item::Pickaxe, 1),
            Self::Jacket => (Item::Jacket, 1),
            Self::FishingRod => (Item::FishingRod, 1),
            Self::AntiRadPills => (Item::AntiRadPills, 1),
            Self::HazmatSuit => (Item::HazmatSuit, 1),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
#[serde(transparent)]
pub struct Blueprints {
    learned: BTreeSet<Recipe>,
}

impl Blueprints {
    pub fn knows(&self, recipe: Recipe) -> bool {
        !recipe.needs_blueprint() || self.learned.contains(&recipe)
    }

    pub(crate) fn learn(&mut self, recipe: Recipe) -> Result<(), String> {
        if self.knows(recipe) {
            return Err(format!("{} is already known", recipe.name()));
        }
        self.learned.insert(recipe);
        Ok(())
    }
}

impl<'de> Deserialize<'de> for Blueprints {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let list = Vec::<Recipe>::deserialize(deserializer)?;
        let mut blueprints = Self::default();
        for recipe in list {
            blueprints.learn(recipe).map_err(serde::de::Error::custom)?;
        }
        Ok(blueprints)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Stack {
    pub item: Item,
    pub quantity: u32,
    #[serde(default, skip_serializing_if = "is_zero")]
    pub wear: u32,
}

fn is_zero(n: &u32) -> bool {
    *n == 0
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct Inventory {
    slots: Vec<Stack>,
}

impl Inventory {
    pub fn stacks(&self) -> &[Stack] {
        &self.slots
    }

    pub fn count(&self, item: Item) -> u32 {
        self.slots
            .iter()
            .filter(|s| s.item == item)
            .map(|s| s.quantity)
            .sum()
    }

    pub fn from_stacks(slots: Vec<Stack>) -> Result<Self, String> {
        if slots.len() > INVENTORY_SLOTS
            || slots.iter().any(|s| {
                s.quantity == 0
                    || s.quantity > s.item.stack_limit()
                    || s.wear > 0 && !s.item.is_tool()
                    || s.wear >= MAX_TOOL_WEAR
            })
        {
            return Err("Invalid inventory capacity or item stack".into());
        }
        Ok(Self { slots })
    }

    pub fn add(&mut self, item: Item, quantity: u32) -> Result<(), String> {
        if quantity == 0 {
            return Err("Item quantity must be positive".into());
        }
        let limit = item.stack_limit();
        let free = self
            .slots
            .iter()
            .filter(|s| s.item == item)
            .map(|s| limit - s.quantity)
            .sum::<u32>()
            + (INVENTORY_SLOTS - self.slots.len()) as u32 * limit;
        if quantity > free {
            return Err("Inventory is full".into());
        }
        let mut remaining = quantity;
        for stack in self.slots.iter_mut().filter(|s| s.item == item) {
            let added = remaining.min(limit - stack.quantity);
            stack.quantity += added;
            remaining -= added;
            if remaining == 0 {
                return Ok(());
            }
        }
        while remaining > 0 {
            let added = remaining.min(limit);
            self.slots.push(Stack {
                item,
                quantity: added,
                wear: 0,
            });
            remaining -= added;
        }
        Ok(())
    }

    pub(crate) fn add_stack(&mut self, stack: Stack) -> Result<(), String> {
        if stack.wear == 0 {
            return self.add(stack.item, stack.quantity);
        }
        if self.slots.len() >= INVENTORY_SLOTS {
            return Err("Inventory is full".into());
        }
        self.slots.push(stack);
        Ok(())
    }

    pub(crate) fn repair(&mut self, slot: usize) -> Result<(), String> {
        let stack = self.slots.get_mut(slot).ok_or("No stack in that slot")?;
        stack.wear = 0;
        Ok(())
    }

    pub(crate) fn wear_tool(&mut self, item: Item) -> Option<bool> {
        let slot = self.slots.iter().position(|s| s.item == item)?;
        self.slots[slot].wear += 1;
        let broke = self.slots[slot].wear >= MAX_TOOL_WEAR;
        if broke {
            self.slots.remove(slot);
        }
        Some(broke)
    }

    pub fn add_up_to(&mut self, item: Item, quantity: u32) -> u32 {
        let limit = item.stack_limit();
        let room = self
            .slots
            .iter()
            .filter(|s| s.item == item)
            .map(|s| limit - s.quantity)
            .sum::<u32>()
            + (INVENTORY_SLOTS - self.slots.len()) as u32 * limit;
        let added = quantity.min(room);
        if added > 0 {
            self.add(item, added)
                .expect("added quantity fits by construction");
        }
        added
    }

    pub fn craft(&mut self, recipe: Recipe, available: Resources) -> Result<Resources, String> {
        let cost = recipe.cost();
        if !available.covers(cost) {
            return Err("Not enough crafting resources".into());
        }
        let (item, quantity) = recipe.output();
        self.add(item, quantity)?;
        Ok(cost)
    }

    pub fn use_item(&mut self, item: Item, quantity: u32) -> Result<Effects, String> {
        let effects = match item {
            Item::Bandage => Effects {
                heal: 25,
                stop_bleeding: true,
                ..Default::default()
            },
            Item::Ammo => Effects {
                ammo: quantity,
                ..Default::default()
            },
            Item::Food => Effects {
                hunger: 25.,
                ..Default::default()
            },
            Item::Water => Effects {
                thirst: 35.,
                ..Default::default()
            },
            Item::Syringe => Effects {
                heal: 15,
                regen: 20.,
                stop_bleeding: true,
                ..Default::default()
            },
            Item::Fish => Effects {
                hunger: 20.,
                ..Default::default()
            },
            Item::CookedFish => Effects {
                heal: 5,
                hunger: 45.,
                ..Default::default()
            },
            Item::AntiRadPills => Effects {
                radiation: 50.,
                ..Default::default()
            },
            Item::FishStew => Effects {
                heal: 10,
                hunger: 60.,
                thirst: 20.,
                ..Default::default()
            },
            Item::Honey => Effects {
                heal: 5,
                hunger: 15.,
                stop_bleeding: true,
                ..Default::default()
            },
            Item::BerryTea => Effects {
                thirst: 30.,
                warmth_seconds: TEA_SECONDS,
                ..Default::default()
            },
            Item::Hatchet
            | Item::Pickaxe
            | Item::Jacket
            | Item::FishingRod
            | Item::HazmatSuit
            | Item::Raincoat
            | Item::SignalFlare
            | Item::BerrySeeds
            | Item::Bait
            | Item::Fertilizer => {
                return Err("That item cannot be used".into());
            }
        };
        if quantity == 0 || item != Item::Ammo && quantity != 1 {
            return Err("Use one consumable or a positive number of rounds".into());
        }
        self.take(item, quantity)?;
        Ok(effects)
    }

    pub(crate) fn take(&mut self, item: Item, quantity: u32) -> Result<(), String> {
        if self.count(item) < quantity {
            return Err("Item is not available".into());
        }
        let mut remaining = quantity;
        for stack in self.slots.iter_mut().filter(|s| s.item == item) {
            let taken = remaining.min(stack.quantity);
            stack.quantity -= taken;
            remaining -= taken;
            if remaining == 0 {
                break;
            }
        }
        self.slots.retain(|s| s.quantity > 0);
        Ok(())
    }

    pub fn split(&mut self, slot: usize, quantity: u32) -> Result<(), String> {
        let stack = *self.slots.get(slot).ok_or("No stack in that slot")?;
        if quantity == 0 || quantity >= stack.quantity {
            return Err("Split part of a stack, leaving at least one item".into());
        }
        if self.slots.len() >= INVENTORY_SLOTS {
            return Err("Inventory is full".into());
        }
        self.slots[slot].quantity -= quantity;
        self.slots.push(Stack {
            item: stack.item,
            quantity,
            wear: 0,
        });
        Ok(())
    }

    pub fn move_stack(&mut self, from: usize, to: usize) -> Result<(), String> {
        if from == to || from >= self.slots.len() || to >= self.slots.len() {
            return Err("Move a stack onto another occupied slot".into());
        }
        let (source, target) = (self.slots[from], self.slots[to]);
        if source.item != target.item || source.item.is_tool() {
            self.slots.swap(from, to);
            return Ok(());
        }
        let moved = source
            .quantity
            .min(target.item.stack_limit() - target.quantity);
        if moved == 0 {
            return Err("That stack is already full".into());
        }
        self.slots[to].quantity += moved;
        self.slots[from].quantity -= moved;
        if self.slots[from].quantity == 0 {
            self.slots.remove(from);
        }
        Ok(())
    }

    pub fn discard(&mut self, slot: usize, quantity: u32) -> Result<Stack, String> {
        let stack = self.slots.get_mut(slot).ok_or("No stack in that slot")?;
        if quantity == 0 || quantity > stack.quantity {
            return Err("Discard between one item and the whole stack".into());
        }
        stack.quantity -= quantity;
        let taken = Stack { quantity, ..*stack };
        if stack.quantity == 0 {
            self.slots.remove(slot);
        }
        Ok(taken)
    }
}

impl<'de> Deserialize<'de> for Inventory {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Saved {
            slots: Vec<Stack>,
        }
        let saved = Saved::deserialize(deserializer)?;
        Self::from_stacks(saved.slots).map_err(serde::de::Error::custom)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Effects {
    pub heal: u32,
    pub hunger: f32,
    pub thirst: f32,
    pub ammo: u32,
    pub regen: f32,
    pub radiation: f32,
    pub warmth_seconds: f32,
    pub stop_bleeding: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct Vitals {
    hunger: f64,
    thirst: f64,
    damage_fraction: f64,
    bleed: f64,
    regen: f64,
    radiation: f64,
}

impl Default for Vitals {
    fn default() -> Self {
        Self {
            hunger: 100.,
            thirst: 100.,
            damage_fraction: 0.,
            bleed: 0.,
            regen: 0.,
            radiation: 0.,
        }
    }
}

impl Vitals {
    pub fn hunger(&self) -> f32 {
        self.hunger as f32
    }

    pub fn thirst(&self) -> f32 {
        self.thirst as f32
    }

    pub fn bleed(&self) -> f32 {
        self.bleed as f32
    }

    pub fn is_bleeding(&self) -> bool {
        self.bleed > 0.
    }

    pub fn regen(&self) -> f32 {
        self.regen as f32
    }

    pub fn radiation(&self) -> f32 {
        self.radiation as f32
    }

    pub fn regenerate(&mut self, dt_seconds: f32) -> Result<u32, String> {
        if !dt_seconds.is_finite() || !(0. ..=MAX_VITAL_SECONDS).contains(&dt_seconds) {
            return Err("Invalid survival time step".into());
        }
        let before = self.regen.ceil();
        self.regen = (self.regen - f64::from(dt_seconds) * REGEN_PER_SECOND).max(0.);
        Ok((before - self.regen.ceil()) as u32)
    }

    pub fn wound(&mut self, damage: u32) {
        if damage >= BLEED_THRESHOLD {
            self.bleed =
                (self.bleed + f64::from(damage) * BLEED_PER_DAMAGE).min(f64::from(MAX_BLEED));
        }
    }

    pub fn advance(&mut self, dt_seconds: f32, temperature: f32) -> Result<u32, String> {
        self.advance_with_exposure(dt_seconds, temperature, 0.)
    }

    pub fn advance_exposed(
        &mut self,
        dt_seconds: f32,
        temperature: f32,
        irradiated: bool,
    ) -> Result<u32, String> {
        let exposure = if irradiated { 1. } else { 0. };
        self.advance_with_exposure(dt_seconds, temperature, exposure)
    }

    pub fn advance_with_exposure(
        &mut self,
        dt_seconds: f32,
        temperature: f32,
        exposure: f32,
    ) -> Result<u32, String> {
        if !dt_seconds.is_finite() || !(0. ..=MAX_VITAL_SECONDS).contains(&dt_seconds) {
            return Err("Invalid survival time step".into());
        }
        if !temperature.is_finite() {
            return Err("Invalid temperature".into());
        }
        if !exposure.is_finite() || !(0. ..=1.).contains(&exposure) {
            return Err("Invalid radiation exposure".into());
        }
        let dt = f64::from(dt_seconds);
        let rise = RADIATION_PER_SECOND * f64::from(exposure);
        let hunger_rate = if temperature < COLD_CELSIUS {
            0.04
        } else {
            0.02
        };
        let freezing = if temperature < FREEZING_CELSIUS {
            dt * FREEZE_PER_SECOND
        } else {
            0.
        };
        let sick = f64::from(RADIATION_SICK);
        let sick_time = if rise > 0. {
            (dt - ((sick - self.radiation) / rise).max(0.)).max(0.)
        } else {
            ((self.radiation - sick) / RADIATION_DECAY_PER_SECOND).clamp(0., dt)
        };
        let sickness = sick_time * RADIATION_DAMAGE_PER_SECOND;
        self.radiation = if rise > 0. {
            (self.radiation + dt * rise).min(f64::from(MAX_RADIATION))
        } else {
            (self.radiation - dt * RADIATION_DECAY_PER_SECOND).max(0.)
        };
        let hungry_time = (dt - self.hunger / hunger_rate).max(0.);
        let thirsty_time = (dt - self.thirst / 0.04).max(0.);
        self.hunger = (self.hunger - dt * hunger_rate).max(0.);
        self.thirst = (self.thirst - dt * 0.04).max(0.);
        let bled = self.bleed.min(dt * BLEED_PER_SECOND);
        self.bleed -= bled;
        let damage =
            self.damage_fraction + hungry_time + thirsty_time * 2. + bled + freezing + sickness;
        let whole = damage.floor();
        self.damage_fraction = damage - whole;
        Ok(whole as u32)
    }

    pub fn apply(&mut self, effects: &Effects) -> Result<(), String> {
        if !effects.hunger.is_finite()
            || !effects.thirst.is_finite()
            || !(0. ..=100.).contains(&effects.hunger)
            || !(0. ..=100.).contains(&effects.thirst)
            || !effects.regen.is_finite()
            || !(0. ..=MAX_REGEN).contains(&effects.regen)
            || !effects.radiation.is_finite()
            || !(0. ..=MAX_RADIATION).contains(&effects.radiation)
        {
            return Err("Invalid consumable vital effects".into());
        }
        self.hunger = (self.hunger + f64::from(effects.hunger)).min(100.);
        self.thirst = (self.thirst + f64::from(effects.thirst)).min(100.);
        self.regen = (self.regen + f64::from(effects.regen)).min(f64::from(MAX_REGEN));
        self.radiation = (self.radiation - f64::from(effects.radiation)).max(0.);
        if effects.stop_bleeding {
            self.bleed = 0.;
        }
        Ok(())
    }
}

impl<'de> Deserialize<'de> for Vitals {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Saved {
            hunger: f64,
            thirst: f64,
            damage_fraction: f64,
            #[serde(default)]
            bleed: f64,
            #[serde(default)]
            regen: f64,
            #[serde(default)]
            radiation: f64,
        }
        let saved = Saved::deserialize(deserializer)?;
        if !saved.hunger.is_finite()
            || !saved.thirst.is_finite()
            || !saved.damage_fraction.is_finite()
            || !(0. ..=100.).contains(&saved.hunger)
            || !(0. ..=100.).contains(&saved.thirst)
            || !(0. ..1.).contains(&saved.damage_fraction)
            || !(0. ..=f64::from(MAX_BLEED)).contains(&saved.bleed)
            || !(0. ..=f64::from(MAX_REGEN)).contains(&saved.regen)
            || !(0. ..=f64::from(MAX_RADIATION)).contains(&saved.radiation)
        {
            return Err(serde::de::Error::custom("Invalid survival vitals"));
        }
        Ok(Self {
            hunger: saved.hunger,
            thirst: saved.thirst,
            damage_fraction: saved.damage_fraction,
            bleed: saved.bleed,
            regen: saved.regen,
            radiation: saved.radiation,
        })
    }
}
