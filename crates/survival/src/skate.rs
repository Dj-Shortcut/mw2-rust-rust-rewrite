use crate::{RailSegment, WORLD_HALF};
use sim::SimWorld;

const MINS: [f32; 3] = [-15., -15., 0.];
const MAXS: [f32; 3] = [15., 15., 70.];
const GRAVITY: f32 = 600.;
const MAX_SPEED: f32 = 650.;
const OLLIE_SPEED: f32 = 300.;
const GROUND_NORMAL: f32 = 0.45;
const MAX_SAVED_SCORE: u64 = 1_000_000_000;
const MAX_SAVED_BAILS: u32 = 1_000_000;

// Rail grinds. A descending rider whose sweep lands on a rail top catches it
// when the rider's origin lies between the endpoints within
// `GRIND_CAPTURE_LATERAL` of the centerline and the board heading and
// horizontal velocity are both within 25 degrees of the rail (either way).
const GRIND_ALIGN_COS: f32 = 0.906_307_8; // cos(25 degrees)
const GRIND_CAPTURE_LATERAL: f32 = 12.;
const GRIND_CAPTURE_HEIGHT: f32 = 1.;
const GRIND_MIN_ENTRY_SPEED: f32 = 60.;
/// Feet clearance above the beam top while grinding.
const GRIND_LIFT: f32 = 0.25;
/// Along-rail deceleration in units/s^2, plus `GRIND_BRAKE` times the brake.
const GRIND_DRAG: f32 = 40.;
const GRIND_BRAKE: f32 = 650.;
/// Below this along-rail speed the rider drops back to the normal controller.
const GRIND_STOP_SPEED: f32 = 30.;
/// After leaving a rail the same rail cannot be caught again for this long.
const GRIND_COOLDOWN: f32 = 0.25;
/// One pending grind point per this many units travelled along the rail.
const GRIND_POINT_DISTANCE: f32 = 12.;
/// Clearance past an endpoint before the box hull no longer overlaps the
/// beam: the hull's half-extent along the rail plus the beam half-width and
/// a margin. Bounded by `MAX_GRIND_OVERHANG` for any yaw.
const GRIND_OVERHANG_MARGIN: f32 = 4. + 1.;
const MAX_GRIND_OVERHANG: f32 = 15. * std::f32::consts::SQRT_2 + GRIND_OVERHANG_MARGIN;
const MAX_PENDING_DISTANCE: f32 = 100_000.;
const MAX_PENDING_TRICK: u32 = 1_000_000;

#[derive(Clone, Copy, Debug, Default)]
pub struct SkateInput {
    pub push: f32,
    pub steer: f32,
    pub brake: f32,
    pub ollie: bool,
    pub spin: f32,
    pub flip: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SkateEvent {
    #[default]
    None,
    Ollie,
    Landed {
        points: u32,
    },
    Bailed,
}

#[derive(Clone, Copy, Debug)]
pub struct SkateStep {
    pub origin: [f32; 3],
    pub velocity: [f32; 3],
    pub yaw: f32,
    pub grounded: bool,
    pub board_roll: f32,
    pub event: SkateEvent,
}

#[derive(Clone, Debug)]
pub struct SkateState {
    pub velocity: [f32; 3],
    pub yaw: f32,
    pub grounded: bool,
    pub total_score: u64,
    pub bails: u32,
    ground_normal: [f32; 3],
    air_time: f32,
    air_spin: f32,
    flip_angle: f32,
    flip_remaining: f32,
    flight: bool,
    grind: Option<Grind>,
    pending_distance: f32,
    pending_trick: u32,
    release_rail: Option<u32>,
    release_cooldown: f32,
}

/// An active grind: the rail's editor ID, the rider's signed distance from
/// the rail start along its centerline, and signed along-rail speed.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Grind {
    rail: u32,
    progress: f32,
    speed: f32,
}

