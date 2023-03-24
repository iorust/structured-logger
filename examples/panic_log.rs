#[cfg(feature = "log-panic")]
fn main() {
    use log::info;
    use structured_logger::Logger;

    // Initialize the logger.
    Logger::new().init();

    info!("going to panic in a moment");
    // {"level":"INFO","message":"going to panic in a moment","target":"panic_log","timestamp":1679655719809}

    // This panic will be logging properly to standard error.
    panic!("oops");
    // {"backtrace":"   0: std::backtrace_rs::backtrace::libunwind::trace\n             at /rustc/8460ca823e8367a30dda430efda790588b8c84d3/library/std/src/../../backtrace/src/backtrace/libunwind.rs:93:5\n   1: std::backtrace_rs::backtrace::trace_unsynchronized\n             at /rustc/8460ca823e8367a30dda430efda790588b8c84d3/library/std/src/../../backtrace/src/backtrace/mod.rs:66:5\n   2: std::backtrace::Backtrace::create\n             at /rustc/8460ca823e8367a30dda430efda790588b8c84d3/library/std/src/backtrace.rs:332:13\n ... 26: _main\n  27: <unknown>\n","file":"examples/panic_log.rs","level":"ERROR","line":13,"message":"thread 'main' panicked at 'oops', examples/panic_log.rs:13:5","target":"panic2","thread_name":"main","timestamp":1679655719809}
}
