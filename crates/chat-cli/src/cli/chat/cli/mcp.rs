use std::io::Write;

use clap::{
    Args,
    Subcommand,
};
use crossterm::{
    queue,
    style,
};

use crate::cli::chat::tool_manager::LoadingRecord;
use crate::cli::chat::{
    ChatError,
    ChatSession,
    ChatState,
};

#[deny(missing_docs)]
#[derive(Debug, PartialEq, Args)]
pub struct McpArgs {
    #[command(subcommand)]
    pub command: Option<McpSubcommand>,
}

#[derive(Debug, PartialEq, Subcommand)]
pub enum McpSubcommand {
    /// Reload MCP servers (all servers if no name specified)
    Reload {
        /// Name of the specific server to reload (optional)
        server_name: Option<String>,
    },
}

impl McpArgs {
    pub async fn execute(self, session: &mut ChatSession) -> Result<ChatState, ChatError> {
        match self.command {
            Some(McpSubcommand::Reload { server_name }) => McpArgs::execute_reload_static(session, server_name).await,
            None => {
                // Default behavior - show MCP server status
                self.execute_status(session).await
            },
        }
    }

    async fn execute_status(self, session: &mut ChatSession) -> Result<ChatState, ChatError> {
        let terminal_width = session.terminal_width();
        let still_loading = session
            .conversation
            .tool_manager
            .pending_clients()
            .await
            .into_iter()
            .map(|name| format!(" - {name}\n"))
            .collect::<Vec<_>>()
            .join("");

        for (server_name, msg) in session.conversation.tool_manager.mcp_load_record.lock().await.iter() {
            let msg = msg
                .iter()
                .map(|record| match record {
                    LoadingRecord::Err(content) | LoadingRecord::Warn(content) | LoadingRecord::Success(content) => {
                        content.clone()
                    },
                })
                .collect::<Vec<_>>()
                .join("\n--- tools refreshed ---\n");

            queue!(
                session.stderr,
                style::Print(server_name),
                style::Print("\n"),
                style::Print(format!("{}\n", "▔".repeat(terminal_width))),
                style::Print(msg),
                style::Print("\n")
            )?;
        }

        if !still_loading.is_empty() {
            queue!(
                session.stderr,
                style::Print("Still loading:\n"),
                style::Print(format!("{}\n", "▔".repeat(terminal_width))),
                style::Print(still_loading),
                style::Print("\n")
            )?;
        }

        session.stderr.flush()?;

        Ok(ChatState::PromptUser {
            skip_printing_tools: true,
        })
    }

    async fn execute_reload_static(
        session: &mut ChatSession,
        server_name: Option<String>,
    ) -> Result<ChatState, ChatError> {
        match server_name {
            Some(name) => {
                queue!(
                    session.stderr,
                    style::SetForegroundColor(style::Color::Blue),
                    style::Print(format!("Reloading MCP server: {}\n", name)),
                    style::SetForegroundColor(style::Color::Reset)
                )?;

                match session.conversation.tool_manager.reload_server(&name).await {
                    Ok(()) => {
                        queue!(
                            session.stderr,
                            style::SetForegroundColor(style::Color::Green),
                            style::Print(format!("✓ Successfully reloaded server: {}\n", name)),
                            style::SetForegroundColor(style::Color::Reset)
                        )?;
                    },
                    Err(e) => {
                        queue!(
                            session.stderr,
                            style::SetForegroundColor(style::Color::Red),
                            style::Print(format!("✗ Failed to reload server {}: {}\n", name, e)),
                            style::SetForegroundColor(style::Color::Reset)
                        )?;
                    },
                }
            },
            None => {
                queue!(
                    session.stderr,
                    style::SetForegroundColor(style::Color::Blue),
                    style::Print("Reloading all MCP servers...\n"),
                    style::SetForegroundColor(style::Color::Reset)
                )?;

                match session.conversation.tool_manager.reload_all_servers().await {
                    Ok(results) => {
                        for (server_name, result) in results {
                            match result {
                                Ok(()) => {
                                    queue!(
                                        session.stderr,
                                        style::SetForegroundColor(style::Color::Green),
                                        style::Print(format!("✓ Successfully reloaded: {}\n", server_name)),
                                        style::SetForegroundColor(style::Color::Reset)
                                    )?;
                                },
                                Err(e) => {
                                    queue!(
                                        session.stderr,
                                        style::SetForegroundColor(style::Color::Red),
                                        style::Print(format!("✗ Failed to reload {}: {}\n", server_name, e)),
                                        style::SetForegroundColor(style::Color::Reset)
                                    )?;
                                },
                            }
                        }
                    },
                    Err(e) => {
                        queue!(
                            session.stderr,
                            style::SetForegroundColor(style::Color::Red),
                            style::Print(format!("✗ Failed to reload servers: {}\n", e)),
                            style::SetForegroundColor(style::Color::Reset)
                        )?;
                    },
                }
            },
        }

        session.stderr.flush()?;

        Ok(ChatState::PromptUser {
            skip_printing_tools: true,
        })
    }
}