impl Default for SkateState {
    fn default() -> Self {
        Self {
            velocity: [0.; 3],
            yaw: 0.,
            grounded: false,
            total_score: 0,
            bails: 0,
            ground_normal: [0., 0., 1.],
            air_time: 0.,
            air_spin: 0.,
            flip_angle: 0.,
            flip_remaining: 0.,
            flight: false,
            grind: None,
            pending_distance: 0.,
            pending_trick: 0,
            release_rail: None,
            release_cooldown: 0.,
        }
    }
}

/// Persisted mounted-skate state. Every field is checked by
/// [`SkateState::from_saved`] before it reaches the controller.
#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SavedSkate {
    pub velocity: [f32; 3],
    pub yaw: f32,
    pub grounded: bool,
    pub total_score: u64,
    pub bails: u32,
    pub ground_normal: [f32; 3],
    pub air_time: f32,
    pub air_spin: f32,
    pub flip_angle: f32,
    pub flip_remaining: f32,
    pub flight: bool,
    /// Grind fields default so mounted format-3 saves from before rail
    /// grinds load as ordinary skating.
    #[serde(default)]
    pub grind: Option<SavedGrind>,
    #[serde(default)]
    pub pending_distance: f32,
    #[serde(default)]
    pub pending_trick: u32,
    #[serde(default)]
    pub release_rail: Option<u32>,
    #[serde(default)]
    pub release_cooldown: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SavedGrind {
    pub rail: u32,
    pub progress: f32,
    pub speed: f32,
}

impl SkateState {
    pub fn new(yaw_degrees: f32) -> Result<Self, String> {
        if !yaw_degrees.is_finite() {
            return Err("Invalid skateboard heading".into());
        }
        Ok(Self {
            yaw: yaw_degrees.rem_euclid(360.),
            ..Default::default()
        })
    }

    pub fn saved(&self) -> SavedSkate {
        SavedSkate {
            velocity: self.velocity,
            yaw: self.yaw,
            grounded: self.grounded,
            total_score: self.total_score,
            bails: self.bails,
            ground_normal: self.ground_normal,
            air_time: self.air_time,
            air_spin: self.air_spin,
            flip_angle: self.flip_angle,
            flip_remaining: self.flip_remaining,
            flight: self.flight,
            grind: self.grind.map(|g| SavedGrind {
                rail: g.rail,
                progress: g.progress,
                speed: g.speed,
            }),
            pending_distance: self.pending_distance,
            pending_trick: self.pending_trick,
            release_rail: self.release_rail,
            release_cooldown: self.release_cooldown,
        }
    }

