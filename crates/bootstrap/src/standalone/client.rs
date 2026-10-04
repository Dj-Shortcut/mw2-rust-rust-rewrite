use super::protocol::{
    ActionRequest, ClientMessage, ServerMessage, decode_server, encode_client, evaluate, hello,
};
use net::CmdSeq;
use net::transport::direct_tcp::FramedTcp;
use net::transport::protocol::{ClientPacket, ConnectionId, HandshakeHello, PacketHeader};
use playerstate_iw4::UserCmd;
use sim::{ClientId, SimWorld};
use std::net::{SocketAddr, TcpStream};
use std::time::{Duration, Instant};
use survival::{SharedReceipt, SharedSnapshot};

const APPLY_TIMEOUT: Duration = Duration::from_secs(5);
const MAX_RECEIPTS: usize = 256;
const MAX_MESSAGES_PER_POLL: usize = 4;

#[derive(Clone, Copy)]
struct Binding {
    client: ClientId,
    connection: ConnectionId,
    epoch: u32,
    server_sequence: u32,
    snapshot_sequence: u32,
    server_ack: u32,
    client_sequence: u32,
    command_sequence: u32,
}

pub struct DirectClient {
    transport: Option<FramedTcp>,
    hello: HandshakeHello,
    binding: Option<Binding>,
    snapshot: Option<SharedSnapshot>,
    replica: Option<SimWorld>,
    receipts: Vec<SharedReceipt>,
    ready: bool,
    apply_deadline: Instant,
    failure: Option<String>,
}

impl DirectClient {
    pub fn connect(address: SocketAddr) -> Result<Self, String> {
        let hello = hello()?;
        let stream = TcpStream::connect_timeout(&address, APPLY_TIMEOUT)
            .map_err(|error| format!("Could not connect to the direct server: {error}"))?;
        let mut transport = FramedTcp::new(stream)
            .map_err(|error| format!("Could not initialize the direct connection: {error}"))?;
        transport
            .queue(encode_client(&ClientMessage::Connect(hello))?)
            .map_err(|error| format!("Could not queue the direct handshake: {error}"))?;
        Ok(Self {
            transport: Some(transport),
            hello,
            binding: None,
            snapshot: None,
            replica: None,
            receipts: Vec::new(),
            ready: false,
            apply_deadline: Instant::now() + APPLY_TIMEOUT,
            failure: None,
        })
    }

    pub fn poll(&mut self) -> Result<(), String> {
        if let Some(error) = &self.failure {
            return Err(error.clone());
        }
        match self.poll_active() {
            Ok(()) => Ok(()),
            Err(error) => Err(self.fail(error)),
        }
    }

    pub fn ready(&self) -> bool {
        self.ready && self.failure.is_none()
    }

    pub fn identity(&self) -> Option<ClientId> {
        self.binding.map(|binding| binding.client)
    }

    pub fn connection(&self) -> Option<ConnectionId> {
        self.binding.map(|binding| binding.connection)
    }

    pub fn epoch(&self) -> Option<u32> {
        self.binding.map(|binding| binding.epoch)
    }

    pub fn snapshot(&self) -> Option<&SharedSnapshot> {
        self.snapshot.as_ref()
    }

    pub fn replica(&self) -> Option<&SimWorld> {
        self.replica.as_ref()
    }

    pub fn take_receipts(&mut self) -> Vec<SharedReceipt> {
        std::mem::take(&mut self.receipts)
    }

    pub fn send(&mut self, command: UserCmd, action: Option<ActionRequest>) -> Result<(), String> {
        if let Some(error) = &self.failure {
            return Err(error.clone());
        }
        if !self.ready {
            return Err("The direct client is not ready for gameplay".into());
        }
        match self.send_active(command, action) {
            Ok(()) => Ok(()),
            Err(error) => Err(self.fail(error)),
        }
    }

    fn poll_active(&mut self) -> Result<(), String> {
        self.check_deadline()?;
        self.write_pending()?;
        for _ in 0..MAX_MESSAGES_PER_POLL {
            let bytes = self
                .transport
                .as_mut()
                .ok_or("The direct connection is closed")?
                .poll_read()
                .map_err(|error| format!("Could not read from the direct server: {error}"))?;
            let Some(bytes) = bytes else {
                break;
            };
            self.apply_message(decode_server(&bytes)?)?;
        }
        self.write_pending()?;
        self.check_deadline()
    }

    fn write_pending(&mut self) -> Result<(), String> {
        self.transport
            .as_mut()
            .ok_or("The direct connection is closed")?
            .poll_write()
            .map_err(|error| format!("Could not write to the direct server: {error}"))
    }

    fn check_deadline(&self) -> Result<(), String> {
        if !self.ready && Instant::now() >= self.apply_deadline {
            Err("The direct server did not complete admission within five seconds".into())
        } else {
            Ok(())
        }
    }

    fn apply_message(&mut self, message: ServerMessage) -> Result<(), String> {
        self.check_deadline()?;
        match message {
            ServerMessage::Accept {
                connection,
                assigned_client,
                hello,
                header,
                snapshot_seq,
                state,
            } => self.accept(
                connection,
                assigned_client,
                hello,
                header,
                snapshot_seq,
                state,
            ),
            ServerMessage::State {
                header,
                snapshot_seq,
                state,
                receipts,
            } => self.apply_state(header, snapshot_seq, state, receipts),
            ServerMessage::Reject(reason) => {
                Err(format!("The direct server rejected admission: {reason}"))
            }
        }
    }

