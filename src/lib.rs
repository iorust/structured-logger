// (c) 2023-present, IO Rust. All rights reserved.
// See the file LICENSE for licensing terms.

//! # Structured Logger
//!
//! A logging implementation for the [`log`] crate that logs structured values
//! either synchronous or asynchronous, as JSON, CBOR, or any other format,
//! into a file, stderr, stdout, or any other destination.
//! To initialize the logger use the [`Builder`] struct.
//! It is inspired by [std-logger](https://github.com/Thomasdezeeuw/std-logger).
//!
//! This crate provides only a logging implementation. To do actual logging use
//! the [`log`] crate and it's various macros.
//!
//! ## Crate features
//!
//! This crate has three features:
//! * `log-panic`, enabled by default.
//!
//! ### Log-panic feature
//!
//! The `log-panic` feature will log all panics using the `error` severity,
//! rather then using the default panic handler. It will log the panic message
//! as well as the location and a backtrace, see the log output for an
//! [`panic_log`] example.
//!
//! ## Examples
//!
//! * Log panics example: <https://github.com/iorust/structured-logger/blob/main/examples/panic_log.rs>
//! * Async log example: <https://github.com/iorust/structured-logger/blob/main/examples/async_log.rs>
//!
//! Muilti writers example:
//! ```rust
//! use serde::Serialize;
//! use std::{fs::File, io::stdout};
//! use structured_logger::{json::new_writer, unix_ms, Builder};
//!
//! fn main() {
//!     // Initialize the logger.
//!     // Optional: create a file to write logs to.
//!     let log_file = File::options()
//!         .create(true)
//!         .append(true)
//!         .open("app.log")
//!         .unwrap();
//!
//!     // Builder::with_level("debug")
//!     Builder::new()
//!         // Optional: set a specific writer (format to JSON, write to stdout) for target "api".
//!         .with_target_writer("api", new_writer(stdout()))
//!         // Optional: set a specific writer (format to JSON, write to app.log file) for target "file".
//!         .with_target_writer("file", new_writer(log_file))
//!         .init();
//!
//!     let kv = ContextLog {
//!         uid: "user123".to_string(),
//!         action: "upate_book".to_string(),
//!     };
//!
//!     log::info!("hello world");
//!     // This log will be written to stderr (default writer):
//!     // {"level":"INFO","message":"hello world","target":"simple","timestamp":1679745592127}
//!
//!     log::info!(target: "api",
//!         method = "GET",
//!         path = "/hello",
//!         status = 200_u16,
//!         start = unix_ms(),
//!         elapsed = 10_u64,
//!         kv = log::as_serde!(kv);
//!         "",
//!     );
//!     // This log will be written to stdout:
//!     // {"elapsed":10,"kv":{"uid":"user123","action":"upate_book"},"level":"INFO","message":"","method":"GET","path":"/hello","start":1679745592127,"status":200,"target":"api","timestamp":1679745592127}
//!
//!     log::info!(target: "file",
//!         method = "GET",
//!         path = "/hello",
//!         status = 200_u16,
//!         start = unix_ms(),
//!         elapsed = 10_u64,
//!         kv = log::as_serde!(kv);
//!         "",
//!     );
//!     // This log will be written to file "app.log":
//!     // {"elapsed":10,"kv":{"uid":"user123","action":"upate_book"},"level":"INFO","message":"","method":"GET","path":"/hello","start":1679745592127,"status":200,"target":"file","timestamp":1679745592127}
//! }
//!
//! #[derive(Serialize)]
//! struct ContextLog {
//!     uid: String,
//!     action: String,
//! }
//! ```
//!
//! [`panic_log`]: https://github.com/iorust/structured-logger/blob/main/examples/panic_log.rs
//! [`log`]: https://crates.io/crates/log
//!

#![doc(html_root_url = "https://docs.rs/structured-logger/latest")]
#![allow(clippy::needless_doctest_main)]

use log::{
    kv::Error, kv::Key, kv::Value, kv::Visitor, Level, LevelFilter, Metadata, Record,
    SetLoggerError,
};
use std::{
    collections::BTreeMap,
    env, io,
    time::{SystemTime, UNIX_EPOCH},
};

