use playerstate_iw4::UserCmd;
use rust_building::{BuildingWorld, CELL, Grade, Kind, Piece, Resources, Socket, WALL_HEIGHT};
use sim::{ClientId, SimBrush, SimContentBuilder, SimWorld, Tick, TickInput};
use std::path::Path;
mod crafting;
mod editor;
mod gathering;
mod inventory;
mod loot;
mod persistence;
mod rules;
mod skate;
mod terrain;
pub use crafting::{CraftJob, CraftQueue, MAX_CRAFT_JOBS};
pub use editor::{EditorState, Geometry, PlacedObject, PropKind, RailSegment};
pub use gathering::{
    GatheringWorld, Harvest, REGROW_RETRY_SECONDS, REGROW_SECONDS, ResourceKind, ResourceNode,
};
pub use inventory::{
    BLEED_THRESHOLD, Inventory, Item, MAX_BLEED, MAX_REGEN, MAX_TOOL_WEAR, Recipe, Stack, Vitals,
};
pub use loot::{LOOT_REACH, LootBag, LootBags, MAX_LOOT_BAGS};
pub use skate::{SavedGrind, SavedSkate, SkateEvent, SkateInput, SkateState, SkateStep};
pub use terrain::Terrain;

pub const UNITS_TO_METERS: f32 = 0.0254;
pub const WORLD_HALF: f32 = 4000.;
pub const LOCAL: ClientId = ClientId(0);
const SPAWN_POINTS: [[f32; 3]; 5] = [
    [0., 0., 1.],
    [-350., 0., 1.],
    [0., 350., 1.],
    [350., 350., 1.],
    [-350., -350., 1.],
];
const PLAYER_MINS: [f32; 3] = [-15., -15., 0.];
const PLAYER_MAXS: [f32; 3] = [15., 15., 70.];

#[derive(Clone, Debug)]
pub struct PropPlacementPreview {
    pub object: Option<PlacedObject>,
    pub error: Option<String>,
}

impl PropPlacementPreview {
    pub fn valid(&self) -> bool {
        self.object.is_some() && self.error.is_none()
    }
}

#[derive(Clone, Debug)]
pub struct BuildingPlacementPreview {
    pub kind: Kind,
    pub socket: Option<Socket>,
    pub bounds: Vec<([f32; 3], [f32; 3])>,
    pub cost: Resources,
    pub resources: Resources,
    pub error: Option<String>,
}

impl BuildingPlacementPreview {
    pub fn valid(&self) -> bool {
        self.socket.is_some() && self.error.is_none()
    }
}

pub struct Session {
    pub world: SimWorld,
    pub tick: u32,
    pub message: String,
    pub terrain: Terrain,
    pub editor: EditorState,
    pub inventory: Inventory,
    pub gathering: GatheringWorld,
    pub vitals: Vitals,
    pub skate: Option<SkateState>,
    pub skate_input: SkateInput,
    pub skate_roll: f32,
    pub last_skate_event: SkateEvent,
    /// Skate score banked while not mounted; carried into the next mount.
    skate_score: u64,
    loot: LootBags,
    queued_damage: u32,
    crafting: CraftQueue,
}

impl Session {
    pub fn ammo(&self) -> (i32, i32) {
        let Some(player) = self.world.player(LOCAL) else {
            return (0, 0);
        };
        let Some(facts) = self.world.weapon_combat_facts(player.weapon) else {
            return (0, 0);
        };
        (
            weapon_iw4::get_clip_for_hand(&player.ammoclip, facts.clip_index, 0),
            weapon_iw4::get_ammo_not_in_clip(&player.ammo, facts.ammo_index),
        )
    }

    pub fn preview_prop_from_view(&self, kind: PropKind, yaw: f32) -> PropPlacementPreview {
        self.prop_placement_from_view(kind, yaw).0
    }

    fn prop_placement_from_view(
        &self,
        kind: PropKind,
        yaw: f32,
    ) -> (PropPlacementPreview, Option<PlacedObject>) {
        let mut preview = PropPlacementPreview {
            object: None,
            error: None,
        };
        let candidate = (|| {
            let (start, end) = self.view_ray()?;
            let hit = self.world.trace_world(start, end, [0.; 3], [0.; 3], 1);
            if hit.startsolid != 0 || hit.fraction >= 1. {
                return Err("Aim at a flat surface within reach".into());
            }
            preview.object = Some(PlacedObject {
                id: 0,
                kind,
                position: hit.endpos,
                yaw: if yaw.is_finite() {
                    yaw.rem_euclid(360.)
                } else {
                    0.
                },
            });
            if hit.normal[2] < 0.7 {
                return Err("Aim at a flat surface within reach".into());
            }
            let object = self.editor.placement_candidate(kind, hit.endpos, yaw)?;
            preview.object = Some(object.clone());
            if props_overlap_players(&self.world, &self.editor)
                || brushes_overlap_players(&self.world, &object.brushes())
            {
                return Err("Editor object overlaps a player".into());
            }
            Ok(object)
        })();
        match candidate {
            Ok(candidate) => (preview, Some(candidate)),
            Err(error) => {
                preview.error = Some(error);
                (preview, None)
            }
        }
    }

    pub fn place_prop_from_view(&mut self, kind: PropKind, yaw: f32) -> Result<u32, String> {
        let (preview, candidate) = self.prop_placement_from_view(kind, yaw);
        let object = candidate.ok_or_else(|| {
            preview
                .error
                .unwrap_or_else(|| "No editor placement target".into())
        })?;
        let mut editor = self.editor.clone();
        let id = editor.place(object.kind, object.position, object.yaw)?;
        self.install_editor(editor)?;
        Ok(id)
    }

    pub fn remove_prop_from_view(&mut self) -> Result<(), String> {
        let id = self.aimed_prop_id()?;
        let mut editor = self.editor.clone();
        editor.remove(id)?;
        self.install_editor(editor)
    }

    /// The editor prop under the crosshair, independent of any background:
    /// removal and rotation need no destination behind it.
    fn aimed_prop_id(&self) -> Result<u32, String> {
        let (start, end) = self.view_ray()?;
        let world_hit = self.world.trace_world(start, end, [0.; 3], [0.; 3], 1);
        self.editor
            .objects()
            .filter_map(|o| {
                o.brushes()
                    .iter()
                    .filter_map(|b| intersect(b, start, end))
                    .min_by(f32::total_cmp)
                    .map(|distance| (o.id, distance))
            })
            .filter(|(_, t)| *t <= world_hit.fraction + 0.002)
            .min_by(|a, b| a.1.total_cmp(&b.1))
            .map(|(id, _)| id)
            .ok_or("Aim at an editor object within reach".into())
    }