    pub fn from_saved(saved: SavedSkate) -> Result<Self, String> {
        let normal_length = length(saved.ground_normal);
        let valid = saved
            .velocity
            .iter()
            .all(|v| v.is_finite() && v.abs() <= MAX_SPEED + 1.)
            && length(saved.velocity) <= MAX_SPEED + 1.
            && saved.yaw.is_finite()
            && (0. ..360.).contains(&saved.yaw)
            && saved.total_score <= MAX_SAVED_SCORE
            && saved.bails <= MAX_SAVED_BAILS
            && saved.ground_normal.iter().all(|v| v.is_finite())
            && (normal_length - 1.).abs() <= 0.01
            && saved.ground_normal[2] >= GROUND_NORMAL
            && saved.air_time.is_finite()
            && (0. ..=30.).contains(&saved.air_time)
            && saved.air_spin.is_finite()
            && saved.air_spin.abs() <= 30. * 540.
            && saved.flip_remaining.is_finite()
            && (0. ..=360.).contains(&saved.flip_remaining)
            && saved.flip_angle.is_finite()
            && (0. ..=30. * 900.).contains(&saved.flip_angle);
        if !valid {
            return Err("Invalid saved skateboard state".into());
        }
        let pending = saved.pending_distance.is_finite()
            && (0. ..=MAX_PENDING_DISTANCE).contains(&saved.pending_distance)
            && saved.pending_trick <= MAX_PENDING_TRICK
            && (!saved.grounded || (saved.pending_distance == 0. && saved.pending_trick == 0));
        let cooldown = saved.release_cooldown.is_finite()
            && (0. ..=GRIND_COOLDOWN).contains(&saved.release_cooldown)
            && saved.release_rail != Some(0)
            && saved.release_rail.is_some() == (saved.release_cooldown > 0.);
        let grind = saved.grind.is_none_or(|g| {
            g.rail != 0
                && g.progress.is_finite()
                && (-MAX_GRIND_OVERHANG..=crate::editor::RAIL_LENGTH + MAX_GRIND_OVERHANG)
                    .contains(&g.progress)
                && g.speed.is_finite()
                && (GRIND_STOP_SPEED..=MAX_SPEED).contains(&g.speed.abs())
                && !saved.grounded
                && !saved.flight
        });
        if !(pending && cooldown && grind) {
            return Err("Invalid saved rail grind state".into());
        }
        Ok(Self {
            velocity: saved.velocity,
            yaw: saved.yaw,
            grounded: saved.grounded,
            total_score: saved.total_score,
            bails: saved.bails,
            ground_normal: normalize(saved.ground_normal),
            air_time: saved.air_time,
            air_spin: saved.air_spin,
            flip_angle: saved.flip_angle,
            flip_remaining: saved.flip_remaining,
            flight: saved.flight,
            grind: saved.grind.map(|g| Grind {
                rail: g.rail,
                progress: g.progress,
                speed: g.speed,
            }),
            pending_distance: saved.pending_distance,
            pending_trick: saved.pending_trick,
            release_rail: saved.release_rail,
            release_cooldown: saved.release_cooldown,
        })
    }

    /// Checks a restored grind against the loaded editor rails and the saved
    /// rider origin: the rail must exist, and the rider must sit on its
    /// centerline moving along it.
    pub fn check_rails(&self, rails: &[RailSegment], origin: [f32; 3]) -> Result<(), String> {
        let Some(grind) = self.grind else {
            return Ok(());
        };
        let rail = rails
            .iter()
            .find(|r| r.id == grind.rail)
            .ok_or("Saved grind references a missing rail")?;
        let (dir, len) = rail_frame(rail).ok_or("Saved grind rail is degenerate")?;
        let reach = overhang(dir);
        let seat = add(rail_point(rail, dir, grind.progress), [0., 0., GRIND_LIFT]);
        if !(-reach..=len + reach).contains(&grind.progress)
            || length(sub(origin, seat)) > 0.5
            || length(sub(self.velocity, scale(dir, grind.speed))) > 1.
            || dot(forward(self.yaw), dir).abs() < GRIND_ALIGN_COS - 0.001
        {
            return Err("Saved grind does not match its rail".into());
        }
        Ok(())
    }

    pub fn is_grinding(&self) -> bool {
        self.grind.is_some()
    }

    /// Trick and grind points waiting for the next safe landing.
    pub fn pending_points(&self) -> u32 {
        let distance = (self.pending_distance / GRIND_POINT_DISTANCE).floor() as u32;
        distance.saturating_add(self.pending_trick)
    }

    pub fn step(
        &mut self,
        world: &SimWorld,
        rails: &[RailSegment],
        dt_seconds: f32,
        input: SkateInput,
        origin: [f32; 3],
    ) -> Result<SkateStep, String> {
        if !dt_seconds.is_finite() || dt_seconds <= 0. || dt_seconds > 0.1 {
            return Err("Skateboard step must be between 0 and 0.1 seconds".into());
        }
        if origin
            .iter()
            .any(|v| !v.is_finite() || v.abs() > WORLD_HALF)
            || self
                .velocity
                .iter()
                .any(|v| !v.is_finite() || v.abs() > 10_000.)
            || !self.yaw.is_finite()
            || [input.push, input.steer, input.brake, input.spin]
                .iter()
                .any(|v| !v.is_finite())
        {
            return Err("Invalid skateboard motion".into());
        }
        let input = SkateInput {
            push: input.push.clamp(0., 1.),
            steer: input.steer.clamp(-1., 1.),
            brake: input.brake.clamp(0., 1.),
            spin: input.spin.clamp(-1., 1.),
            ..input
        };
        let mut next = self.clone();
        let output = next.advance(world, rails, dt_seconds, input, origin)?;
        *self = next;
        Ok(output)
    }

