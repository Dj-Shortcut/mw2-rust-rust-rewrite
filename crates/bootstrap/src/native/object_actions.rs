use survival::Session;

#[derive(Clone, Copy)]
pub(super) enum Action {
    Move,
    Rotate(f32),
}

pub(super) fn apply(session: &mut Session, action: Action) -> Result<String, String> {
    match action {
        Action::Move => session
            .move_prop_to_view()
            .map(|id| format!("Object {id} moved")),
        Action::Rotate(step) => session
            .rotate_prop_from_view(step)
            .map(|id| format!("Object {id} rotated")),
    }
}
