use std::{
    fs,
    time::{Duration, SystemTime},
};

use tracing_appender::{non_blocking, rolling};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};
use walkdir::WalkDir;

pub fn init(default_level: &str) {
    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(default_level));

    fs::create_dir_all("logs").expect("cannot create logs directory");

    let file_appender = rolling::daily("logs", "application.log");

    let (non_blocking, _guard) = non_blocking(file_appender);

    tracing_subscriber::registry()
        .with(filter)
        // terminal
        .with(fmt::layer().pretty().with_target(true))
        // file
        .with(fmt::layer().with_ansi(false).with_writer(non_blocking))
        .init();

    std::mem::forget(_guard);

    cleanup_old_logs();
}

fn cleanup_old_logs() {
    let now = SystemTime::now();

    for entry in WalkDir::new("logs")
        .min_depth(1)
        .max_depth(1)
        .into_iter()
        .filter_map(Result::ok)
    {
        let path = entry.path();

        if !path.is_file() {
            continue;
        }

        let metadata = match fs::metadata(path) {
            Ok(m) => m,
            Err(_) => continue,
        };

        let modified = match metadata.modified() {
            Ok(m) => m,
            Err(_) => continue,
        };

        if let Ok(age) = now.duration_since(modified) {
            if age > Duration::from_secs(7 * 24 * 60 * 60) {
                let _ = fs::remove_file(path);
            }
        }
    }
}
