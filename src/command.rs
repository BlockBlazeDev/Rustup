use std::ffi::OsStr;
use std::io;
use std::process::{self, Command};

use crate::errors::*;
use crate::utils::utils::ExitCode;

pub fn run_command_for_dir<S: AsRef<OsStr>>(
    mut cmd: Command,
    arg0: &str,
    args: &[S],
) -> Result<ExitCode> {
    cmd.args(args);

    // FIXME rust-lang/rust#32254. It's not clear to me
    // when and why this is needed.
    cmd.stdin(process::Stdio::inherit());

    return exec(&mut cmd).chain_err(|| crate::ErrorKind::RunningCommand {
        name: OsStr::new(arg0).to_owned(),
    });

    #[cfg(unix)]
    fn exec(cmd: &mut Command) -> io::Result<ExitCode> {
        use std::os::unix::prelude::*;
        Err(cmd.exec())
    }

    #[cfg(windows)]
    fn exec(cmd: &mut Command) -> io::Result<ExitCode> {
        use winapi::shared::minwindef::{BOOL, DWORD, FALSE, TRUE};
        use winapi::um::consoleapi::SetConsoleCtrlHandler;

        unsafe extern "system" fn ctrlc_handler(_: DWORD) -> BOOL {
            // Do nothing. Let the child process handle it.
            TRUE
        }
        unsafe {
            if SetConsoleCtrlHandler(Some(ctrlc_handler), TRUE) == FALSE {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    "Unable to set console handler",
                ));
            }
        }
        let status = cmd.status()?;
        Ok(ExitCode(status.code().unwrap()))
    }
}
