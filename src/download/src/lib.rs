//! Easy file downloading

#[macro_use]
extern crate error_chain;
extern crate url;

#[cfg(feature = "reqwest-backend")]
extern crate reqwest;

use url::Url;
use std::path::Path;

mod errors;
pub use errors::*;

#[derive(Debug, Copy, Clone)]
pub enum Backend { Curl, Reqwest }

#[derive(Debug, Copy, Clone)]
pub enum Event<'a> {
    ResumingPartialDownload,
    /// Received the Content-Length of the to-be downloaded data.
    DownloadContentLengthReceived(u64),
    /// Received some data.
    DownloadDataReceived(&'a [u8]),
}

fn download_with_backend(backend: Backend,
                         url: &Url,
                         resume_from: u64,
                         callback: &Fn(Event) -> Result<()>)
                         -> Result<()> {
    match backend {
        Backend::Curl => curl::download(url, resume_from, callback),
        Backend::Reqwest => reqwest_be::download(url, resume_from, callback),
    }
}

pub fn download_to_path_with_backend(
    backend: Backend,
    url: &Url,
    path: &Path,
    resume_from_partial: bool,
    callback: Option<&Fn(Event) -> Result<()>>)
    -> Result<()>
{
    use std::cell::RefCell;
    use std::fs::{OpenOptions};
    use std::io::{Read, Write, Seek, SeekFrom};

    || -> Result<()> {
        let (file, resume_from) = if resume_from_partial {
            let possible_partial = OpenOptions::new()
                    .read(true)
                    .open(&path);

            let downloaded_so_far = if let Ok(mut partial) = possible_partial {
                if let Some(cb) = callback {
                    try!(cb(Event::ResumingPartialDownload));

                    let mut buf = vec![0; 32768];
                    let mut downloaded_so_far = 0;
                    loop {
                        let n = try!(partial.read(&mut buf));
                        downloaded_so_far += n as u64;
                        if n == 0 {
                            break;
                        }
                        try!(cb(Event::DownloadDataReceived(&buf[..n])));
                    }

                    downloaded_so_far
                } else {
                    let file_info = try!(partial.metadata());
                    file_info.len()
                }
            } else {
                0
            };

            let mut possible_partial =
                try!(OpenOptions::new()
                        .write(true)
                        .create(true)
                        .open(&path)
                        .chain_err(|| "error opening file for download"));

            try!(possible_partial.seek(SeekFrom::End(0)));

            (possible_partial, downloaded_so_far)
        } else {
            (try!(OpenOptions::new()
                    .write(true)
                    .create(true)
                    .open(&path)
                    .chain_err(|| "error creating file for download")), 0)
        };

        let file = RefCell::new(file);

        try!(download_with_backend(backend, url, resume_from, &|event| {
            if let Event::DownloadDataReceived(data) = event {
                try!(file.borrow_mut().write_all(data)
                     .chain_err(|| "unable to write download to disk"));
            }
            match callback {
                Some(cb) => cb(event),
                None => Ok(())
            }
        }));

        try!(file.borrow_mut().sync_data()
             .chain_err(|| "unable to sync download to disk"));

        Ok(())
    }().map_err(|e| {

        // TODO is there any point clearing up here? What kind of errors will leave us with an unusable partial?
        e
    })
}

/// Download via libcurl; encrypt with the native (or OpenSSl) TLS
/// stack via libcurl
#[cfg(feature = "curl-backend")]
pub mod curl {

    extern crate curl;

    use self::curl::easy::Easy;
    use errors::*;
    use std::cell::RefCell;
    use std::str;
    use std::time::Duration;
    use url::Url;
    use super::Event;

    pub fn download(url: &Url,
                    resume_from: u64,
                    callback: &Fn(Event) -> Result<()> )
                    -> Result<()> {
        // Fetch either a cached libcurl handle (which will preserve open
        // connections) or create a new one if it isn't listed.
        //
        // Once we've acquired it, reset the lifetime from 'static to our local
        // scope.
        thread_local!(static EASY: RefCell<Easy> = RefCell::new(Easy::new()));
        EASY.with(|handle| {
            let mut handle = handle.borrow_mut();

            try!(handle.url(&url.to_string()).chain_err(|| "failed to set url"));
            try!(handle.follow_location(true).chain_err(|| "failed to set follow redirects"));

            if resume_from > 0 {
                try!(handle.resume_from(resume_from)
                    .chain_err(|| "setting the range header for download resumption"));
            } else {
                // an error here indicates that the range header isn't supported by underlying curl,
                // so there's nothing to "clear" - safe to ignore this error.
                let _ = handle.resume_from(0);
            }

            // Take at most 30s to connect
            try!(handle.connect_timeout(Duration::new(30, 0)).chain_err(|| "failed to set connect timeout"));

            {
                let cberr = RefCell::new(None);
                let mut transfer = handle.transfer();

                // Data callback for libcurl which is called with data that's
                // downloaded. We just feed it into our hasher and also write it out
                // to disk.
                try!(transfer.write_function(|data| {
                    match callback(Event::DownloadDataReceived(data)) {
                        Ok(()) => Ok(data.len()),
                        Err(e) => {
                            *cberr.borrow_mut() = Some(e);
                            Ok(0)
                        }
                    }
                }).chain_err(|| "failed to set write"));

                // Listen for headers and parse out a `Content-Length` if it comes
                // so we know how much we're downloading.
                try!(transfer.header_function(|header| {
                    if let Ok(data) = str::from_utf8(header) {
                        let prefix = "Content-Length: ";
                        if data.starts_with(prefix) {
                            if let Ok(s) = data[prefix.len()..].trim().parse::<u64>() {
                                let msg = Event::DownloadContentLengthReceived(s + resume_from);
                                match callback(msg) {
                                    Ok(()) => (),
                                    Err(e) => {
                                        *cberr.borrow_mut() = Some(e);
                                        return false;
                                    }
                                }
                            }
                        }
                    }
                    true
                }).chain_err(|| "failed to set header"));

                // If an error happens check to see if we had a filesystem error up
                // in `cberr`, but we always want to punt it up.
                try!(transfer.perform().or_else(|e| {
                    // If the original error was generated by one of our
                    // callbacks, return it.
                    match cberr.borrow_mut().take() {
                        Some(cberr) => Err(cberr),
                        None => {
                            // Otherwise, return the error from curl
                            if e.is_file_couldnt_read_file() {
                                Err(e).chain_err(|| ErrorKind::FileNotFound)
                            } else {
                                Err(e).chain_err(|| "error during download")
                            }
                        }
                    }
                }));
            }

            // If we didn't get a 20x or 0 ("OK" for files) then return an error
            let code = try!(handle.response_code().chain_err(|| "failed to get response code"));
            match code {
                0 | 200 ... 299 => {},
                _ => { return Err(ErrorKind::HttpStatus(code).into()); }
            };

            Ok(())
        })
    }
}

