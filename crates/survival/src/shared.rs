use super::{
    AuthoredWorld, BuildingPlacementPreview, EditorState, GatheringWorld, Harvest, Inventory,
    ResourceKind, ResourceNode, SPAWN_POINTS, Terrain, actor_building_placement,
    actor_gathering_ray, advance_resources, authored_content, hulls_overlap, new_authored_world,
    persistence, player_blocked,
};
use playerstate_iw4::{UserCmd, buttons};
use rust_building::{Kind, MAX_INVENTORIES, Resources};
use sim::{ClientId, SimWorld, Tick, TickInput};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::atomic::{AtomicU64, Ordering};

pub const SHARED_STEP_MS: i32 = 50;

pub fn shared_replica(nodes: Vec<ResourceNode>) -> Result<SimWorld, String> {
    let terrain = Terrain::new(731);
    let gathering = GatheringWorld::from_nodes(terrain.seed, nodes)?;
    let mut world = SimWorld::new();
    world.install_content(authored_content(
        &terrain,
        &EditorState::default(),
        &gathering,
    ));
    world
        .bootstrap(sim::MatchBootstrap {
            seed: 1,
            ..Default::default()
        })
        .map_err(str::to_owned)?;
    Ok(world)
}
const MAX_ACTORS: usize = 2;
const MAX_RECEIPTS: usize = 256;
const OWNER_BASE: u32 = 64;
const MOVEMENT_BUTTONS: u32 =
    buttons::SPRINT | buttons::PRONE | buttons::CROUCH | buttons::JUMP | buttons::STANCE_HELD;

static NEXT_GENERATION: AtomicU64 = AtomicU64::new(1);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ActorHandle {
    client: ClientId,
    owner: u32,
    generation: u64,
}

impl ActorHandle {
    pub fn client(self) -> ClientId {
        self.client
    }

