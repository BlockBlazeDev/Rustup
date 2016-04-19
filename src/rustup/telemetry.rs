use time;
use rustup_utils::{raw, utils};
use rustc_serialize::json;

use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, PartialEq)]
pub enum TelemetryMode {
    On,
    Off,
}

#[derive(RustcDecodable, RustcEncodable, Debug)]
pub enum TelemetryEvent {
    RustcRun { duration_ms: u64, exit_code: i32, errors: Option<HashSet<String>> },
    ToolchainUpdate { toolchain: String, success: bool } ,
    TargetAdd { toolchain: String, target: String, success: bool },
}

#[derive(RustcDecodable, RustcEncodable, Debug)]
struct LogMessage {
    log_time_s: i64,
    event: TelemetryEvent,
    version: i32,
}

#[derive(Debug)]
pub struct Telemetry {
    telemetry_dir: PathBuf
}

const LOG_FILE_VERSION: i32 = 1;
const MAX_TELEMETRY_FILES: usize = 100;

impl Telemetry {
    pub fn new(telemetry_dir: PathBuf) -> Telemetry {
        Telemetry { telemetry_dir: telemetry_dir }
    }

    pub fn log_telemetry(&self, event: TelemetryEvent) {
        let current_time = time::now_utc();
        let ln = LogMessage { log_time_s: current_time.to_timespec().sec,
                              event: event,
                              version: LOG_FILE_VERSION };

        let json = json::encode(&ln).unwrap();

        let filename = format!("log-{}-{:02}-{:02}.json", current_time.tm_year + 1900, current_time.tm_mon + 1, current_time.tm_mday);

        // Check for the telemetry file. If it doesn't exist, it's a new day.
        // If it is a new day, then attempt to clean the telemetry directory.
        if !raw::is_file(&self.telemetry_dir.join(&filename)) {
            self.clean_telemetry_dir();
        }

        let _ = utils::append_file("telemetry", &self.telemetry_dir.join(&filename), &json);
    }

    pub fn clean_telemetry_dir(&self) {
        let contents = self.telemetry_dir.read_dir().unwrap();

        let mut telemetry_files: Vec<PathBuf> = Vec::new();

        for c in contents {
            let x = c.unwrap();
            let filename = x.path().file_name().unwrap().to_str().unwrap().to_owned();
            if filename.starts_with("log") && filename.ends_with("json") {
                telemetry_files.push(x.path());
            }
        }

        println!("found {:?} files to delete", telemetry_files.len());

        if telemetry_files.len() < MAX_TELEMETRY_FILES {
            return;
        }

        let dl: usize = telemetry_files.len() - MAX_TELEMETRY_FILES;
        let dl = dl + 1 as usize;

        telemetry_files.sort();
        telemetry_files.dedup();

        for i in 0..dl {
            let i = i as usize;
            let _ = fs::remove_file(&telemetry_files[i]);
        }
    }
}
