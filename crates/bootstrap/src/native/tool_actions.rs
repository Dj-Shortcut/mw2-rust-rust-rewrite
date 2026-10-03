use survival::{MAX_TOOL_WEAR, Recipe, Session};

pub(super) fn repair(session: &mut Session, slot: usize) -> Result<String, String> {
    let name = session
        .inventory
        .stacks()
        .get(slot)
        .map_or("tool", |stack| stack.item.name());
    let cost = session.repair_tool(slot)?;
    Ok(format!(
        "Repaired {name} | Cost: {} wood, {} stone, {} metal",
        cost.wood, cost.stone, cost.metal
    ))
}

pub(super) fn recycle(session: &mut Session, slot: usize, quantity: u32) -> Result<String, String> {
    let name = session
        .inventory
        .stacks()
        .get(slot)
        .map_or("item", |stack| stack.item.name());
    let refund = session.recycle_stack(slot, quantity)?;
    Ok(format!(
        "Recycled {name} x{quantity} | Received: {} wood, {} stone, {} metal",
        refund.wood, refund.stone, refund.metal
    ))
}

pub(super) fn status(session: &Session, slot: usize) -> String {
    let Some(stack) = session.inventory.stacks().get(slot) else {
        return "Choose an occupied slot to repair or recycle.".into();
    };
    if stack.item.is_tool() {
        return match Recipe::repair_cost(stack.item, stack.wear) {
            Ok(cost) => format!(
                "Wear {}/{} | T repair: {} wood, {} stone, {} metal",
                stack.wear, MAX_TOOL_WEAR, cost.wood, cost.stone, cost.metal
            ),
            Err(error) => format!("Wear {}/{} | T repair: {error}", stack.wear, MAX_TOOL_WEAR),
        };
    }
    if Recipe::ALL
        .into_iter()
        .any(|recipe| recipe.output().0 == stack.item)
    {
        "N recycle: confirmation required.".into()
    } else {
        "Repair and recycling unavailable for this item.".into()
    }
}
