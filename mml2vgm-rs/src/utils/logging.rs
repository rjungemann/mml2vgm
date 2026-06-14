//! Logging infrastructure
//!
//! Implementation will be done in Phase 1 (already set up via env_logger).

/// Initialize logging with default settings
pub fn init_logging() {
    env_logger::Builder::new()
        .filter_level(log::LevelFilter::Info)
        .format_timestamp(None)
        .init();
}

/// Initialize logging with custom level
pub fn init_logging_with_level(level: log::LevelFilter) {
    env_logger::Builder::new()
        .filter_level(level)
        .format_timestamp(None)
        .init();
}
