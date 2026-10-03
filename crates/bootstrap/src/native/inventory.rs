use survival::{Inventory, LOCAL, Session};

const SLOT_CAPACITY: usize = 24;

#[derive(Clone, Copy)]
pub(super) enum Action {
    Navigate(isize),
    Adjust(isize),
    Activate,
    Split,
    Discard,
    Cancel,
}

enum Pending {
    Move { source: usize },
    Discard { source: usize, quantity: u32 },
}

pub(super) struct InventoryUi {
    pub(super) slot: usize,
    pub(super) quantity: u32,
    pending: Option<Pending>,
    observed: Option<Inventory>,
    notice: Option<String>,
}

impl Default for InventoryUi {
    fn default() -> Self {
        Self {
            slot: 0,
            quantity: 1,
            pending: None,
            observed: None,
            notice: None,
        }
    }
}

impl InventoryUi {
    pub(super) fn reset(&mut self) {
        *self = Self::default();
    }

    pub(super) fn sync(&mut self, session: &Session) {
        self.slot %= SLOT_CAPACITY;
        let changed = self
            .observed
            .as_ref()
            .is_some_and(|inventory| inventory != &session.inventory);
        if changed {
            self.notice = self
                .pending
                .take()
                .map(|_| "Inventory changed; pending action cancelled.".into());
            self.slot = self
                .slot
                .min(session.inventory.stacks().len().saturating_sub(1));
        }
        if changed || self.observed.is_none() {
            self.select_quantity(session);
        }
        if !Self::alive(session) && self.pending.take().is_some() {
            self.notice = Some("Player is not alive; pending action cancelled.".into());
        }
        self.observed = Some(session.inventory.clone());
    }

    pub(super) fn pending(&self) -> bool {
        self.pending.is_some()
    }

    pub(super) fn apply(&mut self, action: Action, session: &mut Session) {
        let had_pending = self.pending();
        self.sync(session);
        if had_pending
            && !self.pending()
            && !matches!(
                action,
                Action::Navigate(_) | Action::Adjust(_) | Action::Cancel
            )
        {
            session.message = self
                .notice
                .as_deref()
                .unwrap_or("Inventory action cancelled.")
                .into();
            return;
        }
        if matches!(action, Action::Activate | Action::Split | Action::Discard)
            && !Self::alive(session)
        {
            self.pending = None;
            session.message = "Player is not alive".into();
            self.notice = Some(session.message.clone());
            return;
        }
        match action {
            Action::Navigate(delta) => {
                let next =
                    (self.slot + delta.rem_euclid(SLOT_CAPACITY as isize) as usize) % SLOT_CAPACITY;
                if next != self.slot {
                    self.slot = next;
                    self.select_quantity(session);
                    if self.cancel_discard() {
                        session.message = "Discard cancelled".into();
                    }
                    self.notice = None;
                }
            }
            Action::Adjust(delta) => {
                let Some(stack) = session.inventory.stacks().get(self.slot) else {
                    session.message = "This slot is empty".into();
                    self.notice = Some(session.message.clone());
                    return;
                };
                let quantity = (i128::from(self.quantity) + delta as i128)
                    .clamp(1, i128::from(stack.quantity)) as u32;
                if quantity != self.quantity {
                    self.quantity = quantity;
                    if self.cancel_discard() {
                        session.message = "Discard cancelled".into();
                    }
                    self.notice = None;
                }
            }
            Action::Activate => self.activate(session),
            Action::Split => {
                self.pending = None;
                match session.split_stack(self.slot, self.quantity) {
                    Ok(()) => {
                        self.after_mutation(session);
                        session.message = "Stack split into the next free slot".into();
                    }
                    Err(error) => {
                        self.pending = None;
                        self.notice = Some(error.clone());
                        session.message = error;
                    }
                }
            }
            Action::Discard => {
                self.pending = None;
                self.notice = None;
                let Some(stack) = session.inventory.stacks().get(self.slot) else {
                    session.message = "This slot is empty".into();
                    self.notice = Some(session.message.clone());
                    return;
                };
                self.pending = Some(Pending::Discard {
                    source: self.slot,
                    quantity: self.quantity,
                });
                self.notice = None;
                session.message = format!(
                    "Destroy {} x{}? Enter / Xbox Y confirms; Backspace / Xbox B cancels.",
                    stack.item.name(),
                    self.quantity
                );
            }
            Action::Cancel => {
                self.pending = None;
                self.notice = None;
                session.message = "Inventory action cancelled".into();
            }
        }
    }

