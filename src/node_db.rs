use std::collections::HashMap;
use std::time::Duration;

use meshtastic::packet::PacketReceiver;
use meshtastic::protobufs;
use meshtastic::protobufs::from_radio::PayloadVariant;
use tokio::time::timeout;

use crate::error::CliError;

const CONFIG_TIMEOUT: Duration = Duration::from_secs(30);

pub struct NodeDb {
    my_node_info: protobufs::MyNodeInfo,
    nodes: HashMap<u32, protobufs::NodeInfo>,
    channels: Vec<protobufs::Channel>,
    metadata: Option<protobufs::DeviceMetadata>,
}

impl NodeDb {
    pub async fn collect_initial(
        receiver: &mut PacketReceiver,
        config_id: u32,
    ) -> Result<Self, CliError> {
        let mut my_node_info: Option<protobufs::MyNodeInfo> = None;
        let mut nodes: HashMap<u32, protobufs::NodeInfo> = HashMap::new();
        let mut channels: Vec<protobufs::Channel> = Vec::new();
        let mut metadata: Option<protobufs::DeviceMetadata> = None;

        loop {
            let packet = timeout(CONFIG_TIMEOUT, receiver.recv())
                .await
                .map_err(|_| {
                    CliError::Timeout("Timed out waiting for configuration packets".into())
                })?
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
                    log::debug!("Received NodeInfo: num={}", info.num);
                    nodes.insert(info.num, info);
                }
                PayloadVariant::Channel(ch) => {
                    log::debug!("Received Channel: index={}", ch.index);
                    channels.push(ch);
                }
                PayloadVariant::Metadata(meta) => {
                    log::debug!("Received DeviceMetadata: fw={}", meta.firmware_version);
                    metadata = Some(meta);
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

        Ok(Self {
            my_node_info,
            nodes,
            channels,
            metadata,
        })
    }

    pub fn my_node_num(&self) -> u32 {
        self.my_node_info.my_node_num
    }

    pub fn nodes(&self) -> &HashMap<u32, protobufs::NodeInfo> {
        &self.nodes
    }

    pub fn my_node_info(&self) -> &protobufs::MyNodeInfo {
        &self.my_node_info
    }

    pub fn local_node(&self) -> Option<&protobufs::NodeInfo> {
        self.nodes.get(&self.my_node_info.my_node_num)
    }

    pub fn channels(&self) -> &[protobufs::Channel] {
        &self.channels
    }

    pub fn metadata(&self) -> Option<&protobufs::DeviceMetadata> {
        self.metadata.as_ref()
    }

    pub fn node_name(&self, node_num: u32) -> Option<&str> {
        self.nodes
            .get(&node_num)
            .and_then(|n| n.user.as_ref())
            .map(|u| u.long_name.as_str())
    }

    pub fn find_by_name(&self, name: &str) -> Vec<(u32, &protobufs::NodeInfo)> {
        let query = name.to_lowercase();
        self.nodes
            .iter()
            .filter(|(_, node)| {
                node.user
                    .as_ref()
                    .is_some_and(|u| u.long_name.to_lowercase() == query)
            })
            .map(|(&num, node)| (num, node))
            .collect()
    }
}