    fn accept(
        &mut self,
        connection: ConnectionId,
        assigned_client: ClientId,
        server_hello: HandshakeHello,
        header: PacketHeader,
        snapshot_sequence: u32,
        state: SharedSnapshot,
    ) -> Result<(), String> {
        if self.binding.is_some() {
            return Err("The direct server sent a duplicate admission".into());
        }
        evaluate(&self.hello, &server_hello)?;
        if connection.0 == 0
            || assigned_client.0 >= 64
            || header.connection != connection
            || header.epoch == 0
            || header.sequence == 0
            || header.ack != 0
            || snapshot_sequence == 0
            || state.recipient != assigned_client
        {
            return Err("The direct server sent invalid admission identity".into());
        }
        let replica = prepare_replica(&state)?;
        self.check_deadline()?;
        let ready_header = PacketHeader {
            connection,
            sequence: 1,
            ack: header.sequence,
            epoch: header.epoch,
        };
        let ready = encode_client(&ClientMessage::Ready {
            header: ready_header,
            snapshot_seq: snapshot_sequence,
        })?;
        self.transport
            .as_mut()
            .ok_or("The direct connection is closed")?
            .queue(ready)
            .map_err(|error| format!("Could not acknowledge the direct bootstrap: {error}"))?;
        self.binding = Some(Binding {
            client: assigned_client,
            connection,
            epoch: header.epoch,
            server_sequence: header.sequence,
            snapshot_sequence,
            server_ack: header.ack,
            client_sequence: ready_header.sequence,
            command_sequence: 0,
        });
        self.snapshot = Some(state);
        self.replica = Some(replica);
        Ok(())
    }

    fn apply_state(
        &mut self,
        header: PacketHeader,
        snapshot_sequence: u32,
        state: SharedSnapshot,
        receipts: Vec<SharedReceipt>,
    ) -> Result<(), String> {
        let mut binding = self
            .binding
            .ok_or("The direct server sent state before admission")?;
        let previous = self
            .snapshot
            .as_ref()
            .ok_or("The direct bootstrap is missing")?;
        if header.connection != binding.connection
            || header.epoch != binding.epoch
            || header.sequence <= binding.server_sequence
            || header.ack < binding.server_ack
            || header.ack > binding.client_sequence
            || snapshot_sequence <= binding.snapshot_sequence
            || state.tick.0 <= previous.tick.0
            || state.recipient != binding.client
            || (!self.ready && header.ack < 1)
        {
            return Err("The direct server sent stale or foreign state".into());
        }
        if self.receipts.len().saturating_add(receipts.len()) > MAX_RECEIPTS {
            return Err(
                "The direct receipt buffer is full; consume receipts before polling again".into(),
            );
        }
        if receipts.iter().any(|receipt| {
            receipt.client != binding.client
                || receipt.request_id == 0
                || receipt.applied_at.0 > state.tick.0
        }) {
            return Err("The direct server sent an invalid action receipt".into());
        }
        let replica = prepare_replica(&state)?;
        self.check_deadline()?;
        binding.server_sequence = header.sequence;
        binding.snapshot_sequence = snapshot_sequence;
        binding.server_ack = header.ack;
        self.binding = Some(binding);
        self.snapshot = Some(state);
        self.replica = Some(replica);
        self.receipts.extend(receipts);
        self.ready = true;
        Ok(())
    }

    fn send_active(
        &mut self,
        command: UserCmd,
        action: Option<ActionRequest>,
    ) -> Result<(), String> {
        let mut binding = self
            .binding
            .ok_or("The direct client has no assigned identity")?;
        let sequence = binding
            .client_sequence
            .checked_add(1)
            .ok_or("The direct packet sequence is exhausted")?;
        let command_sequence = binding
            .command_sequence
            .checked_add(1)
            .ok_or("The direct command sequence is exhausted")?;
        let message = ClientMessage::Input {
            packet: ClientPacket::Commands {
                header: PacketHeader {
                    connection: binding.connection,
                    sequence,
                    ack: binding.server_sequence,
                    epoch: binding.epoch,
                },
                claimed_client: binding.client.0,
                cmds: vec![(CmdSeq(command_sequence), command)],
                samples: Vec::new(),
                actions: Vec::new(),
                reliable_ack: 0,
            },
            action,
        };
        let bytes = encode_client(&message)?;
        self.transport
            .as_mut()
            .ok_or("The direct connection is closed")?
            .queue(bytes)
            .map_err(|error| format!("Could not queue direct gameplay input: {error}"))?;
        binding.client_sequence = sequence;
        binding.command_sequence = command_sequence;
        self.binding = Some(binding);
        Ok(())
    }

    fn fail(&mut self, error: String) -> String {
        self.transport = None;
        self.ready = false;
        self.failure = Some(error.clone());
        error
    }
}

fn prepare_replica(state: &SharedSnapshot) -> Result<SimWorld, String> {
    let mut replica = survival::shared_replica(state.nodes.clone())?;
    let report = replica.adopt_snapshot(&state.sim);
    if report.content_mismatch {
        return Err("The direct snapshot does not match authored replica content".into());
    }
    Ok(replica)
}
