use crate::gathering::height_at;
use crate::{Inventory, Stack, Terrain};

pub const STASH_REACH: f32 = 100.;
const POSITION: [f32; 2] = [140., -180.];

#[derive(Clone, Debug, PartialEq)]
pub struct Stash {
    position: [f32; 3],
    inventory: Inventory,
}

impl Stash {
    pub(crate) fn new(terrain: &Terrain) -> Result<Self, String> {
        let z = height_at(terrain, POSITION).ok_or("Stash lies outside terrain mesh")?;
        Ok(Self {
            position: [POSITION[0], POSITION[1], z],
            inventory: Inventory::default(),
        })
    }

    pub fn position(&self) -> [f32; 3] {
        self.position
    }

    pub fn stacks(&self) -> &[Stack] {
        self.inventory.stacks()
    }

    pub fn in_reach(&self, origin: [f32; 3]) -> bool {
        (0..3)
            .map(|k| (self.position[k] - origin[k]).powi(2))
            .sum::<f32>()
            .sqrt()
            <= STASH_REACH
    }

    pub(crate) fn inventory(&self) -> &Inventory {
        &self.inventory
    }

    pub(crate) fn set_inventory(&mut self, inventory: Inventory) {
        self.inventory = inventory;
    }
}

pub(crate) fn transfer(
    from: &mut Inventory,
    to: &mut Inventory,
    slot: usize,
    quantity: u32,
) -> Result<Stack, String> {
    let stack = *from.stacks().get(slot).ok_or("No stack in that slot")?;
    if quantity == 0 || quantity > stack.quantity {
        return Err("Move between one item and the whole stack".into());
    }
    let moved = from.discard(slot, quantity)?;
    to.add_stack(moved)?;
    Ok(moved)
}
