use std::collections::HashMap;

use meshtastic::protobufs;

pub struct NodeDb {
    my_node_info: protobufs::MyNodeInfo,
    nodes: HashMap<u32, protobufs::NodeInfo>,
    channels: Vec<protobufs::Channel>,
    metadata: Option<protobufs::DeviceMetadata>,
    local_config: protobufs::LocalConfig,
    local_module_config: protobufs::LocalModuleConfig,
}

impl NodeDb {
    pub fn new(
        my_node_info: protobufs::MyNodeInfo,
        nodes: HashMap<u32, protobufs::NodeInfo>,
        channels: Vec<protobufs::Channel>,
        metadata: Option<protobufs::DeviceMetadata>,
        local_config: protobufs::LocalConfig,
        local_module_config: protobufs::LocalModuleConfig,
    ) -> Self {
        Self {
            my_node_info,
            nodes,
            channels,
            metadata,
            local_config,
            local_module_config,
        }
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

    pub fn local_config(&self) -> &protobufs::LocalConfig {
        &self.local_config
    }

    pub fn local_module_config(&self) -> &protobufs::LocalModuleConfig {
        &self.local_module_config
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
