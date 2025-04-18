use bytesize::ByteSize;
use colored::Colorize;
use prettytable::format::Alignment;
use prettytable::*;
use std::cmp::Ordering;
use std::path::PathBuf;
use std::sync::atomic::AtomicU64;
use std::sync::{Arc, Mutex};
use std::time;

#[derive(Debug)]
pub enum ItemType {
    Directory,
    File,
}

#[derive(Debug)]
pub struct LineItem {
    pub path: PathBuf,
    pub item_type: ItemType,
    pub size: Arc<AtomicU64>,
    pub start_time: time::Instant,
    pub completed_time: Arc<Mutex<Option<time::Instant>>>,

    pub parent_size_render_snapshot: u64,
    pub size_render_snapshot: u64,
}

impl LineItem {
    pub fn render_row(&self, is_final: bool) -> Row {
        let mut row = Row::empty();

        if !is_final {
            let completed_time = *self.completed_time.lock().unwrap();
            let time_str = &match completed_time {
                Some(time) => format!("{:.2}", time.duration_since(self.start_time).as_secs_f32())
                    .dimmed()
                    .to_string(),
                None => format!("{:.2}", self.start_time.elapsed().as_secs_f32()),
            };
            row.add_cell(Cell::new(time_str));
        }

        let size_str = ByteSize::b(self.size_render_snapshot).to_string();
        row.add_cell(Cell::new_align(&size_str, Alignment::RIGHT));

        let percent_str = &match self.parent_size_render_snapshot {
            0 => String::from("00.00%"),
            _ => format!(
                "{:.2}%",
                ((self.size_render_snapshot as f64 / self.parent_size_render_snapshot as f64)
                    * 100.0)
                    .min(100.0)
            ),
        };
        row.add_cell(Cell::new_align(percent_str, Alignment::RIGHT));

        let path_str = self.path.to_str().unwrap();
        let path_str = &match self.item_type {
            ItemType::Directory => format!("{}", path_str.bright_cyan()),
            ItemType::File => format!("{}", path_str.bright_white()),
        }
        .to_string();
        row.add_cell(Cell::new(path_str));

        return row;
    }
}

impl Ord for LineItem {
    fn cmp(&self, other: &Self) -> Ordering {
        self.size_render_snapshot.cmp(&other.size_render_snapshot)
    }
}

impl PartialOrd for LineItem {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let size_order = self.size_render_snapshot.cmp(&other.size_render_snapshot);
        if size_order == Ordering::Equal {
            return Some(self.path.cmp(&other.path));
        }
        Some(size_order)
    }
}

impl PartialEq for LineItem {
    fn eq(&self, other: &Self) -> bool {
        self.size_render_snapshot == other.size_render_snapshot
    }
}

impl Eq for LineItem {}
