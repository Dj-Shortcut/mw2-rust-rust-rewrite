use survival::{ResourceKind, Session};

pub(super) fn toggle(session: &mut Session) -> Result<String, String> {
    if session.fishing().is_some() {
        session.reel_in()?;
    } else {
        session.cast_line()?;
    }
    Ok(session.message.clone())
}

pub(super) fn hint(session: &Session) -> Option<String> {
    if let Some(cast) = session.fishing() {
        return Some(format!(
            "Fishing at water #{} | {:.1} s | L / Xbox D-pad Right to reel in",
            cast.node, cast.remaining
        ));
    }
    let node = session
        .gather_target_from_view()
        .ok()
        .flatten()
        .filter(|node| node.kind == ResourceKind::Water)?;
    Some(format!(
        "Water #{} | L / Xbox D-pad Right to cast line",
        node.id
    ))
}