    /// The point the same ray reaches without the aimed prop: the move
    /// destination. Editor brushes are part of the installed world content,
    /// so it is traced against terrain, resource nodes, the other props
    /// and building pieces instead. Only upward faces count, like normal
    /// prop placement, so a prop never lands centered on a wall.
    fn prop_destination(&self, ignore: u32) -> Result<[f32; 3], String> {
        let (start, end) = self.view_ray()?;
        let mut best: Option<(f32, [f32; 3])> = None;
        let mut consider = |t: f32, normal: [f32; 3]| {
            if t < best.map_or(1., |(known, _)| known) {
                best = Some((t, normal));
            }
        };
        let mut solids = self.terrain.brushes();
        solids.extend(self.gathering.brushes());
        solids.extend(
            self.editor
                .objects()
                .filter(|o| o.id != ignore)
                .flat_map(PlacedObject::brushes),
        );
        for brush in &solids {
            if let Some((t, normal)) = intersect_entry(brush, start, end) {
                consider(t, normal);
            }
        }
        let (hit, _) = self
            .world
            .buildings()
            .trace_hit(start, end, [0.; 3], [0.; 3], 1);
        if hit.startsolid == 0 && hit.fraction < 1. {
            consider(hit.fraction, hit.normal);
        }
        let Some((t, normal)) = best else {
            return Err("Aim at a flat surface within reach".into());
        };
        if normal[2] < 0.7 {
            return Err("Aim at a flat surface within reach".into());
        }
        Ok(std::array::from_fn(|k| start[k] + (end[k] - start[k]) * t))
    }

    /// Relocating the rail or ledge being ground would teleport the rider
    /// to the moved segment, so it is rejected until the grind ends.
    fn require_grind_clear(&self, id: u32) -> Result<(), String> {
        if self
            .skate
            .as_ref()
            .is_some_and(|s| s.grind_rail() == Some(id))
        {
            return Err("Leave the grind before moving this".into());
        }
        Ok(())
    }

    /// Moves the aimed prop to the surface point under the crosshair,
    /// keeping its yaw. Player overlap is rejected without moving.
    pub fn move_prop_to_view(&mut self) -> Result<u32, String> {
        let id = self.aimed_prop_id()?;
        self.require_grind_clear(id)?;
        let position = self.prop_destination(id)?;
        let yaw = self
            .editor
            .objects()
            .find(|o| o.id == id)
            .ok_or("Object no longer exists")?
            .yaw;
        let mut editor = self.editor.clone();
        editor.relocate(id, position, yaw)?;
        self.install_editor(editor)?;
        Ok(id)
    }

    /// Rotates the aimed prop in place over the given degrees.
    pub fn rotate_prop_from_view(&mut self, step_degrees: f32) -> Result<u32, String> {
        let id = self.aimed_prop_id()?;
        self.require_grind_clear(id)?;
        let object = self
            .editor
            .objects()
            .find(|o| o.id == id)
            .ok_or("Object no longer exists")?;
        let mut editor = self.editor.clone();
        editor.relocate(id, object.position, object.yaw + step_degrees)?;
        self.install_editor(editor)?;
        Ok(id)
    }

    pub fn undo_props(&mut self) -> Result<(), String> {
        let mut editor = self.editor.clone();
        editor.undo()?;
        self.install_editor(editor)
    }
    pub fn redo_props(&mut self) -> Result<(), String> {
        let mut editor = self.editor.clone();
        editor.redo()?;
        self.install_editor(editor)
    }
    fn install_editor(&mut self, editor: EditorState) -> Result<(), String> {
        if props_overlap_players(&self.world, &editor) {
            return Err("Editor object overlaps a player".into());
        }
        self.world
            .install_content(authored_content(&self.terrain, &editor, &self.gathering));
        self.editor = editor;
        Ok(())
    }
    pub fn new() -> Result<Self, String> {
        let mut world = SimWorld::new();
        let terrain = Terrain::new(terrain::TERRAIN_SEED);
        let editor = EditorState::default();
        let gathering = GatheringWorld::new(&terrain)?;
        world.install_content(authored_content(&terrain, &editor, &gathering));
        world
            .bootstrap(sim::MatchBootstrap {
                seed: 1,
                ..Default::default()
            })
            .map_err(str::to_owned)?;
        rules::install(&mut world)?;
        world.spawn_authored_player(LOCAL, [0., 0., 1.], [0.; 3], 1)?;
        world.spawn_authored_player(ClientId(1), [550., 0., 1.], [0., 180., 0.], 0)?;
        world
            .buildings_mut()
            .grant(
                LOCAL.0,
                Resources {
                    wood: 600,
                    stone: 100,
                    metal: 60,
                },
            )
            .map_err(|e| e.to_string())?;
        let mut inventory = Inventory::default();
        inventory.add(Item::Bandage, 1)?;
        inventory.add(Item::Food, 2)?;
        inventory.add(Item::Water, 2)?;
        let mut session = Self {
            world,
            terrain,
            editor,
            tick: 0,
            message: "Development world: authored content, survival systems in progress".into(),
            inventory,
            gathering,
            vitals: Vitals::default(),
            skate: None,
            skate_input: SkateInput::default(),
            skate_roll: 0.,
            last_skate_event: SkateEvent::None,
            skate_score: 0,
            loot: LootBags::default(),
            queued_damage: 0,
            crafting: CraftQueue::default(),
        };
        session.advance(UserCmd {
            weapon: 1,
            ..Default::default()
        })?;
        Ok(session)
    }