    fn advance(
        &mut self,
        world: &SimWorld,
        rails: &[RailSegment],
        dt: f32,
        input: SkateInput,
        mut origin: [f32; 3],
    ) -> Result<SkateStep, String> {
        origin = clear_origin(world, origin)?;
        let mut event = SkateEvent::None;
        if self
            .grind
            .is_some_and(|g| !rails.iter().any(|r| r.id == g.rail))
        {
            self.cancel_grind();
        }
        self.find_ground(world, rails, &mut origin, &mut event);
        if input.ollie && (self.grounded || self.grind.is_some()) {
            if let Some(grind) = self.grind {
                self.release(grind.rail);
            }
            self.velocity[2] += OLLIE_SPEED;
            self.grounded = false;
            self.begin_flight();
            event = SkateEvent::Ollie;
        }
        if input.flip && !self.grounded && self.grind.is_none() && self.flip_remaining <= 0. {
            self.flip_remaining = 360.;
        }
        let steps = (dt * 120.).ceil() as usize;
        let slice = dt / steps as f32;
        for _ in 0..steps {
            self.release_cooldown = (self.release_cooldown - slice).max(0.);
            if self.release_cooldown <= 0. {
                self.release_rail = None;
            }
            let mut remaining = slice;
            if self.grind.is_some() {
                remaining =
                    self.grind_slice(world, rails, slice, input, &mut origin, &mut event)?;
            }
            if remaining > 0. {
                self.move_free(world, rails, remaining, input, &mut origin, &mut event)?;
            }
            let limit = WORLD_HALF - MAXS[0] - 1.;
            if origin[0].abs() > limit || origin[1].abs() > limit {
                origin[0] = origin[0].clamp(-limit, limit);
                origin[1] = origin[1].clamp(-limit, limit);
                self.bail(&mut event);
            }
        }
        // Points are banked once per step, and never in a step that bailed.
        if let SkateEvent::Landed { points } = event {
            self.total_score = self.total_score.saturating_add(u64::from(points));
        }
        Ok(SkateStep {
            origin,
            velocity: self.velocity,
            yaw: self.yaw,
            grounded: self.grounded || self.grind.is_some(),
            board_roll: self.flip_angle.rem_euclid(360.),
            event,
        })
    }

    /// Ordinary ground or air movement for `dt` seconds.
    fn move_free(
        &mut self,
        world: &SimWorld,
        rails: &[RailSegment],
        dt: f32,
        input: SkateInput,
        origin: &mut [f32; 3],
        event: &mut SkateEvent,
    ) -> Result<(), String> {
        if self.grounded {
            let speed = length(self.velocity);
            let turn = input.steer * (35. + 70. * (speed / MAX_SPEED).min(1.));
            self.yaw = (self.yaw + turn * dt).rem_euclid(360.);
            let heading = forward(self.yaw);
            let tangent = normalize(project(heading, self.ground_normal));
            let along = dot(self.velocity, tangent);
            let grip = (dt * 8.).min(1.);
            let lateral = sub(self.velocity, scale(tangent, along));
            self.velocity = sub(self.velocity, scale(lateral, grip));
            self.velocity = add(self.velocity, scale(tangent, input.push * 245. * dt));
            self.velocity = add(
                self.velocity,
                scale(project([0., 0., -GRAVITY], self.ground_normal), dt),
            );
            let speed = length(self.velocity);
            if speed > 0. {
                self.velocity = scale(
                    self.velocity,
                    (speed - (18. + input.brake * 650.) * dt).max(0.) / speed,
                );
            }
        } else {
            if !self.flight {
                self.begin_flight();
            }
            self.air_time += dt;
            let spin = input.spin * 540. * dt;
            self.air_spin += spin;
            self.yaw = (self.yaw + spin).rem_euclid(360.);
            let flip = self.flip_remaining.min(900. * dt);
            self.flip_remaining -= flip;
            self.flip_angle += flip;
            self.velocity[2] -= GRAVITY * dt;
        }
        let speed = length(self.velocity);
        if speed > MAX_SPEED {
            self.velocity = scale(self.velocity, MAX_SPEED / speed);
        }
        self.sweep(world, rails, dt, origin, event)?;
        self.find_ground(world, rails, origin, event);
        Ok(())
    }

