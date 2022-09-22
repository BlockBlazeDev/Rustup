use std::boxed::Box;
use std::cell::RefCell;
use std::collections::HashMap;
use std::default::Default;
use std::fmt::Debug;
use std::io::Cursor;
use std::panic;
use std::path::{Path, PathBuf};
use std::sync::Once;
use std::sync::{Arc, Mutex};

use home::env as home;
use rand::{thread_rng, Rng};

pub(crate) mod argsource;
pub(crate) mod cwdsource;
pub(crate) mod filesource;
mod homethunk;
pub(crate) mod varsource;

use argsource::*;
use cwdsource::*;
use filesource::*;
use varsource::*;

/// An abstraction for the current process
///
/// This provides replacements env::arg*, env::var*, and the standard files
/// io::std* with traits that are customisable for tests. As a result any macros
/// or code that have non-pluggable usage of those are incompatible with
/// CurrentProcess and must not be used. That includes \[e\]println! as well as
/// third party crates.
///
/// CurrentProcess is used via an instance in a thread local variable; when
/// making new threads, be sure to copy CurrentProcess::process() into the new
/// thread before calling any code that may need to use a CurrentProcess
/// function.
///
/// Run some code using with: this will set the current instance, call your
/// function, then finally reset the instance at the end before returning.
///
/// Testing level interoperation with external code that depends on environment
/// variables could be possible with a hypothetical  `with_projected()` which
/// would be a zero-cost operation in real processes, but in test processes will
/// take a lock out to mutually exclude other code, then overwrite the current
/// value of std::env::vars, restoring it at the end. However, the only use for
/// that today is a test of cargo::home, which is now implemented in a separate
/// crate, so we've just deleted the test.
///
/// A thread local is used to permit the instance to be available to the entire
/// rustup library without needing to explicitly wire this normally global state
/// everywhere; and a trait object with dyn dispatch is likewise used to avoid
/// needing to thread trait parameters across the entire code base: none of the
/// methods are in performance critical loops (except perhaps progress bars -
/// and even there we should be doing debouncing and managing update rates).
/// The real trait is CurrentProcess; HomeProcess is a single trait because
/// Box<T> only allows autotraits to be added to it; so we use a subtrait to add
/// home::Env in.
pub trait HomeProcess: CurrentProcess + home::Env {
    fn clone_boxed(&self) -> Box<dyn HomeProcess>;
}

// Machinery for Cloning boxes

impl<T> HomeProcess for T
where
    T: 'static + CurrentProcess + home::Env + Clone,
{
    fn clone_boxed(&self) -> Box<dyn HomeProcess + 'static> {
        Box::new(T::clone(self))
    }
}

impl Clone for Box<dyn HomeProcess + 'static> {
    fn clone(&self) -> Self {
        HomeProcess::clone_boxed(self.as_ref())
    }
}

pub trait CurrentProcess:
    ArgSource
    + CurrentDirSource
    + VarSource
    + StdoutSource
    + StderrSource
    + StdinSource
    + ProcessSource
    + Debug
{
    fn clone_boxed(&self) -> Box<dyn CurrentProcess>;

    fn name(&self) -> Option<String>;
}

// Machinery for Cloning boxes

impl<T> CurrentProcess for T
where
    T: 'static
        + Clone
        + Debug
        + ArgSource
        + CurrentDirSource
        + VarSource
        + StdoutSource
        + StderrSource
        + StdinSource
        + ProcessSource,
{
    fn clone_boxed(&self) -> Box<dyn CurrentProcess + 'static> {
        Box::new(T::clone(self))
    }

    fn name(&self) -> Option<String> {
        let arg0 = match self.var("RUSTUP_FORCE_ARG0") {
            Ok(v) => Some(v),
            Err(_) => self.args().next(),
        }
        .map(PathBuf::from);

        arg0.as_ref()
            .and_then(|a| a.file_stem())
            .and_then(std::ffi::OsStr::to_str)
            .map(String::from)
    }
}

impl Clone for Box<dyn CurrentProcess + 'static> {
    fn clone(&self) -> Self {
        self.as_ref().clone_boxed()
    }
}

