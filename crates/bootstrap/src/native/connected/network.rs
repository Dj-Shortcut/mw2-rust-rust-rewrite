use crate::standalone::DirectClient;
use crate::standalone::protocol::ActionRequest;
use playerstate_iw4::UserCmd;
use sim::SimWorld;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};
use survival::{SharedAction, SharedReceipt, SharedSnapshot};

const POLL_INTERVAL: Duration = Duration::from_millis(2);
const INPUT_MAX_AGE: Duration = Duration::from_millis(250);
const STALL_AFTER: Duration = Duration::from_secs(5);
const STALL_GRACE: Duration = Duration::from_secs(2);
const MAX_RECEIPTS: usize = 256;
const MAX_MESSAGE_CHARS: usize = 512;

pub(super) struct ReceivedWorld {
    pub snapshot: SharedSnapshot,
    pub replica: SimWorld,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum NetworkStatus {
    Connecting,
    ApplyingBootstrap,
    Connected,
    Stalled,
    Failed(String),
}

impl NetworkStatus {
    pub(super) fn connected(&self) -> bool {
        matches!(self, Self::Connected)
    }
}

pub(super) struct NetworkUpdate {
    pub status: NetworkStatus,
    pub world: Option<ReceivedWorld>,
    pub receipts: Vec<SharedReceipt>,
    pub pending: bool,
    pub notice: Option<String>,
    pub reset_controls: bool,
}

#[derive(Clone, Copy)]
enum PendingAction {
    Queued(SharedAction),
    Sent { id: u32 },
}

struct Mailbox {
    status: NetworkStatus,
    world: Option<ReceivedWorld>,
    receipts: Vec<SharedReceipt>,
    pending: Option<PendingAction>,
    command: UserCmd,
    active: bool,
    input_at: Instant,
    notice: Option<String>,
    reset_controls: bool,
}

impl Mailbox {
    fn cancel_queued(&mut self, reason: &str) {
        if matches!(self.pending, Some(PendingAction::Queued(_))) {
            self.pending = None;
            self.notice = Some(reason.into());
        }
    }

    fn neutralize(&mut self) {
        self.command = neutral(self.command);
        self.active = false;
    }
}

pub(super) struct NetworkHandle {
    shared: Arc<Mutex<Mailbox>>,
    stop: Arc<AtomicBool>,
    worker: Option<JoinHandle<()>>,
}

impl NetworkHandle {
    pub(super) fn start(address: SocketAddr) -> Result<Self, String> {
        let shared = Arc::new(Mutex::new(Mailbox {
            status: NetworkStatus::Connecting,
            world: None,
            receipts: Vec::with_capacity(MAX_RECEIPTS),
            pending: None,
            command: UserCmd::default(),
            active: false,
            input_at: Instant::now(),
            notice: None,
            reset_controls: false,
        }));
        let stop = Arc::new(AtomicBool::new(false));
        let worker_shared = Arc::clone(&shared);
        let worker_stop = Arc::clone(&stop);
        let worker = thread::Builder::new()
            .name("native-direct-network".into())
            .spawn(move || {
                let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    run_worker(address, &worker_shared, &worker_stop)
                }));
                let error = match result {
                    Ok(Ok(())) => None,
                    Ok(Err(error)) => Some(error),
                    Err(_) => Some("The network worker stopped unexpectedly".into()),
                };
                if let Some(error) = error {
                    fail_mailbox(&worker_shared, error);
                }
            })
            .map_err(|error| bounded(format!("Could not start the network worker: {error}")))?;
        Ok(Self {
            shared,
            stop,
            worker: Some(worker),
        })
    }

    pub(super) fn submit(
        &self,
        command: UserCmd,
        action: Option<SharedAction>,
        active: bool,
    ) -> Result<(), String> {
        let mut mailbox = self.lock()?;
        mailbox.input_at = Instant::now();
        if !active || !mailbox.status.connected() || mailbox.reset_controls {
            mailbox.command = neutral(command);
            mailbox.active = false;
            mailbox.cancel_queued("The unsent action was canceled because controls are inactive");
            return if active && action.is_some() {
                Err(
                    "Wait for the connection and active controls before requesting an action"
                        .into(),
                )
            } else {
                Ok(())
            };
        }
        mailbox.command = command;
        mailbox.active = true;
        if let Some(action) = action {
            if mailbox.pending.is_some() {
                return Err("An action is pending; wait for the server receipt".into());
            }
            mailbox.pending = Some(PendingAction::Queued(action));
        }
        Ok(())
    }

    pub(super) fn take_update(&self) -> Result<NetworkUpdate, String> {
        let finished = self.worker.as_ref().is_none_or(JoinHandle::is_finished);
        let mut mailbox = self.lock()?;
        if finished && !matches!(mailbox.status, NetworkStatus::Failed(_)) {
            self.stop.store(true, Ordering::Release);
            mailbox.status =
                NetworkStatus::Failed("The network worker stopped unexpectedly".into());
            mailbox.reset_controls = true;
        }
        let reset_controls = mailbox.reset_controls;
        if reset_controls {
            mailbox.neutralize();
            mailbox.cancel_queued("The unsent action was canceled while controls were reset");
            mailbox.reset_controls = false;
        }
        Ok(NetworkUpdate {
            status: mailbox.status.clone(),
            world: mailbox.world.take(),
            receipts: std::mem::take(&mut mailbox.receipts),
            pending: mailbox.pending.is_some(),
            notice: mailbox.notice.take(),
            reset_controls,
        })
    }

    fn lock(&self) -> Result<std::sync::MutexGuard<'_, Mailbox>, String> {
        self.shared.lock().map_err(|_| {
            self.stop.store(true, Ordering::Release);
            "The network state is unavailable after a worker failure".into()
        })
    }
}

