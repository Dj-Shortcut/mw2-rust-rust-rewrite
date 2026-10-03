use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub const CELL: f32 = 120.0;
pub const WALL_HEIGHT: f32 = 120.0;
pub const MAX_PIECES: usize = 4096;
pub const MAX_SAVE_BYTES: usize = 2_000_000;
pub const MAX_INVENTORIES: usize = 256;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Kind {
    Foundation,
    Floor,
    Wall,
    Doorway,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Grade {
    Wood,
    Stone,
    Metal,
}

impl Grade {
    pub fn health(self) -> u32 {
        match self {
            Self::Wood => 250,
            Self::Stone => 500,
            Self::Metal => 1000,
        }
    }
    pub fn cost(self, kind: Kind) -> Resources {
        let n = match kind {
            Kind::Foundation => 200,
            Kind::Floor => 100,
            Kind::Wall => 100,
            Kind::Doorway => 150,
        };
        match self {
            Self::Wood => Resources {
                wood: n,
                ..Resources::default()
            },
            Self::Stone => Resources {
                stone: n * 2,
                ..Resources::default()
            },
            Self::Metal => Resources {
                metal: n,
                ..Resources::default()
            },
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Resources {
    pub wood: u32,
    pub stone: u32,
    pub metal: u32,
}

impl Resources {
    pub fn covers(self, cost: Self) -> bool {
        self.wood >= cost.wood && self.stone >= cost.stone && self.metal >= cost.metal
    }
    fn spend(&mut self, cost: Self) {
        self.wood -= cost.wood;
        self.stone -= cost.stone;
        self.metal -= cost.metal;
    }
    pub fn add(&mut self, other: Self) {
        self.wood = self.wood.saturating_add(other.wood);
        self.stone = self.stone.saturating_add(other.stone);
        self.metal = self.metal.saturating_add(other.metal);
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Socket {
    pub x: i32,
    pub y: i32,
    pub level: i32,
    pub axis: u8,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Piece {
    pub id: u32,
    pub owner: u32,
    pub kind: Kind,
    pub grade: Grade,
    pub socket: Socket,
    pub health: u32,
    pub open: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BuildingWorld {
    schema: u32,
    pub anchor: [f32; 3],
    next_id: u32,
    pieces: BTreeMap<u32, Piece>,
    inventories: BTreeMap<u32, Resources>,
}

impl Default for BuildingWorld {
    fn default() -> Self {
        Self {
            schema: 1,
            anchor: [0.0; 3],
            next_id: 1,
            pieces: BTreeMap::new(),
            inventories: BTreeMap::new(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BuildError {
    InvalidSocket,
    Occupied,
    Unsupported,
    InsufficientResources,
    NotOwner,
    Missing,
    Limit,
    InvalidGrade,
    InvalidSave,
}

impl std::fmt::Display for BuildError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::InvalidSocket => "Invalid building location or interaction",
            Self::Occupied => "This building location is occupied",
            Self::Unsupported => "Building needs ground or structural support",
            Self::InsufficientResources => "Not enough building resources",
            Self::NotOwner => "You do not own this building piece",
            Self::Missing => "Building piece no longer exists",
            Self::Limit => "Building data limit reached",
            Self::InvalidGrade => "Choose a higher material grade",
            Self::InvalidSave => "Invalid building save",
        })
    }
}
impl std::error::Error for BuildError {}

fn deck(kind: Kind) -> bool {
    matches!(kind, Kind::Foundation | Kind::Floor)
}

impl BuildingWorld {
    pub fn pieces(&self) -> impl Iterator<Item = &Piece> {
        self.pieces.values()
    }
    pub fn piece(&self, id: u32) -> Option<&Piece> {
        self.pieces.get(&id)
    }
    pub fn inventory(&self, owner: u32) -> Resources {
        self.inventories.get(&owner).copied().unwrap_or_default()
    }
    pub fn grant(&mut self, owner: u32, resources: Resources) -> Result<(), BuildError> {
        if !self.inventories.contains_key(&owner) && self.inventories.len() >= MAX_INVENTORIES {
            return Err(BuildError::Limit);
        }
        self.inventories.entry(owner).or_default().add(resources);
        Ok(())
    }
    pub fn consume(&mut self, owner: u32, cost: Resources) -> Result<(), BuildError> {
        let available = self.inventory(owner);
        if !available.covers(cost) {
            return Err(BuildError::InsufficientResources);
        }
        if let Some(inventory) = self.inventories.get_mut(&owner) {
            inventory.spend(cost);
        }
        Ok(())
    }
    pub fn set_anchor(&mut self, anchor: [f32; 3]) -> Result<(), BuildError> {
        if !self.pieces.is_empty()
            || !anchor
                .iter()
                .all(|v| v.is_finite() && v.abs() < 1_000_000.0)
        {
            return Err(BuildError::InvalidSocket);
        }
        self.anchor = anchor;
        Ok(())
    }
    fn valid_socket(kind: Kind, s: Socket) -> bool {
        s.x.abs_diff(0) <= 4096
            && s.y.abs_diff(0) <= 4096
            && (0..=32).contains(&s.level)
            && if deck(kind) { s.axis == 0 } else { s.axis <= 1 }
            && (kind != Kind::Foundation || s.level == 0)
            && (kind != Kind::Floor || s.level > 0)
    }
    fn occupied(&self, kind: Kind, socket: Socket) -> bool {
        self.pieces
            .values()
            .any(|p| p.socket == socket && deck(p.kind) == deck(kind))
    }
    fn deck_at(&self, x: i32, y: i32, level: i32, keep: Option<&BTreeSet<u32>>) -> bool {
        self.pieces.values().any(|p| {
            keep.map_or_else(
                || self.pieces.contains_key(&p.id),
                |keep| keep.contains(&p.id),
            ) && deck(p.kind)
                && p.socket
                    == Socket {
                        x,
                        y,
                        level,
                        axis: 0,
                    }
        })
    }
    fn supported(&self, p: &Piece, keep: Option<&BTreeSet<u32>>) -> bool {
        let s = p.socket;
        match p.kind {
            Kind::Foundation => true,
            Kind::Wall | Kind::Doorway => {
                self.deck_at(s.x, s.y, s.level, keep)
                    || self.deck_at(
                        s.x - i32::from(s.axis == 1),
                        s.y - i32::from(s.axis == 0),
                        s.level,
                        keep,
                    )
            }
            Kind::Floor => self.pieces.values().any(|wall| {
                keep.map_or_else(
                    || self.pieces.contains_key(&wall.id),
                    |keep| keep.contains(&wall.id),
                ) && !deck(wall.kind)
                    && wall.socket.level == s.level - 1
                    && ((wall.socket.x == s.x && wall.socket.y == s.y)
                        || (wall.socket.axis == 0
                            && wall.socket.x == s.x
                            && wall.socket.y == s.y + 1)
                        || (wall.socket.axis == 1
                            && wall.socket.x == s.x + 1
                            && wall.socket.y == s.y))
            }),
        }
    }
    pub fn can_place(
        &self,
        owner: u32,
        kind: Kind,
        socket: Socket,
        grounded: bool,
    ) -> Result<(), BuildError> {
        if !Self::valid_socket(kind, socket) {
            return Err(BuildError::InvalidSocket);
        }
        if self.pieces.len() >= MAX_PIECES || self.next_id == u32::MAX {
            return Err(BuildError::Limit);
        }
        if self.occupied(kind, socket) {
            return Err(BuildError::Occupied);
        }
        let candidate = Piece {
            id: self.next_id,
            owner,
            kind,
            grade: Grade::Wood,
            socket,
            health: 250,
            open: false,
        };
        if (kind == Kind::Foundation && !grounded) || !self.supported(&candidate, None) {
            return Err(BuildError::Unsupported);
        }
        if !self.inventory(owner).covers(Grade::Wood.cost(kind)) {
            return Err(BuildError::InsufficientResources);
        }
        Ok(())
    }
    pub fn place(
        &mut self,
        owner: u32,
        kind: Kind,
        socket: Socket,
        grounded: bool,
    ) -> Result<u32, BuildError> {
        self.can_place(owner, kind, socket, grounded)?;
        self.inventories
            .get_mut(&owner)
            .ok_or(BuildError::InsufficientResources)?
            .spend(Grade::Wood.cost(kind));
        let id = self.next_id;
        self.next_id += 1;
        self.pieces.insert(
            id,
            Piece {
                id,
                owner,
                kind,
                grade: Grade::Wood,
                socket,
                health: 250,
                open: false,
            },
        );
        Ok(id)
    }
    fn owned(&self, owner: u32, id: u32) -> Result<&Piece, BuildError> {
        let p = self.pieces.get(&id).ok_or(BuildError::Missing)?;
        if p.owner != owner {
            return Err(BuildError::NotOwner);
        }
        Ok(p)
    }
    pub fn upgrade(&mut self, owner: u32, id: u32, grade: Grade) -> Result<(), BuildError> {
        let p = self.owned(owner, id)?;
        let rank = |g| match g {
            Grade::Wood => 0,
            Grade::Stone => 1,
            Grade::Metal => 2,
        };
        if rank(grade) <= rank(p.grade) {
            return Err(BuildError::InvalidGrade);
        }
        let cost = grade.cost(p.kind);
        if !self.inventory(owner).covers(cost) {
            return Err(BuildError::InsufficientResources);
        }
        self.inventories
            .get_mut(&owner)
            .ok_or(BuildError::InsufficientResources)?
            .spend(cost);
        let p = self.pieces.get_mut(&id).ok_or(BuildError::Missing)?;
        p.grade = grade;
        p.health = grade.health();
        Ok(())
    }
    pub fn toggle_door(&mut self, owner: u32, id: u32) -> Result<(), BuildError> {
        if self.owned(owner, id)?.kind != Kind::Doorway {
            return Err(BuildError::InvalidSocket);
        }
        let p = self.pieces.get_mut(&id).ok_or(BuildError::Missing)?;
        p.open = !p.open;
        Ok(())
    }
    pub fn demolish(&mut self, owner: u32, id: u32) -> Result<Vec<u32>, BuildError> {
        self.owned(owner, id)?;
        self.pieces.remove(&id);
        let mut removed = vec![id];
        removed.extend(self.collapse());
        Ok(removed)
    }
    pub fn damage(&mut self, id: u32, damage: u32) -> Result<Vec<u32>, BuildError> {
        let p = self.pieces.get_mut(&id).ok_or(BuildError::Missing)?;
        p.health = p.health.saturating_sub(damage);
        if p.health > 0 {
            return Ok(Vec::new());
        }
        self.pieces.remove(&id);
        let mut removed = vec![id];
        removed.extend(self.collapse());
        Ok(removed)
    }
    fn stable_ids(&self) -> BTreeSet<u32> {
        let mut keep: BTreeSet<_> = self
            .pieces
            .values()
            .filter(|p| p.kind == Kind::Foundation)
            .map(|p| p.id)
            .collect();
        loop {
            let more: Vec<_> = self
                .pieces
                .values()
                .filter(|p| !keep.contains(&p.id) && self.supported(p, Some(&keep)))
                .map(|p| p.id)
                .collect();
            if more.is_empty() {
                break;
            }
            keep.extend(more);
        }
        keep
    }
    fn collapse(&mut self) -> Vec<u32> {
        let stable = self.stable_ids();
        let removed = self
            .pieces
            .keys()
            .filter(|id| !stable.contains(id))
            .copied()
            .collect();
        self.pieces.retain(|id, _| stable.contains(id));
        removed
    }
    pub fn bounds(&self, p: &Piece) -> Vec<([f32; 3], [f32; 3])> {
        let s = p.socket;
        let base = [
            self.anchor[0] + s.x as f32 * CELL,
            self.anchor[1] + s.y as f32 * CELL,
            self.anchor[2] + s.level as f32 * WALL_HEIGHT,
        ];
        let ranges: Vec<([f32; 3], [f32; 3])> = match p.kind {
            Kind::Foundation => vec![([0., 0., -12.], [CELL, CELL, 0.])],
            Kind::Floor => vec![([0., 0., -8.], [CELL, CELL, 0.])],
            Kind::Wall => vec![([0., -4., 0.], [CELL, 4., WALL_HEIGHT])],
            Kind::Doorway => {
                let mut boxes = vec![
                    ([0., -4., 0.], [25., 4., WALL_HEIGHT]),
                    ([95., -4., 0.], [CELL, 4., WALL_HEIGHT]),
                    ([25., -4., 92.], [95., 4., WALL_HEIGHT]),
                ];
                if !p.open {
                    boxes.push(([25., -2., 0.], [95., 2., 92.]));
                }
                boxes
            }
        };
        ranges
            .into_iter()
            .map(|(mut lo, mut hi)| {
                if !deck(p.kind) && s.axis == 1 {
                    lo.swap(0, 1);
                    hi.swap(0, 1);
                }
                (
                    std::array::from_fn(|k| base[k] + lo[k]),
                    std::array::from_fn(|k| base[k] + hi[k]),
                )
            })
            .collect()
    }
    pub fn trace(
        &self,
        start: [f32; 3],
        end: [f32; 3],
        mins: [f32; 3],
        maxs: [f32; 3],
        mask: u32,
    ) -> trace_iw4::Trace {
        self.trace_hit(start, end, mins, maxs, mask).0
    }
    pub fn overlaps_piece(
        &self,
        piece: &Piece,
        origin: [f32; 3],
        mins: [f32; 3],
        maxs: [f32; 3],
    ) -> bool {
        let flags = [0; 6];
        self.bounds(piece).into_iter().any(|(lo, hi)| {
            let planes = [
                [1., 0., 0., hi[0]],
                [-1., 0., 0., -lo[0]],
                [0., 1., 0., hi[1]],
                [0., -1., 0., -lo[1]],
                [0., 0., 1., hi[2]],
                [0., 0., -1., -lo[2]],
            ];
            trace_iw4::trace_box(
                std::iter::once(trace_iw4::BrushRef {
                    planes: &planes,
                    contents: 1,
                    plane_surface_flags: &flags,
                    glass_encoded: 0,
                }),
                origin,
                origin,
                mins,
                maxs,
                1,
            )
            .startsolid
                != 0
        })
    }
    pub fn trace_hit(
        &self,
        start: [f32; 3],
        end: [f32; 3],
        mins: [f32; 3],
        maxs: [f32; 3],
        mask: u32,
    ) -> (trace_iw4::Trace, Option<u32>) {
        let mut best = trace_iw4::Trace {
            fraction: 1.,
            endpos: end,
            ..Default::default()
        };
        let mut closest = None;
        let mut solid = None;
        let flags = [0u32; 6];
        for piece in self.pieces.values() {
            let planes: Vec<[[f32; 4]; 6]> = self
                .bounds(piece)
                .into_iter()
                .map(|(lo, hi)| {
                    [
                        [1., 0., 0., hi[0]],
                        [-1., 0., 0., -lo[0]],
                        [0., 1., 0., hi[1]],
                        [0., -1., 0., -lo[1]],
                        [0., 0., 1., hi[2]],
                        [0., 0., -1., -lo[2]],
                    ]
                })
                .collect();
            let hit = trace_iw4::trace_box(
                planes.iter().map(|p| trace_iw4::BrushRef {
                    planes: p,
                    contents: 1,
                    plane_surface_flags: &flags,
                    glass_encoded: 0,
                }),
                start,
                end,
                mins,
                maxs,
                mask,
            );
            if hit.startsolid != 0 && (solid.is_none() || hit.allsolid != 0) {
                solid = Some(piece.id);
            }
            let startsolid = best.startsolid | hit.startsolid;
            let allsolid = best.allsolid | hit.allsolid;
            if hit.fraction < best.fraction || (hit.startsolid != 0 && best.fraction == 1.) {
                best = hit;
                closest = Some(piece.id);
            }
            best.startsolid = startsolid;
            best.allsolid = allsolid;
        }
        (best, solid.or(closest))
    }
    pub fn validate(&self) -> Result<(), BuildError> {
        if self.schema != 1
            || self.next_id == 0
            || self.pieces.len() > MAX_PIECES
            || self.inventories.len() > MAX_INVENTORIES
            || !self
                .anchor
                .iter()
                .all(|v| v.is_finite() && v.abs() < 1_000_000.)
        {
            return Err(BuildError::InvalidSave);
        }
        let mut occupied = BTreeSet::new();
        for (&id, p) in &self.pieces {
            if id == 0
                || id != p.id
                || id >= self.next_id
                || !Self::valid_socket(p.kind, p.socket)
                || p.health == 0
                || p.health > p.grade.health()
                || (p.open && p.kind != Kind::Doorway)
                || !occupied.insert((p.socket, deck(p.kind)))
            {
                return Err(BuildError::InvalidSave);
            }
        }
        if self.stable_ids().len() != self.pieces.len() {
            return Err(BuildError::InvalidSave);
        }
        Ok(())
    }
    pub fn to_json(&self) -> Result<Vec<u8>, BuildError> {
        self.validate()?;
        let data = serde_json::to_vec(self).map_err(|_| BuildError::InvalidSave)?;
        if data.len() > MAX_SAVE_BYTES {
            return Err(BuildError::Limit);
        }
        Ok(data)
    }
    pub fn from_json(data: &[u8]) -> Result<Self, BuildError> {
        if data.len() > MAX_SAVE_BYTES {
            return Err(BuildError::Limit);
        }
        let world: Self = serde_json::from_slice(data).map_err(|_| BuildError::InvalidSave)?;
        world.validate()?;
        Ok(world)
    }
}
