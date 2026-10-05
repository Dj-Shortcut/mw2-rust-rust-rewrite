use super::{asset_directory, filtered_stick, gather_feedback};
use bevy::prelude::*;
use playerstate_iw4::UserCmd;
use sim::ClientId;
use std::net::SocketAddr;
use survival::{SharedAction, SharedEffect};

mod input;
mod inventory;
mod network;
mod scene;

#[derive(Resource)]
struct Connection {
    network: Option<network::NetworkHandle>,
    status: network::NetworkStatus,
    world: Option<network::ReceivedWorld>,
    pending: bool,
    message: String,
}

#[derive(Resource, Default)]
struct Controls {
    pitch: f32,
    yaw: f32,
    paused: bool,
    focused: bool,
    building: bool,
    building_wall: bool,
    wall_axis: u8,
    inventory: bool,
    help: bool,
    captured: bool,
    capture_frames: u8,
    initialized: Option<ClientId>,
    command: UserCmd,
    action: Option<SharedAction>,
    active: bool,
    pad: Option<Entity>,
    notice: Option<String>,
    content_ready: bool,
    content_error: Option<String>,
}

impl Controls {
    fn neutral(&mut self) {
        self.command.buttons = 0;
        self.command.forwardmove = 0;
        self.command.rightmove = 0;
        self.action = None;
        self.active = false;
    }

    fn recapture(&mut self) {
        self.neutral();
        self.captured = false;
        self.initialized = None;
    }
}

pub fn run(address: SocketAddr) -> Result<(), String> {
    let assets = asset_directory()?;
    let network = network::NetworkHandle::start(address)?;
    let mut app = App::new();
    app.insert_resource(Connection {
        network: Some(network),
        status: network::NetworkStatus::Connecting,
        world: None,
        pending: false,
        message: "Connecting to the shared world...".into(),
    })
    .init_resource::<Controls>()
    .insert_resource(ClearColor(Color::srgb(0.53, 0.68, 0.79)))
    .add_plugins(
        DefaultPlugins
            .set(bevy::asset::AssetPlugin {
                file_path: assets.to_string_lossy().into_owned(),
                ..default()
            })
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: format!("Shared Survival | {address}"),
                    resolution: (1280, 720).into(),
                    ..default()
                }),
                ..default()
            }),
    )
    .insert_resource(GlobalAmbientLight {
        color: Color::srgb(0.72, 0.80, 0.90),
        brightness: 180.,
        ..default()
    })
    .add_systems(Startup, scene::setup)
    .add_systems(
        PreUpdate,
        (receive, input::sample, publish)
            .chain()
            .after(bevy::input::InputSystems),
    )
    .add_systems(Update, scene::present);
    app.run();
    Ok(())
}

fn receive(mut connection: ResMut<Connection>, mut controls: ResMut<Controls>) {
    let Some(network) = &connection.network else {
        return;
    };
    let update = match network.take_update() {
        Ok(update) => update,
        Err(error) => {
            connection.status = network::NetworkStatus::Failed(error.clone());
            connection.message = error;
            controls.recapture();
            connection.network.take();
            return;
        }
    };
    if update.reset_controls {
        controls.recapture();
    }
    if update.status.connected() && !connection.status.connected() {
        connection.message = "Connected. Gather trees to build or hemp for Cloth.".into();
    }
    if let network::NetworkStatus::Failed(error) = &update.status {
        connection.message = error.clone();
        controls.recapture();
    }
    connection.status = update.status;
    connection.pending = update.pending;
    if let Some(world) = update.world {
        connection.world = Some(world);
    }
    if let Some(notice) = update.notice {
        connection.message = notice;
    }
    for receipt in update.receipts {
        connection.message = match receipt.result {
            Ok(SharedEffect::Gathered(harvest)) => gather_feedback(harvest),
            Ok(SharedEffect::FoundationPlaced { .. }) => "Wood foundation placed".into(),
            Ok(SharedEffect::WallPlaced { .. }) => "Wood wall placed".into(),
            Ok(SharedEffect::BandageCrafted) => "Bandage crafted | Used 4 Cloth".into(),
            Ok(SharedEffect::TradeOffered { offer_id }) => {
                format!("Trade offer #{offer_id} posted")
            }
            Ok(SharedEffect::TradeAccepted { offer_id }) => {
                format!("Trade #{offer_id} accepted")
            }
            Ok(SharedEffect::TradeClosed { offer_id }) => {
                format!("Trade offer #{offer_id} closed")
            }
            Err(error) => error,
        };
    }
    if matches!(connection.status, network::NetworkStatus::Failed(_)) {
        connection.network.take();
    }
}

fn publish(mut connection: ResMut<Connection>, mut controls: ResMut<Controls>) {
    if let Some(error) = controls.content_error.clone() {
        controls.recapture();
        connection.status = network::NetworkStatus::Failed(error.clone());
        connection.message = error;
        connection.network.take();
        return;
    }
    if let Some(notice) = controls.notice.take() {
        connection.message = notice;
    }
    let action = controls.action.take();
    let Some(network) = &connection.network else {
        return;
    };
    match network.submit(controls.command, action, controls.active) {
        Ok(()) if action.is_some() => {
            connection.pending = true;
            connection.message = "Action pending...".into();
        }
        Ok(()) => {}
        Err(error) => connection.message = error,
    }
}
