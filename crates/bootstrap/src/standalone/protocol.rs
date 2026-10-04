use net::{
    ClientPacket, ConnectionId, Frame, HandshakeHello, PacketHeader, ProtocolLimits, ServerPacket,
    SnapshotDecoder, SnapshotEncoder, WireReader, WireWriter, WorldObjectSyncDecoder,
};
use playerstate_iw4::PlayerState;
use sim::{ClientId, Snapshot, Tick, TickInput};
use std::collections::BTreeSet;
use survival::{
    GatheringWorld, Harvest, Inventory, Item, ResourceKind, ResourceNode, SharedAction,
    SharedEffect, SharedReceipt, SharedSnapshot, Stack, Terrain,
};

pub const MAX_BODY_BYTES: usize = 256 * 1024;
const MAGIC: u32 = 0x3150_4D53;
const VERSION: u16 = 2;
const TERRAIN_SEED: u32 = 731;
const MAX_CLIENTS: usize = 64;
const MAX_OWNERS: usize = 2;
const MAX_NODES: usize = 64;
const MAX_STACKS: usize = 24;
const MAX_RECEIPTS: usize = 256;
const MAX_STRING: usize = 1024;
const OWNER_BASE: u32 = 64;
const OWNER_LIMIT: u32 = OWNER_BASE + rust_building::MAX_INVENTORIES as u32;
const CONNECT: u8 = 1;
const READY: u8 = 2;
const INPUT: u8 = 3;
const ACCEPT: u8 = 11;
const STATE: u8 = 12;
const REJECT: u8 = 13;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ActionRequest {
    pub request_id: u32,
    pub action: SharedAction,
}

#[derive(Clone, Debug)]
pub enum ClientMessage {
    Connect(HandshakeHello),
    Ready {
        header: PacketHeader,
        snapshot_seq: u32,
    },
    Input {
        packet: ClientPacket,
        action: Option<ActionRequest>,
    },
}

#[derive(Clone, Debug)]
pub enum ServerMessage {
    Accept {
        connection: ConnectionId,
        assigned_client: ClientId,
        hello: HandshakeHello,
        header: PacketHeader,
        snapshot_seq: u32,
        state: SharedSnapshot,
    },
    State {
        header: PacketHeader,
        snapshot_seq: u32,
        state: SharedSnapshot,
        receipts: Vec<SharedReceipt>,
    },
    Reject(String),
}

pub fn hello() -> Result<HandshakeHello, String> {
    let terrain = Terrain::new(TERRAIN_SEED);
    let gathering = GatheringWorld::new(&terrain)?;
    let replica = survival::shared_replica(gathering.nodes().cloned().collect())?;
    Ok(HandshakeHello::from_world(&replica))
}

pub fn evaluate(ours: &HandshakeHello, theirs: &HandshakeHello) -> Result<(), String> {
    validate_hello(ours)?;
    validate_hello(theirs)?;
    net::evaluate_handshake(ours, theirs).map_err(|error| error.to_string())?;
    if ours.content != theirs.content {
        return Err("Original-world content fingerprint mismatch".into());
    }
    Ok(())
}

fn validate_hello(value: &HandshakeHello) -> Result<(), String> {
    if value.protocol_version != net::PROTOCOL_VERSION
        || value.limits != ProtocolLimits::default_listen()
        || value.content.map == 0
    {
        return Err("Unsupported standalone handshake profile".into());
    }
    Ok(())
}

pub fn encode_client(message: &ClientMessage) -> Result<Vec<u8>, String> {
    let (tag, packet, action) = match message {
        ClientMessage::Connect(value) => {
            validate_hello(value)?;
            (CONNECT, ClientPacket::Connect(*value), None)
        }
        ClientMessage::Ready {
            header,
            snapshot_seq,
        } => {
            validate_header(*header, *snapshot_seq)?;
            (
                READY,
                ClientPacket::SnapshotAck {
                    header: *header,
                    snapshot_seq: *snapshot_seq,
                },
                None,
            )
        }
        ClientMessage::Input { packet, action } => {
            validate_input(packet, *action)?;
            (INPUT, packet.clone(), *action)
        }
    };
    let mut out = envelope(tag);
    put_blob(&mut out, &packet.to_bytes())?;
    if tag == INPUT {
        put_action(&mut out, action);
    }
    finish(out)
}