    pub fn advance(&mut self, cmd: UserCmd) -> Result<(), String> {
        self.last_skate_event = SkateEvent::None;
        self.tick = self.tick.checked_add(1).ok_or("Session clock exhausted")?;
        let health = self.world.player(LOCAL).map_or(0, |p| p.health);
        authority_step(&mut self.world, self.tick, cmd)?;
        let after = self.world.player(LOCAL).map_or(0, |p| p.health);
        let external = u32::try_from(health - after)
            .unwrap_or(0)
            .saturating_sub(self.queued_damage);
        self.queued_damage = 0;
        self.regrow_resources();
        let alive = after > 0;
        if alive {
            self.vitals.wound(external);
        } else {
            self.refund_crafting()?;
            self.drop_loot()?;
            self.dismount();
            self.world.set_external_motion(LOCAL, false);
            self.skate_input = SkateInput::default();
            return Ok(());
        }
        if let Some(skate) = &mut self.skate {
            let origin = self.world.player(LOCAL).ok_or("Player is missing")?.origin;
            let rails = self.editor.rails();
            let step = skate.step(&self.world, &rails, 0.017, self.skate_input, origin)?;
            self.world.set_origin(LOCAL, step.origin);
            self.skate_roll = step.board_roll;
            self.last_skate_event = step.event;
            match step.event {
                SkateEvent::Bailed => {
                    self.world.queue_environment_damage(LOCAL, 10)?;
                    self.queued_damage += 10;
                    self.message =
                        "Bail: land in line with the board and complete your flip".into();
                }
                SkateEvent::Landed { points } => {
                    self.message = format!("Landed: +{points} skate points");
                }
                _ => {}
            }
        }
        self.skate_input.ollie = false;
        self.skate_input.flip = false;
        if let Some(recipe) = self.crafting.advance(0.017, &mut self.inventory)? {
            self.message = format!("Crafted {}", recipe.name());
        }
        let healed = self.vitals.regenerate(0.017)?;
        if healed > 0 {
            let _ = self.world.heal_player(LOCAL, healed);
        }
        let damage = self.vitals.advance(0.017)?;
        if damage > 0 && self.world.player(LOCAL).is_some_and(|p| p.health > 0) {
            self.world.queue_environment_damage(LOCAL, damage)?;
            self.queued_damage += damage;
        }
        Ok(())
    }

    pub fn toggle_skate(&mut self) -> Result<(), String> {
        let player = self
            .world
            .player(LOCAL)
            .filter(|p| p.health > 0)
            .ok_or("Player is not alive")?;
        if self.skate.is_some() {
            self.dismount();
        } else {
            let mut skate = SkateState::new(player.viewangles[1])?;
            skate.total_score = self.skate_score;
            self.skate = Some(skate);
        }
        self.world.set_external_motion(LOCAL, self.skate.is_some());
        self.skate_input = SkateInput::default();
        self.skate_roll = 0.;
        Ok(())
    }

    /// The skate score earned so far, mounted or not.
    pub fn skate_score(&self) -> u64 {
        self.skate
            .as_ref()
            .map_or(self.skate_score, |k| k.total_score)
    }

    /// Leaves the board. Banked score stays with the session; an unfinished
    /// grind and its pending points are dropped with the board state.
    fn dismount(&mut self) {
        if let Some(skate) = self.skate.take() {
            self.skate_score = skate.total_score;
        }
    }

    pub fn craft(&mut self, recipe: Recipe) -> Result<(), String> {
        self.require_alive()?;
        let mut inventory = self.inventory.clone();
        let cost = inventory.craft(recipe, self.world.buildings().inventory(LOCAL.0))?;
        self.world
            .buildings_mut()
            .consume(LOCAL.0, cost)
            .map_err(|e| e.to_string())?;
        self.inventory = inventory;
        Ok(())
    }

    pub fn crafting_queue(&self) -> &[CraftJob] {
        self.crafting.jobs()
    }

    pub fn queue_craft(&mut self, recipe: Recipe) -> Result<(), String> {
        self.require_alive()?;
        let mut crafting = self.crafting.clone();
        crafting.push(recipe)?;
        self.world
            .buildings_mut()
            .consume(LOCAL.0, recipe.cost())
            .map_err(|_| "Not enough crafting resources".to_string())?;
        self.crafting = crafting;
        Ok(())
    }

    pub fn cancel_craft(&mut self, index: usize) -> Result<Resources, String> {
        self.require_alive()?;
        let mut crafting = self.crafting.clone();
        let refund = crafting.cancel(index)?.cost();
        let mut balance = self.world.buildings().inventory(LOCAL.0);
        balance.add(refund);
        if [balance.wood, balance.stone, balance.metal]
            .iter()
            .any(|&n| n > persistence::MAX_RESOURCE_BALANCE)
        {
            return Err("Resource storage is full".into());
        }
        self.world
            .buildings_mut()
            .grant(LOCAL.0, refund)
            .map_err(|e| e.to_string())?;
        self.crafting = crafting;
        Ok(refund)
    }

    fn refund_crafting(&mut self) -> Result<(), String> {
        let mut refund = self.crafting.clear();
        let balance = self.world.buildings().inventory(LOCAL.0);
        let room = |have: u32| persistence::MAX_RESOURCE_BALANCE.saturating_sub(have);
        refund.wood = refund.wood.min(room(balance.wood));
        refund.stone = refund.stone.min(room(balance.stone));
        refund.metal = refund.metal.min(room(balance.metal));
        self.world
            .buildings_mut()
            .grant(LOCAL.0, refund)
            .map_err(|e| e.to_string())
    }

    fn gathering_ray_from_view(&self) -> Result<([f32; 3], [f32; 3], f32), String> {
        let (start, end) = self.view_ray()?;
        let obstacle = self.world.trace_world(start, end, [0.; 3], [0.; 3], 1);
        if obstacle.startsolid != 0 {
            return Err("Cannot gather through a solid object".into());
        }
        Ok((start, end, obstacle.fraction))
    }

    pub fn gather_target_from_view(&self) -> Result<Option<&ResourceNode>, String> {
        let (start, end, obstacle_fraction) = self.gathering_ray_from_view()?;
        self.gathering
            .target_from_ray(start, end, obstacle_fraction)
    }

    pub fn gather_from_view(&mut self) -> Result<Harvest, String> {
        let (start, end, obstacle_fraction) = self.gathering_ray_from_view()?;
        let mut gathering = self.gathering.clone();
        let tool = self.inventory.count(Item::Hatchet) > 0;
        let mut harvested = gathering.harvest_from_ray(start, end, obstacle_fraction, tool)?;
        let mut inventory = self.inventory.clone();
        harvested.tool_broke =
            tool && harvested.kind.is_solid() && inventory.wear_tool(Item::Hatchet) == Some(true);
        if harvested.tool_broke {
            self.message = "Your stone hatchet broke".into();
        }
        match harvested.kind {
            ResourceKind::Berry => inventory.add(Item::Food, harvested.amount)?,
            ResourceKind::Water => inventory.add(Item::Water, harvested.amount)?,
            _ => {}
        }
        self.world
            .buildings_mut()
            .grant(LOCAL.0, harvested.resources())
            .map_err(|e| e.to_string())?;
        if harvested.remaining == 0 {
            self.world
                .install_content(authored_content(&self.terrain, &self.editor, &gathering));
        }
        self.inventory = inventory;
        self.gathering = gathering;
        Ok(harvested)
    }