impl Drop for NetworkHandle {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}

fn run_worker(
    address: SocketAddr,
    shared: &Mutex<Mailbox>,
    stop: &AtomicBool,
) -> Result<(), String> {
    if stop.load(Ordering::Acquire) {
        return Ok(());
    }
    let mut client = DirectClient::connect(address)?;
    let mut published_tick = None;
    let mut sent_tick = None;
    let mut last_tick_at = Instant::now();
    let mut stalled_at = None;
    let mut highest_request = 0_u32;
    while !stop.load(Ordering::Acquire) {
        if stalled_at.is_some_and(|start: Instant| start.elapsed() >= STALL_GRACE) {
            return Err("The server stopped sending valid ticks; the connection was closed".into());
        }
        let poll_result = client.poll();
        let now = Instant::now();
        let tick = client.snapshot().map(|state| state.tick);
        let changed = tick.is_some() && tick != published_tick;
        let world = if changed {
            match (client.snapshot(), client.replica()) {
                (Some(snapshot), Some(replica)) => Some(ReceivedWorld {
                    snapshot: snapshot.clone(),
                    replica: replica.clone(),
                }),
                _ => return Err("The validated server world is incomplete".into()),
            }
        } else {
            None
        };
        let receipts = client.take_receipts();
        let mut reset_controls = false;
        let status = if let Err(error) = &poll_result {
            reset_controls = true;
            NetworkStatus::Failed(bounded(error.clone()))
        } else if !client.ready() {
            if client.identity().is_some() {
                NetworkStatus::ApplyingBootstrap
            } else {
                NetworkStatus::Connecting
            }
        } else if changed {
            last_tick_at = now;
            reset_controls = stalled_at.take().is_some();
            NetworkStatus::Connected
        } else if now.duration_since(last_tick_at) >= STALL_AFTER {
            reset_controls = stalled_at.is_none();
            let start = stalled_at.get_or_insert(now);
            if now.duration_since(*start) >= STALL_GRACE {
                reset_controls = true;
                NetworkStatus::Failed(
                    "The server stopped sending valid ticks; the connection was closed".into(),
                )
            } else {
                NetworkStatus::Stalled
            }
        } else {
            NetworkStatus::Connected
        };
        let failure = match &status {
            NetworkStatus::Failed(error) => Some(error.clone()),
            _ => None,
        };
        publish(shared, status, world, receipts, reset_controls)?;
        published_tick = tick;
        if let Some(error) = failure {
            return Err(error);
        }
        if stop.load(Ordering::Acquire) {
            break;
        }
        if client.ready() && changed && tick != sent_tick {
            let input = prepare_input(shared, highest_request, Instant::now())?;
            if let Some((command, action)) = input {
                client.send(command, action)?;
                if let Some(action) = action {
                    highest_request = action.request_id;
                }
                sent_tick = tick;
            }
        }
        thread::sleep(POLL_INTERVAL);
    }
    Ok(())
}