    /// Moves a grinding rider along the rail for up to `dt` seconds and
    /// returns the time left over after a release, for normal movement.
    fn grind_slice(
        &mut self,
        world: &SimWorld,
        rails: &[RailSegment],
        dt: f32,
        input: SkateInput,
        origin: &mut [f32; 3],
        event: &mut SkateEvent,
    ) -> Result<f32, String> {
        let Some(grind) = self.grind else {
            return Ok(dt);
        };
        let Some((rail, dir, len)) = rails
            .iter()
            .find(|r| r.id == grind.rail)
            .and_then(|r| rail_frame(r).map(|(dir, len)| (r, dir, len)))
        else {
            self.cancel_grind();
            return Ok(dt);
        };
        // Push adds nothing while latched; drag and brake only slow down.
        let magnitude = (grind.speed.abs() - (GRIND_DRAG + input.brake * GRIND_BRAKE) * dt)
            .clamp(0., MAX_SPEED);
        let speed = magnitude.copysign(grind.speed);
        if magnitude < GRIND_STOP_SPEED {
            self.velocity = scale(dir, speed);
            self.release(grind.rail);
            return Ok(dt);
        }
        let reach = overhang(dir);
        let target = grind.progress + speed * dt;
        let progress = target.clamp(-reach, len + reach);
        let seat = add(rail_point(rail, dir, progress), [0., 0., GRIND_LIFT]);
        let hit = world.trace_world(*origin, seat, MINS, MAXS, 1);
        if hit.startsolid != 0 || hit.allsolid != 0 {
            *origin = clear_origin(world, *origin)?;
            self.bail(event);
            return Ok(0.);
        }
        if hit.fraction < 1. {
            *origin = hit.endpos;
            self.bail(event);
            return Ok(0.);
        }
        // Only distance between the endpoints counts; the overhang that lets
        // the hull clear the beam end earns nothing.
        let travelled = (progress.clamp(0., len) - grind.progress.clamp(0., len)).abs();
        self.pending_distance = (self.pending_distance + travelled).min(MAX_PENDING_DISTANCE);
        *origin = seat;
        self.velocity = scale(dir, speed);
        self.grind = Some(Grind {
            progress,
            speed,
            ..grind
        });
        if progress != target {
            self.release(grind.rail);
            let used = ((progress - grind.progress) / speed).clamp(0., dt);
            return Ok(dt - used);
        }
        Ok(0.)
    }

