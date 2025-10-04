// (c) 2023-present, IO Rust. All rights reserved.
// See the file LICENSE for licensing terms.

//! # Structured Logger
//!
//! A logging implementation for the [`log`] crate that logs structured values
//! either synchronous or asynchronous, in JSON, CBOR, or any other format,
//! to a file, stderr, stdout, or any other destination.
//! To initialize the logger use the [`Builder`] struct.
//! It is inspired by [std-logger](https://github.com/Thomasdezeeuw/std-logger).
//!
//! This crate provides only a logging implementation. To do actual logging use
//! the [`log`] crate and it's various macros.
//!
//! ## Limiting logging targets
//! You can use [`Builder::with_target_writer`] method to log messages related specific target to a specific writer.
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
//! Simple example:
//! ```rust
//! use serde::Serialize;
//! use structured_logger::{async_json::new_writer, unix_ms, Builder};
//!
//! #[tokio::main]
//! async fn main() {
//!     // Initialize the logger.
//!     Builder::with_level("info")
//!         .with_target_writer("*", new_writer(tokio::io::stdout()))
//!         .init();
//!
//!     // Or use the default:
//!     // structured_logger::init();
//!
//!     let kv = ContextLog {
//!         uid: "user123".to_string(),
//!         action: "upate_book".to_string(),
//!     };
//!
//!     log::info!("hello world");
//!     // This log will be written to stdout:
//!     // {"level":"INFO","message":"hello world","target":"simple","timestamp":1679745592127}
//!
//!     log::info!(target: "api",
//!         method = "GET",
//!         path = "/hello",
//!         status = 200_u16,
//!         start = unix_ms(),
//!         elapsed = 10_u64,
//!         kv:serde = kv;
//!         "",
//!     );
//!     // This log will be written to stdout:
//!     // {"elapsed":10,"kv":{"uid":"user123","action":"upate_book"},"level":"INFO","message":"","method":"GET","path":"/hello","start":1679745592127,"status":200,"target":"api","timestamp":1679745592127}
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

use json::new_writer;
use log::{kv::*, Level, LevelFilter, Metadata, Record, SetLoggerError};
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

/// A struct to initialize the logger for [`log`] crate.
pub struct Builder {
    filter: LevelFilter,
    default_writer: Box<dyn Writer>,
    writers: Vec<(Target, Box<dyn Writer>)>,
    with_msg: bool,
}

impl Default for Builder {
    fn default() -> Self {
        Self::new()
    }
}

impl Builder {
    /// Returns a [`Builder`] with default configuration.
    /// The default configuration is:
    /// - level filter: get from the environment variable by `get_env_level()`.
    /// - default writer: write to stderr in JSON format.
    pub fn new() -> Self {
        Builder {
            filter: get_env_level(),
            default_writer: new_writer(io::stderr()),
            writers: Vec::new(),
            with_msg: false,
        }
    }

    /// Returns a [`Builder`] with a given level filter.
    /// `level` is a string that can be parsed to `log::LevelFilter`.
    /// Such as "OFF", "ERROR", "WARN", "INFO", "DEBUG", "TRACE", ignore ascii case.
    pub fn with_level(level: &str) -> Self {
        Builder {
            filter: level.parse().unwrap_or(LevelFilter::Info),
            default_writer: new_writer(io::stderr()),
            writers: Vec::new(),
            with_msg: false,
        }
    }

    /// Returns a [`Builder`] with a given `writer` as default writer.
    pub fn with_default_writer(self, writer: Box<dyn Writer>) -> Self {
        Builder {
            filter: self.filter,
            default_writer: writer,
            writers: self.writers,
            with_msg: false,
        }
    }

    /// Returns a [`Builder`] with a given `targets` pattern and `writer`.
    /// `targets` is a pattern that be used to test log target, if true, the log will be written to the `writer`.
    /// `writer` is a boxed struct that implements the `Writer` trait.
    /// You can call this method multiple times in order to add multiple writers.
    ///
    /// `targets` pattern examples:
    /// - `"api"`: match the target "api".
    /// - `"api,db"`: match the target "api" or "db".
    /// - `"api*,db"`: match the target "db", "api", "api::v1", "api::v2", etc.
    /// - `"*"`: match all targets.
    pub fn with_target_writer(self, targets: &str, writer: Box<dyn Writer>) -> Self {
        let mut cfg = Builder {
            filter: self.filter,
            default_writer: self.default_writer,
            writers: self.writers,
            with_msg: false,
        };

        cfg.writers.push((Target::from(targets), writer));
        cfg
    }

