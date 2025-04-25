use clap::Parser;

/// Calculate the size of a directory
#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about = None)]
pub struct ScanJobArgs {
    /// Directory to scan
    #[arg(default_value = ".")]
    pub directory: String,

    /// List all directories and files in the directory after scanning
    #[arg(short, long, default_value_t = false)]
    pub list_items: bool,

    /// Cache the scan results and allow further traversal
    #[arg(short, long, default_value_t = false)]
    pub interactive_mode: bool,

    /// Max chart width
    #[arg(short, long, default_value_t = 100, conflicts_with = "full_width")]
    pub width: usize,

    /// Use full width of the terminal
    #[arg(short, long("full"), default_value_t = false, conflicts_with = "width")]
    pub full_width: bool,

    /// Log all errors
    #[arg(short, long, default_value_t = false)]
    pub verbose: bool,
}
