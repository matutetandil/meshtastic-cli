use std::collections::HashMap;
use std::time::Duration;

use meshtastic::packet::PacketReceiver;
use meshtastic::protobufs;
use meshtastic::protobufs::from_radio::PayloadVariant;
use tokio::time::timeout;

use crate::error::CliError;
use crate::node_db::NodeDb;

const CONFIG_TIMEOUT: Duration = Duration::from_secs(30);

pub async fn collect_initial(
    receiver: &mut PacketReceiver,
    config_id: u32,
    skip_nodes: bool,
) -> Result<NodeDb, CliError> {
    let mut my_node_info: Option<protobufs::MyNodeInfo> = None;
    let mut nodes: HashMap<u32, protobufs::NodeInfo> = HashMap::new();
    let mut channels: Vec<protobufs::Channel> = Vec::new();
    let mut metadata: Option<protobufs::DeviceMetadata> = None;
    let mut local_config = protobufs::LocalConfig::default();
    let mut local_module_config = protobufs::LocalModuleConfig::default();

    loop {
        let packet = timeout(CONFIG_TIMEOUT, receiver.recv())
            .await
            .map_err(|_| CliError::Timeout("Timed out waiting for configuration packets".into()))?
            .ok_or(CliError::Disconnected)?;

        let Some(variant) = packet.payload_variant else {
            continue;
        };

        match variant {
            PayloadVariant::MyInfo(info) => {
                log::debug!("Received MyNodeInfo: node_num={}", info.my_node_num);
                my_node_info = Some(info);
            }
            PayloadVariant::NodeInfo(info) => {
                if !skip_nodes {
                    log::debug!("Received NodeInfo: num={}", info.num);
                    nodes.insert(info.num, info);
                }
            }
            PayloadVariant::Channel(ch) => {
                log::debug!("Received Channel: index={}", ch.index);
                channels.push(ch);
            }
            PayloadVariant::Metadata(meta) => {
                log::debug!("Received DeviceMetadata: fw={}", meta.firmware_version);
                metadata = Some(meta);
            }
            PayloadVariant::Config(cfg) => {
                log::debug!("Received Config packet");
                fold_config(&mut local_config, cfg);
            }
            PayloadVariant::ModuleConfig(mcfg) => {
                log::debug!("Received ModuleConfig packet");
                fold_module_config(&mut local_module_config, mcfg);
            }
            PayloadVariant::ConfigCompleteId(id) if id == config_id => {
                log::debug!("Configuration complete (id={})", id);
                break;
            }
            _ => {
                log::trace!("Skipping config packet: {:?}", variant);
            }
        }
    }

    let my_node_info = my_node_info.ok_or(CliError::NoLocalNodeInfo)?;
    channels.sort_by_key(|c| c.index);

    Ok(NodeDb::new(
        my_node_info,
        nodes,
        channels,
        metadata,
        local_config,
        local_module_config,
    ))
}

fn fold_config(local: &mut protobufs::LocalConfig, cfg: protobufs::Config) {
    use protobufs::config::PayloadVariant;
    if let Some(variant) = cfg.payload_variant {
        match variant {
            PayloadVariant::Device(v) => local.device = Some(v),
            PayloadVariant::Position(v) => local.position = Some(v),
            PayloadVariant::Power(v) => local.power = Some(v),
            PayloadVariant::Network(v) => local.network = Some(v),
            PayloadVariant::Display(v) => local.display = Some(v),
            PayloadVariant::Lora(v) => local.lora = Some(v),
            PayloadVariant::Bluetooth(v) => local.bluetooth = Some(v),
            PayloadVariant::Security(v) => local.security = Some(v),
            _ => log::trace!("Skipping config variant: {:?}", variant),
        }
    }
}

fn fold_module_config(local: &mut protobufs::LocalModuleConfig, mcfg: protobufs::ModuleConfig) {
    use protobufs::module_config::PayloadVariant;
    if let Some(variant) = mcfg.payload_variant {
        match variant {
            PayloadVariant::Mqtt(v) => local.mqtt = Some(v),
            PayloadVariant::Serial(v) => local.serial = Some(v),
            PayloadVariant::ExternalNotification(v) => local.external_notification = Some(v),
            PayloadVariant::StoreForward(v) => local.store_forward = Some(v),
            PayloadVariant::RangeTest(v) => local.range_test = Some(v),
            PayloadVariant::Telemetry(v) => local.telemetry = Some(v),
            PayloadVariant::CannedMessage(v) => local.canned_message = Some(v),
            PayloadVariant::Audio(v) => local.audio = Some(v),
            PayloadVariant::RemoteHardware(v) => local.remote_hardware = Some(v),
            PayloadVariant::NeighborInfo(v) => local.neighbor_info = Some(v),
            PayloadVariant::AmbientLighting(v) => local.ambient_lighting = Some(v),
            PayloadVariant::DetectionSensor(v) => local.detection_sensor = Some(v),
            PayloadVariant::Paxcounter(v) => local.paxcounter = Some(v),
        }
    }
}
