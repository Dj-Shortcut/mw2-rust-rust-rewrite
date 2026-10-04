use crate::{Inventory, Recipe};
use rust_building::Resources;

pub(super) fn craft_bandage(inventory: &Inventory) -> Result<Inventory, String> {
    let mut candidate = inventory.clone();
    candidate.craft(Recipe::Bandage, Resources::default())?;
    Ok(candidate)
}