pub fn decode_client(bytes: &[u8]) -> Result<ClientMessage, String> {
    let mut input = Reader::new(bytes)?;
    let tag = input.tag()?;
    if !matches!(tag, CONNECT | READY | INPUT) {
        return Err("Unknown standalone client message".into());
    }
    let packet = net::decode_client_packet(
        &input.blob(MAX_BODY_BYTES)?,
        &ProtocolLimits::default_listen(),
    )
    .map_err(|error| error.to_string())?;
    let message = match (tag, packet) {
        (CONNECT, ClientPacket::Connect(value)) => {
            validate_hello(&value)?;
            ClientMessage::Connect(value)
        }
        (
            READY,
            ClientPacket::SnapshotAck {
                header,
                snapshot_seq,
            },
        ) => {
            validate_header(header, snapshot_seq)?;
            ClientMessage::Ready {
                header,
                snapshot_seq,
            }
        }
        (INPUT, packet @ ClientPacket::Commands { .. }) => {
            let action = read_action(&mut input)?;
            validate_input(&packet, action)?;
            ClientMessage::Input { packet, action }
        }
        _ => return Err("Standalone message contains the wrong inherited packet".into()),
    };
    input.end()?;
    Ok(message)
}

fn validate_input(packet: &ClientPacket, action: Option<ActionRequest>) -> Result<(), String> {
    let ClientPacket::Commands {
        header,
        cmds,
        samples,
        actions,
        reliable_ack,
        ..
    } = packet
    else {
        return Err("Standalone input requires Commands".into());
    };
    validate_header(*header, 1)?;
    if cmds.len() != 1
        || cmds[0].0.0 == 0
        || !samples.is_empty()
        || !actions.is_empty()
        || *reliable_ack != 0
    {
        return Err("Standalone input requires one sequenced command and no legacy actions".into());
    }
    let cmd = &cmds[0].1;
    if cmd
        .gun_angle_offset
        .iter()
        .chain(std::iter::once(&cmd.melee_charge_yaw))
        .any(|v| !v.is_finite())
    {
        return Err("Command contains a nonfinite value".into());
    }
    if action.is_some_and(|value| value.request_id == 0) {
        return Err("Action request ID must be nonzero".into());
    }
    Ok(())
}

pub fn encode_server(message: &ServerMessage) -> Result<Vec<u8>, String> {
    let mut out = match message {
        ServerMessage::Accept { .. } => envelope(ACCEPT),
        ServerMessage::State { .. } => envelope(STATE),
        ServerMessage::Reject(_) => envelope(REJECT),
    };
    match message {
        ServerMessage::Accept {
            connection,
            assigned_client,
            hello,
            header,
            snapshot_seq,
            state,
        } => {
            validate_hello(hello)?;
            if *connection != header.connection || *assigned_client != state.recipient {
                return Err("Accept identity does not match its snapshot".into());
            }
            let accept = ServerPacket::Accept {
                connection: *connection,
                assigned_client: assigned_client.0,
                hello: *hello,
            };
            put_blob(&mut out, &accept.to_bytes())?;
            put_snapshot(&mut out, *header, *snapshot_seq, state)?;
            put_state(&mut out, state)?;
        }
        ServerMessage::State {
            header,
            snapshot_seq,
            state,
            receipts,
        } => {
            validate_receipts(state, receipts)?;
            put_snapshot(&mut out, *header, *snapshot_seq, state)?;
            put_state(&mut out, state)?;
            put_receipts(&mut out, receipts)?;
        }
        ServerMessage::Reject(reason) => put_string(&mut out, reason)?,
    }
    finish(out)
}

pub fn decode_server(bytes: &[u8]) -> Result<ServerMessage, String> {
    let mut input = Reader::new(bytes)?;
    let message = match input.tag()? {
        ACCEPT => {
            let packet = net::decode_server_packet(
                &input.blob(MAX_BODY_BYTES)?,
                &ProtocolLimits::default_listen(),
            )
            .map_err(|error| error.to_string())?;
            let ServerPacket::Accept {
                connection,
                assigned_client,
                hello,
            } = packet
            else {
                return Err("Standalone Accept requires the inherited Accept packet".into());
            };
            validate_hello(&hello)?;
            let (header, snapshot_seq, sim) = read_snapshot(&mut input)?;
            let state = read_state(&mut input, sim)?;
            if connection != header.connection || assigned_client != state.recipient.0 {
                return Err("Accept identity does not match its snapshot".into());
            }
            ServerMessage::Accept {
                connection,
                assigned_client: ClientId(assigned_client),
                hello,
                header,
                snapshot_seq,
                state,
            }
        }
        STATE => {
            let (header, snapshot_seq, sim) = read_snapshot(&mut input)?;
            let state = read_state(&mut input, sim)?;
            let receipts = read_receipts(&mut input)?;
            validate_receipts(&state, &receipts)?;
            ServerMessage::State {
                header,
                snapshot_seq,
                state,
                receipts,
            }
        }
        REJECT => ServerMessage::Reject(input.string()?),
        _ => return Err("Unknown standalone server message".into()),
    };
    input.end()?;
    Ok(message)
}

