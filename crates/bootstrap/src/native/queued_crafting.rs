use survival::{LOCAL, MAX_CRAFT_JOBS, Recipe, Session};

#[derive(Clone, Copy)]
pub(super) enum Action {
    Queue(Recipe),
    CancelFirst,
}

pub(super) fn apply(session: &mut Session, action: Action) -> Result<String, String> {
    if session
        .world
        .player(LOCAL)
        .is_none_or(|player| player.health <= 0)
    {
        return Err("Player is not alive".into());
    }
    match action {
        Action::Queue(recipe) => session
            .queue_craft(recipe)
            .map(|()| format!("Queued {}", recipe.name())),
        Action::CancelFirst => {
            let name = session
                .crafting_queue()
                .first()
                .map_or("crafting job", |job| job.recipe.name());
            let refund = session.cancel_craft(0)?;
            Ok(format!(
                "Cancelled {name} | Refunded: {} wood, {} stone, {} metal",
                refund.wood, refund.stone, refund.metal
            ))
        }
    }
}

pub(super) fn summary(session: &Session, inventory_open: bool) -> String {
    let queue = session.crafting_queue();
    let Some(first) = queue.first() else {
        return "Queue: empty".into();
    };
    let progress = if first.remaining <= 0. {
        "Ready; waiting for delivery".into()
    } else {
        format!("{:.1} simulation s", first.remaining)
    };
    let mut text = format!(
        "Queue: {}/{} | {} | {progress}",
        queue.len(),
        MAX_CRAFT_JOBS,
        first.recipe.name()
    );
    if inventory_open {
        text.push_str(" | Close inventory to progress");
    }
    text
}