fn publish(
    shared: &Mutex<Mailbox>,
    status: NetworkStatus,
    world: Option<ReceivedWorld>,
    receipts: Vec<SharedReceipt>,
    reset_controls: bool,
) -> Result<(), String> {
    let displaced = {
        let mut mailbox = shared
            .lock()
            .map_err(|_| "The network state is unavailable after a worker failure")?;
        if mailbox.receipts.len().saturating_add(receipts.len()) > MAX_RECEIPTS {
            return Err("The server receipt buffer is full; the connection was closed".into());
        }
        let became_connected = status.connected() && !mailbox.status.connected();
        mailbox.reset_controls |= reset_controls || became_connected;
        if !status.connected() || mailbox.reset_controls {
            mailbox.neutralize();
            mailbox.cancel_queued("The unsent action was canceled while the connection changed");
        } else if mailbox.input_at.elapsed() > INPUT_MAX_AGE {
            mailbox.neutralize();
            mailbox.cancel_queued("The unsent action was canceled because input became stale");
        }
        for receipt in &receipts {
            if matches!(mailbox.pending, Some(PendingAction::Sent { id }) if id == receipt.request_id)
            {
                mailbox.pending = None;
            }
        }
        mailbox.receipts.extend(receipts);
        mailbox.status = status;
        world.and_then(|world| mailbox.world.replace(world))
    };
    drop(displaced);
    Ok(())
}

fn prepare_input(
    shared: &Mutex<Mailbox>,
    highest_request: u32,
    now: Instant,
) -> Result<Option<(UserCmd, Option<ActionRequest>)>, String> {
    let mut mailbox = shared
        .lock()
        .map_err(|_| "The network state is unavailable after a worker failure")?;
    if !mailbox.status.connected() {
        return Ok(None);
    }
    if now.saturating_duration_since(mailbox.input_at) > INPUT_MAX_AGE {
        mailbox.neutralize();
        mailbox.cancel_queued("The unsent action was canceled because input became stale");
    }
    if !mailbox.active || mailbox.reset_controls {
        mailbox.command = neutral(mailbox.command);
        mailbox.cancel_queued("The unsent action was canceled because controls are inactive");
        return Ok(Some((mailbox.command, None)));
    }
    let action = match mailbox.pending {
        Some(PendingAction::Queued(action)) => {
            let request_id = highest_request
                .checked_add(1)
                .ok_or("The action request sequence is exhausted; reconnect to continue")?;
            // Reserve before unlocking so pause cannot erase an action during socket issuance.
            mailbox.pending = Some(PendingAction::Sent { id: request_id });
            Some(ActionRequest { request_id, action })
        }
        _ => None,
    };
    Ok(Some((mailbox.command, action)))
}

fn fail_mailbox(shared: &Mutex<Mailbox>, error: String) {
    if let Ok(mut mailbox) = shared.lock() {
        mailbox.status = NetworkStatus::Failed(bounded(error));
        mailbox.reset_controls = true;
        mailbox.neutralize();
        mailbox.cancel_queued("The unsent action was canceled because the connection failed");
    }
}

fn neutral(mut command: UserCmd) -> UserCmd {
    command.buttons = 0;
    command.forwardmove = 0;
    command.rightmove = 0;
    command
}

fn bounded(message: String) -> String {
    message.chars().take(MAX_MESSAGE_CHARS).collect()
}
