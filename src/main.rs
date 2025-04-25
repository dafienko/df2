mod scan_job;
use clap::Parser;
use std::collections::HashMap;
use std::io;
use std::io::Write;
use std::path::Path;
use std::sync::{Arc, Mutex};

enum Instruction {
    Index(usize),
    Parent,
}

fn read_instruction(max: usize) -> Instruction {
    let mut input = String::new();

    print!("> ");
    io::stdout().flush().unwrap();

    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");

    let input = input.trim();
    if input == ".." {
        return Instruction::Parent;
    }

    let res = input.parse();
    match res {
        Ok(num) => {
            if num > max {
                eprintln!(
                    "Index is out of range, please enter an index less than or equal to {}",
                    max
                );
                return read_instruction(max);
            }
            Instruction::Index(num)
        }
        Err(_) => {
            eprintln!("Invalid input, please enter an index");
            return read_instruction(max);
        }
    }
}

fn main() {
    let mut args = scan_job::scan_job_args::ScanJobArgs::parse();
    let size_cache = Arc::new(Mutex::new(HashMap::new()));
    let mut dirs = scan_job::scan_dir(args.clone(), size_cache.clone());

    if args.interactive_mode {
        loop {
            match read_instruction(dirs.len()) {
                Instruction::Index(index) => {
                    args.directory = dirs.get(index - 1).unwrap().clone();
                }
                Instruction::Parent => {
                    let path = Path::new(args.directory.as_str());
                    if let Some(parent) = path.parent() {
                        args.directory = parent.to_str().unwrap().to_string();
                    } else {
                        eprintln!("No parent directory found");
                        continue;
                    }
                }
            }
            dirs = scan_job::scan_dir(args.clone(), size_cache.clone());
        }
    }
}
