use std::time::{Duration, Instant};

use anyhow::bail;
use async_trait::async_trait;
use colored::Colorize;
use meshtastic::protobufs::from_radio::PayloadVariant;
use meshtastic::protobufs::mesh_packet::PayloadVariant as MeshPayload;
use meshtastic::protobufs::{self, Data, MeshPacket, PortNum, RouteDiscovery};
use meshtastic::utils::generate_rand_id;
use meshtastic::Message;

use super::{resolve_destination, Command, CommandContext, DestinationSpec};

pub struct TracerouteCommand {
    pub destination: DestinationSpec,
    pub timeout_secs: u64,
}

#[async_trait]
impl Command for TracerouteCommand {
    async fn execute(self: Box<Self>, mut ctx: CommandContext) -> anyhow::Result<()> {
        let (packet_dest, dest_label) = resolve_destination(&self.destination, &ctx.node_db)?;

        let target_node_id = match packet_dest {
            meshtastic::packet::PacketDestination::Node(node_id) => node_id.id(),
            _ => bail!("Traceroute requires a specific node destination"),
        };

        let packet_id: u32 = generate_rand_id();
        let my_node_num = ctx.node_db.my_node_num();

        let route_discovery = RouteDiscovery {
            route: vec![],
            snr_towards: vec![],
            route_back: vec![],
            snr_back: vec![],
        };
        let mut payload_bytes = Vec::new();
        route_discovery.encode(&mut payload_bytes)?;

        let mesh_packet = MeshPacket {
            from: my_node_num,
            to: target_node_id,
            id: packet_id,
            want_ack: true,
            channel: 0,
            hop_limit: 7,
            payload_variant: Some(MeshPayload::Decoded(Data {
                portnum: PortNum::TracerouteApp as i32,
                payload: payload_bytes,
                want_response: true,
                dest: target_node_id,
                ..Default::default()
            })),
            ..Default::default()
        };

        let payload_variant = Some(protobufs::to_radio::PayloadVariant::Packet(mesh_packet));

        println!(
            "{} Tracing route to {}...\n",
            "->".cyan(),
            dest_label.bold()
        );

        let start = Instant::now();

        ctx.api.send_to_radio_packet(payload_variant).await?;

        let timeout = Duration::from_secs(self.timeout_secs);

        loop {
            let remaining = timeout.saturating_sub(start.elapsed());
            if remaining.is_zero() {
                println!(
                    "{} Timeout after {}s — no traceroute response from {}",
                    "x".red(),
                    self.timeout_secs,
                    dest_label
                );
                return Ok(());
            }

            let packet = tokio::time::timeout(remaining, ctx.packet_receiver.recv()).await;

            match packet {
                Err(_) => {
                    println!(
                        "{} Timeout after {}s — no traceroute response from {}",
                        "x".red(),
                        self.timeout_secs,
                        dest_label
                    );
                    return Ok(());
                }
                Ok(None) => {
                    bail!("Disconnected while waiting for traceroute response");
                }
                Ok(Some(from_radio)) => {
                    let Some(PayloadVariant::Packet(mesh_pkt)) = from_radio.payload_variant else {
                        continue;
                    };

                    let Some(MeshPayload::Decoded(ref data)) = mesh_pkt.payload_variant else {
                        continue;
                    };

                    if data.portnum != PortNum::TracerouteApp as i32 || data.request_id != packet_id
                    {
                        continue;
                    }

                    let rtt = start.elapsed();

                    let Ok(route) = RouteDiscovery::decode(data.payload.as_slice()) else {
                        println!(
                            "{} Received traceroute response but failed to decode",
                            "?".yellow()
                        );
                        return Ok(());
                    };

                    print_route(&route, my_node_num, target_node_id, &dest_label, rtt, &ctx);

                    return Ok(());
                }
            }
        }
    }
}

fn print_route(
    route: &RouteDiscovery,
    my_node_num: u32,
    target_node_id: u32,
    dest_label: &str,
    rtt: Duration,
    ctx: &CommandContext,
) {
    let format_node = |num: u32| -> String {
        let id = format!("!{:08x}", num);
        match ctx.node_db.node_name(num) {
            Some(name) => format!("{} ({})", id, name),
            None => id,
        }
    };

    let format_snr = |snr_list: &[i32], idx: usize| -> String {
        snr_list
            .get(idx)
            .map(|&s| format!("{:.1} dB", s as f64 / 4.0))
            .unwrap_or_else(|| "? dB".to_string())
    };

    // Start: local node
    println!("  {} {}", "1".bold(), format_node(my_node_num));

    // Hops towards destination
    let mut hop = 2;
    for (i, &node_num) in route.route.iter().enumerate() {
        let snr = format_snr(&route.snr_towards, i);
        println!(
            "  {} {} {} {}",
            format!("{}", hop).bold(),
            format_node(node_num),
            "SNR:".dimmed(),
            snr
        );
        hop += 1;
    }

    // Destination
    let dest_snr = format_snr(&route.snr_towards, route.route.len());
    println!(
        "  {} {} {} {}",
        format!("{}", hop).bold(),
        format_node(target_node_id),
        "SNR:".dimmed(),
        dest_snr
    );

    // Return path (if different from forward path)
    if !route.route_back.is_empty() {
        println!("\n  {} Return path:", "<<".dimmed());
        let mut hop_back = 1;
        println!(
            "  {} {}",
            format!("{}", hop_back).bold(),
            format_node(target_node_id)
        );
        hop_back += 1;

        for (i, &node_num) in route.route_back.iter().enumerate() {
            let snr = format_snr(&route.snr_back, i);
            println!(
                "  {} {} {} {}",
                format!("{}", hop_back).bold(),
                format_node(node_num),
                "SNR:".dimmed(),
                snr
            );
            hop_back += 1;
        }

        let back_snr = format_snr(&route.snr_back, route.route_back.len());
        println!(
            "  {} {} {} {}",
            format!("{}", hop_back).bold(),
            format_node(my_node_num),
            "SNR:".dimmed(),
            back_snr
        );
    }

    println!(
        "\n{} Route to {} completed in {:.1}s ({} hops)",
        "ok".green(),
        dest_label.bold(),
        rtt.as_secs_f64(),
        route.route.len() + 1
    );
}
