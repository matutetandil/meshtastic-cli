use meshtastic::protobufs::{self, admin_message, mesh_packet, Data, MeshPacket, PortNum};
use meshtastic::utils::generate_rand_id;
use meshtastic::Message;

use super::CommandContext;

pub async fn send_admin_message(
    ctx: &mut CommandContext,
    target_id: u32,
    payload: admin_message::PayloadVariant,
) -> anyhow::Result<()> {
    let admin_msg = protobufs::AdminMessage {
        payload_variant: Some(payload),
        session_passkey: Vec::new(),
    };

    let mesh_packet = MeshPacket {
        from: ctx.node_db.my_node_num(),
        to: target_id,
        id: generate_rand_id(),
        want_ack: true,
        channel: 0,
        hop_limit: 3,
        payload_variant: Some(mesh_packet::PayloadVariant::Decoded(Data {
            portnum: PortNum::AdminApp as i32,
            payload: admin_msg.encode_to_vec(),
            want_response: false,
            ..Default::default()
        })),
        ..Default::default()
    };

    let payload_variant = Some(protobufs::to_radio::PayloadVariant::Packet(mesh_packet));
    ctx.api.send_to_radio_packet(payload_variant).await?;

    Ok(())
}
