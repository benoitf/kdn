/**********************************************************************
 * Copyright (C) 2026 Red Hat, Inc.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 * http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 *
 * SPDX-License-Identifier: Apache-2.0
 ***********************************************************************/

#[cfg(not(unix))]
compile_error!("kdn currently supports Unix-like systems only (raw terminal mode uses termios)");

mod commands;
mod config;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "kdn", version, about = "Manage AI agent workspaces")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Workspace operations
    #[command(alias = "ws")]
    Workspace {
        #[command(subcommand)]
        command: WorkspaceCommands,
    },
}

#[derive(Subcommand)]
enum WorkspaceCommands {
    /// List workspaces
    #[command(alias = "ls")]
    List {
        /// Output format (json)
        #[arg(short, long, value_parser = ["json"])]
        output: Option<String>,
    },
    /// Delete a workspace
    #[command(alias = "rm")]
    Delete {
        /// Workspace name
        name: String,
    },
    /// Open a new interactive shell in a workspace
    #[command(visible_alias = "connect")]
    Terminal {
        /// Workspace name
        name: String,
        /// Command to run (default: /bin/sh)
        #[arg(trailing_var_arg = true)]
        command: Vec<String>,
    },
    /// Attach to the workspace agent session (shared with the Kaiden UI)
    Agent {
        /// Workspace name
        name: String,
    },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Workspace { command } => {
            let (base_url, token) = config::api_endpoint()?;
            match command {
                WorkspaceCommands::List { output } => {
                    let json = output.as_deref() == Some("json");
                    commands::list::run(&base_url, &token, json)?;
                }
                WorkspaceCommands::Delete { name } => {
                    commands::delete::run(&base_url, &token, &name)?;
                }
                WorkspaceCommands::Terminal { name, command } => {
                    let code = commands::terminal::run(&base_url, &token, &name, &command)?;
                    std::process::exit(code);
                }
                WorkspaceCommands::Agent { name } => {
                    let code = commands::agent::run(&base_url, &token, &name)?;
                    std::process::exit(code);
                }
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn cli_is_well_formed() {
        Cli::command().debug_assert();
    }
}
