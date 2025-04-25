pub mod file_util;
mod line_item;
mod lines_component;
mod scan_job;
pub mod scan_job_args;

use crossterm;
use lines_component::LinesComponent;
use scan_job::ScanJob;
use scan_job_args::ScanJobArgs;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use superconsole::components::Blank;
use superconsole::{Component, Dimensions, DrawMode, SuperConsole};

pub fn scan_dir(args: ScanJobArgs, size_cache: Arc<Mutex<HashMap<String, u64>>>) -> Vec<String> {
    let job = Arc::new(ScanJob::new(args.clone()));
    let console = Arc::new(Mutex::new(
        SuperConsole::new()
            .ok_or_else(|| anyhow::anyhow!("Not a TTY"))
            .unwrap(),
    ));

    crossbeam::thread::scope(|s| {
        let stop_flag = Arc::new(AtomicBool::new(false));

        let job_clone = job.clone();
        let stop_flag_clone = stop_flag.clone();
        let console_clone = console.clone();
        s.spawn(move |_| job_clone.render_until_flag(console_clone, stop_flag_clone));

        let console_clone = console.clone();
        job.execute(
            size_cache,
            Arc::new(move |msg: String| {
                {
                    if !args.verbose {
                        return;
                    }

                    let lines = LinesComponent::from_str(&msg)
                        .draw(
                            Dimensions::new(
                                crossterm::terminal::size().unwrap().0.into(),
                                usize::MAX,
                            ),
                            DrawMode::Final,
                        )
                        .unwrap();
                    console_clone.lock().unwrap().emit(lines)
                }
                .into()
            }),
        );

        stop_flag.store(true, Ordering::Relaxed);
    })
    .unwrap();

    if let Ok(console) = Arc::try_unwrap(console) {
        let mut console = console.into_inner().unwrap();
        console.emit(
            job.draw(
                Dimensions::new(crossterm::terminal::size().unwrap().0.into(), usize::MAX),
                DrawMode::Final,
            )
            .unwrap(),
        );
        console.finalize(&Blank).unwrap();
    } else {
        eprintln!("Failed to unlock console");
    }

    let job = Arc::try_unwrap(job).unwrap();
    job.get_result()
}
