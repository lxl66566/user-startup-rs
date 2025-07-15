use std::fs;
#[cfg(target_os = "windows")]
use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueHint};
use log::{LevelFilter, warn};
#[cfg(target_os = "windows")]
use user_startup::utils::run_no_window;
use user_startup::{add_item, get_items_list, open_config_folder, remove_items, utils};

#[derive(Parser)]
#[command(about = "Make any command automatically run on startup")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Add a new startup command
    #[command(visible_alias = "a")]
    Add {
        /// The command to add. Please wrap the command in quotes ('').
        command: String,
        /// The name of the command. If not provided, the first word of the
        /// command will be used.
        #[arg(short, long)]
        name: Option<String>,
        /// Redirect the command's stdout to a file.
        #[arg(long, value_hint(ValueHint::FilePath))]
        stdout: Option<String>,
        /// Redirect the command's stderr to a file.
        #[arg(long, value_hint(ValueHint::FilePath))]
        stderr: Option<String>,
    },
    /// List all startup commands
    #[command(visible_alias = "l", visible_alias = "info", visible_alias = "i")]
    List {
        /// Do not use a table to display the list. Instead, use `\t` to
        /// separate the id and command.
        #[arg(long)]
        no_table: bool,
    },
    /// Remove startup commands
    #[command(visible_alias = "r", visible_alias = "rm")]
    Remove {
        /// The ids of the items to remove
        #[clap(required = true)]
        ids: Vec<String>,
    },
    /// Open the startup folder
    #[command(visible_alias = "o")]
    Open,
    /// Run command with NO_WINDOW
    #[cfg(target_os = "windows")]
    Run {
        /// The command to run
        #[clap(required = true)]
        command: String,
        /// Redirect the command's stdout to a file.
        #[arg(long, value_hint(ValueHint::FilePath))]
        stdout: Option<PathBuf>,
        /// Redirect the command's stderr to a file.
        #[arg(long, value_hint(ValueHint::FilePath))]
        stderr: Option<PathBuf>,
    },
}

fn main() {
    log_init();

    let config_path = &utils::CONFIG_PATH;
    if !config_path.exists() {
        warn!("Config path not found. Creating it...");
        fs::create_dir_all(config_path.as_os_str()).expect("Failed to create config directory");
    }

    let cli = Cli::parse();

    match cli.command {
        Commands::Add {
            command,
            name,
            stdout,
            stderr,
        } => add_item(
            &command,
            name.as_deref(),
            stdout.as_deref(),
            stderr.as_deref(),
        ),
        Commands::List { no_table } => {
            if no_table {
                let temp = get_items_list().into_iter();
                println!("id\tcommand");
                temp.for_each(|(id, c)| println!("{id}\t{c}"));
            } else {
                list_items()
            }
        }
        Commands::Remove { ids } => remove_items(ids),
        Commands::Open => open_config_folder(),
        #[cfg(target_os = "windows")]
        Commands::Run {
            command,
            stdout,
            stderr,
        } => run_no_window(command, stdout, stderr).expect("Failed to run command"),
    }
}

/// List all startup commands with a table.
pub fn list_items() {
    use comfy_table::{
        Table,
        TableComponent::{BottomLeftCorner, BottomRightCorner, TopLeftCorner, TopRightCorner},
        presets::UTF8_FULL,
    };

    let mut table = Table::new();
    // Load the UTF8_FULL preset
    table.load_preset(UTF8_FULL);
    // Set all outer corners to round UTF8 corners
    // This is basically the same as the UTF8_ROUND_CORNERS modifier
    table.set_style(TopLeftCorner, '╭');
    table.set_style(TopRightCorner, '╮');
    table.set_style(BottomLeftCorner, '╰');
    table.set_style(BottomRightCorner, '╯');
    table.set_header(vec!["id", "command"]);

    let items = get_items_list();
    for (id, command) in items {
        table.add_row(vec![id, command]);
    }
    println!("{table}");
}

#[inline]
pub fn log_init() {
    #[cfg(not(debug_assertions))]
    log_init_with_default_level(LevelFilter::Info);
    #[cfg(debug_assertions)]
    log_init_with_default_level(LevelFilter::Debug);
}

#[inline]
pub fn log_init_with_default_level(level: LevelFilter) {
    _ = pretty_env_logger::formatted_builder()
        .filter_level(level)
        .format_timestamp_secs()
        .filter_module("reqwest", LevelFilter::Info)
        .parse_default_env()
        .try_init();
}
