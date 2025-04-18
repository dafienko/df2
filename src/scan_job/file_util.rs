use rayon::prelude::*;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use walkdir::WalkDir;

pub fn get_dir_size(path: &PathBuf, update_size: Arc<Mutex<dyn Fn(u64) + Send + Sync>>) -> u64 {
    WalkDir::new(path)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file())
        .par_bridge()
        .map(|entry| {
            entry
                .metadata()
                .map(|m| {
                    let size = m.len();
                    update_size.lock().unwrap()(size);
                    size
                })
                .expect("map error")
        })
        .sum()
}
