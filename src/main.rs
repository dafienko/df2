use rayon::prelude::*;
use std::cmp::Ordering;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering as AtomicOrdering};
use std::sync::{Arc, Mutex};
use std::{env, fs, thread, time};
use superconsole::components::DrawVertical;
use superconsole::style::Color;
use superconsole::{Component, Dimensions, DrawMode, Line, Lines, Span, SuperConsole};
use thousands::Separable;
use walkdir::WalkDir;

fn get_dir_size(path: &PathBuf, update_size: Arc<Mutex<dyn Fn(u64) + Send + Sync>>) -> u64 {
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

#[derive(Debug)]
enum ItemType {
    Directory,
    File,
}

#[derive(Debug)]
struct LineItem {
    path: PathBuf,
    item_type: ItemType,
    size: Arc<AtomicU64>,
    parent_size: u64,
    start_time: time::Instant,
    completed_time: Arc<Mutex<Option<time::Instant>>>,
}

impl Ord for LineItem {
    fn cmp(&self, other: &Self) -> Ordering {
        self.size
            .load(AtomicOrdering::Relaxed)
            .cmp(&other.size.load(AtomicOrdering::Relaxed))
    }
}

impl PartialOrd for LineItem {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let size_order = self
            .size
            .load(AtomicOrdering::Relaxed)
            .cmp(&other.size.load(AtomicOrdering::Relaxed));
        if size_order == Ordering::Equal {
            return Some(self.path.cmp(&other.path));
        }
        Some(size_order)
    }
}

impl PartialEq for LineItem {
    fn eq(&self, other: &Self) -> bool {
        self.size.load(AtomicOrdering::Relaxed) == other.size.load(AtomicOrdering::Relaxed)
    }
}

impl Eq for LineItem {}

impl Component for LineItem {
    fn draw_unchecked(&self, dimensions: Dimensions, mode: DrawMode) -> anyhow::Result<Lines> {
        let size = self.size.load(AtomicOrdering::Relaxed);
        let completed_time = self.completed_time.lock().unwrap();
        let size_str = size.separate_with_commas();

        let time_span = match *completed_time {
            Some(time) => Span::new_colored_lossy(
                &format!(
                    "{:<6}",
                    format!("{:.2}", time.duration_since(self.start_time).as_secs_f32())
                ),
                Color::Grey,
            ),
            None => Span::new_colored_lossy(
                &format!(
                    "{:<6}",
                    format!("{:.2}", self.start_time.elapsed().as_secs_f32())
                ),
                Color::White,
            ),
        };

        let percent_span = Span::new_unstyled_lossy(&format!(
            "{:<7}",
            &match self.parent_size {
                0 => String::from("00.00%"),
                _ => format!(
                    "{:05.2}%",
                    ((size as f64 / self.parent_size as f64) * 100.0).min(100.0)
                ),
            }
        ));

        let mut spans = match mode {
            DrawMode::Final => vec![],
            DrawMode::Normal => vec![time_span],
        };

        spans.extend(vec![
            Span::new_unstyled_lossy(format!("{:>20}", size_str)),
            Span::padding(3),
            percent_span,
            Span::padding(3),
            Span::new_colored_lossy(
                self.path.to_str().unwrap(),
                match self.item_type {
                    ItemType::Directory => Color::Cyan,
                    ItemType::File => Color::White,
                },
            ),
        ]);

        let mut line = Line::from_iter(spans);
        line.to_exact_width(dimensions.width);
        Ok(Lines::from_iter([line]))
    }
}

#[derive(Debug)]
struct ScanJobResult {
    items: Vec<LineItem>,
}

impl Component for ScanJobResult {
    fn draw_unchecked(&self, dimensions: Dimensions, mode: DrawMode) -> anyhow::Result<Lines> {
        let mut draw_vertical = DrawVertical::new(Dimensions {
            width: dimensions.width - 1,
            height: dimensions.height,
        });

        for item in &self.items {
            draw_vertical.draw(item, mode)?;
        }

        Ok(draw_vertical.finish())
    }
}

#[derive(Debug)]
struct ScanJob {
    path: PathBuf,
    result_items: Arc<Mutex<ScanJobResult>>,
    total_size: Arc<AtomicU64>,
}

