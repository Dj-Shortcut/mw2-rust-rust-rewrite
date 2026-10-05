use super::network::ReceivedWorld;
use survival::{Item, SHARED_STEP_MS, SHARED_TRADE_WOOD, SharedTradeOutcomeKind, SharedTradeParty};

pub(super) fn panel(world: &ReceivedWorld, pending: bool, message: &str) -> String {
    let inventory = &world.snapshot.inventory;
    let owner = world
        .snapshot
        .owners
        .iter()
        .find(|(client, _)| *client == world.snapshot.recipient)
        .map(|(_, owner)| *owner);
    let wood = owner
        .map(|owner| world.replica.buildings().inventory(owner).wood.to_string())
        .unwrap_or_else(|| "unavailable".into());
    let mut content = format!(
        "INVENTORY\nConfirmed: Wood {wood} | Bandages {} | Cloth {}\nBandage cost: 4 Cloth | Confirmed carried items.",
        inventory.count(Item::Bandage),
        inventory.count(Item::Cloth),
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

    let recipient = owner.map(|owner| SharedTradeParty {
        client: world.snapshot.recipient,
        owner,
    });
    content.push_str(&format!("\nTRADE | 1 Bandage for {SHARED_TRADE_WOOD} Wood"));
    let offer = world
        .snapshot
        .trade_offer
        .filter(|offer| recipient == Some(offer.seller) || recipient == Some(offer.buyer));
    if let Some(offer) = offer {
        let tenths = (u64::from(offer.expires_at.0.saturating_sub(world.snapshot.tick.0))
            * SHARED_STEP_MS as u64)
            .div_ceil(100);
        let seller = recipient == Some(offer.seller);
        let role = if seller { "YOU SELL" } else { "YOU BUY" };
        let peer = if seller { offer.buyer } else { offer.seller };
        content.push_str(&format!(
            "\nOffer #{}: {role} | Player {} | {}.{}s left\nBuyer must accept; no items reserved.",
            offer.id,
            peer.client.0,
            tenths / 10,
            tenths % 10,
        ));
        if seller {
            content.push_str("\nBackspace / Controller B cancel outgoing offer");
        } else {
            content.push_str("\nEnter / Controller A accept | Backspace / Controller B decline");
        }
    } else {
        content.push_str("\nNo active offer | V / Controller Right offer 1 Bandage");
    }
    if let Some(outcome) = world
        .snapshot
        .trade_outcome
        .filter(|outcome| recipient == Some(outcome.seller) || recipient == Some(outcome.buyer))
    {
        let seller = recipient == Some(outcome.seller);
        let result = match outcome.kind {
            SharedTradeOutcomeKind::Accepted if seller => "ACCEPTED | Sold 1 Bandage",
            SharedTradeOutcomeKind::Accepted => "ACCEPTED | Bought 1 Bandage",
            SharedTradeOutcomeKind::Cancelled => "CANCELLED by seller",
            SharedTradeOutcomeKind::Declined => "DECLINED by buyer",
            SharedTradeOutcomeKind::Expired => "EXPIRED",
            SharedTradeOutcomeKind::Unavailable => "UNAVAILABLE | Player left or died",
        };
        content.push_str(&format!("\nLast trade #{}: {result}", outcome.offer_id));
    }
    if pending {
        content.push_str("\nAction pending...");
    }
    if !message.is_empty() && !(pending && message == "Action pending...") {
        content.push_str(&format!("\nLast own action: {message}"));
    }
    content.push_str("\nI / Controller Up close | C / Controller X craft 1 Bandage");
    content
}