fn validate_header(header: PacketHeader, snapshot_seq: u32) -> Result<(), String> {
    if header.connection.0 == 0 || header.sequence == 0 || header.epoch == 0 || snapshot_seq == 0 {
        return Err("Standalone connection, epoch and sequence numbers must be nonzero".into());
    }
    Ok(())
}

fn put_snapshot(
    out: &mut WireWriter,
    header: PacketHeader,
    sequence: u32,
    state: &SharedSnapshot,
) -> Result<(), String> {
    validate_header(header, sequence)?;
    validate_state(state)?;
    let mut encoder = SnapshotEncoder::new();
    encoder.reset();
    let frame =
        net::frame_from_acked_tick(&mut encoder, &TickInput::default(), &state.sim, Vec::new());
    if frame.world_objects_wire.len() > u16::MAX as usize {
        return Err("Full world-object snapshot exceeds the inherited wire limit".into());
    }
    let payload = frame.to_bytes();
    if payload.len() > MAX_BODY_BYTES {
        return Err("Full snapshot exceeds the standalone byte limit".into());
    }
    put_blob(
        out,
        &ServerPacket::Snapshot {
            header,
            baseline_seq: 0,
            snapshot_seq: sequence,
            payload,
        }
        .to_bytes(),
    )
}

fn read_snapshot(input: &mut Reader<'_>) -> Result<(PacketHeader, u32, Snapshot), String> {
    let packet = net::decode_server_packet(
        &input.blob(MAX_BODY_BYTES)?,
        &ProtocolLimits::default_listen(),
    )
    .map_err(|error| error.to_string())?;
    let ServerPacket::Snapshot {
        header,
        baseline_seq,
        snapshot_seq,
        payload,
    } = packet
    else {
        return Err("Standalone state requires the inherited Snapshot packet".into());
    };
    validate_header(header, snapshot_seq)?;
    if baseline_seq != 0 {
        return Err("Standalone snapshots require a full baseline".into());
    }
    let mut frame_input = WireReader::new(&payload);
    let frame = Frame::decode(&mut frame_input, &mut WorldObjectSyncDecoder::default())
        .map_err(|error| error.to_string())?;
    if frame.world_objects_wire.get(12) != Some(&6) {
        return Err("Standalone world objects require the full protocol-95 baseline tag".into());
    }
    if !frame_input.is_empty()
        || !frame.cmds.is_empty()
        || !frame.actions.is_empty()
        || !frame.acks.is_empty()
        || frame.reliable != net::ReliablePayload::owes_nothing()
        || !frame.svc_sounds.is_empty()
        || frame.svc_scores.is_some()
        || !frame.svc_card_slots.is_empty()
        || !frame.svc_open_menus.is_empty()
        || !frame.svc_hud_splashes.is_empty()
        || !frame.svc_game_notifies.is_empty()
    {
        return Err("Unsupported command or service data in standalone snapshot".into());
    }
    let mut snapshot = SnapshotDecoder::new()
        .decode(&frame.snapshot_delta)
        .map_err(|error| error.to_string())?;
    if frame.tick != snapshot.tick || frame.state_hash != net::compute_state_hash(&snapshot.players)
    {
        return Err("Standalone frame tick or player hash is inconsistent".into());
    }
    snapshot.meta = frame.snapshot_meta;
    Ok((header, snapshot_seq, snapshot))
}

fn put_state(out: &mut WireWriter, state: &SharedSnapshot) -> Result<(), String> {
    out.put_u32(state.schema);
    out.put_u32(state.tick.0);
    out.put_u32(state.terrain_seed);
    out.put_u32(state.recipient.0);
    out.put_u16(state.owners.len() as u16);
    for (client, owner) in &state.owners {
        out.put_u32(client.0);
        out.put_u32(*owner);
    }
    out.put_u16(state.nodes.len() as u16);
    for node in &state.nodes {
        out.put_u32(node.id);
        out.put_u8(kind_tag(node.kind));
        for value in node.position {
            out.put_f32(value);
        }
        out.put_u32(node.remaining);
        out.put_f32(node.regrow_in);
    }
    out.put_u16(state.inventory.stacks().len() as u16);
    for stack in state.inventory.stacks() {
        let tag = Item::ALL
            .iter()
            .position(|item| *item == stack.item)
            .ok_or("Unsupported item")?;
        out.put_u8(tag as u8 + 1);
        out.put_u32(stack.quantity);
        out.put_u32(stack.wear);
    }
    Ok(())
}

