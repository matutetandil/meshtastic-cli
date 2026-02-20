use anyhow::bail;
use async_trait::async_trait;
use colored::Colorize;
use meshtastic::protobufs::{self, admin_message};

use super::device::send_admin_message;
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
        let target_num = resolve_node_num(&self.destination, &ctx)?;
        let label = format_node_label(target_num, &ctx);

        let my_id = ctx.node_db.my_node_num();

        println!(
            "{} Removing node {} from local NodeDB...",
            "->".cyan(),
            label.bold()
        );

        send_admin_message(
            &mut ctx,
            my_id,
            admin_message::PayloadVariant::RemoveByNodenum(target_num),
        )
        .await?;

        println!("{} Node {} removed.", "ok".green(), label);

        Ok(())
    }
}

// ── SetFavoriteCommand ────────────────────────────────────────────

pub struct SetFavoriteCommand {
    pub destination: DestinationSpec,
}

#[async_trait]
impl Command for SetFavoriteCommand {
    async fn execute(self: Box<Self>, mut ctx: CommandContext) -> anyhow::Result<()> {
        let target_num = resolve_node_num(&self.destination, &ctx)?;
        let label = format_node_label(target_num, &ctx);

        let my_id = ctx.node_db.my_node_num();

        println!("{} Setting {} as favorite...", "->".cyan(), label.bold());

        send_admin_message(
            &mut ctx,
            my_id,
            admin_message::PayloadVariant::SetFavoriteNode(target_num),
        )
        .await?;

        println!("{} {} marked as favorite.", "ok".green(), label);
        Ok(())
    }
}

// ── RemoveFavoriteCommand ────────────────────────────────────────

pub struct RemoveFavoriteCommand {
    pub destination: DestinationSpec,
}

#[async_trait]
impl Command for RemoveFavoriteCommand {
    async fn execute(self: Box<Self>, mut ctx: CommandContext) -> anyhow::Result<()> {
        let target_num = resolve_node_num(&self.destination, &ctx)?;
        let label = format_node_label(target_num, &ctx);

        let my_id = ctx.node_db.my_node_num();

        println!(
            "{} Removing {} from favorites...",
            "->".cyan(),
            label.bold()
        );

        send_admin_message(
            &mut ctx,
            my_id,
            admin_message::PayloadVariant::RemoveFavoriteNode(target_num),
        )
        .await?;

        println!("{} {} removed from favorites.", "ok".green(), label);
        Ok(())
    }
}

// ── SetIgnoredCommand ────────────────────────────────────────────

pub struct SetIgnoredCommand {
    pub destination: DestinationSpec,
}

#[async_trait]
impl Command for SetIgnoredCommand {
    async fn execute(self: Box<Self>, mut ctx: CommandContext) -> anyhow::Result<()> {
        let target_num = resolve_node_num(&self.destination, &ctx)?;
        let label = format_node_label(target_num, &ctx);

        let my_id = ctx.node_db.my_node_num();

        println!("{} Setting {} as ignored...", "->".cyan(), label.bold());

        send_admin_message(
            &mut ctx,
            my_id,
            admin_message::PayloadVariant::SetIgnoredNode(target_num),
        )
        .await?;

        println!("{} {} marked as ignored.", "ok".green(), label);
        Ok(())
    }
}

// ── RemoveIgnoredCommand ─────────────────────────────────────────

pub struct RemoveIgnoredCommand {
    pub destination: DestinationSpec,
}

#[async_trait]
impl Command for RemoveIgnoredCommand {
    async fn execute(self: Box<Self>, mut ctx: CommandContext) -> anyhow::Result<()> {
        let target_num = resolve_node_num(&self.destination, &ctx)?;
        let label = format_node_label(target_num, &ctx);

        let my_id = ctx.node_db.my_node_num();

        println!(
            "{} Removing {} from ignored list...",
            "->".cyan(),
            label.bold()
        );

        send_admin_message(
            &mut ctx,
            my_id,
            admin_message::PayloadVariant::RemoveIgnoredNode(target_num),
        )
        .await?;

        println!("{} {} removed from ignored list.", "ok".green(), label);
        Ok(())
    }
}

// ── SetUnmessageableCommand ──────────────────────────────────────

pub struct SetUnmessageableCommand {
    pub value: bool,
}

#[async_trait]
impl Command for SetUnmessageableCommand {
    async fn execute(self: Box<Self>, mut ctx: CommandContext) -> anyhow::Result<()> {
        let existing_user = ctx
            .node_db
            .local_node()
            .and_then(|n| n.user.clone())
            .unwrap_or_default();

        let label = if self.value {
            "unmessageable"
        } else {
            "messageable"
        };

        println!("{} Setting node as {}...", "->".cyan(), label);

        let new_user = protobufs::User {
            id: existing_user.id,
            long_name: existing_user.long_name,
            short_name: existing_user.short_name,
            hw_model: existing_user.hw_model,
            is_licensed: existing_user.is_licensed,
            role: existing_user.role,
            public_key: existing_user.public_key,
            is_unmessagable: Some(self.value),
            ..Default::default()
        };

        ctx.api.update_user(&mut ctx.router, new_user).await?;

        println!("{} Node marked as {}.", "ok".green(), label);

        Ok(())
    }
}

// ── Helpers ──────────────────────────────────────────────────────

fn resolve_node_num(destination: &DestinationSpec, ctx: &CommandContext) -> anyhow::Result<u32> {
    match destination {
        DestinationSpec::NodeId(id) => Ok(*id),
        DestinationSpec::NodeName(name) => {
            let (_, _) = resolve_destination(destination, &ctx.node_db)?;
            let matches = ctx.node_db.find_by_name(name);
            Ok(matches[0].0)
        }
        DestinationSpec::Broadcast => {
            bail!("Must specify a node via --dest or --to")
        }
    }
}

fn format_node_label(node_num: u32, ctx: &CommandContext) -> String {
    ctx.node_db
        .node_name(node_num)
        .map(|name| format!("{} (!{:08x})", name, node_num))
        .unwrap_or_else(|| format!("!{:08x}", node_num))
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