    fn regrow_resources(&mut self) {
        let (world, editor) = (&self.world, &self.editor);
        let regrown = self.gathering.advance(0.017, |node| {
            let (lo, hi) = node.bounds();
            node.brush()
                .is_some_and(|b| brushes_overlap_players(world, &[b]))
                || editor
                    .brushes()
                    .iter()
                    .any(|b| box_overlaps_brush(lo, hi, b))
                || world.buildings().pieces().any(|p| {
                    world
                        .buildings()
                        .bounds(p)
                        .iter()
                        .any(|(a, b)| (0..3).all(|k| a[k] <= hi[k] + 1. && lo[k] - 1. <= b[k]))
                })
        });
        if !regrown.is_empty() {
            self.world.install_content(authored_content(
                &self.terrain,
                &self.editor,
                &self.gathering,
            ));
        }
    }

    pub fn use_item(&mut self, item: Item) -> Result<(), String> {
        self.require_alive()?;
        let quantity = if item == Item::Ammo {
            self.inventory.count(item).min(30)
        } else {
            1
        };
        let mut inventory = self.inventory.clone();
        let effects = inventory.use_item(item, quantity)?;
        let mut vitals = self.vitals;
        vitals.apply(&effects)?;
        if effects.heal > 0 {
            let healed = self.world.heal_player(LOCAL, effects.heal);
            if !(effects.stop_bleeding && self.vitals.is_bleeding()) {
                healed?;
            }
        }
        if effects.ammo > 0 {
            self.world.add_reserve_ammo(LOCAL, effects.ammo)?;
        }
        self.inventory = inventory;
        self.vitals = vitals;
        Ok(())
    }

    pub fn split_stack(&mut self, slot: usize, quantity: u32) -> Result<(), String> {
        self.require_alive()?;
        self.inventory.split(slot, quantity)
    }

    pub fn move_stack(&mut self, from: usize, to: usize) -> Result<(), String> {
        self.require_alive()?;
        self.inventory.move_stack(from, to)
    }

    pub fn discard_stack(&mut self, slot: usize, quantity: u32) -> Result<Stack, String> {
        self.require_alive()?;
        self.inventory.discard(slot, quantity)
    }

    pub fn recycle_stack(&mut self, slot: usize, quantity: u32) -> Result<Resources, String> {
        self.require_alive()?;
        let stack = *self
            .inventory
            .stacks()
            .get(slot)
            .ok_or("No stack in that slot")?;
        let mut refund = Recipe::recycle_yield(stack.item, quantity)?;
        if stack.wear > 0 {
            let condition = |n: u32| n * (MAX_TOOL_WEAR - stack.wear) / MAX_TOOL_WEAR;
            refund = Resources {
                wood: condition(refund.wood),
                stone: condition(refund.stone),
                metal: condition(refund.metal),
            };
        }
        let mut balance = self.world.buildings().inventory(LOCAL.0);
        balance.add(refund);
        if [balance.wood, balance.stone, balance.metal]
            .iter()
            .any(|&n| n > persistence::MAX_RESOURCE_BALANCE)
        {
            return Err("Resource storage is full".into());
        }
        let mut inventory = self.inventory.clone();
        inventory.discard(slot, quantity)?;
        self.world
            .buildings_mut()
            .grant(LOCAL.0, refund)
            .map_err(|e| e.to_string())?;
        self.inventory = inventory;
        Ok(refund)
    }

    pub fn loot_bags(&self) -> &[LootBag] {
        self.loot.bags()
    }

    pub fn loot_bag_in_reach(&self) -> Option<&LootBag> {
        let player = self.world.player(LOCAL).filter(|p| p.health > 0)?;
        self.loot.nearest(player.origin)
    }

    pub fn pick_up_loot(&mut self) -> Result<u32, String> {
        self.require_alive()?;
        let id = self
            .loot_bag_in_reach()
            .ok_or("No loot bag within reach")?
            .id();
        let mut inventory = self.inventory.clone();
        let mut loot = self.loot.clone();
        let taken = loot.take(id, &mut inventory)?;
        self.inventory = inventory;
        self.loot = loot;
        self.message = format!("Picked up {taken} items");
        Ok(taken)
    }

    fn drop_loot(&mut self) -> Result<(), String> {
        if self.inventory.stacks().is_empty() {
            return Ok(());
        }
        let origin = self.world.player(LOCAL).ok_or("Player is missing")?.origin;
        bag_inventory(&self.world, origin, &mut self.inventory, &mut self.loot)?;
        self.message = "You died; your items are in a bag where you fell".into();
        Ok(())
    }

    fn require_alive(&self) -> Result<(), String> {
        if self.world.player(LOCAL).is_some_and(|p| p.health > 0) {
            Ok(())
        } else {
            Err("Player is not alive".into())
        }
    }

    pub fn respawn(&mut self) -> Result<(), String> {
        let player = self.world.player(LOCAL).ok_or("Player is missing")?;
        if player.health > 0 {
            return Err("Player is already alive".into());
        }
        self.drop_loot()?;
        let player = self.world.player(LOCAL).ok_or("Player is missing")?;
        let angles = player.viewangles;
        let origin = free_spawn(&self.world)
            .ok_or("Spawn area is blocked; remove nearby structures before respawning")?;
        self.world
            .start_gsc(
                "survival/session::respawn",
                sim::script::Value::level(),
                vec![
                    sim::script::Value::Int(LOCAL.0 as i32),
                    sim::script::Value::Vector(origin),
                    sim::script::Value::Vector(angles),
                ],
            )
            .map_err(|e| e.to_string())?;
        self.vitals = Vitals::default();
        self.queued_damage = 0;
        self.dismount();
        self.world.set_external_motion(LOCAL, false);
        self.skate_input = SkateInput::default();
        Ok(())
    }

    fn view_ray(&self) -> Result<([f32; 3], [f32; 3]), String> {
        let player = self
            .world
            .player(LOCAL)
            .filter(|p| p.health > 0)
            .ok_or("Player is not alive")?;
        let mut start = player.origin;
        start[2] += player.view_height_current;
        let (pitch, yaw) = (
            player.viewangles[0].to_radians(),
            player.viewangles[1].to_radians(),
        );
        let direction = [
            pitch.cos() * yaw.cos(),
            pitch.cos() * yaw.sin(),
            -pitch.sin(),
        ];
        Ok((
            start,
            std::array::from_fn(|k| start[k] + direction[k] * 360.),
        ))
    }

