use clap::{Parser, Subcommand};
use colored::Colorize;
use sticky_one::clipboard::write_entry;
use sticky_one::daemon::{is_running, stop, Daemon};
use sticky_one::entry::ContentType;
use sticky_one::error::StickyError;
use sticky_one::Storage;
use tabled::{Table, Tabled};
use tabled::settings::{Style, Modify, object::Columns, Width};

#[derive(Parser)]
#[command(name = "syo")]
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

#[derive(Tabled)]
struct EntryRow {
    #[tabled(rename = "ID")]
    id: String,
    #[tabled(rename = "Type")]
    content_type: String,
    #[tabled(rename = "Time")]
    time: String,
    #[tabled(rename = "Preview")]
    preview: String,
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
        eprintln!("{} {}", "Error:".red().bold(), e);
        std::process::exit(1);
    }
}

async fn run_daemon() -> sticky_one::Result<()> {
    if let Some(pid) = is_running() {
        return Err(StickyError::DaemonRunning(pid));
    }

    println!("{}", "Starting daemon...".green());
    let mut daemon = Daemon::new()?;
    daemon.run().await
}

fn cmd_stop() -> sticky_one::Result<()> {
    stop()?;
    println!("{}", "Daemon stopped".yellow());
    Ok(())
}

fn cmd_status() -> sticky_one::Result<()> {
    match is_running() {
        Some(pid) => println!("{} (pid: {})", "Daemon running".green(), pid),
        None => println!("{}", "Daemon not running".yellow()),
    }
    Ok(())
}

fn format_type(ct: ContentType) -> String {
    match ct {
        ContentType::Text => "text".white().to_string(),
        ContentType::Link => "link".cyan().to_string(),
        ContentType::Image => "image".magenta().to_string(),
    }
}

fn print_entries(entries: Vec<sticky_one::Entry>) {
    if entries.is_empty() {
        println!("{}", "No entries".dimmed());
        return;
    }

    let rows: Vec<EntryRow> = entries
        .into_iter()
        .map(|e| {
            let ts = chrono::DateTime::from_timestamp(e.created_at, 0)
                .map(|dt| dt.format("%H:%M").to_string())
                .unwrap_or_else(|| "???".into());

            EntryRow {
                id: e.id.to_string().bold().to_string(),
                content_type: format_type(e.content_type),
                time: ts.dimmed().to_string(),
                preview: e.display_preview(80),
            }
        })
        .collect();

    let table = Table::new(rows)
        .with(Style::rounded())
        .with(Modify::new(Columns::last()).with(Width::truncate(80).suffix("...")))
        .to_string();

    println!("{}", table);
}

fn cmd_list(limit: usize) -> sticky_one::Result<()> {
    let storage = Storage::open()?;
    let entries = storage.list(limit)?;
    print_entries(entries);
    Ok(())
}

fn cmd_get(id: i64) -> sticky_one::Result<()> {
    let storage = Storage::open()?;
    let entry = storage.get_by_id(id)?;
    write_entry(&entry)?;
    println!("{} {}", "Copied entry".green(), id.to_string().bold());
    Ok(())
}

fn cmd_search(query: &str, limit: usize) -> sticky_one::Result<()> {
    let storage = Storage::open()?;
    let entries = storage.search(query, limit)?;

    if entries.is_empty() {
        println!("{} '{}'", "No matches for".yellow(), query);
        return Ok(());
    }

    print_entries(entries);
    Ok(())
}

fn cmd_clear() -> sticky_one::Result<()> {
    let storage = Storage::open()?;
    let count = storage.clear()?;
    println!("{} {} entries", "Cleared".yellow(), count);
    Ok(())
}
