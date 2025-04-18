mod scan_job;

use std::env;
use std::path::PathBuf;

fn main() {
    let input_path: PathBuf = env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| env::current_dir().expect("Failed to get current dir"));

    scan_job::scan_dir(input_path, None);

    // for portion in scan_job::PortionColors::iter() {
    //     println!("{}", color_portion("█".repeat(10), portion));
    // }
}
