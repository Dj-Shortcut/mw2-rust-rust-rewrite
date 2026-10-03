use crate::{Inventory, WORLD_HALF};
use serde::{Deserialize, Deserializer, Serialize};
pub const MAX_LOOT_BAGS: usize = 16;
pub const LOOT_REACH: f32 = 100.;

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct LootBag {
    id: u32,
    position: [f32; 3],
    inventory: Inventory,
}

impl LootBag {
    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn position(&self) -> [f32; 3] {
        self.position
    }

    pub fn inventory(&self) -> &Inventory {
        &self.inventory
    }
}

impl<'de> Deserialize<'de> for LootBag {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Saved {
            id: u32,
            position: [f32; 3],
            inventory: Inventory,
        }
        let saved = Saved::deserialize(deserializer)?;
        if saved.id == 0
            || saved
                .position
                .iter()
                .any(|v| !v.is_finite() || v.abs() > WORLD_HALF)
            || saved.inventory.stacks().is_empty()
        {
            return Err(serde::de::Error::custom("Invalid loot bag"));
        }
        Ok(Self {
            id: saved.id,
            position: saved.position,
            inventory: saved.inventory,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct LootBags {
    next_id: u32,
    bags: Vec<LootBag>,
}

impl Default for LootBags {
    fn default() -> Self {
        Self {
            next_id: 1,
            bags: Vec::new(),
        }
    }
}

impl LootBags {
    pub fn bags(&self) -> &[LootBag] {
        &self.bags
    }
    pub(crate) fn drop_bag(
        &mut self,
        position: [f32; 3],
        inventory: Inventory,
    ) -> Result<Option<u32>, String> {
        if inventory.stacks().is_empty() {
            return Ok(None);
        }
        if position
            .iter()
            .any(|v| !v.is_finite() || v.abs() > WORLD_HALF)
        {
            return Err("Loot bag position is out of range".into());
        }
        let id = self.next_id;
        let next_id = id.checked_add(1).ok_or("Loot bag IDs exhausted")?;
        if self.bags.len() >= MAX_LOOT_BAGS {
            self.bags.remove(0);
        }
        self.bags.push(LootBag {
            id,
            position,
            inventory,
        });
        self.next_id = next_id;
        Ok(Some(id))
    }
    pub fn nearest(&self, origin: [f32; 3]) -> Option<&LootBag> {
        let distance = |bag: &LootBag| {
            let p = bag.position();
            ((p[0] - origin[0]).powi(2) + (p[1] - origin[1]).powi(2) + (p[2] - origin[2]).powi(2))
                .sqrt()
        };
        self.bags
            .iter()
            .filter(|b| distance(b) <= LOOT_REACH)
            .min_by(|a, b| distance(a).total_cmp(&distance(b)))
    }
    pub(crate) fn take(&mut self, id: u32, inventory: &mut Inventory) -> Result<u32, String> {
        let index = self
            .bags
            .iter()
            .position(|b| b.id == id)
            .ok_or("That loot bag is gone")?;
        let mut moved = 0;
        let mut left = Vec::new();
        for stack in self.bags[index].inventory.stacks() {
            if stack.wear > 0 {
                if inventory.add_stack(*stack).is_ok() {
                    moved += stack.quantity;
                } else {
                    left.push(*stack);
                }
                continue;
            }
            let added = inventory.add_up_to(stack.item, stack.quantity);
            moved += added;
            if added < stack.quantity {
                left.push(crate::Stack {
                    item: stack.item,
                    quantity: stack.quantity - added,
                    wear: 0,
                });
            }
        }
        if moved == 0 {
            return Err("Inventory is full".into());
        }
        if left.is_empty() {
            self.bags.remove(index);
        } else {
            self.bags[index].inventory = Inventory::from_stacks(left)?;
        }
        Ok(moved)
    }
}

impl<'de> Deserialize<'de> for LootBags {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Saved {
            next_id: u32,
            bags: Vec<LootBag>,
        }
        let saved = Saved::deserialize(deserializer)?;
        let mut ids: Vec<u32> = saved.bags.iter().map(|b| b.id).collect();
        ids.sort_unstable();
        ids.dedup();
        if saved.bags.len() > MAX_LOOT_BAGS
            || ids.len() != saved.bags.len()
            || ids.last().is_some_and(|&max| max >= saved.next_id)
            || saved.next_id == 0
            || saved.next_id == u32::MAX
        {
            return Err(serde::de::Error::custom("Invalid loot bags"));
        }
        Ok(Self {
            next_id: saved.next_id,
            bags: saved.bags,
        })
    }
}
