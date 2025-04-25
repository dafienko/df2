use super::scan_job::{color_portion, PortionColor};
use bytesize::ByteSize;
use colored::Colorize;
use prettytable::format::Alignment;
use prettytable::*;
use std::cmp::Ordering;
use std::time;

#[derive(Debug, PartialEq)]
pub enum ItemType {
    Directory,
    File,
}

#[derive(Debug)]
pub struct LineItem {
    pub path: String,
    pub item_type: ItemType,
    pub start_time: time::Instant,
    pub completed_time: Option<time::Instant>,
    pub size_snapshot: u64,
}

impl LineItem {
    pub fn render_progress_row(&self, parent_size: u64, is_final: bool) -> Row {
        let mut row = Row::empty();

        if !is_final {
            let time_str = &match self.completed_time {
                Some(time) => format!("{:.2}", time.duration_since(self.start_time).as_secs_f32())
                    .dimmed()
                    .to_string(),
                None => format!("{:.2}", self.start_time.elapsed().as_secs_f32()),
            };
            row.add_cell(Cell::new(time_str));
        }

        let size_str = ByteSize::b(self.size_snapshot).to_string();
        row.add_cell(Cell::new_align(&size_str, Alignment::RIGHT));

        let percent_str = &match parent_size {
            0 => String::from("00.00%"),
            _ => format!(
                "{:.2}%",
                ((self.size_snapshot as f64 / parent_size as f64) * 100.0).min(100.0)
            ),
        };
        row.add_cell(Cell::new_align(percent_str, Alignment::RIGHT));

        let path_str = &match self.item_type {
            ItemType::Directory => format!("{}", self.path.bright_cyan()),
            ItemType::File => format!("{}", self.path.bright_white()),
        }
        .to_string();
        row.add_cell(Cell::new(path_str));

        return row;
    }

    pub fn render_legend_row(
        &self,
        i: usize,
        portion: PortionColor,
        aggregated_other: bool,
    ) -> (Row, bool) {
        let item_name = self.path.clone();
        let item_name = match self.item_type {
            ItemType::Directory => item_name.bright_cyan(),
            ItemType::File => item_name.bright_white(),
        }
        .to_string();

        let show_index = !(portion == PortionColor::PortionLast && aggregated_other)
            && self.item_type == ItemType::Directory;
        let index_str = match show_index {
            true => format!("[{}]", i + 1),
            false => String::from(""),
        };
        let index_str = color_portion(index_str, portion);

        let item_size = ByteSize::b(self.size_snapshot).to_string();

        (
            Row::new(vec![
                Cell::new(&index_str),
                Cell::new(&item_name),
                Cell::new_align(&item_size, Alignment::RIGHT),
            ]),
            show_index,
        )
    }

    pub fn render_legend_row_other(label: &str, size: u64) -> Row {
        Row::new(vec![
            Cell::new(&"".bright_white().to_string()),
            Cell::new(label),
            Cell::new_align(&ByteSize::b(size).to_string(), Alignment::RIGHT),
        ])
    }
}

impl Ord for LineItem {
    fn cmp(&self, other: &Self) -> Ordering {
        self.size_snapshot.cmp(&other.size_snapshot)
    }
}

impl PartialOrd for LineItem {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let size_order = self.size_snapshot.cmp(&other.size_snapshot);
        if size_order == Ordering::Equal {
            return Some(self.path.cmp(&other.path));
        }
        Some(size_order)
    }
}

impl PartialEq for LineItem {
    fn eq(&self, other: &Self) -> bool {
        self.size_snapshot == other.size_snapshot
    }
}

impl Eq for LineItem {}
