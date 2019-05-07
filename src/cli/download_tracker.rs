use crate::term2;
use rustup::dist::Notification as In;
use rustup::utils::tty;
use rustup::utils::Notification as Un;
use rustup::Notification;
use std::collections::VecDeque;
use std::fmt;
use std::io::Write;
use term::Terminal;
use time::precise_time_s;

/// Keep track of this many past download amounts
const DOWNLOAD_TRACK_COUNT: usize = 5;

/// Tracks download progress and displays information about it to a terminal.
pub struct DownloadTracker {
    /// Content-Length of the to-be downloaded object.
    content_len: Option<usize>,
    /// Total data downloaded in bytes.
    total_downloaded: usize,
    /// Data downloaded this second.
    downloaded_this_sec: usize,
    /// Keeps track of amount of data downloaded every last few secs.
    /// Used for averaging the download speed.
    downloaded_last_few_secs: VecDeque<usize>,
    /// Time stamp of the last second
    last_sec: Option<f64>,
    /// How many seconds have elapsed since the download started
    seconds_elapsed: u32,
    /// The terminal we write the information to.
    /// XXX: Could be a term trait, but with #1818 on the horizon that
    ///      is a pointless change to make - better to let that transition
    ///      happen and take stock after that.
    term: term2::StdoutTerminal,
    /// Whether we displayed progress for the download or not.
    ///
    /// If the download is quick enough, we don't have time to
    /// display the progress info.
    /// In that case, we do not want to do some cleanup stuff we normally do.
    ///
    /// If we have displayed progress, this is the number of characters we
    /// rendered, so we can erase it cleanly.
    displayed_charcount: Option<usize>,
}

impl DownloadTracker {
    /// Creates a new DownloadTracker.
    pub fn new() -> Self {
        DownloadTracker {
            content_len: None,
            total_downloaded: 0,
            downloaded_this_sec: 0,
            downloaded_last_few_secs: VecDeque::with_capacity(DOWNLOAD_TRACK_COUNT),
            seconds_elapsed: 0,
            last_sec: None,
            term: term2::stdout(),
            displayed_charcount: None,
        }
    }

    pub fn handle_notification(&mut self, n: &Notification<'_>) -> bool {
        match *n {
            Notification::Install(In::Utils(Un::DownloadContentLengthReceived(content_len))) => {
                self.content_length_received(content_len);

                true
            }
            Notification::Install(In::Utils(Un::DownloadDataReceived(data))) => {
                if tty::stdout_isatty() {
                    self.data_received(data.len());
                }
                true
            }
            Notification::Install(In::Utils(Un::DownloadFinished)) => {
                self.download_finished();
                true
            }
            _ => false,
        }
    }

    /// Notifies self that Content-Length information has been received.
    pub fn content_length_received(&mut self, content_len: u64) {
        self.content_len = Some(content_len as usize);
    }

    /// Notifies self that data of size `len` has been received.
    pub fn data_received(&mut self, len: usize) {
        self.total_downloaded += len;
        self.downloaded_this_sec += len;

        let current_time = precise_time_s();

        match self.last_sec {
            None => self.last_sec = Some(current_time),
            Some(start) => {
                let elapsed = current_time - start;
                if elapsed >= 1.0 {
                    self.seconds_elapsed += 1;

                    self.display();
                    self.last_sec = Some(current_time);
                    if self.downloaded_last_few_secs.len() == DOWNLOAD_TRACK_COUNT {
                        self.downloaded_last_few_secs.pop_back();
                    }
                    self.downloaded_last_few_secs
                        .push_front(self.downloaded_this_sec);
                    self.downloaded_this_sec = 0;
                }
            }
        }
    }
    /// Notifies self that the download has finished.
    pub fn download_finished(&mut self) {
        if self.displayed_charcount.is_some() {
            // Display the finished state
            self.display();
            let _ = writeln!(self.term);
        }
        self.prepare_for_new_download();
    }
    /// Resets the state to be ready for a new download.
    fn prepare_for_new_download(&mut self) {
        self.content_len = None;
        self.total_downloaded = 0;
        self.downloaded_this_sec = 0;
        self.downloaded_last_few_secs.clear();
        self.seconds_elapsed = 0;
        self.last_sec = None;
        self.displayed_charcount = None;
    }
    /// Display the tracked download information to the terminal.
    fn display(&mut self) {
        let total_h = Size(self.total_downloaded);
        let sum = self.downloaded_last_few_secs.iter().fold(0, |a, &v| a + v);
        let len = self.downloaded_last_few_secs.len();
        let speed = if len > 0 { sum / len } else { 0 };
        let speed_h = Size(speed);

        // First, move to the start of the current line and clear it.
        let _ = self.term.carriage_return();
        // We'd prefer to use delete_line() but on Windows it seems to
        // sometimes do unusual things
        // let _ = self.term.as_mut().unwrap().delete_line();
        // So instead we do:
        if let Some(n) = self.displayed_charcount {
            // This is not ideal as very narrow terminals might mess up,
            // but it is more likely to succeed until term's windows console
            // fixes whatever's up with delete_line().
            let _ = write!(self.term, "{}", " ".repeat(n));
            let _ = self.term.flush();
            let _ = self.term.carriage_return();
        }

        let output = match self.content_len {
            Some(content_len) => {
                let content_len_h = Size(content_len);
                let content_len = content_len as f64;
                let percent = (self.total_downloaded as f64 / content_len) * 100.;
                let remaining = content_len - self.total_downloaded as f64;
                let eta_h = Duration(remaining / speed as f64);
                format!(
                    "{} / {} ({:3.0} %) {}/s ETA: {}",
                    total_h, content_len_h, percent, speed_h, eta_h
                )
            }
            None => format!("Total: {} Speed: {}/s", total_h, speed_h),
        };

        let _ = write!(self.term, "{}", output);
        // Since stdout is typically line-buffered and we don't print a newline, we manually flush.
        let _ = self.term.flush();
        self.displayed_charcount = Some(output.chars().count());
    }
}

/// Human readable representation of duration(seconds).
struct Duration(f64);

impl fmt::Display for Duration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // repurposing the alternate mode for ETA
        let sec = self.0;

        if sec.is_infinite() {
            write!(f, "Unknown")
        } else {
            // we're doing modular arithmetic, treat as integer
            let sec = sec as u32;
            if sec > 48 * 3600 {
                let d = sec / (24 * 3600);
                let h = sec % (24 * 3600);
                let min = sec % 3600;
                let sec = sec % 60;

                write!(f, "{:3} days {:2} h {:2} min {:2} s", d, h, min, sec) // XYZ days PQ h RS min TU s
            } else if sec > 6_000 {
                let h = sec / 3600;
                let min = sec % 3600;
                let sec = sec % 60;

                write!(f, "{:3} h {:2} min {:2} s", h, min, sec) // XYZ h PQ min RS s
            } else if sec > 100 {
                let min = sec / 60;
                let sec = sec % 60;

                write!(f, "{:3} min {:2} s", min, sec) // XYZ min PQ s
            } else {
                write!(f, "{:3.0} s", self.0) // XYZ s
            }
        }
    }
}

/// Human readable size (bytes)
struct Size(usize);

impl fmt::Display for Size {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        const KIB: f64 = 1024.0;
        const MIB: f64 = KIB * KIB;
        let size = self.0 as f64;

        if size >= MIB {
            write!(f, "{:5.1} MiB", size / MIB) // XYZ.P MiB
        } else if size >= KIB {
            write!(f, "{:5.1} KiB", size / KIB)
        } else {
            write!(f, "{:3.0} B", size)
        }
    }
}