    pub(super) fn status(&self, session: &Session) -> String {
        let selected = session.inventory.stacks().get(self.slot);
        let mut status = match selected {
            Some(stack) => format!(
                "Slot {:02}: {} x{} | Split/discard quantity {}",
                self.slot + 1,
                stack.item.name(),
                stack.quantity,
                self.quantity
            ),
            None => format!(
                "Slot {:02}: empty. New stacks fill empty slots automatically.",
                self.slot + 1
            ),
        };
        if self
            .observed
            .as_ref()
            .is_some_and(|inventory| inventory != &session.inventory)
        {
            status.push_str("\nInventory changed; choose a stack again.");
            return status;
        }
        match self.pending {
            Some(Pending::Move { source }) => {
                if let Some(stack) = session.inventory.stacks().get(source) {
                    status.push_str(&format!(
                        "\nMove whole {} x{} from slot {:02}. Choose an occupied target.\nEnter / Xbox Y merges or swaps; Backspace / Xbox B cancels.",
                        stack.item.name(), stack.quantity, source + 1
                    ));
                }
            }
            Some(Pending::Discard { source, quantity }) => {
                if let Some(stack) = session.inventory.stacks().get(source) {
                    status.push_str(&format!(
                        "\nDESTROY {} x{} from slot {:02}. Items are destroyed.\nEnter / Xbox Y confirms; Backspace / Xbox B cancels.",
                        stack.item.name(), quantity, source + 1
                    ));
                }
            }
            None => {
                if selected.is_none() {
                    status.push_str("\nMoves need an occupied target.");
                }
                if let Some(notice) = self.notice.as_deref() {
                    status.push_str(&format!("\n{notice}"));
                }
            }
        }
        status
    }

    fn activate(&mut self, session: &mut Session) {
        match self.pending {
            Some(Pending::Move { source }) => {
                let target = self.slot;
                let count = session.inventory.stacks().len();
                let merges = session
                    .inventory
                    .stacks()
                    .get(source)
                    .zip(session.inventory.stacks().get(target))
                    .is_some_and(|(source, target)| {
                        source.item == target.item && !source.item.is_tool()
                    });
                match session.move_stack(source, target) {
                    Ok(()) => {
                        if session.inventory.stacks().len() < count && source < target {
                            self.slot = target - 1;
                        }
                        self.after_mutation(session);
                        session.message = if merges {
                            "Stacks merged"
                        } else {
                            "Stacks swapped"
                        }
                        .into();
                    }
                    Err(error) => {
                        self.pending = None;
                        self.notice = Some(error.clone());
                        session.message = error;
                    }
                }
            }
            Some(Pending::Discard { source, quantity }) => {
                match session.discard_stack(source, quantity) {
                    Ok(stack) => {
                        self.after_mutation(session);
                        session.message =
                            format!("Destroyed {} x{}", stack.item.name(), stack.quantity);
                    }
                    Err(error) => {
                        self.pending = None;
                        self.notice = Some(error.clone());
                        session.message = error;
                    }
                }
            }
            None => {
                if session.inventory.stacks().get(self.slot).is_none() {
                    session.message = "Choose an occupied stack to move".into();
                    self.notice = Some(session.message.clone());
                    return;
                }
                self.pending = Some(Pending::Move { source: self.slot });
                self.notice = None;
                session.message = "Whole stack selected; choose an occupied target".into();
            }
        }
    }

    fn alive(session: &Session) -> bool {
        session
            .world
            .player(LOCAL)
            .is_some_and(|player| player.health > 0)
    }

    fn select_quantity(&mut self, session: &Session) {
        self.quantity = session
            .inventory
            .stacks()
            .get(self.slot)
            .map_or(1, |stack| (stack.quantity / 2).max(1));
    }

    fn cancel_discard(&mut self) -> bool {
        if matches!(self.pending, Some(Pending::Discard { .. })) {
            self.pending = None;
            true
        } else {
            false
        }
    }

    fn after_mutation(&mut self, session: &Session) {
        self.pending = None;
        self.notice = None;
        self.slot = self
            .slot
            .min(session.inventory.stacks().len().saturating_sub(1));
        self.select_quantity(session);
        self.observed = Some(session.inventory.clone());
    }
}