/// Obtain the current instance of CurrentProcess
pub fn process() -> Box<dyn CurrentProcess> {
    CurrentProcess::clone_boxed(&*home_process())
}

/// Obtain the current instance of HomeProcess
pub(crate) fn home_process() -> Box<dyn HomeProcess> {
    match PROCESS.with(|p| p.borrow().clone()) {
        None => panic!("No process instance"),
        Some(p) => p,
    }
}

static HOOK_INSTALLED: Once = Once::new();

/// Run a function in the context of a process definition.
///
/// If the function panics, the process definition *in that thread* is cleared
/// by an implicitly installed global panic hook.
pub fn with<F, R>(process: Box<dyn HomeProcess>, f: F) -> R
where
    F: FnOnce() -> R,
{
    HOOK_INSTALLED.call_once(|| {
        let orig_hook = panic::take_hook();
        panic::set_hook(Box::new(move |info| {
            clear_process();
            orig_hook(info);
        }));
    });

    PROCESS.with(|p| {
        if let Some(old_p) = &*p.borrow() {
            panic!("current process already set {:?}", old_p);
        }
        *p.borrow_mut() = Some(process);
        let result = f();
        *p.borrow_mut() = None;
        result
    })
}

/// Internal - for the panic hook only
fn clear_process() {
    PROCESS.with(|p| p.replace(None));
}

thread_local! {
    pub(crate) static PROCESS:RefCell<Option<Box<dyn HomeProcess>>> = RefCell::new(None);
}

// PID related things

pub trait ProcessSource {
    /// Returns a unique id for the process.
    ///
    /// Real process ids are <= u32::MAX.
    /// Test process ids are > u32::MAX
    fn id(&self) -> u64;
}

// ----------- real process -----------------

#[derive(Clone, Debug, Default)]
pub struct OSProcess {}

impl ProcessSource for OSProcess {
    fn id(&self) -> u64 {
        std::process::id() as u64
    }
}

// ------------ test process ----------------

#[derive(Clone, Debug, Default)]
pub struct TestProcess {
    pub cwd: PathBuf,
    pub args: Vec<String>,
    pub vars: HashMap<String, String>,
    pub id: u64,
    pub stdin: TestStdinInner,
    pub stdout: TestWriterInner,
    pub stderr: TestWriterInner,
}

impl TestProcess {
    pub fn new<P: AsRef<Path>, A: AsRef<str>>(
        cwd: P,
        args: &[A],
        vars: HashMap<String, String>,
        stdin: &str,
    ) -> Self {
        TestProcess {
            cwd: cwd.as_ref().to_path_buf(),
            args: args.iter().map(|s| s.as_ref().to_string()).collect(),
            vars,
            id: TestProcess::new_id(),
            stdin: Arc::new(Mutex::new(Cursor::new(stdin.to_string()))),
            stdout: Arc::new(Mutex::new(Vec::new())),
            stderr: Arc::new(Mutex::new(Vec::new())),
        }
    }
    fn new_id() -> u64 {
        let low_bits: u64 = std::process::id() as u64;
        let mut rng = thread_rng();
        let high_bits = rng.gen_range(0..u32::MAX) as u64;
        high_bits << 32 | low_bits
    }

    /// Extracts the stdout from the process
    pub fn get_stdout(&self) -> Vec<u8> {
        self.stdout
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone()
    }

    /// Extracts the stderr from the process
    pub fn get_stderr(&self) -> Vec<u8> {
        self.stderr
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone()
    }
}

impl ProcessSource for TestProcess {
    fn id(&self) -> u64 {
        self.id
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::env;

    use super::{process, with, ProcessSource, TestProcess};

    #[test]
    fn test_instance() {
        let proc = TestProcess::new(
            env::current_dir().unwrap(),
            &["foo", "bar", "baz"],
            HashMap::default(),
            "",
        );
        with(Box::new(proc.clone()), || {
            assert_eq!(proc.id(), process().id(), "{:?} != {:?}", proc, process())
        });
    }
}
