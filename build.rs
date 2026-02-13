use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::{generate_to, Shell};
use clap_mangen::Man;
use std::fs;
use std::path::PathBuf;

// Mirror of the CLI struct for generation purposes
#[derive(Parser)]
#[command(name = "syo")]
#[command(version)]
#[command(about = "Clipboard manager with 12-hour history")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Daemon,
    Stop,
    Status,
    List {
        #[arg(short, long, default_value = "20")]
        limit: usize,
    },
    Get {
        id: i64,
    },
    Search {
        query: String,
        #[arg(short, long, default_value = "20")]
        limit: usize,
    },
    Clear,
    Popup,
}

fn main() {
    let out = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());

    // Shell completions
    let comp_dir = out.join("completions");
    fs::create_dir_all(&comp_dir).unwrap();
    let mut cmd = Cli::command();
    for shell in [Shell::Bash, Shell::Zsh, Shell::Fish] {
        generate_to(shell, &mut cmd, "syo", &comp_dir).unwrap();
    }

    // Man page
    let man_dir = out.join("man");
    fs::create_dir_all(&man_dir).unwrap();
    let man = Man::new(Cli::command());
    let mut buf = Vec::new();
    man.render(&mut buf).unwrap();
    fs::write(man_dir.join("syo.1"), buf).unwrap();
}
