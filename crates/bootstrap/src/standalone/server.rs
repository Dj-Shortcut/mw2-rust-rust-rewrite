use super::protocol::{self, ActionRequest, ClientMessage, ServerMessage};
use net::transport::direct_tcp::FramedTcp;
use net::transport::protocol::{ClientPacket, ConnectionId, HandshakeHello, PacketHeader};
use playerstate_iw4::UserCmd;
use sim::ClientId;
use std::collections::BTreeMap;
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::time::{Duration, Instant};
use survival::{ActorHandle, SharedAction, SharedReceipt, SharedRequest, SharedSession};

const STEP: Duration = Duration::from_millis(50);
const JOIN_TIMEOUT: Duration = Duration::from_secs(5);

fn report_peer_error(error: &str, last: &mut Option<Instant>) {
    if last.is_none_or(|time| time.elapsed() >= Duration::from_secs(1)) {
        eprintln!("Direct connection closed: {error}");
        *last = Some(Instant::now());
    }
}

struct Peer {
    io: FramedTcp,
    admitted_at: Instant,
    actor: Option<ActorHandle>,
    connection: ConnectionId,
    ready: bool,
    closing: bool,
    input_sequence: u32,
    command_sequence: u32,
    output_sequence: u32,
    bootstrap_sequence: u32,
    command: Option<UserCmd>,
    request: Option<ActionRequest>,
    highest_request: u32,
    history: BTreeMap<u32, SharedAction>,
}

impl Peer {
    fn new(stream: TcpStream) -> Result<Self, String> {
        Ok(Self {
            io: FramedTcp::new(stream).map_err(|error| error.to_string())?,
            admitted_at: Instant::now(),
            actor: None,
            connection: ConnectionId(0),
            ready: false,
            closing: false,
            input_sequence: 0,
            command_sequence: 0,
            output_sequence: 0,
            bootstrap_sequence: 0,
            command: None,
            request: None,
            highest_request: 0,
            history: BTreeMap::new(),
        })
    }

    fn header(&mut self) -> Result<PacketHeader, String> {
        self.output_sequence = self
            .output_sequence
            .checked_add(1)
            .ok_or("Server sequence exhausted")?;
        Ok(PacketHeader {
            connection: self.connection,
            sequence: self.output_sequence,
            ack: self.input_sequence,
            epoch: 1,
        })
    }

    fn check_header(&self, header: PacketHeader) -> Result<(), String> {
        if header.connection != self.connection
            || header.epoch != 1
            || header.sequence == 0
            || header.sequence <= self.input_sequence
            || header.ack > self.output_sequence
        {
            return Err("Invalid connection, epoch or packet sequence".into());
        }
        Ok(())
    }

    fn queue(&mut self, message: &ServerMessage) -> Result<(), String> {
        self.io
            .queue(protocol::encode_server(message)?)
            .map_err(|error| error.to_string())
    }

    fn receive(
        &mut self,
        message: ClientMessage,
        session: &mut SharedSession,
        hello: HandshakeHello,
        next_connection: &mut u64,
    ) -> Result<(), String> {
        match message {
            ClientMessage::Connect(client_hello) => {
                if self.actor.is_some() {
                    return Err("Connection is already admitted".into());
                }
                let admission = (|| {
                    protocol::evaluate(&hello, &client_hello)?;
                    let connection = next_connection
                        .checked_add(1)
                        .ok_or("Connection identities exhausted")?;
                    let actor = (0..64)
                        .find_map(|slot| session.connect(ClientId(slot)).ok())
                        .ok_or("The shared world is full or has no available identity")?;
                    *next_connection = connection;
                    self.connection = ConnectionId(connection);
                    self.actor = Some(actor);
                    let header = self.header()?;
                    self.bootstrap_sequence = header.sequence;
                    self.queue(&ServerMessage::Accept {
                        connection: self.connection,
                        assigned_client: actor.client(),
                        hello,
                        header,
                        snapshot_seq: header.sequence,
                        state: session.snapshot_for(actor)?,
                    })
                })();
                if let Err(error) = admission {
                    self.queue(&ServerMessage::Reject(error))?;
                    self.closing = true;
                }
                Ok(())
            }
            ClientMessage::Ready {
                header,
                snapshot_seq,
            } => {
                if self.actor.is_none() || self.ready {
                    return Err("Unexpected bootstrap acknowledgement".into());
                }
                self.check_header(header)?;
                if snapshot_seq != self.bootstrap_sequence || header.ack != self.bootstrap_sequence
                {
                    return Err("Bootstrap acknowledgement does not match this connection".into());
                }
                self.input_sequence = header.sequence;
                self.ready = true;
                Ok(())
            }
            ClientMessage::Input { packet, action } => {
                if !self.ready {
                    return Err("Apply the bootstrap before sending gameplay".into());
                }
                let ClientPacket::Commands {
                    header,
                    cmds,
                    samples,
                    actions,
                    reliable_ack,
                    ..
                } = packet
                else {
                    return Err("Unsupported gameplay packet".into());
                };
                self.check_header(header)?;
                if cmds.len() != 1
                    || cmds[0].0.0 <= self.command_sequence
                    || !samples.is_empty()
                    || !actions.is_empty()
                    || reliable_ack != 0
                {
                    return Err("Invalid movement or unsupported raw actions".into());
                }
                if let Some(action) = action {
                    if action.request_id == 0 {
                        return Err("Request ID must be nonzero".into());
                    }
                    if let Some(pending) = self.request {
                        if pending != action {
                            return Err("Only one pending action is allowed per tick".into());
                        }
                    } else {
                        if let Some(previous) = self.history.get(&action.request_id) {
                            if *previous != action.action {
                                return Err("Request ID was reused with a different action".into());
                            }
                        } else if action.request_id <= self.highest_request {
                            return Err("Request ID is expired or out of order".into());
                        }
                        self.request = Some(action);
                    }
                }
                self.command = Some(cmds[0].1);
                self.command_sequence = cmds[0].0.0;
                self.input_sequence = header.sequence;
                Ok(())
            }
        }
    }