    pub fn preview_building_from_view(&self, kind: Kind, axis: u8) -> BuildingPlacementPreview {
        self.building_placement_from_view(kind, axis).0
    }

    fn building_placement_from_view(
        &self,
        kind: Kind,
        axis: u8,
    ) -> (BuildingPlacementPreview, Option<(Socket, bool)>) {
        let mut preview = BuildingPlacementPreview {
            kind,
            socket: None,
            bounds: Vec::new(),
            cost: Grade::Wood.cost(kind),
            resources: self.world.buildings().inventory(LOCAL.0),
            error: None,
        };
        let candidate = (|| {
            let (start, end) = self.view_ray()?;
            let hit = self.world.trace_world(start, end, [0.; 3], [0.; 3], 1);
            if hit.fraction >= 1. || hit.startsolid != 0 {
                return Err("Aim at ground or a building within reach".into());
            }
            let buildings = self.world.buildings();
            let anchor = buildings.anchor;
            let pos =
                std::array::from_fn::<_, 3, _>(|k| hit.endpos[k] + hit.normal[k] * 0.2 - anchor[k]);
            let deck = matches!(kind, Kind::Foundation | Kind::Floor);
            let axis = if deck { 0 } else { axis % 2 };
            let socket = Socket {
                x: if !deck && axis == 1 {
                    (pos[0] / CELL).round() as i32
                } else {
                    (pos[0] / CELL).floor() as i32
                },
                y: if !deck && axis == 0 {
                    (pos[1] / CELL).round() as i32
                } else {
                    (pos[1] / CELL).floor() as i32
                },
                level: if kind == Kind::Foundation {
                    0
                } else {
                    (pos[2] / WALL_HEIGHT).round().max(0.) as i32
                },
                axis,
            };
            preview.socket = Some(socket);
            let piece = Piece {
                id: 0,
                owner: LOCAL.0,
                kind,
                grade: Grade::Wood,
                socket,
                health: Grade::Wood.health(),
                open: false,
            };
            preview.bounds = buildings.bounds(&piece);
            let grounded = if kind == Kind::Foundation {
                [(0.1, 0.1), (0.9, 0.1), (0.1, 0.9), (0.9, 0.9)]
                    .iter()
                    .all(|&(x, y)| {
                        let p = [
                            anchor[0] + (socket.x as f32 + x) * CELL,
                            anchor[1] + (socket.y as f32 + y) * CELL,
                            anchor[2],
                        ];
                        let tr = self.world.trace_static_world(
                            [p[0], p[1], p[2] + 16.],
                            [p[0], p[1], p[2] - 16.],
                            [0.; 3],
                            [0.; 3],
                            1,
                        );
                        tr.fraction < 1.
                            && tr.startsolid == 0
                            && tr.normal[2] >= 0.7
                            && (tr.endpos[2] - p[2]).abs() <= 1.
                    })
            } else {
                false
            };
            buildings
                .can_place(LOCAL.0, kind, socket, grounded)
                .map_err(|error| error.to_string())?;
            let mut player_overlap = overlaps_players(&self.world, buildings);
            self.world.visit_players(|_, player| {
                if player.health > 0
                    && buildings.overlaps_piece(&piece, player.origin, PLAYER_MINS, PLAYER_MAXS)
                {
                    player_overlap = true;
                }
            });
            if player_overlap {
                return Err("Building overlaps a player".into());
            }
            if kind != Kind::Foundation {
                for (lo, hi) in &preview.bounds {
                    let center = std::array::from_fn(|k| (lo[k] + hi[k]) * 0.5);
                    let mins = std::array::from_fn(|k| lo[k] - center[k] + 0.5);
                    let maxs = std::array::from_fn(|k| hi[k] - center[k] - 0.5);
                    if self
                        .world
                        .trace_static_world(center, center, mins, maxs, 1)
                        .startsolid
                        != 0
                    {
                        return Err("Building overlaps terrain".into());
                    }
                }
            }
            Ok((socket, grounded))
        })();
        match candidate {
            Ok(candidate) => (preview, Some(candidate)),
            Err(error) => {
                preview.error = Some(error);
                (preview, None)
            }
        }
    }

    pub fn place_from_view(&mut self, kind: Kind, axis: u8) -> Result<u32, String> {
        let (preview, candidate) = self.building_placement_from_view(kind, axis);
        let (socket, grounded) = candidate.ok_or_else(|| {
            preview
                .error
                .unwrap_or_else(|| "No building placement target".into())
        })?;
        self.world
            .buildings_mut()
            .place(LOCAL.0, kind, socket, grounded)
            .map_err(|error| error.to_string())
    }

    pub fn toggle_door_from_view(&mut self) -> Result<(), String> {
        let (start, end) = self.view_ray()?;
        let outcome = self.world.bullet_trace(
            sim::BulletTraceQuery {
                start,
                end,
                mask: 1,
                ignore: Some(LOCAL),
                ignore_hit: None,
                ignore_model: None,
            },
            None,
        );
        let sim::TraceOutcome::Hit {
            collider:
                sim::ColliderId::World {
                    building_id: Some(id),
                    ..
                },
            ..
        } = outcome
        else {
            return Err("Aim at a door within reach".into());
        };
        let mut candidate = self.world.buildings().clone();
        candidate
            .toggle_door(LOCAL.0, id)
            .map_err(|e| e.to_string())?;
        if overlaps_players(&self.world, &candidate) {
            return Err("Door cannot close through a player".into());
        }
        *self.world.buildings_mut() = candidate;
        Ok(())
    }

    /// The building piece under the player view ray. Buildings win
    /// coplanar ties with the terrain: the shared bullet trace only reports
    /// a building when it is strictly nearer, which makes ground-level
    /// foundation tops unselectable. Pieces are traced directly and static
    /// geometry occludes them instead.
    fn aimed_building(&self) -> Result<u32, String> {
        let (start, end) = self.view_ray()?;
        let (hit, piece) = self
            .world
            .buildings()
            .trace_hit(start, end, [0.; 3], [0.; 3], 1);
        let blocked = self
            .world
            .trace_static_world(start, end, [0.; 3], [0.; 3], 1);
        let ray_len = dot(sub(end, start), sub(end, start)).sqrt();
        piece
            .filter(|_| {
                hit.startsolid == 0
                    && hit.fraction < 1.
                    && blocked.startsolid == 0
                    && hit.fraction * ray_len <= blocked.fraction * ray_len + 0.5
            })
            .ok_or_else(|| "Aim at a building within reach".into())
    }

