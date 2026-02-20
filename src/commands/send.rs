use async_trait::async_trait;
use colored::Colorize;
use meshtastic::types::MeshChannel;

use super::{resolve_destination, Command, CommandContext, DestinationSpec};

pub struct SendCommand {
    pub message: String,
    pub destination: DestinationSpec,
    pub channel: MeshChannel,
}

#[async_trait]
impl Command for SendCommand {
    async fn execute(self: Box<Self>, mut ctx: CommandContext) -> anyhow::Result<()> {
        let (packet_dest, dest_label) = resolve_destination(&self.destination, &ctx.node_db)?;

        ctx.api
            .send_text(
                &mut ctx.router,
                self.message.clone(),
                packet_dest,
                true,
                self.channel,
            )
            .await?;

        println!(
            "{} Message sent to {} on channel {}",
            "✓".green(),
            dest_label.bold(),
            self.channel.channel()
        );

        Ok(())
    }
}
