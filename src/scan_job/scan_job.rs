use super::file_util;
use super::line_item::{ItemType, LineItem};
use super::lines_component::LinesComponent;
use super::scan_job_args::ScanJobArgs;
use bytesize::ByteSize;
use colored::Colorize;
use once_cell::sync::Lazy;
use prettytable::format::Alignment;
use prettytable::format::TableFormat;
use prettytable::*;
use rayon::prelude::*;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering as AtomicOrdering};
use std::sync::{Arc, Mutex};
use std::{fs, thread, time};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;
use superconsole::components::bordering::{Bordered, BorderedSpec};
use superconsole::components::DrawVertical;
use superconsole::style::ContentStyle;
use superconsole::{Component, Dimensions, DrawMode, Lines, SuperConsole};

static TABLE_FROMAT: Lazy<TableFormat> = Lazy::new(|| {
    format::FormatBuilder::new()
        .column_separator(' ')
        .separators(&[], format::LineSeparator::new(' ', ' ', ' ', ' '))
        .padding(0, 2)
        .build()
});

static EMPTY_LINE: Lazy<LinesComponent> =
    Lazy::new(|| LinesComponent::new(Lines::from_multiline_string("\n", ContentStyle::default())));

#[derive(Debug, EnumIter, PartialEq, Copy, Clone)]
enum PortionColors {
    Portion1,
    Portion2,
    Portion3,
    Portion4,
    Portion5,
    PortionLast,
}

fn color_portion(str: String, portion: PortionColors) -> String {
    match portion {
        PortionColors::Portion1 => str.bright_red(),
        PortionColors::Portion2 => str.bright_yellow(),
        PortionColors::Portion3 => str.bright_green(),
        PortionColors::Portion4 => str.bright_blue(),
        PortionColors::Portion5 => str.bright_magenta(),
        PortionColors::PortionLast => str.white(),
    }
    .to_string()
}

#[derive(Debug)]
pub struct ScanJob {
    result_items: Arc<Mutex<Vec<LineItem>>>,
    total_size: Arc<AtomicU64>,
    args: ScanJobArgs,
}

impl Component for ScanJob {
    fn draw_unchecked(&self, dimensions: Dimensions, mode: DrawMode) -> anyhow::Result<Lines> {
        if !self.pre_render() {
            return Ok(Lines::from_multiline_string(
                "Directory is empty",
                ContentStyle::default(),
            ));
        }

        let stacked_bar = LinesComponent::new(self.render_stacked_bar(dimensions));

        let item_table = LinesComponent::new(Lines::from_colored_multiline_string(
            &self.render_size_table(mode == DrawMode::Final).to_string(),
        ))
        .with_fill_width(true);

        let mut drew_something = false;
        let mut draw_vertical = DrawVertical::new(dimensions);
        match mode {
            DrawMode::Normal => {
                drew_something = true;
                let mut bordered_spec = BorderedSpec::default();
                bordered_spec.left = None;
                bordered_spec.right = None;
                draw_vertical.draw(&Bordered::new(item_table, bordered_spec), mode)?;
            }
            DrawMode::Final => {
                if self.args.list_items {
                    drew_something = true;
                    draw_vertical.draw(&item_table, mode)?;
                }
            }
        }

        if drew_something {
            draw_vertical.draw(&*EMPTY_LINE, mode)?;
        }

        draw_vertical.draw(&stacked_bar, mode)?;

        Ok(draw_vertical.finish())
    }
}

impl ScanJob {
    pub fn new(args: ScanJobArgs) -> Self {
        Self {
            result_items: Arc::new(Mutex::new(Vec::new())),
            total_size: Arc::new(AtomicU64::new(0)),
            args,
        }
    }

    fn render_size_table(&self, is_final: bool) -> Table {
        let mut table = Table::new();
        table.set_format(*TABLE_FROMAT);

        let scan_res = self.result_items.lock().unwrap();
        for item in scan_res.iter() {
            if !is_final && item.completed_time.lock().unwrap().is_some() {
                continue;
            }

            table.add_row(item.render_row(is_final));
        }

        table
    }

