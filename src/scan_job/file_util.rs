use crossbeam::channel::{unbounded, Receiver, Sender};
use std::collections::HashMap;
use std::fs;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time;

#[derive(Debug)]
pub struct DirectoryScanProgress {
    pub start_time: time::Instant,
    pub completed_time: Arc<Mutex<Option<time::Instant>>>,
    pub size: Arc<AtomicU64>,
}

#[derive(Debug)]
pub enum ItemView {
    Directory(String, DirectoryScanProgress),
    File(String, u64),
}

type ErrorHandler = dyn Fn(String) + Send + Sync + 'static;

struct ChannelCtrl {
    r: Receiver<Arc<ProcessMessage>>,
    n_active_workers: u32,
}

struct Ctrl {
    s: Sender<Arc<ProcessMessage>>,
    channel_ctrl: Mutex<ChannelCtrl>,
    stop_flag: AtomicBool,
    on_error: Arc<ErrorHandler>,
}

impl Ctrl {
    fn get_message(&self) -> Option<Arc<ProcessMessage>> {
        let mut channel_ctrl = self.channel_ctrl.lock().unwrap();
        if let Ok(msg) = channel_ctrl.r.try_recv() {
            channel_ctrl.n_active_workers += 1;
            Some(msg)
        } else {
            None
        }
    }

    fn on_message_processed(&self, msg: &Arc<ProcessMessage>) {
        let mut channel_ctrl = msg.ctrl.channel_ctrl.lock().unwrap();
        channel_ctrl.n_active_workers -= 1;
        if channel_ctrl.n_active_workers == 0 && channel_ctrl.r.is_empty() {
            msg.ctrl.stop_flag.store(true, Ordering::Release);
        }
    }

    fn err(&self, msg: String) {
        let err_closure = &self.on_error;
        err_closure(msg);
    }

    fn work(ctrl: Arc<Self>) {
        loop {
            match ctrl.get_message() {
                Some(msg) => {
                    ProcessMessage::process(&msg);
                    ctrl.on_message_processed(&msg);
                }
                None => {
                    if ctrl.stop_flag.load(Ordering::Acquire) {
                        return;
                    }
                }
            }
        }
    }
}

struct ProcessMessage {
    path: String,
    size: Arc<AtomicU64>,
    cache_ref: Arc<Mutex<HashMap<String, u64>>>,
    parent: Option<Arc<ProcessMessage>>,
    ctrl: Arc<Ctrl>,
    render_children: Option<Arc<Mutex<Vec<Arc<ItemView>>>>>,
    view: Option<Arc<ItemView>>,
}

impl ProcessMessage {
    fn new(
        path: String,
        size_cache: Arc<Mutex<HashMap<String, u64>>>,
        ctrl: Arc<Ctrl>,
        render_children: Option<Arc<Mutex<Vec<Arc<ItemView>>>>>,
    ) -> Self {
        Self {
            path,
            size: Arc::new(AtomicU64::new(0)),
            cache_ref: size_cache.clone(),
            parent: None,
            ctrl,
            render_children,
            view: None,
        }
    }

    fn from_parent(parent: Arc<ProcessMessage>, child_path: String) -> Self {
        Self {
            path: child_path,
            size: Arc::new(AtomicU64::new(0)),
            cache_ref: parent.cache_ref.clone(),
            ctrl: parent.ctrl.clone(),
            parent: Some(parent),
            render_children: None,
            view: None,
        }
    }

    fn add_size(&self, size: u64) {
        self.size.fetch_add(size, Ordering::Release);

        if let Some(parent) = &self.parent {
            parent.add_size(size);
        }
    }

    fn traverse_path(msg: &Arc<Self>) {
        match fs::read_dir(&msg.path) {
            Ok(entries) => {
                let mut greedy_msg = None;
                let entries = entries.filter_map(Result::ok);
                let mut files = vec![];
                for entry in entries {
                    if let Ok(file_type) = entry.file_type() {
                        if file_type.is_dir() {
                            let mut child_msg = ProcessMessage::from_parent(
                                msg.clone(),
                                entry.path().to_string_lossy().to_string(),
                            );

                            if let Some(render_children) = &msg.render_children {
                                let render_line_item = Arc::new(ItemView::Directory(
                                    entry.path().to_string_lossy().to_string(),
                                    DirectoryScanProgress {
                                        start_time: time::Instant::now(),
                                        completed_time: Arc::new(Mutex::new(None)),
                                        size: child_msg.size.clone(),
                                    },
                                ));
                                child_msg.view = Some(render_line_item.clone());
                                render_children.lock().unwrap().push(render_line_item);
                            }

                            let child_msg = Arc::from(child_msg);
                            if greedy_msg.is_none() {
                                greedy_msg = Some(child_msg);
                            } else {
                                msg.ctrl.s.send(child_msg).unwrap();
                            }
                        } else if file_type.is_file() || file_type.is_symlink() {
                            files.push(entry);
                        }
                    }
                }

                files.iter().for_each(|entry| {
                    let file_size = entry.metadata().map(|m| m.len()).unwrap_or(0);
                    msg.add_size(file_size);

                    if let Some(render_children) = &msg.render_children {
                        let mut render_children = render_children.lock().unwrap();
                        render_children.push(Arc::new(ItemView::File(
                            entry.path().to_string_lossy().to_string(),
                            file_size,
                        )));
                    }
                });

                if let Some(greedy_msg) = greedy_msg {
                    ProcessMessage::process(&greedy_msg);
                    return;
                }
            }
            Err(e) => {
                msg.ctrl
                    .err(format!("Error reading directory '{}': {}", &msg.path, e));
            }
        };
    }

    fn process(msg: &Arc<Self>) {
        if msg.render_children.is_none() {
            if let Some(cache) = msg.cache_ref.lock().unwrap().get(&msg.path) {
                msg.add_size(*cache);
                return;
            }
        }

        ProcessMessage::traverse_path(msg);
    }
}

impl Drop for ProcessMessage {
    fn drop(&mut self) {
        if let Some(view) = &self.view {
            if let ItemView::Directory(_, progress) = view.as_ref() {
                progress
                    .completed_time
                    .lock()
                    .unwrap()
                    .replace(time::Instant::now());
            }
        }

        let size = self.size.load(Ordering::Acquire);
        if size > 1_000_000_000 {
            self.cache_ref
                .lock()
                .unwrap()
                .insert(self.path.clone(), size);
        }
    }
}

pub fn get_dir_size(
    root: &str,
    size_cache: Arc<Mutex<HashMap<String, u64>>>,
    render_view: Arc<Mutex<Vec<Arc<ItemView>>>>,
    on_error: Arc<ErrorHandler>,
) {
    let (dir_s, dir_r) = unbounded::<Arc<ProcessMessage>>();
    let ctrl = Arc::new(Ctrl {
        s: dir_s.clone(),
        channel_ctrl: Mutex::new(ChannelCtrl {
            r: dir_r.clone(),
            n_active_workers: 0,
        }),
        stop_flag: AtomicBool::new(false),
        on_error,
    });
    let root_msg = Arc::new(ProcessMessage::new(
        root.to_string(),
        size_cache.clone(),
        ctrl.clone(),
        Some(render_view),
    ));

    dir_s.send(root_msg.clone()).unwrap();
    thread::scope(|scope| {
        for _ in 0..(num_cpus::get_physical() * 2 / 3).max(1) {
            let ctrl = ctrl.clone();
            scope.spawn(move || Ctrl::work(ctrl));
        }
    });
}
