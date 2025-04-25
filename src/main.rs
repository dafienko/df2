mod scan_job;
use clap::Parser;

fn main() {
    let args = scan_job::scan_job_args::ScanJobArgs::parse();
    scan_job::scan_dir(args);
}