// /// A type alias for BTreeMap<Key<'a>, Value<'a>>.
// /// BTreeMap is used to keep the order of the keys.
// type Log<'a> = BTreeMap<Key<'a>, Value<'a>>;

/// A trait that defines how to write a log. You can implement this trait for your custom formatting and writing destination.
///
/// Implementation examples:
/// * <https://github.com/iorust/structured-logger/blob/main/src/json.rs>
/// * <https://github.com/iorust/structured-logger/blob/main/src/async_json.rs>
pub trait Writer {
    /// Writes a structured log to the underlying io::Write instance.
    fn write_log(&self, value: &BTreeMap<Key, Value>) -> Result<(), io::Error>;
}

pub mod async_json;
pub mod json;
use json::new_writer;

/// A struct to initialize the logger.
pub struct Builder {
    filter: LevelFilter,
    default_writer: Box<dyn Writer>,
    writers: BTreeMap<&'static str, Box<dyn Writer>>,
}

impl Default for Builder {
    fn default() -> Self {
        Self::new()
    }
}

impl Builder {
    /// Returns a new Logger with default configuration.
    /// The default configuration is:
    /// - level filter: get from the environment variable by `get_env_level()`.
    /// - default writer: write to stderr in JSON format.
    pub fn new() -> Self {
        Builder {
            filter: get_env_level(),
            default_writer: new_writer(io::stderr()),
            writers: BTreeMap::new(),
        }
    }

    /// Returns a new Logger with a given level filter.
    /// `level` is a string that can be parsed to `log::LevelFilter`.
    /// Such as "OFF", "ERROR", "WARN", "INFO", "DEBUG", "TRACE", ignore ascii case.
    pub fn with_level(level: &str) -> Self {
        Builder {
            filter: level.parse().unwrap_or(LevelFilter::Info),
            default_writer: new_writer(io::stderr()),
            writers: BTreeMap::new(),
        }
    }

    /// Returns a new Logger with a given `target` and `writer`.
    /// `target` is a string that be used as a log target.
    /// `writer` is a struct that implements the `Writer` trait.
    /// You can call this method multiple times to add multiple writers.
    pub fn with_target_writer(self, target: &'static str, writer: Box<dyn Writer>) -> Self {
        let mut cfg = Builder {
            filter: self.filter,
            default_writer: self.default_writer,
            writers: self.writers,
        };
        cfg.writers.insert(target, writer);
        cfg
    }

    /// Initialize the logger.
    ///
    /// See the [crate level documentation] for more.
    ///
    /// [crate level documentation]: index.html
    ///
    /// # Panics
    ///
    /// This will panic if the logger fails to initialize. Use [`Builder::try_init`] if
    /// you want to handle the error yourself.
    pub fn init(self) {
        self.try_init()
            .unwrap_or_else(|err| panic!("failed to initialize the logger: {err}"));
    }

    /// Try to initialize the logger.
    ///
    /// Unlike [`Builder::init`] this doesn't panic when the logger fails to initialize.
    /// See the [crate level documentation] for more.
    ///
    /// [`init`]: fn.init.html
    /// [crate level documentation]: index.html
    pub fn try_init(self) -> Result<(), SetLoggerError> {
        let logger = Box::new(Logger {
            filter: self.filter,
            default_writer: self.default_writer,
            writers: self.writers,
        });
        log::set_boxed_logger(logger)?;
        log::set_max_level(self.filter);

        #[cfg(feature = "log-panic")]
        std::panic::set_hook(Box::new(log_panic));
        Ok(())
    }
}

/// Returns the current unix timestamp in milliseconds.
pub fn unix_ms() -> u64 {
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time before Unix epoch");
    ts.as_millis() as u64
}

/// Returns the log level from the environment variables.
pub fn get_env_level() -> LevelFilter {
    for var in &["LOG", "LOG_LEVEL"] {
        if let Ok(level) = env::var(var) {
            if let Ok(level) = level.parse() {
                return level;
            }
        }
    }

    if env::var("TRACE").is_ok() {
        LevelFilter::Trace
    } else if env::var("DEBUG").is_ok() {
        LevelFilter::Debug
    } else {
        LevelFilter::Info
    }
}

