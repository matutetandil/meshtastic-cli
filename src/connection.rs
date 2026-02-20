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

    let node_db = NodeDb::collect_initial(&mut packet_receiver, config_id).await?;
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
