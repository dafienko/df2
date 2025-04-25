use super::file_util::get_dir_size;
use super::file_util::ItemView;
use super::line_item::{ItemType, LineItem};
use super::lines_component::LinesComponent;
use super::scan_job_args::ScanJobArgs;
use colored::Colorize;
use once_cell::sync::Lazy;
use prettytable::format::TableFormat;
use prettytable::*;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
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
pub enum PortionColor {
    Portion1,
    Portion2,
    Portion3,
    Portion4,
    Portion5,
    PortionLast,
}

pub fn color_portion(str: String, portion: PortionColor) -> String {
    match portion {
        PortionColor::Portion1 => str.bright_red(),
        PortionColor::Portion2 => str.bright_yellow(),
        PortionColor::Portion3 => str.bright_green(),
        PortionColor::Portion4 => str.bright_blue(),
        PortionColor::Portion5 => str.bright_magenta(),
        PortionColor::PortionLast => str.white(),
    }
    .to_string()
}

pub struct ScanJob {
    scan_view: Arc<Mutex<Vec<Arc<ItemView>>>>,
    args: ScanJobArgs,
}

impl Component for ScanJob {
    fn draw_unchecked(&self, dimensions: Dimensions, mode: DrawMode) -> anyhow::Result<Lines> {
        let line_items = self.pre_render();
        let total_size = line_items
            .iter()
            .fold(0, |acc, item| acc + item.size_snapshot);

        if total_size == 0 {
            return Ok(Lines::from_multiline_string(
                "Directory is empty",
                ContentStyle::default(),
            ));
        }

        let stacked_bar = LinesComponent::new(self.render_stacked_bar(
            dimensions,
            mode,
            &line_items,
            total_size,
        )?);
        let item_table = LinesComponent::new(Lines::from_colored_multiline_string(
            &self
                .render_size_table(&line_items, total_size, mode == DrawMode::Final)
                .to_string(),
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
                // if self.args.list_items {
                //     drew_something = true;
                //     draw_vertical.draw(&item_table, mode)?;
                // }
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
            scan_view: Arc::new(Mutex::new(Vec::new())),
            args,
        }
    }

    fn render_size_table(
        &self,
        line_items: &Vec<LineItem>,
        total_size: u64,
        is_final: bool,
    ) -> Table {
        let mut table = Table::new();
        table.set_format(*TABLE_FROMAT);

        let mut remaining_list_items = match is_final {
            true => u64::MAX,
            false => 6,
        };
        for item in line_items.iter().rev() {
            if !is_final && item.completed_time.is_some() {
                continue;
            }

            table.add_row(item.render_row(total_size, is_final));

            remaining_list_items -= 1;
            if remaining_list_items == 0 {
                break;
            }
        }

        table
    }

    fn render_stacked_bar(
        &self,
        dimensions: Dimensions,
        mode: DrawMode,
        line_items: &Vec<LineItem>,
        total_size: u64,
    ) -> anyhow::Result<Lines> {
        let mut bar_str = String::new();
        let total_width = dimensions.width - 1;
        let mut remaining_width = total_width;
        let len = line_items.len();
        let mut did_aggregate_other = false;
        let mut legend_table = Table::new();
        legend_table.set_format(*TABLE_FROMAT);
        let mut other_size = total_size;
        for (i, mut portion) in PortionColor::iter().enumerate() {
            if i == len || did_aggregate_other {
                break;
            }

            let j = len - i - 1;
            let item = &line_items[j];
            let proportion = item.size_snapshot as f64 / total_size as f64;
            let item_width = (proportion * total_width as f64).floor() as usize;
            let is_last = portion == PortionColor::PortionLast || i == len - 1;
            let width = if is_last || item_width == 0 {
                did_aggregate_other = i != len - 1;
                if did_aggregate_other {
                    portion = PortionColor::PortionLast;
                }
                remaining_width
            } else {
                item_width
            };

            let portion_str = "█".repeat(width);
            let portion_str = color_portion(portion_str, portion);
            bar_str.push_str(&portion_str);
            remaining_width = remaining_width.saturating_sub(width);

            if did_aggregate_other && self.args.list_items && mode == DrawMode::Final {
                break;
            }

            if did_aggregate_other {
                legend_table.add_row(LineItem::render_legend_row_other(
                    &color_portion(String::from("Other"), PortionColor::PortionLast),
                    other_size,
                ));
            } else {
                other_size -= item.size_snapshot;
                legend_table.add_row(item.render_legend_row(i, portion));
            }
        }
        if did_aggregate_other && self.args.list_items && mode == DrawMode::Final {
            for i in PortionColor::iter().count() - 1..line_items.len() {
                let j = len - i - 1;
                let item = &line_items[j];
                legend_table.add_row(item.render_legend_row(i, PortionColor::PortionLast));
            }
        }
        legend_table.add_row(LineItem::render_legend_row_other(
            &"Total".bright_white().bold().to_string(),
            total_size,
        ));

        let mut draw_vertical = DrawVertical::new(dimensions);
        draw_vertical.draw(&LinesComponent::from_str(&legend_table.to_string()), mode)?;
        draw_vertical.draw(&*EMPTY_LINE, mode)?;
        draw_vertical.draw(&LinesComponent::from_str(&bar_str), mode)?;
        Ok(draw_vertical.finish())
    }

    fn pre_render(&self) -> Vec<LineItem> {
        let mut items = self
            .scan_view
            .lock()
            .unwrap()
            .iter()
            .map(|item| match item.as_ref() {
                ItemView::Directory(path, progress) => LineItem {
                    path: path.clone(),
                    item_type: ItemType::Directory,
                    start_time: progress.start_time,
                    completed_time: progress.completed_time.lock().unwrap().clone(),
                    size_snapshot: progress.size.load(Ordering::Acquire),
                },
                ItemView::File(path, size) => LineItem {
                    path: path.clone(),
                    item_type: ItemType::File,
                    start_time: std::time::Instant::now(),
                    completed_time: Some(std::time::Instant::now()),
                    size_snapshot: *size,
                },
            })
            .collect::<Vec<_>>();

        items.sort();

        items
    }

    pub fn render_until_flag(&self, console: Arc<Mutex<SuperConsole>>, stop_flag: Arc<AtomicBool>) {
        while !stop_flag.load(Ordering::Relaxed) {
            console.lock().unwrap().render(self).unwrap();
            thread::sleep(std::time::Duration::from_millis(100));
        }
    }

    pub fn execute<F>(&self, on_error: Arc<F>)
    where
        F: Fn(String) + Send + Sync + 'static,
    {
        get_dir_size(
            &self.args.directory,
            Arc::new(Mutex::new(HashMap::new())),
            self.scan_view.clone(),
            on_error,
        );
    }
}