    pub fn repair_from_view(&mut self) -> Result<(), String> {
        let id = self.aimed_building()?;
        self.world
            .buildings_mut()
            .repair(LOCAL.0, id)
            .map_err(|e| e.to_string())
    }

    pub fn upgrade_from_view(&mut self, grade: Grade) -> Result<(), String> {
        let id = self.aimed_building()?;
        self.world
            .buildings_mut()
            .upgrade(LOCAL.0, id, grade)
            .map_err(|e| e.to_string())
    }

    /// Demolishes the aimed piece and anything left unsupported by it,
    /// returning every removed piece ID.
    pub fn demolish_from_view(&mut self) -> Result<Vec<u32>, String> {
        let id = self.aimed_building()?;
        self.world
            .buildings_mut()
            .demolish(LOCAL.0, id)
            .map_err(|e| e.to_string())
    }

    /// Writes the complete session (scene and local player) atomically.
    /// See `persistence` for the exact list of persisted state.
    pub fn save(&self, path: &Path) -> Result<(), String> {
        let player = self.world.player(LOCAL).ok_or("Player is missing")?;
        let meta = self.world.client_meta(LOCAL).ok_or("Player is missing")?;
        let facts = self
            .world
            .weapon_combat_facts(persistence::AUTHORED_WEAPON)
            .ok_or("Weapon has no combat data")?;
        let alive = player.health > 0;
        let (clip, reserve) = if alive { self.ammo() } else { (0, 0) };
        let saved = persistence::SavedPlayer {
            alive,
            origin: player.origin,
            velocity: player.velocity,
            view: normalize_view(player.viewangles),
            health: player.health.max(0),
            weapon: persistence::AUTHORED_WEAPON,
            clip,
            reserve,
            kills: meta.kills,
            deaths: meta.deaths,
            score: meta.score,
            skate: self.skate.as_ref().map(SkateState::saved),
            skate_score: Some(self.skate_score()),
        };
        saved.validate(player.max_health, facts.clip_size, facts.max_ammo)?;
        let scene = persistence::SavedSession {
            version: persistence::SAVE_VERSION,
            seed: self.terrain.seed,
            buildings: String::from_utf8(
                self.world
                    .buildings()
                    .to_json()
                    .map_err(|e| e.to_string())?,
            )
            .map_err(|e| e.to_string())?,
            objects: self.editor.objects().cloned().collect(),
            inventory: self.inventory.clone(),
            loot: self.loot.clone(),
            vitals: self.vitals,
            pending_damage: self.queued_damage,
            crafting: self.crafting.clone(),
            gathering: self.gathering.clone(),
            player: saved,
        };
        persistence::write_atomic(path, &scene)
    }

    /// Loads a format-3 save, or migrates a format-2 scene save. The whole
    /// candidate is validated and restored on a copy of the world through
    /// one authority tick; the session changes only if that succeeds.
    pub fn load(&mut self, path: &Path) -> Result<(), String> {
        let loaded = persistence::read(path)?;
        self.restore(loaded)
    }

    fn restore(&mut self, loaded: persistence::Loaded) -> Result<(), String> {
        let persistence::Loaded {
            session: scene,
            migrated,
        } = loaded;
        if scene.seed != self.terrain.seed || scene.gathering.seed() != self.terrain.seed {
            return Err("Save belongs to a different terrain seed".into());
        }
        let buildings = BuildingWorld::from_json(scene.buildings.as_bytes())
            .map_err(|e| format!("Saved buildings are invalid: {e}"))?;
        if buildings.anchor != self.world.buildings().anchor {
            return Err("Saved buildings use a different anchor".into());
        }
        let balance = buildings.inventory(LOCAL.0);
        if [balance.wood, balance.stone, balance.metal]
            .iter()
            .any(|&n| n > persistence::MAX_RESOURCE_BALANCE)
        {
            return Err("Saved resource balance is out of range".into());
        }
        let editor = EditorState::from_objects(scene.objects)?;
        let facts = self
            .world
            .weapon_combat_facts(persistence::AUTHORED_WEAPON)
            .ok_or("Weapon has no combat data")?;
        let max_health = self
            .world
            .player(LOCAL)
            .ok_or("Player is missing")?
            .max_health;
        let mut player = scene.player;
        let skate = player.skate.map(SkateState::from_saved).transpose()?;
        // Saves from before the session-level score omit it and carry only
        // the mounted total; otherwise both must agree.
        let skate_score = match (&skate, player.skate_score) {
            (Some(k), None) => k.total_score,
            (Some(k), Some(score)) if score == k.total_score => score,
            (Some(_), Some(_)) => return Err("Saved skate scores do not match".into()),
            (None, score) => score.unwrap_or(0),
        };

        let mut world = self.world.clone();
        world.install_content(authored_content(&self.terrain, &editor, &scene.gathering));
        *world.buildings_mut() = buildings;
        if migrated {
            player.origin = free_spawn(&world)
                .ok_or("Spawn area is blocked; the version 2 save cannot place the player")?;
        }
        player.validate(max_health, facts.clip_size, facts.max_ammo)?;
        if player.alive && player_blocked(&world, player.origin) {
            return Err("Saved player position is blocked".into());
        }
        if let Some(skate) = &skate {
            skate.check_rails(&editor.rails(), player.origin)?;
        }
        let mut others_blocked = false;
        world.visit_players(|id, p| {
            others_blocked |= id != LOCAL
                && p.health > 0
                && (player_blocked(&world, p.origin)
                    || player.alive && hulls_overlap(p.origin, player.origin));
        });
        if others_blocked {
            return Err("Saved scene overlaps another player".into());
        }

        use sim::script::Value;
        world
            .start_gsc(
                "survival/session::restore",
                Value::level(),
                vec![
                    Value::Int(LOCAL.0 as i32),
                    Value::Int(player.alive.into()),
                    Value::Vector(player.origin),
                    Value::Vector(player.view),
                    Value::Vector(player.velocity),
                    Value::Int(player.health),
                    Value::Int(player.clip),
                    Value::Int(player.reserve),
                    Value::Int(player.kills),
                    Value::Int(player.deaths),
                    Value::Int(player.score),
                ],
            )
            .map_err(|e| e.to_string())?;
        world.set_external_motion(LOCAL, skate.is_some());
        let tick = self.tick.checked_add(1).ok_or("Session clock exhausted")?;
        // The view is the command angles plus the player's delta angles, so
        // the restoring command carries the saved view rather than zero. A
        // scripted spawn rebases the delta on the previous command instead.
        let current = world.player(LOCAL).ok_or("Player is missing")?;
        let angles = if player.alive && current.health <= 0 {
            world
                .old_cmd_angles(LOCAL)
                .ok_or("Player command history is missing")?
        } else {
            let delta = current.delta_angles;
            std::array::from_fn(|k| ((player.view[k] - delta[k]) * 65536. / 360.).round() as i32)
        };
        let restore_cmd = UserCmd {
            angles,
            ..Default::default()
        };
        authority_step(&mut world, tick, restore_cmd)?;
        if let Some(fault) = world.take_script_fault() {
            return Err(format!("Restoring the player failed: {fault}"));
        }
        verify_restored(&world, &player)?;
        if !player.alive && !scene.crafting.jobs().is_empty() {
            return Err("A dead player cannot have queued crafting".into());
        }
        let mut inventory = scene.inventory;
        let mut loot = scene.loot;
        if !player.alive {
            bag_inventory(&world, player.origin, &mut inventory, &mut loot)?;
        }
        let queued_damage = if player.alive && scene.pending_damage > 0 {
            world.queue_environment_damage(LOCAL, scene.pending_damage)?;
            scene.pending_damage
        } else {
            0
        };

        self.world = world;
        self.tick = tick;
        self.editor = editor;
        self.inventory = inventory;
        self.loot = loot;
        self.vitals = scene.vitals;
        self.gathering = scene.gathering;
        self.skate = skate;
        self.skate_score = skate_score;
        self.skate_input = SkateInput::default();
        self.skate_roll = 0.;
        self.last_skate_event = SkateEvent::None;
        self.queued_damage = queued_damage;
        self.crafting = scene.crafting;
        Ok(())
    }
}