fn read_state(input: &mut Reader<'_>, sim: Snapshot) -> Result<SharedSnapshot, String> {
    let schema = input.u32()?;
    let tick = Tick(input.u32()?);
    let terrain_seed = input.u32()?;
    let recipient = ClientId(input.u32()?);
    let count = input.count(MAX_OWNERS)?;
    let mut owners = Vec::with_capacity(count);
    for _ in 0..count {
        owners.push((ClientId(input.u32()?), input.u32()?));
    }
    let count = input.count(MAX_NODES)?;
    let mut nodes = Vec::with_capacity(count);
    for _ in 0..count {
        nodes.push(ResourceNode {
            id: input.u32()?,
            kind: read_kind(input.u8()?)?,
            position: [input.f32()?, input.f32()?, input.f32()?],
            remaining: input.u32()?,
            regrow_in: input.f32()?,
        });
    }
    let count = input.count(MAX_STACKS)?;
    let mut stacks = Vec::with_capacity(count);
    for _ in 0..count {
        let tag = input.u8()?;
        let item = tag
            .checked_sub(1)
            .and_then(|index| Item::ALL.get(usize::from(index)))
            .copied()
            .ok_or("Unknown standalone item tag")?;
        stacks.push(Stack {
            item,
            quantity: input.u32()?,
            wear: input.u32()?,
        });
    }
    let state = SharedSnapshot {
        schema,
        tick,
        terrain_seed,
        sim,
        nodes,
        owners,
        recipient,
        inventory: Inventory::from_stacks(stacks)?,
    };
    validate_state(&state)?;
    Ok(state)
}

fn validate_state(state: &SharedSnapshot) -> Result<(), String> {
    if state.schema != 1
        || state.terrain_seed != TERRAIN_SEED
        || state.tick != state.sim.tick
        || state.tick.0 > (i32::MAX / survival::SHARED_STEP_MS) as u32
        || state.nodes.len() > MAX_NODES
        || state.inventory.stacks().len() > MAX_STACKS
        || state.owners.is_empty()
        || state.owners.len() > MAX_OWNERS
        || state.recipient.0 >= MAX_CLIENTS as u32
    {
        return Err("Unsupported standalone snapshot schema, clock or counts".into());
    }
    let gathering = GatheringWorld::from_nodes(TERRAIN_SEED, state.nodes.clone())?;
    if gathering.nodes().cloned().collect::<Vec<_>>() != state.nodes {
        return Err("Resource nodes must have canonical ordering and regrowth state".into());
    }
    Inventory::from_stacks(state.inventory.stacks().to_vec())?;
    let mut clients = BTreeSet::new();
    let mut owner_ids = BTreeSet::new();
    for &(client, owner) in &state.owners {
        if client.0 >= MAX_CLIENTS as u32
            || !(OWNER_BASE..OWNER_LIMIT).contains(&owner)
            || !clients.insert(client)
            || !owner_ids.insert(owner)
        {
            return Err("Invalid or duplicate standalone owner identity".into());
        }
    }
    if !clients.contains(&state.recipient) {
        return Err("Snapshot recipient is not an active owner".into());
    }
    validate_sim(&state.sim, &clients)?;
    validate_buildings(&state.sim.meta.world_objects.buildings, &state.owners)?;
    Ok(())
}

fn validate_buildings(
    buildings: &rust_building::BuildingWorld,
    owners: &[(ClientId, u32)],
) -> Result<(), String> {
    let bytes = buildings.to_json().map_err(|error| error.to_string())?;
    if bytes.len() > MAX_BODY_BYTES {
        return Err("Building state exceeds the standalone byte limit".into());
    }
    rust_building::BuildingWorld::from_json(&bytes).map_err(|error| error.to_string())?;
    let value: serde_json::Value =
        serde_json::from_slice(&bytes).map_err(|error| error.to_string())?;
    let ledgers = value
        .get("inventories")
        .and_then(serde_json::Value::as_object)
        .ok_or("Missing public building ledgers")?;
    if ledgers.len() > rust_building::MAX_INVENTORIES {
        return Err("Too many public building ledgers".into());
    }
    if owners
        .iter()
        .any(|(_, owner)| !ledgers.contains_key(&owner.to_string()))
        || buildings
            .pieces()
            .any(|piece| !ledgers.contains_key(&piece.owner.to_string()))
    {
        return Err("Building owner is missing its public resource ledger".into());
    }
    for (owner, resources) in ledgers {
        let owner: u32 = owner.parse().map_err(|_| "Invalid building ledger owner")?;
        if !(OWNER_BASE..OWNER_LIMIT).contains(&owner) {
            return Err("Unsupported building ledger owner".into());
        }
        for name in ["wood", "stone", "metal"] {
            if resources
                .get(name)
                .and_then(serde_json::Value::as_u64)
                .is_none_or(|amount| amount > 1_000_000)
            {
                return Err("Public resource ledger exceeds the authority balance limit".into());
            }
        }
    }
    Ok(())
}

