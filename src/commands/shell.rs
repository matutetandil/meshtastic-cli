use std::borrow::Cow;

use async_trait::async_trait;
use colored::Colorize;
use rustyline::completion::{Completer, Pair};
use rustyline::error::ReadlineError;
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::validate::Validator;
use rustyline::{Context, Editor, Helper};

use super::{Command, CommandContext};
use crate::cli::Commands;
use crate::commands::create_command;

pub struct ShellCommand;

const SHELL_BLOCKED: &[&str] = &["shell", "completions", "config-file"];

fn get_command_names() -> Vec<String> {
    use clap::CommandFactory;
    let mut names: Vec<String> = crate::cli::Cli::command()
        .get_subcommands()
        .map(|sub| sub.get_name().to_string())
        .filter(|n| !SHELL_BLOCKED.contains(&n.as_str()))
        .collect();
    names.extend(["help", "exit", "quit"].iter().map(|s| s.to_string()));
    names.sort();
    names
}

struct ShellHelper {
    command_names: Vec<String>,
}

impl Helper for ShellHelper {}
impl Validator for ShellHelper {}
impl Hinter for ShellHelper {
    type Hint = String;
    fn hint(&self, _line: &str, _pos: usize, _ctx: &Context<'_>) -> Option<String> {
        None
    }
}
impl Highlighter for ShellHelper {
    fn highlight_prompt<'b, 's: 'b, 'p: 'b>(&'s self, prompt: &'p str, _: bool) -> Cow<'b, str> {
        Cow::Owned(prompt.cyan().bold().to_string())
    }
}

impl Completer for ShellHelper {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &Context<'_>,
    ) -> rustyline::Result<(usize, Vec<Pair>)> {
        let prefix = &line[..pos];
        let word_start = prefix.rfind(' ').map(|i| i + 1).unwrap_or(0);
        let partial = &prefix[word_start..];

        let candidates: Vec<Pair> = self
            .command_names
            .iter()
            .filter(|name| name.starts_with(partial))
            .map(|name| Pair {
                display: name.clone(),
                replacement: name.clone(),
            })
            .collect();

        Ok((word_start, candidates))
    }
}

#[async_trait]
impl Command for ShellCommand {
    async fn execute(&self, ctx: &mut CommandContext) -> anyhow::Result<()> {
        println!(
            "{} Meshtastic interactive shell. Type {} for help, {} to exit.\n",
            "->".cyan(),
            "help".bold(),
            "exit".bold()
        );

        let history_path = crate::config_file::config_dir().join("history.txt");
        let _ = std::fs::create_dir_all(crate::config_file::config_dir());

        let helper = ShellHelper {
            command_names: get_command_names(),
        };
        let mut rl = Editor::new()?;
        rl.set_helper(Some(helper));
        let _ = rl.load_history(&history_path);

        loop {
            match rl.readline("mesh> ") {
                Ok(line) => {
                    let trimmed = line.trim();
                    if trimmed.is_empty() {
                        continue;
                    }

                    rl.add_history_entry(trimmed)?;

                    match trimmed {
                        "exit" | "quit" => {
                            println!("Goodbye.");
                            break;
                        }
                        "help" => {
                            print_help();
                            continue;
                        }
                        _ => {}
                    }

                    // Block commands that don't work inside the shell
                    let first_word = trimmed.split_whitespace().next().unwrap_or("");
                    if SHELL_BLOCKED.contains(&first_word) {
                        println!(
                            "{} Use '{}' from the command line, not inside the shell.",
                            "x".red(),
                            first_word
                        );
                        continue;
                    }

                    // Parse the line as a CLI command
                    let args = match shlex::split(trimmed) {
                        Some(args) => args,
                        None => {
                            println!("{} Invalid input (unmatched quotes).", "x".red());
                            continue;
                        }
                    };

                    // Build a full argv with the binary name prepended
                    let mut full_args = vec!["meshtastic-cli".to_string()];
                    full_args.extend(args);

                    match parse_shell_command(&full_args) {
                        Ok(cmd_enum) => match create_command(&cmd_enum, false) {
                            Ok(command) => {
                                if let Err(e) = command.execute(ctx).await {
                                    println!("{} {}", "Error:".red(), e);
                                }
                            }
                            Err(e) => {
                                println!("{} {}", "Error:".red(), e);
                            }
                        },
                        Err(msg) => {
                            println!("{}", msg);
                        }
                    }

                    println!();
                }
                Err(ReadlineError::Interrupted) => {
                    println!("^C");
                    continue;
                }
                Err(ReadlineError::Eof) => {
                    println!("Goodbye.");
                    break;
                }
                Err(e) => {
                    println!("{} Readline error: {}", "x".red(), e);
                    break;
                }
            }
        }

        let _ = rl.save_history(&history_path);
        Ok(())
    }
}

fn parse_shell_command(args: &[String]) -> Result<Commands, String> {
    use clap::Parser;

    match crate::cli::Cli::try_parse_from(args) {
        Ok(cli) => match cli.command {
            Some(cmd) => Ok(cmd),
            None => Err("No command specified. Type 'help' for available commands.".to_string()),
        },
        Err(e) => Err(e.to_string()),
    }
}

fn print_help() {
    use clap::CommandFactory;

    println!("{}", "Available commands:".bold());

    let app = crate::cli::Cli::command();
    let mut entries: Vec<(&str, String)> = app
        .get_subcommands()
        .filter(|sub| !SHELL_BLOCKED.contains(&sub.get_name()))
        .map(|sub| {
            let desc = sub.get_about().map(|a| a.to_string()).unwrap_or_default();
            (sub.get_name(), desc)
        })
        .collect();
    entries.sort_by_key(|(name, _)| *name);
    entries.push(("exit/quit", "Exit the shell".to_string()));

    for (name, desc) in &entries {
        println!("  {:<16} {}", name, desc);
    }
    println!(
        "\n{}",
        "Type a command with --help for detailed usage.".dimmed()
    );
}