    /// Latches onto a rail when a descending rider's contact with `normal`
    /// is the top of an aligned rail. Airborne trick points become pending
    /// rather than banked, so they are credited once at the final landing.
    fn try_catch(
        &mut self,
        world: &SimWorld,
        rails: &[RailSegment],
        origin: &mut [f32; 3],
        normal: [f32; 3],
        impact: f32,
    ) -> bool {
        let roll = self.flip_angle.rem_euclid(360.);
        if !self.flight
            || self.grounded
            || self.grind.is_some()
            || self.velocity[2] > 0.
            || normal[2] < 0.99
            || impact > 520.
            || roll.min(360. - roll) > 35.
        {
            return false;
        }
        let horizontal = [self.velocity[0], self.velocity[1], 0.];
        let heading = forward(self.yaw);
        let mut best: Option<(f32, &RailSegment, [f32; 3], f32, f32)> = None;
        for rail in rails {
            if self.release_rail == Some(rail.id) {
                continue;
            }
            let Some((dir, len)) = rail_frame(rail) else {
                continue;
            };
            let offset = sub(*origin, rail.start);
            let along = dot(offset, dir);
            let lateral = length(sub([offset[0], offset[1], 0.], scale(dir, along)));
            let height = offset[2];
            let speed = dot(horizontal, dir);
            if !(0. ..=len).contains(&along)
                || lateral > GRIND_CAPTURE_LATERAL
                || !(-0.1..=GRIND_CAPTURE_HEIGHT).contains(&height)
                || speed.abs() < GRIND_MIN_ENTRY_SPEED
                || speed.abs() < GRIND_ALIGN_COS * length(horizontal)
                || dot(heading, dir).abs() < GRIND_ALIGN_COS
            {
                continue;
            }
            if best.is_none_or(|(nearest, ..)| lateral < nearest) {
                best = Some((lateral, rail, dir, along, speed));
            }
        }
        let Some((_, rail, dir, along, speed)) = best else {
            return false;
        };
        let seat = add(rail_point(rail, dir, along), [0., 0., GRIND_LIFT]);
        let hit = world.trace_world(*origin, seat, MINS, MAXS, 1);
        if hit.startsolid != 0 || hit.allsolid != 0 || hit.fraction < 1. {
            return false;
        }
        if self.air_time > 0.08 {
            self.pending_trick = self
                .pending_trick
                .saturating_add(self.trick_points())
                .min(MAX_PENDING_TRICK);
        }
        self.end_flight();
        self.grounded = false;
        self.grind = Some(Grind {
            rail: rail.id,
            progress: along,
            speed,
        });
        self.velocity = scale(dir, speed);
        *origin = seat;
        true
    }

    /// Leaves the rail with momentum kept; the same rail cannot be caught
    /// again until the cooldown runs out.
    fn release(&mut self, rail: u32) {
        self.grind = None;
        self.grounded = false;
        self.flight = false;
        self.release_rail = Some(rail);
        self.release_cooldown = GRIND_COOLDOWN;
    }

    /// The rail disappeared under the rider (removed or undone): drop into
    /// normal motion and forfeit the unbanked points.
    fn cancel_grind(&mut self) {
        self.grind = None;
        self.grounded = false;
        self.flight = false;
        self.clear_pending();
    }

    fn clear_pending(&mut self) {
        self.pending_distance = 0.;
        self.pending_trick = 0;
    }

    fn trick_points(&self) -> u32 {
        let flips = (self.flip_angle / 360.).floor() as u32;
        let turns = ((self.air_spin.abs() + 15.) / 180.).floor() as u32;
        10 + turns * 50 + flips * 100
    }