#[cfg(feature = "reqwest-backend")]
pub mod reqwest_be {
    extern crate env_proxy;

    use std::io;
    use errors::*;
    use url::Url;
    use super::Event;
    use reqwest::{header, Client, Proxy};

    pub fn download(url: &Url,
                    resume_from: u64,
                    callback: &Fn(Event) -> Result<()>)
                    -> Result<()> {

        // Short-circuit reqwest for the "file:" URL scheme
        if try!(download_from_file_url(url, callback)) {
            return Ok(());
        }

        let maybe_proxy = env_proxy::for_url(url)
            .map(|(addr, port)| {
                String::from(format!("{}:{}", addr, port))
            });

        let client = match url.scheme() {
            "https" => match maybe_proxy {
                None => Client::new()?,
                Some(host_port) => {
                    Client::builder()?
                        .proxy(Proxy::https(&host_port)?)
                        .build()?
                }
            },
            "http" => match maybe_proxy {
                None => Client::new()?,
                Some(host_port) => {
                    Client::builder()?
                        .proxy(Proxy::http(&host_port)?)
                        .build()?
                }
            },
            _ => return Err(format!("unsupported URL scheme: '{}'", url.scheme()).into())
        };

        let mut res = if resume_from > 0 {
            client
                .get(url.clone())?
                .header(header::Range::Bytes(
                    vec![header::ByteRangeSpec::AllFrom(resume_from)]
                ))
                .send()
        } else {
            client.get(url.clone())?.send()
        }.chain_err(|| "failed to make network request")?;

        if !res.status().is_success() {
            let code: u16 = res.status().into();
            return Err(ErrorKind::HttpStatus(code as u32).into());
        }

        let buffer_size = 0x10000;
        let mut buffer = vec![0u8; buffer_size];

        if let Some(len) = res.headers().get::<header::ContentLength>().cloned() {
            try!(callback(Event::DownloadContentLengthReceived(len.0)));
        }

        loop {
            let bytes_read = try!(io::Read::read(&mut res, &mut buffer)
                                  .chain_err(|| "error reading from socket"));

            if bytes_read != 0 {
                try!(callback(Event::DownloadDataReceived(&buffer[0..bytes_read])));
            } else {
                return Ok(());
            }
        }
    }

    fn download_from_file_url(url: &Url,
                              callback: &Fn(Event) -> Result<()>)
                              -> Result<bool> {

        use std::fs;
        use std::io;

        // The file scheme is mostly for use by tests to mock the dist server
        if url.scheme() == "file" {
            let src = try!(url.to_file_path()
                           .map_err(|_| Error::from(format!("bogus file url: '{}'", url))));
            if !src.is_file() {
                // Because some of rustup's logic depends on checking
                // the error when a downloaded file doesn't exist, make
                // the file case return the same error value as the
                // network case.
                return Err(ErrorKind::FileNotFound.into());
            }

            let ref mut f = try!(fs::File::open(src)
                                 .chain_err(|| "unable to open downloaded file"));

            let ref mut buffer = vec![0u8; 0x10000];
            loop {
                let bytes_read = try!(io::Read::read(f, buffer)
                                      .chain_err(|| "unable to read downloaded file"));
                if bytes_read == 0 { break }
                try!(callback(Event::DownloadDataReceived(&buffer[0..bytes_read])));
            }

            Ok(true)
        } else {
            Ok(false)
        }
    }
}

#[cfg(not(feature = "curl-backend"))]
pub mod curl {

    use errors::*;
    use url::Url;
    use super::Event;

    pub fn download(_url: &Url,
                    _resume_from: u64,
                    _callback: &Fn(Event) -> Result<()> )
                    -> Result<()> {
        Err(ErrorKind::BackendUnavailable("curl").into())
    }
}

#[cfg(not(feature = "reqwest-backend"))]
pub mod reqwest_be {

    use errors::*;
    use url::Url;
    use super::Event;

    pub fn download(_url: &Url,
                    _resume_from: u64,
                    _callback: &Fn(Event) -> Result<()> )
                    -> Result<()> {
        Err(ErrorKind::BackendUnavailable("reqwest").into())
    }
}
