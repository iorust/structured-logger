#[cfg(feature = "log-panic")]
fn main() {
    use log::info;
    use structured_logger::Logger;

    // Initialize the logger.
    Logger::new().init();

    info!("going to panic in a moment");
    // {"message":"going to panic in a moment","target":"panic_log","level":"INFO"}

    // This panic will be logging properly to standard error.
    panic!("oops");
    // {"thread_name":"main","message":"thread 'main' panicked at 'oops', examples/panic.rs:16:5","level":"ERROR","file":"examples/panic.rs","line":16,"backtrace":"   0: std::backtrace_rs::backtrace::libunwind::trace\n             at /rustc/8460ca823e8367a30dda430efda790588b8c84d3/library/std/src/../../backtrace/src/backtrace/libunwind.rs:93:5\n   1: std::backtrace_rs::backtrace::trace_unsynchronized\n             at /rustc/8460ca823e8367a30dda430efda790588b8c84d3/library/std/src/../../backtrace/src/backtrace/mod.rs:66:5\n ...","target":"panic"}
}
