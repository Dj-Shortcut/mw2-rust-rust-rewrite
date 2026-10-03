use rust_building::Grade;
use survival::Session;

#[derive(Clone, Copy)]
pub(super) enum Action {
    Repair,
    Upgrade(Grade),
}

pub(super) fn apply(session: &mut Session, action: Action) -> Result<String, String> {
    match action {
        Action::Repair => session
            .repair_from_view()
            .map(|()| "Building repaired".into()),
        Action::Upgrade(grade) => session
            .upgrade_from_view(grade)
            .map(|()| format!("Building upgraded to {grade:?}")),
    }
}
