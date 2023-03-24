// (c) 2022-present, IO Rust. All rights reserved.
// See the file LICENSE for licensing terms.

//! # Structured Logger
//!
//! A logging implementation for the log crate that logs structured values
//! as JSON (CBOR, or any other) into a file, stderr, stdout, or any other.
//! Inspired by [std-logger](https://github.com/Thomasdezeeuw/std-logger).
//!
//! This crate provides only a logging implementation. To do actual logging use
//! the [`log`] crate and it's various macros.
//!
//!
//! # Setting severity
//!
//! You can use various environment variables to change the severity (log level)
//! of the messages to actually log and which to ignore.
//!
//! `LOG` and `LOG_LEVEL` can be used to set the severity to a specific value,
//! see the [`log`]'s package [`LevelFilter`] type for available values.
//!
//! ```bash
//! ## In your shell of your choice:
//!
//! ## Set the log severity to only print log messages with info severity or
//! ## higher, trace and debug messages won't be printed anymore.
//! $ LOG=info ./my_binary
//!
//! ## Set the log severity to only print log messages with warning severity or
//! ## higher, informational (or lower severity) messages won't be printed
//! ## anymore.
//! $ LOG=warn ./my_binary
//! ```
//!
//! Alternatively setting the `TRACE` variable (e.g. `TRACE=1`) sets the
//! severity to the trace, meaning it will log everything. Setting `DEBUG` will
//! set the severity to debug.
//!
//! ```bash
//! ## In your shell of your choice:
//!
//! ## Enables trace logging.
//! $ TRACE=1 ./my_binary
//!
//! ## Enables debug logging.
//! $ DEBUG=1 ./my_binary
//! ```
//!
//! If none of these environment variables are found it will default to an
//! information severity.
//!
//! # Crate features
//!
//! This crate has three features:
//! * *log-panic*, enabled by default.
//!
//! ## Log-panic feature
//!
//! The *log-panic* feature will log all panics using the `error` severity,
//! rather then using the default panic handler. It will log the panic message
//! as well as the location and a backtrace, see the log output for an
//! [`panic_log`] example.
//!
//! # Examples
//!
//! ```rust
//! use serde::Serialize;
//! use std::io::stdout;
//! use structured_logger::{json::new_json_writer, unix_ms, Logger};
//!
//! fn main() {
//!     // Initialize the logger.
//!     Logger::new()
//!         // set a specific writer (format to JSON, write to stdout) for target "request".
//!         .with_target_writer("request", new_json_writer(stdout()))
//!         .init();
//!
//!     let kv = ContextLog {
//!         uid: "user123".to_string(),
//!         action: "upate_book".to_string(),
//!     };
//!
//!     log::info!("hello world");
//!     // {"level":"INFO","message":"hello world","target":"simple","timestamp":1679655670735}
//!
//!     // mock request data
//!     log::info!(target: "request",
//!         method = "GET",
//!         path = "/hello",
//!         status = 200_u16,
//!         start = unix_ms(),
//!         elapsed = 10_u64,
//!         kv = log::as_serde!(kv);
//!         "",
//!     );
//!     // {"elapsed":10,"kv":{"uid":"user123","action":"upate_book"},"level":"INFO","message":"","method":"GET","path":"/hello","start":1679655670735,"status":200,"target":"request","timestamp":1679655670735}
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

use log::{kv::Error, kv::Visitor, Level, LevelFilter, Metadata, Record, SetLoggerError};
use std::{
    collections::BTreeMap,
    env, io,
    time::{SystemTime, UNIX_EPOCH},
};

pub use log::kv::Key;
pub use log::kv::Value;

/// A type alias for BTreeMap<Key<'a>, Value<'a>>.
/// BTreeMap is used to keep the order of the keys.
pub type Log<'a> = BTreeMap<Key<'a>, Value<'a>>;

/// A trait that defines how to write a log.
pub trait Writer {
    /// Writes a structured log to the underlying io::Write instance.
    fn write_log(&self, value: &Log) -> Result<(), io::Error>;
}

pub mod json;
use json::new_json_writer;

/// A struct that holds the configuration for the logger.
pub struct Logger {
    filter: LevelFilter,
    default_writer: Box<dyn Writer>,
    writers: BTreeMap<&'static str, Box<dyn Writer>>,
}

impl Default for Logger {
    fn default() -> Self {
        Self::new()
    }
}

impl Logger {
    /// Returns a new Logger with default configuration.
    /// The default configuration is:
    /// - level filter: get from the environment variable by `get_env_level()`.
    /// - default writer: write to stderr in JSON format.
    pub fn new() -> Self {
        Logger {
            filter: get_env_level(),
            default_writer: new_json_writer(io::stderr()),
            writers: BTreeMap::new(),
        }
    }

    /// Returns a new Logger with a given level filter.
    /// `level` is a string that can be parsed to `log::LevelFilter`.
    /// Such as "OFF", "ERROR", "WARN", "INFO", "DEBUG", "TRACE", ignore ascii case.
    pub fn with_level(level: &str) -> Self {
        Logger {
            filter: level.parse().unwrap_or(LevelFilter::Info),
            default_writer: new_json_writer(io::stderr()),
            writers: BTreeMap::new(),
        }
    }

    /// Returns a new Logger with a given `target` and `writer`.
    /// `target` is a string that be used as a log target.
    /// `writer` is a struct that implements the `Writer` trait.
    /// You can call this method multiple times to add multiple writers.
    pub fn with_target_writer(self, target: &'static str, writer: Box<dyn Writer>) -> Self {
        let mut cfg = Logger {
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
    /// This will panic if the logger fails to initialize. Use [`Logger::try_init`] if
    /// you want to handle the error yourself.
    pub fn init(self) {
        self.try_init()
            .unwrap_or_else(|err| panic!("failed to initialize the logger: {err}"));
    }

    /// Try to initialize the logger.
    ///
    /// Unlike [`Logger::init`] this doesn't panic when the logger fails to initialize.
    /// See the [crate level documentation] for more.
    ///
    /// [`init`]: fn.init.html
    /// [crate level documentation]: index.html
    pub fn try_init(self) -> Result<(), SetLoggerError> {
        let logger = Box::new(InnerLogger {
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

struct InnerLogger {
    filter: LevelFilter,
    default_writer: Box<dyn Writer>,
    writers: BTreeMap<&'static str, Box<dyn Writer>>,
}

impl InnerLogger {
    fn get_writer(&self, target: &str) -> &dyn Writer {
        if let Some(writer) = self.writers.get(target) {
            writer.as_ref()
        } else {
            self.default_writer.as_ref()
        }
    }
}

unsafe impl Sync for InnerLogger {}
unsafe impl Send for InnerLogger {}

impl log::Log for InnerLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        self.filter >= metadata.level()
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let kvs = record.key_values();
            let mut visitor = KeyValueVisitor(BTreeMap::new());
            kvs.visit(&mut visitor).unwrap();

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
            let writer = self.get_writer(record.target());
            writer.write_log(&visitor.0).unwrap();
        }
    }

    fn flush(&self) {}
}

struct KeyValueVisitor<'kvs>(Log<'kvs>);

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
        .target("panic2")
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

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn it_works() {
//         assert_eq!(4, 4);
//     }
// }
