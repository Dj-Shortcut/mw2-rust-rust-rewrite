//! Versioned session saves.
//!
//! Format 3 stores the scene (buildings with the canonical resource balance,
//! editor objects, inventory, vitals, resource-node depletion) together with
//! the local player: lifecycle, transform, velocity, view, health, weapon,
//! clip/reserve ammunition, kills/deaths/score and the mounted skate state,
//! plus the loot bags dead players left behind.
//! Format 2 saves (scene only) are migrated with the explicit defaults in
//! [`SavedPlayer::v2_default`].

use crate::airdrop::SavedAirdrops;
use crate::beehive::SavedHive;
use crate::cooking::SavedFires;
use crate::crates::{SavedCrates, SavedLocked};
use crate::farming::SavedGarden;
use crate::fishtrap::SavedTrap;
use crate::markers::Waypoint;
use crate::rainbarrel::SavedBarrel;
use crate::trading::SavedRequest;
use crate::{
    Blueprints, Cast, CraftQueue, GatheringWorld, Inventory, Item, LootBags, PlacedObject,
    SavedSkate, Vitals, WORLD_HALF, Weather, WorldClock,
};
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};
use std::path::Path;

pub(crate) const SAVE_VERSION: u32 = 3;
pub(crate) const MAX_SAVE_BYTES: usize = 4 * 1024 * 1024;
pub(crate) const MAX_RESOURCE_BALANCE: u32 = 1_000_000;
const MAX_COUNTER: i32 = 1_000_000;
const MAX_PLAYER_SPEED: f32 = 5_000.;
/// Same bound as the mounted total in `SkateState::from_saved`.
const MAX_SKATE_SCORE: u64 = 1_000_000_000;
pub(crate) const AUTHORED_WEAPON: u32 = 1;

/// Format-2 player defaults: a fresh spawn of the authored carbine.
const V2_HEALTH: i32 = 100;
const V2_CLIP: i32 = 30;
const V2_RESERVE: i32 = 90;

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct SavedSession {
    pub version: u32,
    pub seed: u32,
    pub buildings: String,
    pub objects: Vec<PlacedObject>,
    pub inventory: Inventory,
    /// Absent in saves from before loot bags.
    #[serde(default)]
    pub loot: LootBags,
    pub vitals: Vitals,
    #[serde(default)]
    pub pending_damage: u32,
    #[serde(default)]
    pub crafting: CraftQueue,
    #[serde(default)]
    pub blueprints: Blueprints,
    #[serde(default)]
    pub clock: WorldClock,
    #[serde(default)]
    pub weather: Weather,
    #[serde(default)]
    pub worn: Option<Item>,
    #[serde(default, deserialize_with = "present_crates")]
    pub crates: Option<SavedCrates>,
    #[serde(default)]
    pub locked_crate: SavedLocked,
    #[serde(default)]
    pub campfires: SavedFires,
    #[serde(default)]
    pub airdrops: SavedAirdrops,
    #[serde(default)]
    pub garden: SavedGarden,
    #[serde(default)]
    pub waypoint: Option<Waypoint>,
    #[serde(default)]
    pub stash: Inventory,
    #[serde(default)]
    pub fish_trap: SavedTrap,
    #[serde(default)]
    pub rain_barrel: SavedBarrel,
    #[serde(default)]
    pub beehive: SavedHive,
    #[serde(default)]
    pub trader_request: SavedRequest,
    #[serde(default)]
    pub tea_warmth: f32,
    #[serde(default)]
    pub fishing: Option<Cast>,
    #[serde(default)]
    pub casts: u32,
    pub gathering: GatheringWorld,
    pub player: SavedPlayer,
}