impl Component for ScanJob {
    fn draw_unchecked(&self, dimensions: Dimensions, mode: DrawMode) -> anyhow::Result<Lines> {
        let mut scan_res = self.result_items.lock().unwrap();
        scan_res.items.sort();
        let result_lines = scan_res.draw(dimensions, mode)?;
        Ok(result_lines)
    }
}

impl ScanJob {
    fn new(path: PathBuf) -> Self {
        Self {
            path: path,
            result_items: Arc::new(Mutex::new(ScanJobResult { items: Vec::new() })),
            total_size: Arc::new(AtomicU64::new(0)),
        }
    }

    fn pre_render(&self) {
        let mut scan_res = self.result_items.lock().unwrap();
        scan_res.items.sort();

        let current_total_size = self.total_size.load(AtomicOrdering::Relaxed);
        for item in &mut scan_res.items {
            item.parent_size = current_total_size;
        }
    }

    fn execute(self, mut console: SuperConsole) {
        let stop_flag = Arc::new(AtomicBool::new(false));
        let flag_clone = Arc::clone(&stop_flag);
        crossbeam::thread::scope(|s| {
            s.spawn(|_| {
                while !flag_clone.load(AtomicOrdering::Relaxed) {
                    self.pre_render();
                    console.render(&self).unwrap();
                    thread::sleep(std::time::Duration::from_millis(100));
                }
            });

            let mut children_dirs = Vec::new();
            if let Ok(entries) = fs::read_dir(&self.path) {
                for entry in entries.filter_map(Result::ok) {
                    if let Ok(file_type) = entry.file_type() {
                        let path = entry.path();
                        if file_type.is_dir() {
                            children_dirs.push(path);
                        } else if file_type.is_file() {
                            let file_size = entry.path().metadata().map(|m| m.len()).unwrap_or(0);
                            self.total_size
                                .fetch_add(file_size, AtomicOrdering::Relaxed);

                            self.result_items.lock().unwrap().items.push(LineItem {
                                path: path.clone().strip_prefix(&self.path).unwrap().to_path_buf(),
                                size: Arc::new(AtomicU64::new(file_size)),
                                item_type: ItemType::File,
                                start_time: time::Instant::now(),
                                completed_time: Arc::new(Mutex::new(Some(time::Instant::now()))),
                                parent_size: self.total_size.load(AtomicOrdering::Relaxed),
                            });
                        }
                    }
                }
            }

            children_dirs.par_iter().for_each(|dir| {
                let child_size = Arc::new(AtomicU64::new(0));
                let child_completed_time = Arc::new(Mutex::new(None));
                self.result_items.lock().unwrap().items.push(LineItem {
                    path: dir.clone().strip_prefix(&self.path).unwrap().to_path_buf(),
                    size: child_size.clone(),
                    item_type: ItemType::Directory,
                    start_time: time::Instant::now(),
                    completed_time: child_completed_time.clone(),
                    parent_size: self.total_size.load(AtomicOrdering::Relaxed),
                });

                let total_size_clone = self.total_size.clone();
                let update_size = Arc::new(Mutex::new(move |add: u64| {
                    child_size.fetch_add(add, AtomicOrdering::Relaxed);
                    total_size_clone.fetch_add(add, AtomicOrdering::Relaxed);
                }));
                get_dir_size(dir, update_size);

                child_completed_time
                    .lock()
                    .unwrap()
                    .replace(time::Instant::now());
            });

            stop_flag.store(true, AtomicOrdering::Relaxed);
        })
        .unwrap();

        self.pre_render();
        console.finalize(&self).unwrap();
    }
}

fn main() {
    let console = SuperConsole::new()
        .ok_or_else(|| anyhow::anyhow!("Not a TTY"))
        .unwrap();

    let input_path: PathBuf = env::args()
        .nth(1) // Get first argument if present
        .map(PathBuf::from) // Convert String to PathBuf
        .unwrap_or_else(|| env::current_dir().expect("Failed to get current dir"));

    let scan_job = ScanJob::new(input_path);
    scan_job.execute(console);
}
