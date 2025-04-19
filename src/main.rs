mod scan_job;

use clap::Parser;

fn main() {
    let args = scan_job::scan_job_args::ScanJobArgs::parse();

    scan_job::scan_dir(args);

    // for portion in scan_job::PortionColors::iter() {
    //     println!("{}", color_portion("█".repeat(10), portion));
    // }
}
