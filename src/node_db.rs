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
}

impl NodeDb {
    pub async fn collect_initial(
        receiver: &mut PacketReceiver,
        config_id: u32,
    ) -> Result<Self, CliError> {
        let mut my_node_info: Option<protobufs::MyNodeInfo> = None;
        let mut nodes: HashMap<u32, protobufs::NodeInfo> = HashMap::new();

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

        Ok(Self {
            my_node_info,
            nodes,
        })
    }

    pub fn my_node_num(&self) -> u32 {
        self.my_node_info.my_node_num
    }

    pub fn nodes(&self) -> &HashMap<u32, protobufs::NodeInfo> {
        &self.nodes
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