fn validate_sim(snapshot: &Snapshot, owners: &BTreeSet<ClientId>) -> Result<(), String> {
    let meta = &snapshot.meta;
    if !snapshot.projectiles.is_empty()
        || !meta.entities.is_empty()
        || !meta.script_movers.is_empty()
        || !meta.item_ammo.is_empty()
        || !meta.item_pickups.is_empty()
        || !meta.entity_dobjs.is_empty()
        || !meta.pellet_fx.is_empty()
        || meta.corpses.slots.iter().any(|slot| slot.occupied)
        || !meta.world_objects.glass_pieces.is_empty()
        || !meta.world_objects.destructible_loop_sounds.is_empty()
    {
        return Err("Unsupported dynamic entities in original-world snapshot".into());
    }
    if usize::from(meta.corpses.spawn_ring) >= meta.corpses.slots.len()
        || meta.corpses.slots.iter().any(|slot| {
            slot.origin
                .iter()
                .chain(slot.viewangles.iter())
                .chain(slot.tr_delta.iter())
                .chain(slot.tr_base.iter())
                .chain(std::iter::once(&slot.view_height_current))
                .any(|v| !v.is_finite())
        })
    {
        return Err("Invalid empty corpse pool".into());
    }
    meta.entity_kernel
        .validate()
        .map_err(|error| format!("Invalid entity kernel: {error:?}"))?;
    if let Some(area) = &meta.area_entities {
        area.validate()
            .map_err(|error| format!("Invalid area entity state: {error:?}"))?;
    }
    for slot in &meta.entity_kernel.slots {
        if let Some(occupied) = &slot.occupied {
            if occupied.kind != sim::EntityRunKind::TempEvent {
                return Err("Unsupported dynamic entity kernel occupant".into());
            }
            if occupied
                .relations
                .parent_link_axis
                .iter()
                .flatten()
                .chain(occupied.relations.parent_link_origin.iter())
                .any(|v| !v.is_finite())
            {
                return Err("Nonfinite entity relation transform".into());
            }
        }
    }
    if snapshot.players.len() > MAX_CLIENTS
        || meta.clients.len() > MAX_CLIENTS
        || meta.rng.root_seed != 1
        || meta.rng.scheme != 1
    {
        return Err("Unsupported player counts or replica seed".into());
    }
    let mut players = BTreeSet::new();
    for (client, player) in &snapshot.players {
        if client.0 >= MAX_CLIENTS as u32 || !players.insert(*client) {
            return Err("Invalid or duplicate public player".into());
        }
        validate_player(player)?;
    }
    let mut clients = BTreeSet::new();
    for (client, row) in &meta.clients {
        if client.0 >= MAX_CLIENTS as u32 || !clients.insert(*client) {
            return Err("Invalid or duplicate player metadata".into());
        }
        if row.shield.is_some()
            || row.shield_collision.is_some()
            || row.loadout.is_some()
            || row.weapon_lock != sim::WeaponLock::default()
            || row.killcam_hud.is_some()
            || row.remote_missile.is_some()
            || row.shellshock.is_some()
            || row.location_selection.is_some()
            || row.view_effects != sim::ViewEffects::default()
            || !row.hud_archival.is_empty()
            || !row.hud_current.is_empty()
            || !row.menu_commands.is_empty()
            || row.ammo_by_weapon.len() > 64
            || row.taped_mag_spent.len() > 64
        {
            return Err("Unsupported scripted or weapon player metadata".into());
        }
        validate_pairs(&row.client_dvars)?;
    }
    if players != clients || !owners.is_subset(&players) {
        return Err("Player, metadata and active-owner identities do not agree".into());
    }
    let objectives = &meta.objectives;
    if !objectives.compass.is_empty()
        || !objectives.vehicles.is_empty()
        || objectives.slow_motion.is_some()
        || objectives.ambient.as_ref().is_some_and(|ambient| {
            ambient.alias.is_some() || ambient.start_ms != 0 || ambient.end_ms != 0
        })
        || objectives.ac130_ambient.as_ref().is_some_and(|ambient| {
            ambient.alias.is_some() || ambient.start_ms != 0 || ambient.end_ms != 0
        })
        || !objectives.effects.is_empty()
        || objectives.fog.is_some()
        || !objectives.earthquakes.is_empty()
        || objectives.naked_vision.is_some()
        || objectives.thermal_vision.is_some()
        || objectives.missile_vision.is_some()
        || objectives.night_vision.is_some()
        || objectives.pain_vision.is_some()
        || !objectives.rumble_aliases.is_empty()
    {
        return Err("Unsupported scripted world presentation".into());
    }
    validate_pairs(&objectives.server_info)?;
    for rows in [&meta.sound_aliases, &meta.effect_names, &meta.hud_materials] {
        if rows.len() > u8::MAX as usize {
            return Err("Too many presentation strings".into());
        }
        let mut indices = BTreeSet::new();
        for (index, value) in rows {
            if *index == 0 || !indices.insert(*index) || value.len() > u8::MAX as usize {
                return Err("Invalid presentation string".into());
            }
        }
    }
    if meta.hud_strings.len() >= sim::CS_LOCALIZED_STRINGS_SLOTS {
        return Err("Too many HUD strings".into());
    }
    let mut indices = BTreeSet::new();
    for (index, value) in &meta.hud_strings {
        if *index == 0
            || usize::from(*index) >= sim::CS_LOCALIZED_STRINGS_SLOTS
            || !indices.insert(*index)
            || value.len() > MAX_STRING
        {
            return Err("Invalid HUD string".into());
        }
    }
    if meta.journal.len() > MAX_RECEIPTS || meta.entity_events.len() > MAX_RECEIPTS {
        return Err("Too many public events".into());
    }
    for event in &meta.entity_events {
        validate_audience(&event.audience)?;
        if event.tick > snapshot.tick
            || event
                .payload
                .origin
                .iter()
                .chain(event.payload.origin2.iter())
                .chain(event.payload.direction.iter())
                .any(|v| !v.is_finite())
        {
            return Err("Invalid public entity event".into());
        }
    }
    for event in &meta.journal {
        validate_audience(&event.audience)?;
        if event.tick > snapshot.tick {
            return Err("Public event is ahead of the snapshot".into());
        }
    }
    Ok(())
}