fn bag_inventory(
    world: &SimWorld,
    origin: [f32; 3],
    inventory: &mut Inventory,
    loot: &mut LootBags,
) -> Result<(), String> {
    let below = [origin[0], origin[1], origin[2] - 4096.];
    let ground = world.trace_world(origin, below, [0.; 3], [0.; 3], 1);
    let position = if ground.startsolid != 0 {
        origin
    } else {
        ground.endpos
    };
    loot.drop_bag(position, inventory.clone())?;
    *inventory = Inventory::default();
    Ok(())
}

fn authority_step(world: &mut SimWorld, tick: u32, mut cmd: UserCmd) -> Result<(), String> {
    cmd.server_time = i32::try_from(u64::from(tick) * 17).map_err(|_| "Session clock exhausted")?;
    cmd.weapon = 1;
    sim::try_step(
        world,
        Tick(tick),
        &TickInput::from_cmds(vec![(LOCAL, cmd)]),
        17,
        sim::StepReason::AuthorityFrame,
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

fn free_spawn(world: &SimWorld) -> Option<[f32; 3]> {
    SPAWN_POINTS
        .into_iter()
        .find(|p| !player_blocked(world, *p))
}

fn player_blocked(world: &SimWorld, origin: [f32; 3]) -> bool {
    world
        .trace_world(origin, origin, PLAYER_MINS, PLAYER_MAXS, 1)
        .startsolid
        != 0
}

fn hulls_overlap(a: [f32; 3], b: [f32; 3]) -> bool {
    (0..3).all(|k| (a[k] - b[k]).abs() < PLAYER_MAXS[k] - PLAYER_MINS[k])
}

/// Pitch is stored in [-90, 90] and yaw/roll in (-180, 180].
fn normalize_view(view: [f32; 3]) -> [f32; 3] {
    view.map(|a| {
        let a = a.rem_euclid(360.);
        if a > 180. { a - 360. } else { a }
    })
}

/// Confirms the authority tick produced the saved player state.
fn verify_restored(world: &SimWorld, saved: &persistence::SavedPlayer) -> Result<(), String> {
    let failed = || "Saved player state could not be restored".to_string();
    let player = world.player(LOCAL).ok_or_else(failed)?;
    let meta = world.client_meta(LOCAL).ok_or_else(failed)?;
    if (player.health > 0) != saved.alive
        || player.health.max(0) != saved.health
        || meta.kills != saved.kills
        || meta.deaths != saved.deaths
        || meta.score != saved.score
    {
        return Err(failed());
    }
    let turned = (0..2).any(|k| {
        let d = (player.viewangles[k] - saved.view[k]).rem_euclid(360.);
        d.min(360. - d) > 0.1
    });
    let drift = (0..3)
        .map(|k| (player.origin[k] - saved.origin[k]).abs())
        .fold(0., f32::max);
    if turned {
        return Err(failed());
    }
    if saved.alive {
        let facts = world
            .weapon_combat_facts(player.weapon)
            .ok_or_else(failed)?;
        let clip = weapon_iw4::get_clip_for_hand(&player.ammoclip, facts.clip_index, 0);
        let reserve = weapon_iw4::get_ammo_not_in_clip(&player.ammo, facts.ammo_index);
        // A living player has had one movement step since the restore.
        let travel = dot(saved.velocity, saved.velocity).sqrt() * 0.017;
        if player.weapon != saved.weapon
            || clip != saved.clip
            || reserve != saved.reserve
            || drift > 1. + travel
        {
            return Err(failed());
        }
    } else {
        let skid = (0..3)
            .map(|k| (player.velocity[k] - saved.velocity[k]).abs())
            .fold(0., f32::max);
        if drift > 1. || skid > 1. {
            return Err(failed());
        }
    }
    Ok(())
}

fn overlaps_players(world: &SimWorld, buildings: &BuildingWorld) -> bool {
    let mut overlaps = false;
    world.visit_players(|_, p| {
        if p.health > 0
            && buildings
                .trace(p.origin, p.origin, [-15., -15., 0.], [15., 15., 70.], 1)
                .startsolid
                != 0
        {
            overlaps = true;
        }
    });
    overlaps
}

fn props_overlap_players(world: &SimWorld, editor: &EditorState) -> bool {
    let brushes = editor.brushes();
    brushes_overlap_players(world, &brushes)
}

fn brushes_overlap_players(world: &SimWorld, brushes: &[SimBrush]) -> bool {
    let mut overlap = false;
    world.visit_players(|_, p| {
        if p.health <= 0 {
            return;
        }
        let center = [p.origin[0], p.origin[1], p.origin[2] + 35.];
        overlap |= brushes.iter().any(|brush| {
            brush.planes.iter().all(|plane| {
                let extent = plane[0].abs() * 15. + plane[1].abs() * 15. + plane[2].abs() * 35.;
                dot([plane[0], plane[1], plane[2]], center) - extent < plane[3] - 0.1
            })
        });
    });
    overlap
}

fn box_overlaps_brush(lo: [f32; 3], hi: [f32; 3], brush: &SimBrush) -> bool {
    let normals: Vec<[f32; 3]> = brush.planes.iter().map(|p| [p[0], p[1], p[2]]).collect();
    let mut vertices = Vec::new();
    for (i, a) in brush.planes.iter().enumerate() {
        for (j, b) in brush.planes.iter().enumerate().skip(i + 1) {
            for c in brush.planes.iter().skip(j + 1) {
                let (na, nb, nc) = ([a[0], a[1], a[2]], [b[0], b[1], b[2]], [c[0], c[1], c[2]]);
                let det = dot(na, cross(nb, nc));
                if det.abs() < 1e-6 {
                    continue;
                }
                let terms = [
                    cross(nb, nc).map(|v| v * a[3]),
                    cross(nc, na).map(|v| v * b[3]),
                    cross(na, nb).map(|v| v * c[3]),
                ];
                let point: [f32; 3] =
                    std::array::from_fn(|k| (terms[0][k] + terms[1][k] + terms[2][k]) / det);
                if brush
                    .planes
                    .iter()
                    .all(|p| dot([p[0], p[1], p[2]], point) <= p[3] + 0.01)
                {
                    vertices.push(point);
                }
            }
        }
    }
    if vertices.is_empty() {
        return false;
    }
    let mut axes = normals.clone();
    for k in 0..3 {
        let mut box_axis = [0.; 3];
        box_axis[k] = 1.;
        axes.push(box_axis);
        for (i, a) in normals.iter().enumerate() {
            for b in normals.iter().skip(i + 1) {
                let axis = cross(box_axis, cross(*a, *b));
                if dot(axis, axis) > 1e-6 {
                    axes.push(axis);
                }
            }
        }
    }
    let center: [f32; 3] = std::array::from_fn(|k| (lo[k] + hi[k]) / 2.);
    let half: [f32; 3] = std::array::from_fn(|k| (hi[k] - lo[k]) / 2.);
    axes.iter().all(|axis| {
        let length = dot(*axis, *axis).sqrt();
        let middle = dot(*axis, center);
        let extent = axis[0].abs() * half[0] + axis[1].abs() * half[1] + axis[2].abs() * half[2];
        let (low, high) = vertices.iter().fold((f32::MAX, f32::MIN), |(l, h), v| {
            let d = dot(*axis, *v);
            (l.min(d), h.max(d))
        });
        middle - extent < high - 0.1 * length && low + 0.1 * length < middle + extent
    })
}

fn intersect(brush: &SimBrush, start: [f32; 3], end: [f32; 3]) -> Option<f32> {
    intersect_entry(brush, start, end).map(|(t, _)| t)
}

/// Like [`intersect`], but also reports the entry plane's outward normal
/// for the nearest contact. A ray starting inside yields fraction zero
/// with a zero normal, which never passes an upward-face check.
fn intersect_entry(brush: &SimBrush, start: [f32; 3], end: [f32; 3]) -> Option<(f32, [f32; 3])> {
    let mut enter: f32 = 0.;
    let mut leave: f32 = 1.;
    let mut normal = [0.; 3];
    for plane in &brush.planes {
        let n = [plane[0], plane[1], plane[2]];
        let a = dot(n, start) - plane[3];
        let b = dot(n, end) - plane[3];
        if a > 0. && b > 0. {
            return None;
        }
        if a <= 0. && b <= 0. {
            continue;
        }
        let t = a / (a - b);
        if a > b {
            if t > enter {
                enter = t;
                normal = n;
            }
        } else {
            leave = leave.min(t);
        }
        if enter > leave {
            return None;
        }
    }
    Some((enter, normal))
}

fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}
fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    std::array::from_fn(|i| a[i] - b[i])
}
fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}
fn normalize(v: [f32; 3]) -> [f32; 3] {
    let length = dot(v, v).sqrt();
    v.map(|x| x / length)
}
fn triangle_prism(v: [[f32; 3]; 3], bottom: f32, flags: u32) -> SimBrush {
    let normal = normalize(cross(sub(v[1], v[0]), sub(v[2], v[0])));
    let mut planes = vec![
        [normal[0], normal[1], normal[2], dot(normal, v[0])],
        [0., 0., -1., -bottom],
    ];
    for i in 0..3 {
        let a = v[i];
        let b = v[(i + 1) % 3];
        let n = normalize([b[1] - a[1], a[0] - b[0], 0.]);
        planes.push([n[0], n[1], 0., dot(n, a)]);
    }
    SimBrush {
        plane_surface_flags: vec![flags; planes.len()],
        planes,
        contents: 1,
        glass_encoded: 0,
    }
}

