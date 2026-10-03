use survival::Session;

pub(super) fn wear(session: &mut Session, slot: usize) -> Result<String, String> {
    session.wear(slot)?;
    Ok(session.message.clone())
}

pub(super) fn take_off(session: &mut Session) -> Result<String, String> {
    let name = session.worn().map(|item| item.name());
    session.take_off()?;
    Ok(format!("Removed {}", name.unwrap_or("clothing")))
}

pub(super) fn status(session: &Session) -> String {
    let name = session.worn().map_or("None", |item| item.name());
    format!("Worn: {name} | O wear / P remove")
}
