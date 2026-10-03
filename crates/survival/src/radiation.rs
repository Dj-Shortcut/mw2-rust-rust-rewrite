use crate::{CrateTier, LootCrates};

pub const RADIATION_RADIUS: f32 = 450.;
pub const MAX_RADIATION: f32 = 100.;
pub const RADIATION_SICK: f32 = 40.;
pub(crate) const RADIATION_PER_SECOND: f64 = 2.;
pub(crate) const RADIATION_DECAY_PER_SECOND: f64 = 0.5;
pub(crate) const RADIATION_DAMAGE_PER_SECOND: f64 = 0.1;

pub(crate) fn in_zone(crates: &LootCrates, origin: [f32; 3]) -> bool {
    crates
        .crates()
        .iter()
        .filter(|c| c.tier() == CrateTier::Military)
        .any(|c| {
            let p = c.position();
            (p[0] - origin[0]).hypot(p[1] - origin[1]) < RADIATION_RADIUS
        })
}
