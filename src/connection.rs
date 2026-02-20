use meshtastic::api::{ConnectedStreamApi, StreamApi};
use meshtastic::packet::PacketReceiver;
use meshtastic::utils;

use crate::cli::ConnectionArgs;
use crate::error::CliError;
use crate::node_db::NodeDb;

pub struct EstablishedConnection {
    pub api: ConnectedStreamApi,
    pub node_db: NodeDb,
    pub packet_receiver: PacketReceiver,
}

pub async fn establish(args: &ConnectionArgs) -> Result<EstablishedConnection, CliError> {
    let stream_api = StreamApi::new();

    let (mut packet_receiver, connected_api) = if let Some(serial_path) = &args.serial {
        log::info!("Connecting via serial: {}", serial_path);
        let serial_stream =
            utils::stream::build_serial_stream(serial_path.clone(), None, None, None)
                .map_err(|e| CliError::Serial(e.to_string()))?;
        stream_api.connect(serial_stream).await
    } else if let Some(ble_target) = &args.ble {
        connect_ble(stream_api, ble_target).await?
    } else {
        let address = format!("{}:{}", args.host, args.port);
        log::info!("Connecting via TCP: {}", address);
        let tcp_stream = utils::stream::build_tcp_stream(address)
            .await
            .map_err(|e| CliError::Connection(e.to_string()))?;
        stream_api.connect(tcp_stream).await
    };

    let config_id: u32 = utils::generate_rand_id();
    log::debug!("Configuring with id={}", config_id);

    let api = connected_api
        .configure(config_id)
        .await
        .map_err(|e| CliError::Configuration(e.to_string()))?;

    let node_db = NodeDb::collect_initial(&mut packet_receiver, config_id, args.no_nodes).await?;
    log::info!(
        "Connected. Local node: !{:08x}, {} nodes in mesh",
        node_db.my_node_num(),
        node_db.nodes().len()
    );

    Ok(EstablishedConnection {
        api,
        node_db,
        packet_receiver,
    })
}

#[cfg(feature = "ble")]
async fn connect_ble(
    stream_api: StreamApi,
    ble_target: &str,
) -> Result<
    (
        PacketReceiver,
        meshtastic::api::ConnectedStreamApi<meshtastic::api::state::Connected>,
    ),
    CliError,
> {
    use std::time::Duration;

    use meshtastic::BleId;

    let ble_id = if ble_target.contains(':') {
        BleId::from_mac_address(ble_target).map_err(|e| CliError::Ble(e.to_string()))?
    } else {
        BleId::from_name(ble_target)
    };

    log::info!("Connecting via BLE: {}", ble_id);

    let ble_stream = utils::build_ble_stream(ble_id, Duration::from_secs(30))
        .await
        .map_err(|e| CliError::Ble(e.to_string()))?;

    Ok(stream_api.connect(ble_stream).await)
}

#[cfg(not(feature = "ble"))]
async fn connect_ble(
    _stream_api: StreamApi,
    _ble_target: &str,
) -> Result<
    (
        PacketReceiver,
        meshtastic::api::ConnectedStreamApi<meshtastic::api::state::Connected>,
    ),
    CliError,
> {
    Err(CliError::Ble(
        "BLE support not compiled. Rebuild with: cargo build --features ble".to_string(),
    ))
}

#[cfg(feature = "ble")]
pub async fn scan_ble_devices() -> Result<(), CliError> {
    use std::time::Duration;

    use colored::Colorize;
    use meshtastic::available_ble_devices;

    println!("{} Scanning for BLE Meshtastic devices...", "->".cyan());

    let devices = available_ble_devices(Duration::from_secs(5))
        .await
        .map_err(|e| CliError::Ble(e.to_string()))?;

    if devices.is_empty() {
        println!("No BLE Meshtastic devices found.");
    } else {
        println!("\n{:<30} {}", "Name".bold(), "MAC Address".bold());
        println!("{}", "-".repeat(50));
        for device in &devices {
            let name = device.name.as_deref().unwrap_or("(unknown)");
            println!("{:<30} {}", name, device.mac_address);
        }
        println!("\nFound {} device(s).", devices.len());
    }

    Ok(())
}

#[cfg(not(feature = "ble"))]
pub async fn scan_ble_devices() -> Result<(), CliError> {
    Err(CliError::Ble(
        "BLE support not compiled. Rebuild with: cargo build --features ble".to_string(),
    ))
}
