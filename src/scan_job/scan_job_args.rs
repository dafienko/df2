use clap::Parser;

/// Calculate the size of a directory and its contents
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
}
