use meshtastic::packet::PacketRouter;
use meshtastic::protobufs;
use meshtastic::types::NodeId;

use crate::error::CliError;

pub struct MeshRouter {
    node_id: NodeId,
}

impl MeshRouter {
    pub fn new(node_id: u32) -> Self {
        Self {
            node_id: NodeId::new(node_id),
        }
    }
}

impl PacketRouter<(), CliError> for MeshRouter {
    fn handle_packet_from_radio(&mut self, _packet: protobufs::FromRadio) -> Result<(), CliError> {
        Ok(())
    }

    fn handle_mesh_packet(&mut self, _packet: protobufs::MeshPacket) -> Result<(), CliError> {
        Ok(())
    }

    fn source_node_id(&self) -> NodeId {
        self.node_id
    }
}