    fn sweep(
        &mut self,
        world: &SimWorld,
        rails: &[RailSegment],
        dt: f32,
        origin: &mut [f32; 3],
        event: &mut SkateEvent,
    ) -> Result<(), String> {
        let mut remaining = dt;
        let mut planes = Vec::with_capacity(5);
        for _ in 0..5 {
            let end = add(*origin, scale(self.velocity, remaining));
            let hit = world.trace_world(*origin, end, MINS, MAXS, 1);
            if hit.startsolid != 0 || hit.allsolid != 0 {
                *origin = clear_origin(world, *origin)?;
                self.bail(event);
                break;
            }
            *origin = hit.endpos;
            if hit.fraction >= 1. {
                break;
            }
            if !hit.normal.iter().all(|v| v.is_finite()) || length(hit.normal) < 0.5 {
                return Err("Invalid skateboard contact plane".into());
            }
            let normal = normalize(hit.normal);
            if self.grounded && normal[2] < GROUND_NORMAL {
                if let Some((raised, ground)) = cross_curb(world, *origin, end) {
                    *origin = raised;
                    self.ground_normal = ground;
                    self.velocity = project(self.velocity, ground);
                    break;
                }
            }
            let impact = -dot(self.velocity, normal);
            if normal[2] >= GROUND_NORMAL && self.velocity[2] <= 0. {
                if self.try_catch(world, rails, origin, normal, impact) {
                    break;
                }
                self.land(normal, impact, event);
            } else if normal[2] < GROUND_NORMAL && impact > 220. {
                self.bail(event);
            }
            *origin = add(*origin, scale(normal, 0.04));
            remaining *= (1. - hit.fraction).clamp(0., 1.);
            planes.push(normal);
            for plane in &planes {
                let into = dot(self.velocity, *plane);
                if into < 0. {
                    self.velocity = sub(self.velocity, scale(*plane, into));
                }
            }
            if planes.iter().any(|n| dot(self.velocity, *n) < -0.1) {
                self.velocity = [0.; 3];
                break;
            }
            if remaining <= 0.00001 || length(self.velocity) < 0.01 {
                break;
            }
        }
        Ok(())
    }

    fn find_ground(
        &mut self,
        world: &SimWorld,
        rails: &[RailSegment],
        origin: &mut [f32; 3],
        event: &mut SkateEvent,
    ) {
        if self.grind.is_some() || (!self.grounded && self.velocity[2] > 0.) {
            return;
        }
        let hit = world.trace_world(
            add(*origin, [0., 0., 0.5]),
            add(*origin, [0., 0., -3.]),
            MINS,
            MAXS,
            1,
        );
        if hit.fraction < 1. && hit.startsolid == 0 && hit.normal[2] >= GROUND_NORMAL {
            let normal = normalize(hit.normal);
            if !self.grounded {
                let impact = -dot(self.velocity, normal);
                let mut seat = hit.endpos;
                if self.try_catch(world, rails, &mut seat, normal, impact) {
                    *origin = seat;
                    return;
                }
                self.land(normal, impact, event);
            }
            self.grounded = true;
            self.ground_normal = normal;
            self.velocity = project(self.velocity, normal);
            *origin = add(hit.endpos, scale(normal, 0.04));
        } else {
            self.grounded = false;
        }
    }

    fn begin_flight(&mut self) {
        self.flight = true;
        self.air_time = 0.;
        self.air_spin = 0.;
        self.flip_angle = 0.;
        self.flip_remaining = 0.;
    }

    /// A safe landing reports the flight's trick points plus any pending
    /// grind points as `Landed`; [`Self::advance`] banks them at the end of
    /// the step unless the step bailed.
    fn land(&mut self, normal: [f32; 3], impact: f32, event: &mut SkateEvent) {
        let bailed = self.flight && {
            let horizontal = [self.velocity[0], self.velocity[1], 0.];
            let sideways = length(horizontal) > 80.
                && dot(normalize(horizontal), forward(self.yaw)).abs() < 0.64;
            let roll = self.flip_angle.rem_euclid(360.);
            let unfinished_flip = roll.min(360. - roll) > 35.;
            impact > 520. || sideways || unfinished_flip
        };
        if bailed {
            self.bail(event);
        } else {
            let mut points = self.pending_points();
            if self.flight && self.air_time > 0.08 {
                points = points.saturating_add(self.trick_points());
            }
            self.clear_pending();
            if points > 0 {
                match event {
                    SkateEvent::Bailed => {}
                    SkateEvent::Landed { points: earlier } => {
                        *earlier = earlier.saturating_add(points);
                    }
                    _ => *event = SkateEvent::Landed { points },
                }
            }
        }
        if self.flight {
            self.end_flight();
        }
        self.grounded = true;
        self.ground_normal = normal;
    }