fn validate_audience(audience: &sim::EventAudience) -> Result<(), String> {
    match audience {
        sim::EventAudience::All => Ok(()),
        sim::EventAudience::AllExcept(client) | sim::EventAudience::Client(client) => {
            if client.0 >= MAX_CLIENTS as u32 {
                return Err("Invalid event audience client".into());
            }
            Ok(())
        }
        sim::EventAudience::Clients(clients) => {
            let unique: BTreeSet<_> = clients.iter().copied().collect();
            if clients.len() > MAX_CLIENTS
                || unique.len() != clients.len()
                || clients.iter().any(|client| client.0 >= MAX_CLIENTS as u32)
            {
                return Err("Invalid event audience collection".into());
            }
            Ok(())
        }
    }
}

fn validate_player(player: &PlayerState) -> Result<(), String> {
    if player.weapon != 0
        || player.weapon_primary != 0
        || player.off_hand_index != 0
        || player.offhand_primary != 0
        || player.offhand_secondary != 0
    {
        return Err("Standalone shared-world players must be unarmed".into());
    }
    let arrays = [
        player.origin,
        player.velocity,
        player.delta_angles,
        player.v_ladder_vec,
        player.viewangles,
        player.link_weapon_angles,
    ];
    let scalars = [
        player.leanf,
        player.jump_origin_z,
        player.view_height_current,
        player.move_speed_scale_multiplier,
        player.mantle_yaw,
        player.f_weapon_pos_frac,
        player.hold_breath_scale,
        player.aim_spread_scale,
        player.melee_charge_yaw,
    ];
    if arrays
        .iter()
        .flatten()
        .chain(scalars.iter())
        .any(|value| !value.is_finite() || value.abs() > 1_000_000.)
    {
        return Err("Invalid public player transform or movement value".into());
    }
    Ok(())
}

fn validate_pairs(values: &[(String, String)]) -> Result<(), String> {
    if values.len() > 64
        || values
            .iter()
            .any(|(key, value)| key.len() > MAX_STRING || value.len() > MAX_STRING)
    {
        return Err("Too many or oversized configuration strings".into());
    }
    Ok(())
}

fn put_action(out: &mut WireWriter, action: Option<ActionRequest>) {
    match action {
        None => out.put_u8(0),
        Some(value) => {
            out.put_u8(match value.action {
                SharedAction::GatherTree => 1,
                SharedAction::PlaceWoodFoundation => 2,
                SharedAction::GatherCloth => 3,
                SharedAction::CraftBandage => 4,
            });
            out.put_u32(value.request_id);
        }
    }
}