    pub fn owner(self) -> u32 {
        self.owner
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SharedAction {
    GatherTree,
    PlaceWoodFoundation,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SharedRequest {
    pub actor: ActorHandle,
    pub request_id: u32,
    pub action: SharedAction,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SharedEffect {
    Gathered(Harvest),
    FoundationPlaced { id: u32 },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SharedReceipt {
    pub client: ClientId,
    pub request_id: u32,
    pub applied_at: Tick,
    pub result: Result<SharedEffect, String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SharedSnapshot {
    pub schema: u32,
    pub tick: Tick,
    pub terrain_seed: u32,
    pub sim: sim::Snapshot,
    pub nodes: Vec<ResourceNode>,
    pub owners: Vec<(ClientId, u32)>,
    pub recipient: ClientId,
    pub inventory: Inventory,
}

impl SharedSnapshot {
    pub fn tree_target(&self, replica: &SimWorld) -> Result<Option<ResourceNode>, String> {
        self.preview_owner(replica)?;
        let gathering = GatheringWorld::from_nodes(self.terrain_seed, self.nodes.clone())?;
        let (start, end, obstacle) = actor_gathering_ray(replica, self.recipient)?;
        Ok(gathering
            .target_from_ray(start, end, obstacle)?
            .filter(|node| node.kind == ResourceKind::Tree)
            .cloned())
    }

    pub fn foundation_preview(
        &self,
        replica: &SimWorld,
    ) -> Result<BuildingPlacementPreview, String> {
        let owner = self.preview_owner(replica)?;
        Ok(actor_building_placement(replica, self.recipient, owner, Kind::Foundation, 0).0)
    }

    fn preview_owner(&self, replica: &SimWorld) -> Result<u32, String> {
        if self.schema != 1 || self.terrain_seed != 731 || self.tick != self.sim.tick {
            return Err("Unsupported shared snapshot for preview".into());
        }
        let mut owners = self
            .owners
            .iter()
            .filter(|(client, _)| *client == self.recipient);
        let owner = owners
            .next()
            .map(|(_, owner)| *owner)
            .ok_or("Preview recipient is not an active owner")?;
        if self.recipient.0 >= OWNER_BASE
            || !(OWNER_BASE..OWNER_BASE + MAX_INVENTORIES as u32).contains(&owner)
            || owners.next().is_some()
        {
            return Err("Invalid preview recipient ownership".into());
        }
        let mut players = self
            .sim
            .players
            .iter()
            .filter(|(client, _)| *client == self.recipient);
        let expected = players
            .next()
            .map(|(_, player)| player)
            .ok_or("Preview recipient has no confirmed player")?;
        if players.next().is_some()
            || replica.player(self.recipient) != Some(expected)
            || replica.match_elapsed_ms() != self.sim.meta.match_elapsed_ms
            || replica.buildings() != &self.sim.meta.world_objects.buildings
        {
            return Err("Preview snapshot and replica do not match".into());
        }
        Ok(owner)
    }
}

struct Actor {
    handle: ActorHandle,
    inventory: Inventory,
    last_angles: [i32; 3],
    initializing: bool,
    highest_request: u32,
    receipts: BTreeMap<u32, (SharedAction, SharedReceipt)>,
}

struct PreparedRequest {
    request: SharedRequest,
    replay: Option<SharedReceipt>,
}

pub struct SharedSession {
    world: SimWorld,
    terrain: Terrain,
    editor: EditorState,
    gathering: GatheringWorld,
    actors: BTreeMap<ClientId, Actor>,
    retiring: BTreeSet<ClientId>,
    admissions: usize,
    tick: u32,
    fault: Option<String>,
}

impl SharedSession {
    pub fn new() -> Result<Self, String> {
        let AuthoredWorld {
            world,
            terrain,
            editor,
            gathering,
        } = new_authored_world()?;
        Ok(Self {
            world,
            terrain,
            editor,
            gathering,
            actors: BTreeMap::new(),
            retiring: BTreeSet::new(),
            admissions: 0,
            tick: 0,
            fault: None,
        })
    }

    pub fn connect(&mut self, client: ClientId) -> Result<ActorHandle, String> {
        self.check_running()?;
        if client.0 >= OWNER_BASE {
            return Err("Invalid client slot".into());
        }
        if self.actors.contains_key(&client)
            || self.retiring.contains(&client)
            || self.world.player(client).is_some()
        {
            return Err("Client slot is already occupied or retiring".into());
        }
        if self.actors.len() >= MAX_ACTORS {
            return Err("The shared world is full".into());
        }
        if self.admissions >= MAX_INVENTORIES {
            return Err("Shared world owner records are full".into());
        }
        let owner = OWNER_BASE
            .checked_add(u32::try_from(self.admissions).map_err(|_| "Owner ID exhausted")?)
            .ok_or("Owner ID exhausted")?;
        let origin = SPAWN_POINTS
            .into_iter()
            .find(|&origin| {
                let mut occupied = player_blocked(&self.world, origin);
                self.world.visit_players(|_, player| {
                    occupied |= player.health > 0 && hulls_overlap(origin, player.origin);
                });
                !occupied
            })
            .ok_or("No free player spawn")?;

        // Stage the ledger before spawning; a refused admission leaks no owner.
        let mut buildings = self.world.buildings().clone();
        buildings
            .grant(owner, Resources::default())
            .map_err(|error| error.to_string())?;
        let generation = NEXT_GENERATION
            .try_update(Ordering::Relaxed, Ordering::Relaxed, |value| {
                value.checked_add(1)
            })
            .map_err(|_| "Actor generation exhausted")?;
        self.world
            .spawn_authored_player(client, origin, [0.; 3], 0)?;
        *self.world.buildings_mut() = buildings;
        // Late joins must stamp command time before their first PMove.
        self.world.set_external_motion(client, true);
        let handle = ActorHandle {
            client,
            owner,
            generation,
        };
        self.actors.insert(
            client,
            Actor {
                handle,
                inventory: Inventory::default(),
                last_angles: self.world.old_cmd_angles(client).unwrap_or([0; 3]),
                initializing: true,
                highest_request: 0,
                receipts: BTreeMap::new(),
            },
        );
        self.admissions += 1;
        Ok(handle)
    }

    pub fn disconnect(&mut self, actor: ActorHandle) -> Result<(), String> {
        self.check_running()?;
        self.actor(actor)?;
        self.actors.remove(&actor.client);
        // Script-owned players retire on a later authority step.
        self.retiring.insert(actor.client);
        self.world.retire_client(actor.client);
        Ok(())
    }

    pub fn step(
        &mut self,
        commands: &[(ActorHandle, UserCmd)],
        requests: &[SharedRequest],
    ) -> Result<Vec<SharedReceipt>, String> {
        self.check_running()?;
        let next_tick = self.tick.checked_add(1).ok_or("Shared clock exhausted")?;
        let server_time = i32::try_from(u64::from(next_tick) * SHARED_STEP_MS as u64)
            .map_err(|_| "Shared clock exhausted")?;
        let mut supplied = BTreeMap::new();
        for &(actor, command) in commands {
            self.actor(actor)?;
            let command = UserCmd {
                angles: command.angles.map(|angle| angle & 0xffff),
                forwardmove: command.forwardmove,
                rightmove: command.rightmove,
                buttons: command.buttons & MOVEMENT_BUTTONS,
                ..Default::default()
            };
            if supplied.insert(actor.client, command).is_some() {
                return Err("Only one command per player is allowed in a tick".into());
            }
        }
        let mut prepared = Vec::with_capacity(requests.len().min(MAX_ACTORS));
        let mut requesting = BTreeSet::new();
        for &request in requests {
            let state = self.actor(request.actor)?;
            if !requesting.insert(request.actor.client) {
                return Err("Only one action per player is allowed in a tick".into());
            }
            if request.request_id == 0 {
                return Err("Request ID must be nonzero".into());
            }
            let replay = if let Some((action, receipt)) = state.receipts.get(&request.request_id) {
                if *action != request.action {
                    return Err("Request ID was reused with a different action".into());
                }
                Some(receipt.clone())
            } else {
                if request.request_id <= state.highest_request {
                    return Err("Request ID is expired or out of order".into());
                }
                None
            };
            prepared.push(PreparedRequest { request, replay });
        }
        prepared.sort_by_key(|entry| (entry.request.actor.client, entry.request.request_id));

        let initializing: BTreeSet<_> = self
            .actors
            .iter()
            .filter_map(|(&client, actor)| actor.initializing.then_some(client))
            .collect();
        let cmds = self
            .actors
            .iter()
            .map(|(&client, actor)| {
                let accepted = (!actor.initializing)
                    .then(|| supplied.get(&client))
                    .flatten();
                let command = UserCmd {
                    server_time,
                    angles: accepted.map_or(actor.last_angles, |command| command.angles),
                    forwardmove: accepted.map_or(0, |command| command.forwardmove),
                    rightmove: accepted.map_or(0, |command| command.rightmove),
                    buttons: accepted.map_or(0, |command| command.buttons & MOVEMENT_BUTTONS),
                    weapon: 0,
                    ..Default::default()
                };
                (client, command)
            })
            .collect();
        if let Err(error) = sim::try_step(
            &mut self.world,
            Tick(next_tick),
            &TickInput::from_cmds(cmds),
            SHARED_STEP_MS,
            sim::StepReason::AuthorityFrame,
        ) {
            // The engine may already have mutated; this authority cannot resume.
            let message = format!("Shared simulation stopped: {error}");
            self.fault = Some(message.clone());
            return Err(message);
        }
        self.tick = next_tick;
        for (&client, actor) in &mut self.actors {
            if actor.initializing {
                if self
                    .world
                    .player(client)
                    .is_some_and(|player| player.command_time == server_time)
                {
                    self.world.set_external_motion(client, false);
                    actor.initializing = false;
                }
            } else if let Some(command) = supplied.get(&client) {
                // Keep canonical packed angles; reconstructed view loses delta angles.
                actor.last_angles = command.angles;
            }
        }
        self.retiring
            .retain(|&client| self.world.player(client).is_some());
        advance_resources(
            &mut self.world,
            &self.terrain,
            &self.editor,
            &mut self.gathering,
            SHARED_STEP_MS as f32 / 1000.,
        );

        let mut receipts = Vec::with_capacity(prepared.len());
        for entry in prepared {
            if let Some(receipt) = entry.replay {
                receipts.push(receipt);
                continue;
            }
            let request = entry.request;
            let result = if initializing.contains(&request.actor.client) {
                Err("Player is initializing".into())
            } else {
                self.apply_action(request.actor, request.action)
            };
            let receipt = SharedReceipt {
                client: request.actor.client,
                request_id: request.request_id,
                applied_at: Tick(self.tick),
                result,
            };
            let actor = self
                .actors
                .get_mut(&request.actor.client)
                .expect("preflight validated the actor");
            actor.highest_request = request.request_id;
            actor
                .receipts
                .insert(request.request_id, (request.action, receipt.clone()));
            while actor.receipts.len() > MAX_RECEIPTS {
                if let Some((&oldest, _)) = actor.receipts.first_key_value() {
                    actor.receipts.remove(&oldest);
                }
            }
            receipts.push(receipt);
        }
        Ok(receipts)
    }

    pub fn snapshot_for(&self, actor: ActorHandle) -> Result<SharedSnapshot, String> {
        self.check_running()?;
        let state = self.actor(actor)?;
        Ok(SharedSnapshot {
            schema: 1,
            tick: Tick(self.tick),
            terrain_seed: self.terrain.seed,
            sim: self.world.snapshot(Tick(self.tick)),
            nodes: self.gathering.nodes().cloned().collect(),
            owners: self
                .actors
                .iter()
                .map(|(&client, actor)| (client, actor.handle.owner))
                .collect(),
            recipient: actor.client,
            inventory: state.inventory.clone(),
        })
    }

    pub fn world(&self) -> &SimWorld {
        &self.world
    }

    fn check_running(&self) -> Result<(), String> {
        self.fault
            .as_ref()
            .map_or(Ok(()), |error| Err(error.clone()))
    }

    fn actor(&self, handle: ActorHandle) -> Result<&Actor, String> {
        self.actors
            .get(&handle.client)
            .filter(|actor| actor.handle == handle)
            .ok_or_else(|| "Actor handle is stale or belongs to another world".into())
    }

    fn apply_action(
        &mut self,
        actor: ActorHandle,
        action: SharedAction,
    ) -> Result<SharedEffect, String> {
        if !self
            .world
            .player(actor.client)
            .is_some_and(|player| player.health > 0)
        {
            return Err("Player is not alive".into());
        }
        match action {
            SharedAction::GatherTree => self.gather_tree(actor).map(SharedEffect::Gathered),
            SharedAction::PlaceWoodFoundation => {
                let (preview, candidate) = actor_building_placement(
                    &self.world,
                    actor.client,
                    actor.owner,
                    Kind::Foundation,
                    0,
                );
                let (socket, grounded) = candidate.ok_or_else(|| {
                    preview
                        .error
                        .unwrap_or_else(|| "No building placement target".into())
                })?;
                let id = self
                    .world
                    .buildings_mut()
                    .place(actor.owner, Kind::Foundation, socket, grounded)
                    .map_err(|error| error.to_string())?;
                Ok(SharedEffect::FoundationPlaced { id })
            }
        }
    }

    fn gather_tree(&mut self, actor: ActorHandle) -> Result<Harvest, String> {
        let (start, end, obstacle) = actor_gathering_ray(&self.world, actor.client)?;
        let target = self
            .gathering
            .target_from_ray(start, end, obstacle)?
            .ok_or("Aim at a tree within reach")?;
        if target.kind != ResourceKind::Tree {
            return Err("Only tree gathering is available in the shared world".into());
        }
        let mut gathering = self.gathering.clone();
        let harvest = gathering.harvest_from_ray(start, end, obstacle, |_| false)?;
        let balance = self.world.buildings().inventory(actor.owner);
        let resources = harvest.resources();
        if [
            (balance.wood, resources.wood),
            (balance.stone, resources.stone),
            (balance.metal, resources.metal),
        ]
        .into_iter()
        .any(|(have, add)| {
            have.checked_add(add)
                .is_none_or(|total| total > persistence::MAX_RESOURCE_BALANCE)
        }) {
            return Err("Resource storage is full".into());
        }
        self.world
            .buildings_mut()
            .grant(actor.owner, resources)
            .map_err(|error| error.to_string())?;
        if harvest.remaining == 0 {
            self.world
                .install_content(authored_content(&self.terrain, &self.editor, &gathering));
        }
        self.gathering = gathering;
        Ok(harvest)
    }
}