    fn render_stacked_bar(&self, dimensions: Dimensions) -> Lines {
        let mut bar_str = String::new();
        let items = self.result_items.lock().unwrap();
        let total_width = dimensions.width - 1;
        let mut remaining_width = total_width;
        let len = items.len();
        let mut did_aggregate_other = false;
        let mut legend_table = Table::new();
        legend_table.set_format(*TABLE_FROMAT);
        for (i, portion) in PortionColors::iter().enumerate() {
            if i == len || did_aggregate_other {
                break;
            }

            let j = len - i - 1;
            let item = &items[j];
            let proportion =
                item.size_render_snapshot as f64 / item.parent_size_render_snapshot as f64;
            let item_width = (proportion * total_width as f64).floor() as usize;
            let is_last = portion == PortionColors::PortionLast || i == len - 1;
            let width = if is_last || item_width == 0 {
                did_aggregate_other = i != len - 1;
                remaining_width
            } else {
                item_width
            };

            let portion_str = "█".repeat(width);
            let portion_str = color_portion(portion_str, portion);
            remaining_width = remaining_width.saturating_sub(width);
            bar_str.push_str(&portion_str);

            let item_name = match did_aggregate_other {
                true => String::from("Other"),
                false => item.path.to_str().unwrap().to_string(),
            };
            let item_name = format!("[{}] {}", i + 1, item_name);
            let item_size = ByteSize::b(item.size_render_snapshot).to_string();
            legend_table.add_row(Row::new(vec![
                Cell::new(&color_portion(item_name, portion)),
                Cell::new_align(&item_size, Alignment::RIGHT),
            ]));
        }

        let mut lines = Lines::new();
        lines
            .0
            .extend(Lines::from_colored_multiline_string(&legend_table.to_string()).0);
        lines
            .0
            .extend(Lines::from_multiline_string("\n", ContentStyle::default()));
        lines
            .0
            .extend(Lines::from_colored_multiline_string(&bar_str).0);
        lines
    }

    fn pre_render(&self) -> bool {
        let mut scan_res = self.result_items.lock().unwrap();
        let computed_total_size = scan_res.iter_mut().fold(0, |acc, item| {
            item.size_render_snapshot = item.size.load(AtomicOrdering::Relaxed);
            acc + item.size_render_snapshot
        });
        scan_res.sort();

        for item in scan_res.iter_mut() {
            item.parent_size_render_snapshot = computed_total_size;
        }

        computed_total_size > 0
    }

    pub fn render_until_flag(&self, console: Arc<Mutex<SuperConsole>>, stop_flag: Arc<AtomicBool>) {
        while !stop_flag.load(AtomicOrdering::Relaxed) {
            console.lock().unwrap().render(&self).unwrap();
            thread::sleep(std::time::Duration::from_millis(100));
        }
    }

    pub fn execute(&self) {
        let mut children_dirs = Vec::new();
        if let Ok(entries) = fs::read_dir(&self.args.directory) {
            for entry in entries.filter_map(Result::ok) {
                if let Ok(file_type) = entry.file_type() {
                    let path = entry.path();
                    if file_type.is_dir() {
                        children_dirs.push(path);
                    } else if file_type.is_file() {
                        let file_size = entry.path().metadata().map(|m| m.len()).unwrap_or(0);
                        self.total_size
                            .fetch_add(file_size, AtomicOrdering::Relaxed);

                        self.result_items.lock().unwrap().push(LineItem {
                            path: path
                                .clone()
                                .strip_prefix(&self.args.directory)
                                .unwrap()
                                .to_path_buf(),
                            size: Arc::new(AtomicU64::new(file_size)),
                            item_type: ItemType::File,
                            start_time: time::Instant::now(),
                            completed_time: Arc::new(Mutex::new(Some(time::Instant::now()))),
                            size_render_snapshot: file_size,
                            parent_size_render_snapshot: 0,
                        });
                    }
                }
            }
        }

        children_dirs.par_iter().for_each(|dir| {
            let child_size = Arc::new(AtomicU64::new(0));
            let child_completed_time = Arc::new(Mutex::new(None));
            self.result_items.lock().unwrap().push(LineItem {
                path: dir
                    .clone()
                    .strip_prefix(&self.args.directory)
                    .unwrap()
                    .to_path_buf(),
                size: child_size.clone(),
                item_type: ItemType::Directory,
                start_time: time::Instant::now(),
                completed_time: child_completed_time.clone(),
                size_render_snapshot: 0,
                parent_size_render_snapshot: 0,
            });

            let total_size_clone = self.total_size.clone();
            let update_size = Arc::new(Mutex::new(move |add: u64| {
                child_size.fetch_add(add, AtomicOrdering::Relaxed);
                total_size_clone.fetch_add(add, AtomicOrdering::Relaxed);
            }));
            file_util::get_dir_size(dir, update_size);

            child_completed_time
                .lock()
                .unwrap()
                .replace(time::Instant::now());
        });
    }
}