fn read_action(input: &mut Reader<'_>) -> Result<Option<ActionRequest>, String> {
    let action = match input.u8()? {
        0 => return Ok(None),
        1 => SharedAction::GatherTree,
        2 => SharedAction::PlaceWoodFoundation,
        3 => SharedAction::GatherCloth,
        4 => SharedAction::CraftBandage,
        _ => return Err("Unknown standalone action tag".into()),
    };
    let request_id = input.u32()?;
    if request_id == 0 {
        return Err("Action request ID must be nonzero".into());
    }
    Ok(Some(ActionRequest { request_id, action }))
}

fn validate_receipts(state: &SharedSnapshot, receipts: &[SharedReceipt]) -> Result<(), String> {
    if receipts.len() > MAX_RECEIPTS {
        return Err("Too many action receipts".into());
    }
    for receipt in receipts {
        if receipt.client != state.recipient
            || receipt.request_id == 0
            || receipt.applied_at > state.tick
        {
            return Err("Action receipt has an invalid recipient, ID or tick".into());
        }
        match &receipt.result {
            Ok(SharedEffect::Gathered(harvest)) => {
                if !matches!(harvest.kind, ResourceKind::Tree | ResourceKind::Hemp)
                    || harvest.amount == 0
                    || harvest.amount > harvest.kind.harvest_amount()
                    || harvest.remaining > harvest.kind.capacity()
                    || harvest.tool_broke
                    || harvest.tool_almost_broken
                    || !state
                        .nodes
                        .iter()
                        .any(|node| node.id == harvest.node_id && node.kind == harvest.kind)
                {
                    return Err("Invalid shared harvest receipt".into());
                }
            }
            Ok(SharedEffect::FoundationPlaced { id }) => {
                let owner = state
                    .owners
                    .iter()
                    .find(|(client, _)| *client == state.recipient)
                    .map(|(_, owner)| *owner);
                if *id == 0
                    || !state
                        .sim
                        .meta
                        .world_objects
                        .buildings
                        .pieces()
                        .any(|piece| {
                            piece.id == *id
                                && Some(piece.owner) == owner
                                && piece.kind == rust_building::Kind::Foundation
                        })
                {
                    return Err("Invalid foundation receipt".into());
                }
            }
            Ok(SharedEffect::BandageCrafted) => {}
            Err(error) if error.len() > MAX_STRING => {
                return Err("Action refusal exceeds the string limit".into());
            }
            Err(_) => {}
        }
    }
    Ok(())
}

fn put_receipts(out: &mut WireWriter, receipts: &[SharedReceipt]) -> Result<(), String> {
    out.put_u16(receipts.len() as u16);
    for receipt in receipts {
        out.put_u32(receipt.client.0);
        out.put_u32(receipt.request_id);
        out.put_u32(receipt.applied_at.0);
        match &receipt.result {
            Ok(SharedEffect::Gathered(harvest)) => {
                out.put_u8(1);
                out.put_u32(harvest.node_id);
                out.put_u8(kind_tag(harvest.kind));
                out.put_u32(harvest.amount);
                out.put_u32(harvest.remaining);
                out.put_u8(u8::from(harvest.tool_broke));
                out.put_u8(u8::from(harvest.tool_almost_broken));
            }
            Ok(SharedEffect::FoundationPlaced { id }) => {
                out.put_u8(2);
                out.put_u32(*id);
            }
            Ok(SharedEffect::BandageCrafted) => out.put_u8(4),
            Err(error) => {
                out.put_u8(3);
                put_string(out, error)?;
            }
        }
    }
    Ok(())
}

fn read_receipts(input: &mut Reader<'_>) -> Result<Vec<SharedReceipt>, String> {
    let count = input.count(MAX_RECEIPTS)?;
    let mut receipts = Vec::with_capacity(count);
    for _ in 0..count {
        let client = ClientId(input.u32()?);
        let request_id = input.u32()?;
        let applied_at = Tick(input.u32()?);
        let result = match input.u8()? {
            1 => Ok(SharedEffect::Gathered(Harvest {
                node_id: input.u32()?,
                kind: read_kind(input.u8()?)?,
                amount: input.u32()?,
                remaining: input.u32()?,
                tool_broke: input.boolean()?,
                tool_almost_broken: input.boolean()?,
            })),
            2 => Ok(SharedEffect::FoundationPlaced { id: input.u32()? }),
            3 => Err(input.string()?),
            4 => Ok(SharedEffect::BandageCrafted),
            _ => return Err("Unknown standalone receipt result".into()),
        };
        receipts.push(SharedReceipt {
            client,
            request_id,
            applied_at,
            result,
        });
    }
    Ok(receipts)
}

