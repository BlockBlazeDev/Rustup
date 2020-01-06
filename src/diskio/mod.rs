/// Disk IO abstraction for rustup.
///
/// This exists to facilitate high performance extraction even though OS's are
/// imperfect beasts. For detailed design notes see the module source.
//
// When performing IO we have a choice:
// - perform some IO in this thread
// - dispatch some or all IO to another thread
// known tradeoffs:
// NFS: network latency incurred on create, chmod, close calls
// WSLv1: Defender latency incurred on close calls; mutex shared with create calls
// Windows: Defender latency incurred on close calls
// Unix: limited open file count
// Defender : CPU limited, so more service points than cores brings no gain.
// Some machines: IO limited, more service points than cores brings more efficient
// Hello world footprint ~350MB, so around 400MB to install is considered ok.
// IO utilisation.
// All systems: dispatching to a thread has some overhead.
// Basic idea then is a locally measured congestion control problem.
// Underlying system has two
// dimensions - how much work we have queued, and how much work we execute
// at once. Queued work is both memory footprint, and unless each executor
// is performing complex logic, potentially concurrent work.
// Single core machines - thread anyway, they probably don't have SSDs?
// How many service points? Blocking latency due to networks and disks
// is independent of CPU: more threads will garner more throughput up
// to actual resource service capability.
// so:
// a) measure time around each IO op from dispatch to completion.
// b) create more threads than CPUs - 2x for now (because threadpool
//    doesn't allow creating dynamically), with very shallow stacks
//    (say 1MB)
// c) keep adding work while the P95? P80? of completion stays the same
//    when pNN starts to increase either (i) we've saturated the system
//    or (ii) other work coming in has saturated the system or (iii) this
//    sort of work is a lot harder to complete. We use NN<100 to avoid
//    having jitter throttle us inappropriately. We use a high NN to
//    avoid making the system perform poorly for the user / other users
//    on shared components. Perhaps time-to-completion should be scaled by size.
// d) if we have a lot of (iii) we should respond to it the same as (i), so
//    lets reduce this to (i) and (ii). Being unable to tell the difference
//    between load we created and others, we have to throttle back when
//    the system saturates. Our most throttled position will be one service
//    worker: dispatch IO, extract the next text, wait for IO completion,
//    repeat.
// e) scaling up and down: TCP's lessons here are pretty good. So exponential
//    up - single thread and measure. two, 4 etc. When Pnn goes bad back off
//    for a time and then try again with linear increase (it could be case (ii)
//    - lots of room to experiment here; working with a time based approach is important
//    as that is the only way we can detect saturation: we are not facing
//    loss or errors in this model.
// f) data gathering: record (name, bytes, start, duration)
//    write to disk afterwards as a csv file?
pub mod immediate;
pub mod threaded;

use crate::utils::notifications::Notification;

use std::env;
use std::fs::OpenOptions;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use std::time::{Duration, Instant};

#[derive(Debug)]
pub enum Kind {
    Directory,
    File(Vec<u8>),
}

#[derive(Debug)]
pub struct Item {
    /// The path to operate on
    pub full_path: PathBuf,
    /// The operation to perform
    pub kind: Kind,
    /// When the operation started
    pub start: Option<Instant>,
    /// Amount of time the operation took to finish
    pub finish: Option<Duration>,
    /// The length of the file, for files (for stats)
    pub size: Option<usize>,
    /// The result of the operation
    pub result: io::Result<()>,
    /// The mode to apply
    pub mode: u32,
}

impl Item {
    pub fn make_dir(full_path: PathBuf, mode: u32) -> Self {
        Self {
            full_path,
            kind: Kind::Directory,
            start: None,
            finish: None,
            size: None,
            result: Ok(()),
            mode,
        }
    }

    pub fn write_file(full_path: PathBuf, content: Vec<u8>, mode: u32) -> Self {
        let len = content.len();
        Self {
            full_path,
            kind: Kind::File(content),
            start: None,
            finish: None,
            size: Some(len),
            result: Ok(()),
            mode,
        }
    }
}

/// Trait object for performing IO. At this point the overhead
/// of trait invocation is not a bottleneck, but if it becomes
/// one we could consider an enum variant based approach instead.
pub trait Executor {
    /// Perform a single operation.
    /// During overload situations previously queued items may
    /// need to be completed before the item is accepted:
    /// consume the returned iterator.
    fn execute(&mut self, mut item: Item) -> Box<dyn Iterator<Item = Item> + '_> {
        item.start = Some(Instant::now());
        self.dispatch(item)
    }

    /// Actually dispatch a operation.
    /// This is called by the default execute() implementation and
    /// should not be called directly.
    fn dispatch(&mut self, item: Item) -> Box<dyn Iterator<Item = Item> + '_>;

    /// Wrap up any pending operations and iterate over them.
    /// All operations submitted before the join will have been
    /// returned either through ready/complete or join once join
    /// returns.
    fn join(&mut self) -> Box<dyn Iterator<Item = Item> + '_>;

    /// Iterate over completed items.
    fn completed(&mut self) -> Box<dyn Iterator<Item = Item> + '_>;
}

/// Trivial single threaded IO to be used from executors.
/// (Crazy sophisticated ones can obviously ignore this)
pub fn perform(item: &mut Item) {
    // directories: make them, TODO: register with the dir existence cache.
    // Files, write them.
    item.result = match item.kind {
        Kind::Directory => create_dir(&item.full_path),
        Kind::File(ref contents) => write_file(&item.full_path, &contents, item.mode),
    };
    item.finish = item.start.map(|s| s.elapsed());
}

#[allow(unused_variables)]
pub fn write_file<P: AsRef<Path>, C: AsRef<[u8]>>(
    path: P,
    contents: C,
    mode: u32,
) -> io::Result<()> {
    let mut opts = OpenOptions::new();
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        opts.mode(mode);
    }
    let path = path.as_ref();
    let path_display = format!("{}", path.display());
    let mut f = {
        trace_scoped!("creat", "name": path_display);
        opts.write(true).create(true).truncate(true).open(path)?
    };
    let contents = contents.as_ref();
    let len = contents.len();
    {
        trace_scoped!("write", "name": path_display, "len": len);
        f.write_all(contents)?;
    }
    {
        trace_scoped!("close", "name:": path_display);
        drop(f);
    }
    Ok(())
}

pub fn create_dir<P: AsRef<Path>>(path: P) -> io::Result<()> {
    let path = path.as_ref();
    let path_display = format!("{}", path.display());
    trace_scoped!("create_dir", "name": path_display);
    std::fs::create_dir(path)
}

/// Get the executor for disk IO.
pub fn get_executor<'a>(
    notify_handler: Option<&'a dyn Fn(Notification<'_>)>,
) -> Box<dyn Executor + 'a> {
    // If this gets lots of use, consider exposing via the config file.
    if let Ok(thread_str) = env::var("RUSTUP_IO_THREADS") {
        if thread_str == "disabled" {
            Box::new(immediate::ImmediateUnpacker::new())
        } else if let Ok(thread_count) = thread_str.parse::<usize>() {
            Box::new(threaded::Threaded::new_with_threads(
                notify_handler,
                thread_count,
            ))
        } else {
            Box::new(threaded::Threaded::new(notify_handler))
        }
    } else {
        Box::new(threaded::Threaded::new(notify_handler))
    }
}