    fn record(&mut self, action: ActionRequest) {
        self.highest_request = self.highest_request.max(action.request_id);
        self.history.insert(action.request_id, action.action);
        while self.history.len() > 256 {
            if let Some((&first, _)) = self.history.first_key_value() {
                self.history.remove(&first);
            }
        }
    }
}

pub fn run_server(bind: SocketAddr) -> Result<(), String> {
    let listener =
        TcpListener::bind(bind).map_err(|error| format!("Cannot bind server: {error}"))?;
    listener
        .set_nonblocking(true)
        .map_err(|error| error.to_string())?;
    let mut session = SharedSession::new()?;
    let hello = protocol::hello()?;
    let mut peers: Vec<Peer> = Vec::new();
    let mut next_connection = 0;
    let mut last_peer_error = None;
    let mut next_tick = Instant::now() + STEP;
    println!(
        "Authored-world server listening on {}",
        listener.local_addr().map_err(|error| error.to_string())?
    );
    loop {
        for _ in 0..4 {
            match listener.accept() {
                Ok((stream, _)) => {
                    let pending = peers.iter().filter(|peer| !peer.ready).count();
                    if peers.len() < 4
                        && pending < 2
                        && let Ok(peer) = Peer::new(stream)
                    {
                        peers.push(peer);
                    }
                }
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => break,
                Err(error) => return Err(format!("Server accept failed: {error}")),
            }
        }
        let mut index = 0;
        while index < peers.len() {
            let peer = &mut peers[index];
            let result = (|| {
                if !peer.ready && peer.admitted_at.elapsed() > JOIN_TIMEOUT {
                    return Err("Connection admission timed out".to_owned());
                }
                if !peer.closing {
                    for _ in 0..4 {
                        let Some(body) = peer.io.poll_read().map_err(|error| error.to_string())?
                        else {
                            break;
                        };
                        peer.receive(
                            protocol::decode_client(&body)?,
                            &mut session,
                            hello,
                            &mut next_connection,
                        )?;
                        if peer.closing {
                            break;
                        }
                    }
                }
                peer.io.poll_write().map_err(|error| error.to_string())?;
                if peer.closing && peer.io.is_output_empty() {
                    return Err("Connection rejected".into());
                }
                Ok(())
            })();
            if let Err(error) = result {
                report_peer_error(&error, &mut last_peer_error);
                let peer = peers.remove(index);
                if let Some(actor) = peer.actor {
                    session.disconnect(actor)?;
                }
            } else {
                index += 1;
            }
        }
        for _ in 0..4 {
            if Instant::now() < next_tick {
                break;
            }
            let commands = peers
                .iter_mut()
                .filter_map(|peer| Some((peer.actor?, peer.command.take()?)))
                .collect::<Vec<_>>();
            let mut actions = Vec::new();
            for peer in &mut peers {
                if let (Some(actor), Some(action)) = (peer.actor, peer.request.take()) {
                    peer.record(action);
                    actions.push(SharedRequest {
                        actor,
                        request_id: action.request_id,
                        action: action.action,
                    });
                }
            }
            let receipts = session.step(&commands, &actions)?;
            let mut index = 0;
            while index < peers.len() {
                let peer = &mut peers[index];
                if !peer.ready || peer.closing {
                    index += 1;
                    continue;
                }
                let actor = peer.actor.ok_or("Ready connection lacks an actor")?;
                let header = peer.header()?;
                let receipts: Vec<SharedReceipt> = receipts
                    .iter()
                    .filter(|receipt| receipt.client == actor.client())
                    .cloned()
                    .collect();
                let result = peer
                    .queue(&ServerMessage::State {
                        header,
                        snapshot_seq: header.sequence,
                        state: session.snapshot_for(actor)?,
                        receipts,
                    })
                    .and_then(|()| peer.io.poll_write().map_err(|error| error.to_string()));
                if let Err(error) = result {
                    report_peer_error(&error, &mut last_peer_error);
                    let peer = peers.remove(index);
                    session.disconnect(peer.actor.ok_or("Connection lacks an actor")?)?;
                } else {
                    index += 1;
                }
            }
            next_tick += STEP;
        }
        if Instant::now() >= next_tick {
            eprintln!("Server clock is behind; limiting catch-up to four ticks");
            next_tick = Instant::now() + STEP;
        }
        std::thread::sleep(Duration::from_millis(1));
    }
}