fn present_crates<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<SavedCrates>, D::Error> {
    SavedCrates::deserialize(deserializer).map(Some)
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct SavedPlayer {
    pub alive: bool,
    pub origin: [f32; 3],
    pub velocity: [f32; 3],
    pub view: [f32; 3],
    pub health: i32,
    pub weapon: u32,
    pub clip: i32,
    pub reserve: i32,
    pub kills: i32,
    pub deaths: i32,
    pub score: i32,
    pub skate: Option<SavedSkate>,
    /// Banked skate score, kept across dismount, death and respawn. `None`
    /// only for older format-3 saves that omit it.
    #[serde(default)]
    pub skate_score: Option<u64>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SavedSceneV2 {
    version: u32,
    seed: u32,
    buildings: String,
    objects: Vec<PlacedObject>,
    inventory: Inventory,
    vitals: Vitals,
    gathering: GatheringWorld,
}

#[derive(Deserialize)]
struct Header {
    version: u32,
}

/// A parsed save. `migrated` is set for format-2 input, whose player
/// position is a default that the session resolves to a free spawn point.
pub(crate) struct Loaded {
    pub session: SavedSession,
    pub migrated: bool,
}

impl SavedPlayer {
    pub(crate) fn v2_default() -> Self {
        Self {
            alive: true,
            origin: [0., 0., 1.],
            velocity: [0.; 3],
            view: [0.; 3],
            health: V2_HEALTH,
            weapon: AUTHORED_WEAPON,
            clip: V2_CLIP,
            reserve: V2_RESERVE,
            kills: 0,
            deaths: 0,
            score: 0,
            skate: None,
            skate_score: None,
        }
    }

    /// Checks everything that does not depend on the world the player is
    /// restored into. `clip_size`/`max_ammo` come from the weapon table.
    pub(crate) fn validate(
        &self,
        max_health: i32,
        clip_size: i32,
        max_ammo: i32,
    ) -> Result<(), String> {
        let finite = |v: &[f32; 3], limit: f32| v.iter().all(|x| x.is_finite() && x.abs() <= limit);
        if !finite(&self.origin, WORLD_HALF)
            || !finite(&self.velocity, MAX_PLAYER_SPEED)
            || !finite(&self.view, 360.)
            || !(-90. ..=90.).contains(&self.view[0])
        {
            return Err("Saved player transform is out of range".into());
        }
        let health_ok = if self.alive {
            (1..=max_health).contains(&self.health)
        } else {
            self.health == 0
        };
        if !health_ok {
            return Err("Saved player health does not match their alive state".into());
        }
        if self.weapon != AUTHORED_WEAPON
            || !(0..=clip_size).contains(&self.clip)
            || !(0..=max_ammo).contains(&self.reserve)
        {
            return Err("Saved weapon or ammunition is out of range".into());
        }
        if !(0..=MAX_COUNTER).contains(&self.kills)
            || !(0..=MAX_COUNTER).contains(&self.deaths)
            || !(0..=MAX_COUNTER).contains(&self.score)
            || (!self.alive && self.deaths == 0)
        {
            return Err("Saved kills, deaths or score are out of range".into());
        }
        if self.skate_score.is_some_and(|s| s > MAX_SKATE_SCORE) {
            return Err("Saved skate score is out of range".into());
        }
        if self.skate.is_some() && !self.alive {
            return Err("A dead player cannot be saved on a skateboard".into());
        }
        Ok(())
    }
}

pub(crate) fn read(path: &Path) -> Result<Loaded, String> {
    let mut bytes = Vec::new();
    std::fs::File::open(path)
        .map_err(|e| format!("Cannot open save: {e}"))?
        .take((MAX_SAVE_BYTES + 1) as u64)
        .read_to_end(&mut bytes)
        .map_err(|e| format!("Cannot read save: {e}"))?;
    parse(&bytes)
}

pub(crate) fn parse(bytes: &[u8]) -> Result<Loaded, String> {
    if bytes.len() > MAX_SAVE_BYTES {
        return Err("Save file is too large".into());
    }
    let header: Header =
        serde_json::from_slice(bytes).map_err(|e| format!("Save file is malformed: {e}"))?;
    match header.version {
        SAVE_VERSION => Ok(Loaded {
            session: serde_json::from_slice(bytes)
                .map_err(|e| format!("Save file is invalid: {e}"))?,
            migrated: false,
        }),
        2 => {
            let old: SavedSceneV2 = serde_json::from_slice(bytes)
                .map_err(|e| format!("Version 2 save is invalid: {e}"))?;
            debug_assert_eq!(old.version, 2);
            Ok(Loaded {
                session: SavedSession {
                    version: SAVE_VERSION,
                    seed: old.seed,
                    buildings: old.buildings,
                    objects: old.objects,
                    inventory: old.inventory,
                    loot: LootBags::default(),
                    vitals: old.vitals,
                    pending_damage: 0,
                    crafting: CraftQueue::default(),
                    blueprints: Blueprints::default(),
                    clock: WorldClock::default(),
                    weather: Weather::default(),
                    worn: None,
                    crates: None,
                    locked_crate: SavedLocked::default(),
                    campfires: SavedFires::default(),
                    airdrops: SavedAirdrops::default(),
                    garden: SavedGarden::default(),
                    waypoint: None,
                    stash: Inventory::default(),
                    fish_trap: SavedTrap::default(),
                    rain_barrel: SavedBarrel::default(),
                    beehive: SavedHive::default(),
                    trader_request: SavedRequest::default(),
                    tea_warmth: 0.,
                    fishing: None,
                    casts: 0,
                    gathering: old.gathering,
                    player: SavedPlayer::v2_default(),
                },
                migrated: true,
            })
        }
        other => Err(format!("Unsupported save version {other}")),
    }
}

/// Writes to a sibling `.pending` file, flushes it, then renames it over the
/// destination so a crash leaves either the old or the new save.
pub(crate) fn write_atomic(path: &Path, session: &SavedSession) -> Result<(), String> {
    let data = serde_json::to_vec(session).map_err(|e| e.to_string())?;
    if data.len() > MAX_SAVE_BYTES {
        return Err("Save file would be too large".into());
    }
    if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let pending = path.with_extension("pending");
    let mut file = std::fs::File::create(&pending).map_err(|e| e.to_string())?;
    file.write_all(&data).map_err(|e| e.to_string())?;
    file.sync_all().map_err(|e| e.to_string())?;
    drop(file);
    std::fs::rename(pending, path).map_err(|e| e.to_string())
}