fn authored_content(
    terrain: &Terrain,
    editor: &EditorState,
    gathering: &GatheringWorld,
) -> std::sync::Arc<sim::SimContent> {
    let mut content = SimContentBuilder::default();
    let mut brushes = terrain.brushes();
    brushes.extend(editor.brushes());
    brushes.extend(gathering.brushes());
    content.set_clip_brushes(brushes);
    content.set_weapon_combat_table(vec![
        weapon_iw4::WeaponCombatFacts::none(),
        weapon_iw4::WeaponCombatFacts {
            damage: 30,
            min_damage: 20,
            max_damage_range: 600.,
            min_damage_range: 2000.,
            fire_time_ms: 95,
            raise_time_ms: 250,
            drop_time_ms: 200,
            reload_time_ms: 2000,
            reload_empty_time_ms: 2400,
            reload_add_time_ms: 1600,
            reload_empty_add_time_ms: 1800,
            clip_size: 30,
            start_ammo: 120,
            max_ammo: 180,
            ammo_index: 1,
            clip_index: 1,
            shots_per_fire: 1,
            ads_in_rate: 0.005,
            ads_out_rate: 0.006,
            melee_damage: 45,
            hip_spread_stand_min: 1.5,
            hip_spread_stand_max: 4.,
            ..weapon_iw4::WeaponCombatFacts::none()
        },
    ]);
    content.set_weapon_runnable_table(vec![false, true]);
    content.set_weapon_script_names(vec![String::new(), "authored_carbine".into()]);

    content.finish()
}
