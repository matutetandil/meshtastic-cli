use anyhow::bail;
use async_trait::async_trait;
use colored::Colorize;
use meshtastic::protobufs::{self, admin_message};
use serde::Serialize;

use super::{Command, CommandContext};

#[derive(Serialize)]
struct PositionJson {
    latitude: f64,
    longitude: f64,
    altitude: i32,
    sats_in_view: u32,
    fix_quality: u32,
    fix_type: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    gps_accuracy: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    ground_speed: Option<u32>,
    location_source: String,
}

/// Parse position flags from either a numeric bitmask or comma-separated flag names.
pub fn parse_position_flags(input: &str) -> anyhow::Result<u32> {
    // Try numeric first (decimal or 0x hex)
    if let Some(hex) = input
        .strip_prefix("0x")
        .or_else(|| input.strip_prefix("0X"))
    {
        return u32::from_str_radix(hex, 16)
            .map_err(|_| anyhow::anyhow!("Invalid hex flags: '{}'", input));
    }
    if let Ok(n) = input.parse::<u32>() {
        return Ok(n);
    }

    // Parse as comma-separated flag names
    let mut flags: u32 = 0;
    for name in input.split(',') {
        let flag = match name.trim().to_uppercase().as_str() {
            "ALTITUDE" => 1,
            "ALTITUDE_MSL" => 2,
            "GEOIDAL_SEPARATION" => 4,
            "DOP" => 8,
            "HVDOP" => 16,
            "SATINVIEW" => 32,
            "SEQ_NO" => 64,
            "TIMESTAMP" => 128,
            "HEADING" => 256,
            "SPEED" => 512,
            other => bail!(
                "Unknown position flag '{}'. Valid flags: ALTITUDE, ALTITUDE_MSL, GEOIDAL_SEPARATION, DOP, HVDOP, SATINVIEW, SEQ_NO, TIMESTAMP, HEADING, SPEED",
                other
            ),
        };
        flags |= flag;
    }
    Ok(flags)
}

// ── PositionRemoveCommand ────────────────────────────────────────

pub struct PositionRemoveCommand;

#[async_trait]
impl Command for PositionRemoveCommand {
    async fn execute(&self, ctx: &mut CommandContext) -> anyhow::Result<()> {
        let my_id = ctx.node_db.my_node_num();

        println!("{} Removing fixed position...", "->".cyan());

        super::admin::send_admin_message(
            ctx,
            my_id,
            admin_message::PayloadVariant::RemoveFixedPosition(true),
        )
        .await?;

        println!(
            "{} Fixed position removed. Device will use GPS if available.",
            "ok".green()
        );

        Ok(())
    }
}

// ── PositionGetCommand ────────────────────────────────────────────

pub struct PositionGetCommand;

#[async_trait]
impl Command for PositionGetCommand {
    async fn execute(&self, ctx: &mut CommandContext) -> anyhow::Result<()> {
        let local = ctx.node_db.local_node();

        let position = local.and_then(|n| n.position);

        match position {
            Some(pos) => {
                let lat = pos.latitude_i.unwrap_or(0) as f64 / 1e7;
                let lon = pos.longitude_i.unwrap_or(0) as f64 / 1e7;
                let alt = pos.altitude.unwrap_or(0);
                let source = protobufs::position::LocSource::try_from(pos.location_source)
                    .map(|s| s.as_str_name().to_string())
                    .unwrap_or_else(|_| pos.location_source.to_string());

                if ctx.json {
                    let json = PositionJson {
                        latitude: lat,
                        longitude: lon,
                        altitude: alt,
                        sats_in_view: pos.sats_in_view,
                        fix_quality: pos.fix_quality,
                        fix_type: pos.fix_type,
                        gps_accuracy: if pos.gps_accuracy > 0 {
                            Some(pos.gps_accuracy)
                        } else {
                            None
                        },
                        ground_speed: pos.ground_speed.filter(|&v| v > 0),
                        location_source: source,
                    };
                    println!("{}", serde_json::to_string_pretty(&json)?);
                    return Ok(());
                }

                println!("{}", "Position".bold().underline());
                println!("  {:<20} {:.7}", "latitude:".dimmed(), lat);
                println!("  {:<20} {:.7}", "longitude:".dimmed(), lon);
                println!("  {:<20} {} m", "altitude:".dimmed(), alt);
                println!("  {:<20} {}", "sats_in_view:".dimmed(), pos.sats_in_view);
                println!("  {:<20} {}", "fix_quality:".dimmed(), pos.fix_quality);
                println!("  {:<20} {}", "fix_type:".dimmed(), pos.fix_type);

                if pos.gps_accuracy > 0 {
                    println!("  {:<20} {} mm", "gps_accuracy:".dimmed(), pos.gps_accuracy);
                }
                if pos.ground_speed.unwrap_or(0) > 0 {
                    println!(
                        "  {:<20} {} m/s",
                        "ground_speed:".dimmed(),
                        pos.ground_speed.unwrap_or(0)
                    );
                }
                println!("  {:<20} {}", "location_source:".dimmed(), source);
            }
            None => {
                if ctx.json {
                    println!("null");
                    return Ok(());
                }
                println!("{}", "(no position available)".dimmed());
            }
        }

        Ok(())
    }
}

// ── PositionSetCommand ────────────────────────────────────────────

pub struct PositionSetCommand {
    pub latitude: f64,
    pub longitude: f64,
    pub altitude: i32,
    pub flags: Option<u32>,
}

#[async_trait]
impl Command for PositionSetCommand {
    async fn execute(&self, ctx: &mut CommandContext) -> anyhow::Result<()> {
        let lat_i = (self.latitude * 1e7) as i32;
        let lon_i = (self.longitude * 1e7) as i32;

        println!(
            "{} Setting fixed position: lat={}, lon={}, alt={}m",
            "->".cyan(),
            self.latitude,
            self.longitude,
            self.altitude
        );

        let position = protobufs::Position {
            latitude_i: Some(lat_i),
            longitude_i: Some(lon_i),
            altitude: Some(self.altitude),
            time: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs() as u32,
            location_source: protobufs::position::LocSource::LocManual as i32,
            ..Default::default()
        };

        let my_id = ctx.node_db.my_node_num();
        super::admin::send_admin_message(
            ctx,
            my_id,
            admin_message::PayloadVariant::SetFixedPosition(position),
        )
        .await?;

        println!(
            "{} Fixed position set. Device will use this location instead of GPS.",
            "ok".green()
        );

        if let Some(flags) = self.flags {
            let mut pos_config = ctx.node_db.local_config().position.unwrap_or_default();
            pos_config.position_flags = flags;

            let config_packet = protobufs::Config {
                payload_variant: Some(protobufs::config::PayloadVariant::Position(pos_config)),
            };

            ctx.api
                .update_config(&mut ctx.router, config_packet)
                .await?;

            println!("{} Position flags set to {}.", "ok".green(), flags);
        }

        Ok(())
    }
}
