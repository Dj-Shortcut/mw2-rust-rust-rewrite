use survival::{Recipe, Session};

pub(super) fn apply(session: &mut Session, recipe: Recipe) -> Result<String, String> {
    let cost = session.research(recipe)?;
    Ok(format!(
        "Researched {} | wood {} / stone {} / metal {}",
        recipe.name(),
        cost.wood,
        cost.stone,
        cost.metal
    ))
}

pub(super) fn status(session: &Session, recipe: Recipe) -> String {
    if !recipe.needs_blueprint() {
        format!("{} | No research required", recipe.name())
    } else if session.blueprints().knows(recipe) {
        format!("{} | Blueprint known", recipe.name())
    } else {
        let cost = recipe.research_cost();
        format!(
            "{} | R research | wood {} / stone {} / metal {}",
            recipe.name(),
            cost.wood,
            cost.stone,
            cost.metal
        )
    }
}
