use async_trait::async_trait;
use chrono::{DateTime, Utc};
use colored::Colorize;
use meshtastic::protobufs::{self, HardwareModel, NodeInfo};
use serde::Serialize;

use super::{Command, CommandContext};

const DEFAULT_FIELDS: &[&str] = &["id", "name", "battery", "snr", "hops", "last_heard"];
const ALL_FIELDS: &[&str] = &[
    "id",
    "name",
    "battery",
    "snr",
    "hops",
    "last_heard",
    "hw_model",
    "role",
    "position",
];

pub struct NodesCommand {
    pub fields: Option<Vec<String>>,
}

#[derive(Serialize)]
struct NodeJson {
    id: String,
    num: u32,
    name: String,
    short_name: String,
    hw_model: String,
    role: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    battery: Option<u32>,
    snr: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    hops: Option<u32>,
    last_heard: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    latitude: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    longitude: Option<f64>,
    is_local: bool,
}

fn node_to_json(node: &NodeInfo, is_local: bool) -> NodeJson {
    let user = node.user.as_ref();
    let pos = node.position.as_ref();

    let (lat, lon) = pos
        .map(|p| {
            let la = p.latitude_i.unwrap_or(0) as f64 / 1e7;
            let lo = p.longitude_i.unwrap_or(0) as f64 / 1e7;
            if la == 0.0 && lo == 0.0 {
                (None, None)
            } else {
                (Some(la), Some(lo))
            }
        })
        .unwrap_or((None, None));

    NodeJson {
        id: format!("!{:08x}", node.num),
        num: node.num,
        name: user
            .map(|u| u.long_name.clone())
            .unwrap_or_else(|| "Unknown".into()),
        short_name: user.map(|u| u.short_name.clone()).unwrap_or_default(),
        hw_model: user
            .map(|u| {
                HardwareModel::try_from(u.hw_model)
                    .map(|m| m.as_str_name().to_string())
                    .unwrap_or_else(|_| format!("Unknown({})", u.hw_model))
            })
            .unwrap_or_else(|| "N/A".into()),
        role: user
            .map(|u| {
                protobufs::config::device_config::Role::try_from(u.role)
                    .map(|r| format!("{:?}", r))
                    .unwrap_or_else(|_| u.role.to_string())
            })
            .unwrap_or_else(|| "N/A".into()),
        battery: node.device_metrics.as_ref().and_then(|m| m.battery_level),
        snr: node.snr,
        hops: node.hops_away,
        last_heard: node.last_heard,
        latitude: lat,
        longitude: lon,
        is_local,
    }
}

#[async_trait]
impl Command for NodesCommand {
    async fn execute(&self, ctx: &mut CommandContext) -> anyhow::Result<()> {
        let local_node_num = ctx.node_db.my_node_num();
        let nodes = ctx.node_db.nodes();

        if nodes.is_empty() {
            if ctx.json {
                println!("[]");
            } else {
                println!("No nodes found in mesh.");
            }
            return Ok(());
        }

        let mut sorted_nodes: Vec<_> = nodes.values().collect();
        sorted_nodes.sort_by_key(|n| n.num);

        if ctx.json {
            let json_nodes: Vec<NodeJson> = sorted_nodes
                .iter()
                .map(|n| node_to_json(n, n.num == local_node_num))
                .collect();
            println!("{}", serde_json::to_string_pretty(&json_nodes)?);
            return Ok(());
        }

        let fields: Vec<&str> = match &self.fields {
            Some(f) => f.iter().map(|s| s.as_str()).collect(),
            None => DEFAULT_FIELDS.to_vec(),
        };

        for f in &fields {
            if !ALL_FIELDS.contains(f) {
                anyhow::bail!(
                    "Unknown field '{}'. Valid fields: {}",
                    f,
                    ALL_FIELDS.join(", ")
                );
            }
        }

        let header = build_header(&fields);
        println!("{header}");
        let sep_len: usize = fields.iter().map(|f| field_width(f) + 1).sum();
        println!("{}", "-".repeat(sep_len));

        for node in sorted_nodes {
            let is_local = node.num == local_node_num;
            let line = build_row(node, &fields);

            if is_local {
                println!("{}", line.green());
            } else {
                println!("{}", line);
            }
        }

        Ok(())
    }
}

fn field_width(field: &str) -> usize {
    match field {
        "id" => 12,
        "name" => 20,
        "battery" => 8,
        "snr" => 8,
        "hops" => 6,
        "last_heard" => 20,
        "hw_model" => 20,
        "role" => 14,
        "position" => 28,
        _ => 12,
    }
}

fn build_header(fields: &[&str]) -> String {
    fields
        .iter()
        .map(|f| {
            let label = match *f {
                "id" => "ID",
                "name" => "Name",
                "battery" => "Battery",
                "snr" => "SNR",
                "hops" => "Hops",
                "last_heard" => "Last Heard",
                "hw_model" => "Hardware",
                "role" => "Role",
                "position" => "Position",
                _ => *f,
            };
            format!("{:<width$}", label, width = field_width(f))
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn build_row(node: &NodeInfo, fields: &[&str]) -> String {
    fields
        .iter()
        .map(|f| {
            let value = get_field_value(node, f);
            format!("{:<width$}", value, width = field_width(f))
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn get_field_value(node: &NodeInfo, field: &str) -> String {
    match field {
        "id" => format!("!{:08x}", node.num),
        "name" => node
            .user
            .as_ref()
            .map(|u| u.long_name.clone())
            .unwrap_or_else(|| "Unknown".into()),
        "battery" => node
            .device_metrics
            .as_ref()
            .and_then(|m| m.battery_level)
            .map(|b| format!("{}%", b))
            .unwrap_or_else(|| "N/A".into()),
        "snr" => {
            if node.snr != 0.0 {
                format!("{:.1}", node.snr)
            } else {
                "N/A".into()
            }
        }
        "hops" => node
            .hops_away
            .map(|h| h.to_string())
            .unwrap_or_else(|| "N/A".into()),
        "last_heard" => {
            if node.last_heard > 0 {
                DateTime::<Utc>::from_timestamp(node.last_heard as i64, 0)
                    .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                    .unwrap_or_else(|| "Invalid".into())
            } else {
                "Never".into()
            }
        }
        "hw_model" => node
            .user
            .as_ref()
            .map(|u| {
                HardwareModel::try_from(u.hw_model)
                    .map(|m| m.as_str_name().to_string())
                    .unwrap_or_else(|_| format!("Unknown({})", u.hw_model))
            })
            .unwrap_or_else(|| "N/A".into()),
        "role" => node
            .user
            .as_ref()
            .map(|u| {
                protobufs::config::device_config::Role::try_from(u.role)
                    .map(|r| format!("{:?}", r))
                    .unwrap_or_else(|_| u.role.to_string())
            })
            .unwrap_or_else(|| "N/A".into()),
        "position" => node
            .position
            .as_ref()
            .map(|p| {
                let lat = p.latitude_i.unwrap_or(0) as f64 / 1e7;
                let lon = p.longitude_i.unwrap_or(0) as f64 / 1e7;
                if lat == 0.0 && lon == 0.0 {
                    "N/A".to_string()
                } else {
                    format!("{:.5}, {:.5}", lat, lon)
                }
            })
            .unwrap_or_else(|| "N/A".into()),
        _ => "N/A".into(),
    }
}
