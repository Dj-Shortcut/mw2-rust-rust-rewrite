use rust_building::Resources;
use serde::{Deserialize, Deserializer, Serialize};

pub const INVENTORY_SLOTS: usize = 24;
pub const MAX_VITAL_SECONDS: f32 = 3600.;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Item {
    Bandage,
    Ammo,
    Food,
    Water,
}

impl Item {
    pub const ALL: [Self; 4] = [Self::Bandage, Self::Ammo, Self::Food, Self::Water];

    pub fn name(self) -> &'static str {
        match self {
            Self::Bandage => "Bandage",
            Self::Ammo => "Carbine ammunition",
            Self::Food => "Food",
            Self::Water => "Water",
        }
    }

    pub fn stack_limit(self) -> u32 {
        match self {
            Self::Bandage | Self::Water => 10,
            Self::Ammo => 60,
            Self::Food => 20,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Recipe {
    Bandage,
    Ammo,
}

impl Recipe {
    pub const ALL: [Self; 2] = [Self::Bandage, Self::Ammo];

    pub fn name(self) -> &'static str {
        match self {
            Self::Bandage => "Bandage",
            Self::Ammo => "30 carbine rounds",
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
        }
    }

    pub fn output(self) -> (Item, u32) {
        match self {
            Self::Bandage => (Item::Bandage, 1),
            Self::Ammo => (Item::Ammo, 30),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Stack {
    pub item: Item,
    pub quantity: u32,
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
            || slots
                .iter()
                .any(|s| s.quantity == 0 || s.quantity > s.item.stack_limit())
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
            });
            remaining -= added;
        }
        Ok(())
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
        if quantity == 0 || item != Item::Ammo && quantity != 1 {
            return Err("Use one consumable or a positive number of rounds".into());
        }
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
        Ok(match item {
            Item::Bandage => Effects {
                heal: 25,
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
        })
    }

    /// Moves `quantity` items from the stack in `slot` into a new stack in
    /// the next free slot. Both stacks stay non-empty.
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
        });
        Ok(())
    }

    /// Moves the stack in `from` onto `to`. Stacks of the same item merge up
    /// to the stack limit and any rest stays in `from`; different items swap
    /// slots.
    pub fn move_stack(&mut self, from: usize, to: usize) -> Result<(), String> {
        if from == to || from >= self.slots.len() || to >= self.slots.len() {
            return Err("Move a stack onto another occupied slot".into());
        }
        let (source, target) = (self.slots[from], self.slots[to]);
        if source.item != target.item {
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

    /// Removes `quantity` items from the stack in `slot` and returns what
    /// was removed. The items are destroyed; there are no world drops yet.
    pub fn discard(&mut self, slot: usize, quantity: u32) -> Result<Stack, String> {
        let stack = self.slots.get_mut(slot).ok_or("No stack in that slot")?;
        if quantity == 0 || quantity > stack.quantity {
            return Err("Discard between one item and the whole stack".into());
        }
        stack.quantity -= quantity;
        let item = stack.item;
        if stack.quantity == 0 {
            self.slots.remove(slot);
        }
        Ok(Stack { item, quantity })
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
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct Vitals {
    hunger: f64,
    thirst: f64,
    damage_fraction: f64,
}

impl Default for Vitals {
    fn default() -> Self {
        Self {
            hunger: 100.,
            thirst: 100.,
            damage_fraction: 0.,
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

    pub fn advance(&mut self, dt_seconds: f32) -> Result<u32, String> {
        if !dt_seconds.is_finite() || !(0. ..=MAX_VITAL_SECONDS).contains(&dt_seconds) {
            return Err("Invalid survival time step".into());
        }
        let dt = f64::from(dt_seconds);
        let hungry_time = (dt - self.hunger / 0.02).max(0.);
        let thirsty_time = (dt - self.thirst / 0.04).max(0.);
        self.hunger = (self.hunger - dt * 0.02).max(0.);
        self.thirst = (self.thirst - dt * 0.04).max(0.);
        let damage = self.damage_fraction + hungry_time + thirsty_time * 2.;
        let whole = damage.floor();
        self.damage_fraction = damage - whole;
        Ok(whole as u32)
    }

    pub fn apply(&mut self, effects: &Effects) -> Result<(), String> {
        if !effects.hunger.is_finite()
            || !effects.thirst.is_finite()
            || !(0. ..=100.).contains(&effects.hunger)
            || !(0. ..=100.).contains(&effects.thirst)
        {
            return Err("Invalid consumable vital effects".into());
        }
        self.hunger = (self.hunger + f64::from(effects.hunger)).min(100.);
        self.thirst = (self.thirst + f64::from(effects.thirst)).min(100.);
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
        }
        let saved = Saved::deserialize(deserializer)?;
        if !saved.hunger.is_finite()
            || !saved.thirst.is_finite()
            || !saved.damage_fraction.is_finite()
            || !(0. ..=100.).contains(&saved.hunger)
            || !(0. ..=100.).contains(&saved.thirst)
            || !(0. ..1.).contains(&saved.damage_fraction)
        {
            return Err(serde::de::Error::custom("Invalid survival vitals"));
        }
        Ok(Self {
            hunger: saved.hunger,
            thirst: saved.thirst,
            damage_fraction: saved.damage_fraction,
        })
    }
}