    /// Use "msg" field for log message instead of "message".
    pub fn with_msg_field(mut self) -> Self {
        self.with_msg = true;
        self
    }

    /// Builds the logger without registering it in the [`log`] crate.
    ///
    /// Unlike [`Builder::init`] and [`Builder::try_init`] this does not register
    /// the logger into the [`log`] system, allowing it to be combined with
    /// other logging crates.
    pub fn build(self) -> impl log::Log {
        Logger {
            filter: self.filter,
            default_writer: self.default_writer,
            writers: self
                .writers
                .into_iter()
                .map(|(t, w)| (InnerTarget::from(t), w))
                .collect(),
            message_field: if self.with_msg {
                "msg".to_string()
            } else {
                "message".to_string()
            },
        }
    }

    /// Initialize the logger for [`log`] crate.
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
            .unwrap_or_else(|err| panic!("failed to initialize the logger: {}", err));
    }

    /// Try to initialize the logger for [`log`] crate.
    ///
    /// Unlike [`Builder::init`] this doesn't panic when the logger fails to initialize.
    /// See the [crate level documentation] for more.
    ///
    /// [`init`]: fn.init.html
    /// [crate level documentation]: index.html
    pub fn try_init(self) -> Result<(), SetLoggerError> {
        let filter = self.filter;
        let logger = Box::new(self.build());

        log::set_boxed_logger(logger)?;
        log::set_max_level(filter);

        #[cfg(feature = "log-panic")]
        std::panic::set_hook(Box::new(log_panic));
        Ok(())
    }
}

/// Initializes the logger for [`log`] crate with default configuration.
pub fn init() {
    Builder::new().init();
}

/// Returns the current unix timestamp in milliseconds.
#[inline]
pub fn unix_ms() -> u64 {
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time before Unix epoch");
    ts.as_millis() as u64
}

/// Returns the log level from the environment variables: `LOG`, `LOG_LEVEL`, `RUST_LOG`, `TRACE` or `DEBUG`.
/// Default is `INFO`.
pub fn get_env_level() -> LevelFilter {
    for var in &["LOG", "LOG_LEVEL", "RUST_LOG"] {
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
    writers: Box<[(InnerTarget, Box<dyn Writer>)]>,
    message_field: String,
}

impl Logger {
    fn get_writer(&self, target: &str) -> &dyn Writer {
        for t in self.writers.iter() {
            if t.0.test(target) {
                return t.1.as_ref();
            }
        }

        self.default_writer.as_ref()
    }

    fn try_log(&self, record: &Record) -> Result<(), io::Error> {
        let kvs = record.key_values();
        let mut visitor = KeyValueVisitor(BTreeMap::new());
        let _ = kvs.visit(&mut visitor);

        visitor
            .0
            .insert(Key::from("target"), Value::from(record.target()));

        let args = record.args();
        visitor.0.insert(
            Key::from(self.message_field.as_str()),
            if let Some(msg) = args.as_str() {
                Value::from(msg)
            } else {
                Value::from_display(args)
            },
        );

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
                log_failure(format!("Logger failed to log: {}", err).as_str());
            }
        }
    }

    fn flush(&self) {}
}

struct Target {
    all: bool,
    prefix: Vec<String>,
    items: Vec<String>,
}

impl Target {
    fn from(targets: &str) -> Self {
        let mut target = Target {
            all: false,
            prefix: Vec::new(),
            items: Vec::new(),
        };
        for t in targets.split(',') {
            let t = t.trim();
            if t == "*" {
                target.all = true;
                break;
            } else if t.ends_with('*') {
                target.prefix.push(t.trim_end_matches('*').to_string());
            } else {
                target.items.push(t.to_string());
            }
        }
        target
    }
}

struct InnerTarget {
    all: bool,
    prefix: Box<[Box<str>]>,
    items: Box<[Box<str>]>,
}

impl InnerTarget {
    fn from(t: Target) -> Self {
        InnerTarget {
            all: t.all,
            prefix: t.prefix.into_iter().map(|s| s.into_boxed_str()).collect(),
            items: t.items.into_iter().map(|s| s.into_boxed_str()).collect(),
        }
    }

    fn test(&self, target: &str) -> bool {
        if self.all {
            return true;
        }
        if self.items.iter().any(|i| i.as_ref() == target) {
            return true;
        }
        if self.prefix.iter().any(|p| target.starts_with(p.as_ref())) {
            return true;
        }
        false
    }
}