struct Logger {
    filter: LevelFilter,
    default_writer: Box<dyn Writer>,
    writers: BTreeMap<&'static str, Box<dyn Writer>>,
}

impl Logger {
    fn get_writer(&self, target: &str) -> &dyn Writer {
        if let Some(writer) = self.writers.get(target) {
            writer.as_ref()
        } else {
            self.default_writer.as_ref()
        }
    }

    fn try_log(&self, record: &Record) -> Result<(), io::Error> {
        let kvs = record.key_values();
        let mut visitor = KeyValueVisitor(BTreeMap::new());
        let _ = kvs.visit(&mut visitor);

        visitor
            .0
            .insert(Key::from("target"), Value::from(record.target()));

        let args = record.args();
        let msg: String;
        if let Some(msg) = args.as_str() {
            visitor.0.insert(Key::from("message"), Value::from(msg));
        } else {
            msg = args.to_string();
            visitor.0.insert(Key::from("message"), Value::from(&msg));
        }

        let level = record.level();
        visitor
            .0
            .insert(Key::from("level"), Value::from(level.as_str()));

        if level <= Level::Warn {
            if let Some(val) = record.module_path() {
                visitor.0.insert(Key::from("module"), Value::from(val));
            }
            if let Some(val) = record.file() {
                visitor.0.insert(Key::from("file"), Value::from(val));
            }
            if let Some(val) = record.line() {
                visitor.0.insert(Key::from("line"), Value::from(val));
            }
        }

        visitor
            .0
            .insert(Key::from("timestamp"), Value::from(unix_ms()));
        self.get_writer(record.target()).write_log(&visitor.0)?;
        Ok(())
    }
}

unsafe impl Sync for Logger {}
unsafe impl Send for Logger {}

impl log::Log for Logger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        self.filter >= metadata.level()
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            if let Err(err) = self.try_log(record) {
                // should never happen, but if it does, we log it.
                eprintln!(
                    "{{\"level\":\"ERROR\",\"message\":\"failed to write log: {}\",\"target\":\"Logger\",\"timestamp\":{}}}",
                     err, unix_ms(),
                );
            }
        }
    }

    fn flush(&self) {}
}

struct KeyValueVisitor<'kvs>(BTreeMap<Key<'kvs>, Value<'kvs>>);

impl<'kvs> Visitor<'kvs> for KeyValueVisitor<'kvs> {
    fn visit_pair(&mut self, key: Key<'kvs>, value: Value<'kvs>) -> Result<(), Error> {
        self.0.insert(key, value);
        Ok(())
    }
}

/// Panic hook that logs the panic using [`log::error!`].
#[cfg(feature = "log-panic")]
fn log_panic(info: &std::panic::PanicInfo<'_>) {
    use std::backtrace::Backtrace;
    use std::thread;

    let mut record = log::Record::builder();
    let thread = thread::current();
    let thread_name = thread.name().unwrap_or("unnamed");
    let backtrace = Backtrace::force_capture();

    let key_values = [
        ("backtrace", Value::capture_display(&backtrace)),
        ("thread_name", Value::from(thread_name)),
    ];
    let key_values = key_values.as_slice();

    let _ = record
        .level(log::Level::Error)
        .target("panic")
        .key_values(&key_values);

    if let Some(location) = info.location() {
        let _ = record
            .file(Some(location.file()))
            .line(Some(location.line()));
    };

    log::logger().log(
        &record
            .args(format_args!("thread '{thread_name}' {info}"))
            .build(),
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::task::JoinSet;

    #[tokio::test(flavor = "multi_thread", worker_threads = 3)]
    async fn multiple_threads_works() {
        Builder::new().init();

        let mut set = JoinSet::new();

        for i in 0..1000 {
            set.spawn(async move {
                log::info!("hello {}", i);
                i
            });
        }

        let mut seen = [false; 1000];
        while let Some(res) = set.join_next().await {
            let idx = res.unwrap();
            seen[idx] = true;
        }

        for i in 0..1000 {
            assert!(seen[i]);
        }
    }
}
