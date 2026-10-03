use crate::WORLD_HALF;
use sim::SimWorld;

const MINS: [f32; 3] = [-15., -15., 0.];
const MAXS: [f32; 3] = [15., 15., 70.];
const GRAVITY: f32 = 600.;
const MAX_SPEED: f32 = 650.;
const OLLIE_SPEED: f32 = 300.;
const GROUND_NORMAL: f32 = 0.45;
const MAX_SAVED_SCORE: u64 = 1_000_000_000;
const MAX_SAVED_BAILS: u32 = 1_000_000;

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
        })
    }

    pub fn step(
        &mut self,
        world: &SimWorld,
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
        let output = next.advance(world, dt_seconds, input, origin)?;
        *self = next;
        Ok(output)
    }

    fn advance(
        &mut self,
        world: &SimWorld,
        dt: f32,
        input: SkateInput,
        mut origin: [f32; 3],
    ) -> Result<SkateStep, String> {
        origin = clear_origin(world, origin)?;
        let mut event = SkateEvent::None;
        self.find_ground(world, &mut origin, &mut event);
        if input.ollie && self.grounded {
            self.velocity[2] += OLLIE_SPEED;
            self.grounded = false;
            self.begin_flight();
            event = SkateEvent::Ollie;
        }
        if input.flip && !self.grounded && self.flip_remaining <= 0. {
            self.flip_remaining = 360.;
        }
        let steps = (dt * 120.).ceil() as usize;
        let slice = dt / steps as f32;
        for _ in 0..steps {
            if self.grounded {
                let speed = length(self.velocity);
                let turn = input.steer * (35. + 70. * (speed / MAX_SPEED).min(1.));
                self.yaw = (self.yaw + turn * slice).rem_euclid(360.);
                let heading = forward(self.yaw);
                let tangent = normalize(project(heading, self.ground_normal));
                let along = dot(self.velocity, tangent);
                let grip = (slice * 8.).min(1.);
                let lateral = sub(self.velocity, scale(tangent, along));
                self.velocity = sub(self.velocity, scale(lateral, grip));
                self.velocity = add(self.velocity, scale(tangent, input.push * 245. * slice));
                self.velocity = add(
                    self.velocity,
                    scale(project([0., 0., -GRAVITY], self.ground_normal), slice),
                );
                let speed = length(self.velocity);
                if speed > 0. {
                    self.velocity = scale(
                        self.velocity,
                        (speed - (18. + input.brake * 650.) * slice).max(0.) / speed,
                    );
                }
            } else {
                if !self.flight {
                    self.begin_flight();
                }
                self.air_time += slice;
                let spin = input.spin * 540. * slice;
                self.air_spin += spin;
                self.yaw = (self.yaw + spin).rem_euclid(360.);
                let flip = self.flip_remaining.min(900. * slice);
                self.flip_remaining -= flip;
                self.flip_angle += flip;
                self.velocity[2] -= GRAVITY * slice;
            }
            let speed = length(self.velocity);
            if speed > MAX_SPEED {
                self.velocity = scale(self.velocity, MAX_SPEED / speed);
            }
            self.sweep(world, slice, &mut origin, &mut event)?;
            self.find_ground(world, &mut origin, &mut event);
            let limit = WORLD_HALF - MAXS[0] - 1.;
            if origin[0].abs() > limit || origin[1].abs() > limit {
                origin[0] = origin[0].clamp(-limit, limit);
                origin[1] = origin[1].clamp(-limit, limit);
                self.bail(&mut event);
            }
        }
        Ok(SkateStep {
            origin,
            velocity: self.velocity,
            yaw: self.yaw,
            grounded: self.grounded,
            board_roll: self.flip_angle.rem_euclid(360.),
            event,
        })
    }

    fn sweep(
        &mut self,
        world: &SimWorld,
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

    fn find_ground(&mut self, world: &SimWorld, origin: &mut [f32; 3], event: &mut SkateEvent) {
        if !self.grounded && self.velocity[2] > 0. {
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
                self.land(normal, -dot(self.velocity, normal), event);
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

    fn land(&mut self, normal: [f32; 3], impact: f32, event: &mut SkateEvent) {
        if self.flight {
            let horizontal = [self.velocity[0], self.velocity[1], 0.];
            let sideways = length(horizontal) > 80.
                && dot(normalize(horizontal), forward(self.yaw)).abs() < 0.64;
            let roll = self.flip_angle.rem_euclid(360.);
            let unfinished_flip = roll.min(360. - roll) > 35.;
            if impact > 520. || sideways || unfinished_flip {
                self.bail(event);
            } else if self.air_time > 0.08 {
                let flips = (self.flip_angle / 360.).floor() as u32;
                let turns = ((self.air_spin.abs() + 15.) / 180.).floor() as u32;
                let points = 10 + turns * 50 + flips * 100;
                self.total_score = self.total_score.saturating_add(u64::from(points));
                if *event != SkateEvent::Bailed {
                    *event = SkateEvent::Landed { points };
                }
            }
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
