use log::{kv::Error, kv::Visitor, Level, LevelFilter, Metadata, Record, SetLoggerError};
use std::collections::{BTreeMap, HashMap};
use std::{env, io};

/// re-export log::kv::Key and log::kv::Value.
pub use log::kv::{Key, Value};

/// Log is a type alias for HashMap<Key<'a>, Value<'a>>.
pub type Log<'a> = HashMap<Key<'a>, Value<'a>>;

/// Writer is a trait that defines how to write a log.
pub trait Writer {
    /// write_log writes a log to the underlying io::Write instance.
    fn write_log(&self, value: &Log) -> Result<(), io::Error>;
}

mod json;
pub use json::{new_json_writer, JSONWriter};

/// Logger is a struct that holds the configuration for the logger.
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
    /// new creates a new Logger with default configuration.
    pub fn new() -> Self {
        Logger {
            filter: get_max_level(),
            default_writer: new_json_writer(io::stderr()),
            writers: BTreeMap::new(),
        }
    }

    /// with_level creates a new Logger with a given level filter.
    pub fn with_level(level: LevelFilter) -> Self {
        Logger {
            filter: level,
            default_writer: new_json_writer(io::stderr()),
            writers: BTreeMap::new(),
        }
    }

    /// with_target_writer creates a new Logger with a given target and writer.
    pub fn with_target_writer(self, target: &'static str, writer: Box<dyn Writer>) -> Self {
        let mut cfg = Logger {
            filter: self.filter,
            default_writer: self.default_writer,
            writers: self.writers,
        };
        cfg.writers.insert(target, writer);
        cfg
    }

    /// initialize the logger.
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

/// Get the maximum log level based on the environment.
pub fn get_max_level() -> LevelFilter {
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
            let mut visitor = KeyValueVisitor(HashMap::new());
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