    fn end_flight(&mut self) {
        self.flight = false;
        self.air_time = 0.;
        self.air_spin = 0.;
        self.flip_angle = 0.;
        self.flip_remaining = 0.;
    }

    fn bail(&mut self, event: &mut SkateEvent) {
        self.velocity = [0.; 3];
        self.end_flight();
        self.grind = None;
        self.clear_pending();
        if *event != SkateEvent::Bailed {
            self.bails = self.bails.saturating_add(1);
        }
        *event = SkateEvent::Bailed;
    }
}

fn clear_origin(world: &SimWorld, origin: [f32; 3]) -> Result<[f32; 3], String> {
    for lift in [0., 0.25, 1., 2., 4.] {
        let candidate = add(origin, [0., 0., lift]);
        let hit = world.trace_world(candidate, candidate, MINS, MAXS, 1);
        if hit.startsolid == 0 && hit.allsolid == 0 {
            return Ok(candidate);
        }
    }
    Err("Skateboard rider overlaps a solid object".into())
}

fn cross_curb(world: &SimWorld, origin: [f32; 3], end: [f32; 3]) -> Option<([f32; 3], [f32; 3])> {
    let lift = [0., 0., 6.];
    let up = world.trace_world(origin, add(origin, lift), MINS, MAXS, 1);
    if up.startsolid != 0 || up.fraction < 1. {
        return None;
    }
    let across = world.trace_world(up.endpos, add(end, lift), MINS, MAXS, 1);
    if across.startsolid != 0
        || across.fraction < 1.
        || length(sub(across.endpos, up.endpos)) < 0.01
    {
        return None;
    }
    let down = world.trace_world(
        across.endpos,
        add(across.endpos, [0., 0., -9.]),
        MINS,
        MAXS,
        1,
    );
    if down.startsolid != 0
        || down.fraction >= 1.
        || down.normal[2] < GROUND_NORMAL
        || down.endpos[2] - origin[2] > 6.
    {
        return None;
    }
    let normal = normalize(down.normal);
    Some((add(down.endpos, scale(normal, 0.04)), normal))
}

/// Horizontal unit direction and length of a rail; `None` if degenerate.
fn rail_frame(rail: &RailSegment) -> Option<([f32; 3], f32)> {
    let span = sub(rail.end, rail.start);
    let len = length([span[0], span[1], 0.]);
    (len > 1. && span[2].abs() < 0.01).then(|| (scale([span[0], span[1], 0.], 1. / len), len))
}

fn rail_point(rail: &RailSegment, dir: [f32; 3], progress: f32) -> [f32; 3] {
    add(rail.start, scale(dir, progress))
}

/// How far past an endpoint the axis-aligned hull must travel to clear the
/// beam end.
fn overhang(dir: [f32; 3]) -> f32 {
    MAXS[0] * (dir[0].abs() + dir[1].abs()) + GRIND_OVERHANG_MARGIN
}

fn forward(yaw: f32) -> [f32; 3] {
    let radians = yaw.to_radians();
    [radians.cos(), radians.sin(), 0.]
}
fn add(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    std::array::from_fn(|i| a[i] + b[i])
}
fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    std::array::from_fn(|i| a[i] - b[i])
}
fn scale(a: [f32; 3], factor: f32) -> [f32; 3] {
    a.map(|v| v * factor)
}
fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a.into_iter().zip(b).map(|(a, b)| a * b).sum()
}
fn length(a: [f32; 3]) -> f32 {
    dot(a, a).sqrt()
}
fn normalize(a: [f32; 3]) -> [f32; 3] {
    let magnitude = length(a);
    if magnitude > 0.0001 {
        scale(a, 1. / magnitude)
    } else {
        [0.; 3]
    }
}
fn project(vector: [f32; 3], normal: [f32; 3]) -> [f32; 3] {
    sub(vector, scale(normal, dot(vector, normal)))
}
