mod client;
mod output;
mod types;

use anyhow::Result;
use clap::{Parser, Subcommand};

use client::SyncfuClient;
use types::*;

const DEFAULT_SERVER: &str = "http://127.0.0.1:9868";

#[derive(Parser)]
#[command(name = "syncfu", about = "CLI for syncfu — send notifications from anywhere")]
#[command(version, propagate_version = true)]
struct Cli {
    /// Server URL
    #[arg(long, env = "SYNCFU_SERVER", default_value = DEFAULT_SERVER, global = true)]
    server: String,

    /// Force JSON output
    #[arg(long, global = true)]
    json: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Send a notification
    Send {
        /// Notification body text
        body: String,

        /// Title
        #[arg(short, long, default_value = "Notification")]
        title: String,

        /// Sender name
        #[arg(short, long, default_value_t = default_sender())]
        sender: String,

        /// Priority: low, normal, high, critical
        #[arg(short, long, default_value = "normal")]
        priority: Priority,

        /// Lucide icon name
        #[arg(short, long)]
        icon: Option<String>,

        /// Timeout: "never", "default", or seconds
        #[arg(long)]
        timeout: Option<String>,

        /// Action button: "id:label:style" (repeatable)
        #[arg(short, long = "action")]
        actions: Vec<String>,

        /// Progress value 0.0-1.0
        #[arg(long)]
        progress: Option<f64>,

        /// Progress label
        #[arg(long)]
        progress_label: Option<String>,

        /// Progress style: bar, ring
        #[arg(long, default_value = "bar")]
        progress_style: ProgressStyle,

        /// Group/category
        #[arg(long)]
        group: Option<String>,

        /// Theme name
        #[arg(long)]
        theme: Option<String>,

        /// Sound name
        #[arg(long)]
        sound: Option<String>,

        /// Google Font name
        #[arg(long)]
        font: Option<String>,

        /// Webhook callback URL
        #[arg(long)]
        callback_url: Option<String>,

        /// Style overrides as JSON
        #[arg(long)]
        style_json: Option<String>,
    },

    /// Update an existing notification
    Update {
        /// Notification ID
        id: String,

        /// New body text
        #[arg(long)]
        body: Option<String>,

        /// New progress value 0.0-1.0
        #[arg(long)]
        progress: Option<f64>,

        /// Progress label
        #[arg(long)]
        progress_label: Option<String>,

        /// Progress style: bar, ring
        #[arg(long, default_value = "bar")]
        progress_style: ProgressStyle,
    },

    /// Trigger an action on a notification
    Action {
        /// Notification ID
        id: String,
        /// Action ID to trigger
        action_id: String,
    },

    /// Dismiss a notification
    Dismiss {
        /// Notification ID
        id: String,
    },

    /// Dismiss all active notifications
    DismissAll,

    /// List active notifications (JSON)
    List,

    /// Check server health (JSON)
    Health,
}

fn default_sender() -> String {
    std::env::var("USER")
        .or_else(|_| std::env::var("USERNAME"))
        .unwrap_or_else(|_| "syncfu-cli".to_string())
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let cli = Cli::parse();
    let client = SyncfuClient::new(&cli.server);

    let result = run(cli, &client).await;
    if let Err(e) = result {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}

async fn run(cli: Cli, client: &SyncfuClient) -> Result<()> {
    match cli.command {
        Commands::Send {
            body,
            title,
            sender,
            priority,
            icon,
            timeout,
            actions,
            progress,
            progress_label,
            progress_style,
            group,
            theme,
            sound,
            font: _,
            callback_url,
            style_json,
        } => {
            let parsed_actions: Vec<Action> = actions
                .iter()
                .map(|s| parse_action_spec(s))
                .collect::<Result<Vec<_>, _>>()
                .map_err(|e| anyhow::anyhow!(e))?;

            let progress_info = progress.map(|value| ProgressInfo {
                value,
                label: progress_label,
                style: progress_style,
            });

            let style: Option<StyleOverrides> = match style_json {
                Some(ref json) => Some(serde_json::from_str(json)?),
                None => None,
            };

            let req = NotifyRequest {
                sender,
                title,
                body,
                icon,
                priority,
                timeout: timeout.map(|t| parse_timeout(&t)),
                actions: parsed_actions,
                progress: progress_info,
                group,
                theme,
                sound,
                callback_url,
                style,
            };

            let resp = client.send_notification(&req).await?;
            output::print_send_result(&resp, cli.json);
        }

        Commands::Update {
            id,
            body,
            progress,
            progress_label,
            progress_style,
        } => {
            let progress_info = progress.map(|value| ProgressInfo {
                value,
                label: progress_label,
                style: progress_style,
            });

            let req = UpdateRequest {
                body,
                progress: progress_info,
            };
            client.update_notification(&id, &req).await?;
            eprintln!("Updated: {id}");
        }

        Commands::Action { id, action_id } => {
            let resp = client.trigger_action(&id, &action_id).await?;
            output::print_action_result(&resp);
        }

        Commands::Dismiss { id } => {
            client.dismiss(&id).await?;
            eprintln!("Dismissed: {id}");
        }

        Commands::DismissAll => {
            let resp = client.dismiss_all().await?;
            output::print_dismiss_all(&resp, cli.json);
        }

        Commands::List => {
            let notifications = client.list_active().await?;
            output::print_active(&notifications);
        }

        Commands::Health => {
            let resp = client.health().await?;
            output::print_health(&resp);
        }
    }
    Ok(())
}
