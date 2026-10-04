use super::network::ReceivedWorld;
use survival::Item;

pub(super) fn panel(world: &ReceivedWorld, pending: bool, message: &str) -> String {
    let inventory = &world.snapshot.inventory;
    let mut content = format!(
        "INVENTORY\nCloth: {} | Bandage cost: 4 Cloth\nConfirmed carried items.",
        inventory.count(Item::Cloth)
    );
    if inventory.stacks().is_empty() {
        content.push_str("\nInventory is empty");
    }
    for (row, pair) in inventory.stacks().chunks(2).enumerate() {
        let first = pair[0];
        content.push_str(&format!(
            "\n{} {} x{}",
            row * 2 + 1,
            first.item.name(),
            first.quantity
        ));
        if let Some(second) = pair.get(1) {
            content.push_str(&format!(
                " | {} {} x{}",
                row * 2 + 2,
                second.item.name(),
                second.quantity
            ));
        }
    }
    if pending {
        content.push_str("\nAction pending...");
    }
    if !message.is_empty() && !(pending && message == "Action pending...") {
        content.push_str(&format!("\n{message}"));
    }
    content.push_str("\nI / Controller Up close | C / Controller X craft 1 Bandage");
    content
}
