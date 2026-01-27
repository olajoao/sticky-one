use clap::{Parser, Subcommand};
use sticky_one::clipboard::write_entry;
use sticky_one::daemon::{is_running, stop, Daemon};
use sticky_one::error::StickyError;
use sticky_one::Storage;

#[derive(Parser)]
#[command(name = "sticky_one")]
#[command(about = "Clipboard manager with 12-hour history")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the clipboard monitoring daemon
    Daemon,
    /// Stop the running daemon
    Stop,
    /// Check daemon status
    Status,
    /// List recent clipboard entries
    List {
        /// Max entries to show
        #[arg(short, long, default_value = "20")]
        limit: usize,
    },
    /// Copy a specific entry back to clipboard
    Get {
        /// Entry ID
        id: i64,
    },
    /// Search text/link entries
    Search {
        /// Search query
        query: String,
        /// Max results
        #[arg(short, long, default_value = "20")]
        limit: usize,
    },
    /// Clear all history
    Clear,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Daemon => run_daemon().await,
        Commands::Stop => cmd_stop(),
        Commands::Status => cmd_status(),
        Commands::List { limit } => cmd_list(limit),
        Commands::Get { id } => cmd_get(id),
        Commands::Search { query, limit } => cmd_search(&query, limit),
        Commands::Clear => cmd_clear(),
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

async fn run_daemon() -> sticky_one::Result<()> {
    if let Some(pid) = is_running() {
        return Err(StickyError::DaemonRunning(pid));
    }

    println!("Starting daemon...");
    let mut daemon = Daemon::new()?;
    daemon.run().await
}

fn cmd_stop() -> sticky_one::Result<()> {
    stop()?;
    println!("Daemon stopped");
    Ok(())
}

fn cmd_status() -> sticky_one::Result<()> {
    match is_running() {
        Some(pid) => println!("Daemon running (pid: {})", pid),
        None => println!("Daemon not running"),
    }
    Ok(())
}

fn cmd_list(limit: usize) -> sticky_one::Result<()> {
    let storage = Storage::open()?;
    let entries = storage.list(limit)?;

    if entries.is_empty() {
        println!("No entries");
        return Ok(());
    }

    for entry in entries {
        let ts = chrono::DateTime::from_timestamp(entry.created_at, 0)
            .map(|dt| dt.format("%H:%M:%S").to_string())
            .unwrap_or_else(|| "???".into());

        println!(
            "{:>4} │ {:>5} │ {} │ {}",
            entry.id,
            entry.content_type.as_str(),
            ts,
            entry.display_preview(50)
        );
    }

    Ok(())
}

fn cmd_get(id: i64) -> sticky_one::Result<()> {
    let storage = Storage::open()?;
    let entry = storage.get_by_id(id)?;
    write_entry(&entry)?;
    println!("Copied entry {} to clipboard", id);
    Ok(())
}

fn cmd_search(query: &str, limit: usize) -> sticky_one::Result<()> {
    let storage = Storage::open()?;
    let entries = storage.search(query, limit)?;

    if entries.is_empty() {
        println!("No matches for '{}'", query);
        return Ok(());
    }

    for entry in entries {
        let ts = chrono::DateTime::from_timestamp(entry.created_at, 0)
            .map(|dt| dt.format("%H:%M:%S").to_string())
            .unwrap_or_else(|| "???".into());

        println!(
            "{:>4} │ {:>5} │ {} │ {}",
            entry.id,
            entry.content_type.as_str(),
            ts,
            entry.display_preview(50)
        );
    }

    Ok(())
}

fn cmd_clear() -> sticky_one::Result<()> {
    let storage = Storage::open()?;
    let count = storage.clear()?;
    println!("Cleared {} entries", count);
    Ok(())
}