struct KeyValueVisitor<'kvs>(BTreeMap<Key<'kvs>, Value<'kvs>>);

impl<'kvs> VisitSource<'kvs> for KeyValueVisitor<'kvs> {
    #[inline]
    fn visit_pair(&mut self, key: Key<'kvs>, value: Value<'kvs>) -> Result<(), Error> {
        self.0.insert(key, value);
        Ok(())
    }
}

/// A fallback logging function that is used in case of logging failure in [`Writer`] implementation.
/// It will write failure information in JSON to `stderr`.
pub fn log_failure(msg: &str) {
    match serde_json::to_string(msg) {
        Ok(msg) => {
            eprintln!(
                "{{\"level\":\"ERROR\",\"message\":{},\"target\":\"structured_logger\",\"timestamp\":{}}}",
                &msg,
                unix_ms()
            );
        }
        Err(err) => {
            // should never happen
            panic!("log_failure serialize error: {}", err)
        }
    }
}

/// Panic hook that logs the panic using [`log::error!`].
#[cfg(feature = "log-panic")]
fn log_panic(info: &std::panic::PanicHookInfo<'_>) {
    use std::backtrace::Backtrace;
    use std::thread;

    let mut record = log::Record::builder();
    let thread = thread::current();
    let thread_name = thread.name().unwrap_or("unnamed");
    let backtrace = Backtrace::force_capture();

    let key_values = [
        ("backtrace", Value::from_display(&backtrace)),
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
    use gag::BufferRedirect;
    use serde_json::{de, value};
    use std::io::Read;

    #[test]
    fn unix_ms_works() {
        let now = unix_ms();
        assert!(now > 1670123456789_u64);
    }

    #[test]
    fn get_env_level_works() {
        assert_eq!(Level::Info, get_env_level());

        env::set_var("LOG", "error");
        assert_eq!(Level::Error, get_env_level());
        env::remove_var("LOG");

        env::set_var("LOG_LEVEL", "Debug");
        assert_eq!(Level::Debug, get_env_level());
        env::remove_var("LOG_LEVEL");

        env::set_var("RUST_LOG", "WARN");
        assert_eq!(Level::Warn, get_env_level());
        env::remove_var("RUST_LOG");

        env::set_var("TRACE", "");
        assert_eq!(Level::Trace, get_env_level());
        env::remove_var("TRACE");

        env::set_var("DEBUG", "");
        assert_eq!(Level::Debug, get_env_level());
        env::remove_var("DEBUG");
    }

    #[test]
    fn target_works() {
        let target = InnerTarget::from(Target::from("*"));
        assert!(target.test(""));
        assert!(target.test("api"));
        assert!(target.test("hello"));

        let target = InnerTarget::from(Target::from("api*, file,db"));
        assert!(!target.test(""));
        assert!(!target.test("apx"));
        assert!(!target.test("err"));
        assert!(!target.test("dbx"));
        assert!(target.test("api"));
        assert!(target.test("apiinfo"));
        assert!(target.test("apierr"));
        assert!(target.test("file"));
        assert!(target.test("db"));

        let target = InnerTarget::from(Target::from("api*, file, *"));
        assert!(target.test(""));
        assert!(target.test("apx"));
        assert!(target.test("err"));
        assert!(target.test("api"));
        assert!(target.test("apiinfo"));
        assert!(target.test("apierr"));
        assert!(target.test("error"));
    }

    #[test]
    fn log_failure_works() {
        let cases: Vec<&str> = vec!["", "\"", "hello", "\"hello >", "hello\n", "hello\r"];
        for case in cases {
            let buf = BufferRedirect::stderr().unwrap();
            log_failure(case);
            let mut msg: String = String::new();
            buf.into_inner().read_to_string(&mut msg).unwrap();
            let msg = msg.as_str();
            // println!("JSON: {}", msg);
            assert_eq!('\n', msg.chars().last().unwrap());

            let res = de::from_str::<BTreeMap<String, value::Value>>(msg);
            assert!(res.is_ok());
            let res = res.unwrap();
            assert_eq!("ERROR", res.get("level").unwrap());
            assert_eq!(case, res.get("message").unwrap());
            assert_eq!("structured_logger", res.get("target").unwrap());
            assert!(unix_ms() - 999 <= res.get("timestamp").unwrap().as_u64().unwrap());
        }
    }
}