fn kind_tag(kind: ResourceKind) -> u8 {
    match kind {
        ResourceKind::Tree => 1,
        ResourceKind::Stone => 2,
        ResourceKind::Metal => 3,
        ResourceKind::Berry => 4,
        ResourceKind::Water => 5,
        ResourceKind::Hemp => 6,
    }
}

fn read_kind(tag: u8) -> Result<ResourceKind, String> {
    match tag {
        1 => Ok(ResourceKind::Tree),
        2 => Ok(ResourceKind::Stone),
        3 => Ok(ResourceKind::Metal),
        4 => Ok(ResourceKind::Berry),
        5 => Ok(ResourceKind::Water),
        6 => Ok(ResourceKind::Hemp),
        _ => Err("Unknown resource kind".into()),
    }
}

fn envelope(tag: u8) -> WireWriter {
    let mut out = WireWriter::new();
    out.put_u32(MAGIC);
    out.put_u16(VERSION);
    out.put_u8(tag);
    out
}

fn put_blob(out: &mut WireWriter, bytes: &[u8]) -> Result<(), String> {
    if bytes.is_empty()
        || bytes.len() > MAX_BODY_BYTES
        || out.len() + 4 + bytes.len() > MAX_BODY_BYTES
    {
        return Err("Standalone message exceeds the byte limit".into());
    }
    out.put_u32(bytes.len() as u32);
    out.put_bytes(bytes);
    Ok(())
}

fn put_string(out: &mut WireWriter, value: &str) -> Result<(), String> {
    if value.len() > MAX_STRING {
        return Err("Standalone string exceeds the byte limit".into());
    }
    out.put_u16(value.len() as u16);
    out.put_bytes(value.as_bytes());
    Ok(())
}

fn finish(out: WireWriter) -> Result<Vec<u8>, String> {
    if out.len() > MAX_BODY_BYTES {
        return Err("Standalone message exceeds the byte limit".into());
    }
    Ok(out.finish())
}

struct Reader<'a> {
    wire: WireReader<'a>,
}

impl<'a> Reader<'a> {
    fn new(bytes: &'a [u8]) -> Result<Self, String> {
        if bytes.is_empty() || bytes.len() > MAX_BODY_BYTES {
            return Err("Invalid standalone body length".into());
        }
        let mut value = Self {
            wire: WireReader::new(bytes),
        };
        if value.u32()? != MAGIC || value.u16()? != VERSION {
            return Err("Unsupported standalone magic or schema version".into());
        }
        Ok(value)
    }
    fn tag(&mut self) -> Result<u8, String> {
        self.u8()
    }
    fn u8(&mut self) -> Result<u8, String> {
        self.wire.get_u8().map_err(|error| error.to_string())
    }
    fn u16(&mut self) -> Result<u16, String> {
        self.wire.get_u16().map_err(|error| error.to_string())
    }
    fn u32(&mut self) -> Result<u32, String> {
        self.wire.get_u32().map_err(|error| error.to_string())
    }
    fn f32(&mut self) -> Result<f32, String> {
        self.wire.get_f32().map_err(|error| error.to_string())
    }
    fn count(&mut self, max: usize) -> Result<usize, String> {
        let count = usize::from(self.u16()?);
        if count > max {
            return Err("Standalone collection count exceeds its limit".into());
        }
        Ok(count)
    }
    fn bytes(&mut self, count: usize) -> Result<Vec<u8>, String> {
        if count > self.wire.remaining() {
            return Err("Truncated standalone field".into());
        }
        let mut bytes = vec![0; count];
        self.wire
            .get_bytes(&mut bytes)
            .map_err(|error| error.to_string())?;
        Ok(bytes)
    }
    fn blob(&mut self, max: usize) -> Result<Vec<u8>, String> {
        let count = self.u32()? as usize;
        if count == 0 || count > max {
            return Err("Invalid inherited packet length".into());
        }
        self.bytes(count)
    }
    fn string(&mut self) -> Result<String, String> {
        let count = self.count(MAX_STRING)?;
        String::from_utf8(self.bytes(count)?).map_err(|_| "Standalone string is not UTF-8".into())
    }
    fn boolean(&mut self) -> Result<bool, String> {
        match self.u8()? {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err("Invalid standalone boolean".into()),
        }
    }
    fn end(&self) -> Result<(), String> {
        if !self.wire.is_empty() {
            return Err("Trailing bytes after standalone message".into());
        }
        Ok(())
    }
}
