use crossbeam::queue::SegQueue;
use ignore::WalkBuilder;
use rayon::prelude::*;
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use superconsole::style::ContentStyle;
use superconsole::{Lines, SuperConsole};
use walkdir::WalkDir;

// pub fn get_dir_size(
//     path: &PathBuf,
//     update_size: Arc<Mutex<dyn Fn(u64) + Send + Sync>>,
//     console: Arc<Mutex<SuperConsole>>,
// ) -> u64 {
//     WalkDir::new(path)
//         .into_iter()
//         .filter_map(Result::ok)
//         .filter(|entry| entry.file_type().is_file())
//         .par_bridge()
//         .map(|entry| {
//             entry
//                 .metadata()
//                 .map(|m| {
//                     let size = m.len();
//                     update_size.lock().unwrap()(size);
//                     size
//                 })
//                 .unwrap_or_else(|e| {
//                     console.lock().unwrap().emit(Lines::from_multiline_string(
//                         &format!("Error reading directory: {}", e),
//                         ContentStyle::default(),
//                     ));
//                     0
//                 })
//         })
//         .sum()
// }

pub fn get_dir_size(
    path: &PathBuf,
    update_size: Arc<Mutex<dyn Fn(u64) + Send + Sync>>,
    console: Arc<Mutex<SuperConsole>>,
) {
    WalkBuilder::new(path)
        .standard_filters(false)
        .follow_links(false)
        .threads(num_cpus::get())
        .build_parallel()
        .run(|| {
            let update_size = update_size.clone();
            let console = console.clone();
            Box::new(move |result| {
                match result {
                    Ok(entry) => {
                        let path = entry.path();
                        if path.is_file() {
                            if let Ok(metadata) = fs::metadata(path) {
                                update_size.lock().unwrap()(metadata.len());
                            }
                        }
                    }
                    Err(e) => {
                        console.lock().unwrap().emit(Lines::from_multiline_string(
                            &format!("Error reading directory: {}", e),
                            ContentStyle::default(),
                        ));
                    }
                }
                ignore::WalkState::Continue
            })
        });
}

// pub fn get_dir_size(
//     path: &PathBuf,
//     update_size: Arc<Mutex<dyn Fn(u64) + Send + Sync>>,
//     console: Arc<Mutex<SuperConsole>>,
// ) -> u64 {
//     let entries: Vec<_> = WalkDir::new(path)
//         .into_iter()
//         .filter_map(Result::ok)
//         .filter(|entry| entry.file_type().is_file())
//         .collect();

//     entries
//         .par_iter()
//         .map(|entry| {
//             entry
//                 .metadata()
//                 .map(|m| {
//                     let size = m.len();
//                     update_size.lock().unwrap()(size);
//                     size
//                 })
//                 .unwrap_or_else(|e| {
//                     console.lock().unwrap().emit(Lines::from_multiline_string(
//                         &format!("Error reading directory: {}", e),
//                         ContentStyle::default(),
//                     ));
//                     0
//                 })
//         })
//         .sum()
// }

// pub fn get_dir_size(
//     path: &PathBuf,
//     update_size: Arc<Mutex<dyn Fn(u64) + Send + Sync>>,
//     console: Arc<Mutex<SuperConsole>>,
// ) -> u64 {
//     let entries: Vec<_> = WalkDir::new(path)
//         .into_iter()
//         .filter_map(Result::ok)
//         .filter(|entry| entry.file_type().is_file())
//         .collect();

//     entries
//         .par_iter()
//         .fold(
//             || 0u64,
//             |acc, entry| {
//                 let size = entry.metadata().map(|m| m.len()).unwrap_or_else(|e| {
//                     console.lock().unwrap().emit(Lines::from_multiline_string(
//                         &format!("Error reading directory: {}", e),
//                         ContentStyle::default(),
//                     ));
//                     0
//                 });
//                 update_size.lock().unwrap()(size);
//                 acc + size
//             },
//         )
//         .reduce(|| 0u64, |a, b| a + b)
// }

// pub fn get_dir_size2(path: &PathBuf, update_size: Arc<Mutex<dyn Fn(u64) + Send + Sync>>) -> u64 {
//     let entries: Vec<_> = WalkDir::new(path)
//         .into_iter()
//         .filter_map(Result::ok)
//         .filter(|entry| entry.file_type().is_file())
//         .collect();

//     entries
//         .par_iter()
//         .fold(
//             || 0u64,
//             |acc, entry| {
//                 let size = entry.metadata().map(|m| m.len()).unwrap_or(0);
//                 update_size.lock().unwrap()(size);
//                 acc + size
//             },
//         )
//         .reduce(|| 0u64, |a, b| a + b)
// }

// pub fn get_dir_size4(source_path: PathBuf, update_size: Arc<Mutex<dyn Fn(u64) + Send + Sync>>) {
//     if let Ok(entries) = fs::read_dir(&source_path) {
//         let x: Vec<_> = entries
//             .filter_map(Result::ok)
//             .map(|entry| {
//                 if let Ok(file_type) = entry.file_type() {
//                     let path = entry.path();
//                     if file_type.is_dir() {
//                         get_dir_size4(path, update_size.clone());
//                     } else if file_type.is_file() {
//                     }
//                 }
//             })
//             .collect();

//         x.par_iter().for_each(|entry| {
//             if let Ok(file_type) = entry.file_type() {
//                 let path = entry.path();
//                 if file_type.is_file() {
//                     if let Ok(metadata) = entry.metadata() {
//                         let size = metadata.len();
//                         update_size.lock().unwrap()(size);
//                     }
//                 }
//             }
//         });
//     }
// }

// pub fn get_dir_size3(path: &PathBuf, update_size: Arc<Mutex<dyn Fn(u64) + Send + Sync>>) -> u64 {
//     WalkDir::new(path)
//         .into_iter()
//         .filter_map(Result::ok)
//         .filter(|entry| entry.file_type().is_file())
//         .par_bridge()
//         .map(|entry| {
//             entry
//                 .metadata()
//                 .map(|m| {
//                     let size = m.len();
//                     update_size.lock().unwrap()(size);
//                     size
//                 })
//                 .unwrap_or(0)
//         })
//         .sum()
// }

// pub fn get_dir_size5(path: PathBuf, update_size: Arc<Mutex<dyn Fn(u64) + Send + Sync>>) -> u64 {
//     WalkDir::new(path)
//         .into_iter()
//         .filter_map(Result::ok)
//         .filter(|entry| entry.file_type().is_file())
//         .par_bridge()
//         .map(|entry| {
//             if entry.file_type().is_dir() {
//                 get_dir_size5(entry.path().to_path_buf(), update_size.clone())
//             } else {
//                 entry
//                     .metadata()
//                     .map(|m| {
//                         let size = m.len();
//                         update_size.lock().unwrap()(size);
//                         size
//                     })
//                     .unwrap_or(0)
//             }
//         })
//         .sum()
// }
