use anyhow::bail;
use async_trait::async_trait;
use colored::Colorize;
use meshtastic::protobufs::{self, admin_message};

use super::{resolve_destination, Command, CommandContext, DestinationSpec};

// ── SetOwnerCommand ───────────────────────────────────────────────

pub struct SetOwnerCommand {
    pub long_name: String,
    pub short_name: Option<String>,
}

#[async_trait]
impl Command for SetOwnerCommand {
    async fn execute(self: Box<Self>, mut ctx: CommandContext) -> anyhow::Result<()> {
        if self.long_name.len() > 40 {
            bail!(
                "Long name must be 40 characters or less, got {}",
                self.long_name.len()
            );
        }

        let short_name = match &self.short_name {
            Some(s) => {
                if s.len() > 5 {
                    bail!("Short name must be 5 characters or less, got {}", s.len());
                }
                s.clone()
            }
            None => abbreviate(&self.long_name),
        };

        let existing_user = ctx
            .node_db
            .local_node()
            .and_then(|n| n.user.clone())
            .unwrap_or_default();

        println!(
            "{} Setting owner: long_name={}, short_name={}",
            "->".cyan(),
            self.long_name.bold(),
            short_name.bold()
        );

        let new_user = protobufs::User {
            id: existing_user.id,
            long_name: self.long_name.clone(),
            short_name: short_name.clone(),
            hw_model: existing_user.hw_model,
            is_licensed: existing_user.is_licensed,
            role: existing_user.role,
            public_key: existing_user.public_key,
            ..Default::default()
        };

        ctx.api.update_user(&mut ctx.router, new_user).await?;

        println!(
            "{} Owner updated: {} ({})",
            "ok".green(),
            self.long_name,
            short_name
        );

        Ok(())
    }
}

// ── RemoveNodeCommand ──────────────────────────────────────────────

pub struct RemoveNodeCommand {
    pub destination: DestinationSpec,
}

#[async_trait]
impl Command for RemoveNodeCommand {
    async fn execute(self: Box<Self>, mut ctx: CommandContext) -> anyhow::Result<()> {
        let (_, label) = resolve_destination(&self.destination, &ctx.node_db)?;

        let target_num = match &self.destination {
            DestinationSpec::NodeId(id) => *id,
            DestinationSpec::NodeName(_) => {
                let matches = match &self.destination {
                    DestinationSpec::NodeName(name) => ctx.node_db.find_by_name(name),
                    _ => unreachable!(),
                };
                matches[0].0
            }
            DestinationSpec::Broadcast => bail!("Must specify a node to remove via --dest or --to"),
        };

        println!(
            "{} Removing node {} from local NodeDB...",
            "->".cyan(),
            label.bold()
        );

        let my_id = ctx.node_db.my_node_num();
        super::device::send_admin_message(
            &mut ctx,
            my_id,
            admin_message::PayloadVariant::RemoveByNodenum(target_num),
        )
        .await?;

        println!("{} Node {} removed.", "ok".green(), label);

        Ok(())
    }
}

/// Generate a short name from a long name by taking the first character
/// of up to the first 4 words, or the first 4 characters if single word.
fn abbreviate(long_name: &str) -> String {
    let words: Vec<&str> = long_name.split_whitespace().collect();
    if words.len() >= 2 {
        words
            .iter()
            .take(4)
            .filter_map(|w| w.chars().next())
            .collect::<String>()
            .to_uppercase()
    } else {
        long_name.chars().take(4).collect::<String>().to_uppercase()
    }
}
