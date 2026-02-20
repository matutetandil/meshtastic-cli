use async_trait::async_trait;
use colored::Colorize;
use meshtastic::protobufs::{self, admin_message};

use super::{Command, CommandContext};

// ── PositionGetCommand ────────────────────────────────────────────

pub struct PositionGetCommand;

#[async_trait]
impl Command for PositionGetCommand {
    async fn execute(self: Box<Self>, ctx: CommandContext) -> anyhow::Result<()> {
        let local = ctx.node_db.local_node();

        let position = local.and_then(|n| n.position);

        match position {
            Some(pos) => {
                println!("{}", "Position".bold().underline());

                let lat = pos.latitude_i.unwrap_or(0) as f64 / 1e7;
                let lon = pos.longitude_i.unwrap_or(0) as f64 / 1e7;
                let alt = pos.altitude.unwrap_or(0);

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

                let source = protobufs::position::LocSource::try_from(pos.location_source)
                    .map(|s| s.as_str_name().to_string())
                    .unwrap_or_else(|_| pos.location_source.to_string());
                println!("  {:<20} {}", "location_source:".dimmed(), source);
            }
            None => {
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
}

#[async_trait]
impl Command for PositionSetCommand {
    async fn execute(self: Box<Self>, mut ctx: CommandContext) -> anyhow::Result<()> {
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
        super::device::send_admin_message(
            &mut ctx,
            my_id,
            admin_message::PayloadVariant::SetFixedPosition(position),
        )
        .await?;

        println!(
            "{} Fixed position set. Device will use this location instead of GPS.",
            "ok".green()
        );

        Ok(())
    }
}
