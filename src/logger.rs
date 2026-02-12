use env_logger::{Builder, Env};
use log::{Level, LevelFilter, Log, Metadata, Record};
use rusqlite::{params, Connection};
use std::io::Write;
use std::path::PathBuf;


/// Custom logger that writes to both stdout and optionally to SQLite database
pub struct DualLogger {
    env_logger: env_logger::Logger,
    db_path: Option<PathBuf>,
    db_level: LevelFilter,
}

impl DualLogger {
    /// Create a new dual logger with optional database support
    pub fn new(log_level: Option<&str>, db_path: Option<PathBuf>, db_level: Option<LevelFilter>) -> Self {
        let default_level = log_level.unwrap_or("error");

        let env = Env::default()
            .filter_or("LOG_LEVEL", default_level)
            .write_style_or("LOG_STYLE", "always");

        let env_logger = Builder::from_env(env)
            .format(|buf, record| {
                let level = record.level();
                let info_style = buf.default_level_style(record.level());
                writeln!(buf, "{info_style}{level}: {info_style:#}{}", record.args())
            })
            .build();

        Self {
            env_logger,
            db_path,
            db_level: db_level.unwrap_or(LevelFilter::Info),
        }
    }

    /// Log a message to the database (synchronously)
    fn log_to_db(&self, record: &Record) {
        if let Some(db_path) = &self.db_path {
            // Open a new connection for each log entry (simple but works)
            // In production, you might want to use a connection pool
            if let Ok(conn) = Connection::open(db_path) {
                let timestamp = chrono::Utc::now().timestamp();
                let level = record.level().to_string().to_lowercase();
                let target = record.target();
                let message = format!("{}", record.args());

                // Best effort - don't panic if logging fails
                let _ = conn.execute(
                    "INSERT INTO logs (timestamp, level, target, message, session_id) VALUES (?1, ?2, ?3, ?4, NULL)",
                    params![timestamp, level, target, message],
                );
            }
        }
    }
}

impl Log for DualLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        self.env_logger.enabled(metadata)
    }

    fn log(&self, record: &Record) {
        // Log to stdout via env_logger
        self.env_logger.log(record);

        // Log to database if enabled and level is appropriate
        if self.db_path.is_some() && record.level() <= self.db_level {
            self.log_to_db(record);
        }
    }

    fn flush(&self) {
        self.env_logger.flush();
    }
}

/// Initialize the logger without database support
pub fn init(log_level: Option<&str>) {
    init_with_db(log_level, None, None);
}

/// Initialize the logger with optional database support
pub fn init_with_db(log_level: Option<&str>, db_path: Option<PathBuf>, db_level: Option<LevelFilter>) {
    let logger = DualLogger::new(log_level, db_path, db_level);
    let max_level = logger.env_logger.filter();

    log::set_boxed_logger(Box::new(logger))
        .map(|()| log::set_max_level(max_level))
        .expect("Failed to initialize logger");
}

/// Query logs from database with optional filters
pub fn query_logs(
    db_path: &str,
    limit: Option<usize>,
    level_filter: Option<&str>,
    session_id: Option<&str>,
) -> Result<Vec<LogEntry>, rusqlite::Error> {
    let conn = Connection::open(db_path)?;

    let mut query = String::from("SELECT id, timestamp, level, target, message, session_id FROM logs WHERE 1=1");
    let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

    if let Some(level) = level_filter {
        query.push_str(" AND level = ?");
        params.push(Box::new(level.to_string()));
    }

    if let Some(sid) = session_id {
        query.push_str(" AND session_id = ?");
        params.push(Box::new(sid.to_string()));
    }

    query.push_str(" ORDER BY timestamp DESC");

    if let Some(lim) = limit {
        query.push_str(" LIMIT ?");
        params.push(Box::new(lim as i64));
    }

    let mut stmt = conn.prepare(&query)?;
    let param_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();

    let logs = stmt
        .query_map(param_refs.as_slice(), |row| {
            Ok(LogEntry {
                id: row.get(0)?,
                timestamp: row.get(1)?,
                level: row.get(2)?,
                target: row.get(3)?,
                message: row.get(4)?,
                session_id: row.get(5)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(logs)
}

/// Clean up old logs from database
pub fn cleanup_old_logs(db_path: &str, max_age_seconds: i64) -> Result<usize, rusqlite::Error> {
    let conn = Connection::open(db_path)?;
    let cutoff_time = chrono::Utc::now().timestamp() - max_age_seconds;

    let deleted = conn.execute(
        "DELETE FROM logs WHERE timestamp < ?1",
        params![cutoff_time],
    )?;

    Ok(deleted)
}

/// A log entry from the database
#[derive(Debug, Clone)]
pub struct LogEntry {
    pub id: i64,
    pub timestamp: i64,
    pub level: String,
    pub target: String,
    pub message: String,
    pub session_id: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use log::{error, info, warn};
    use std::path::Path;

    #[test]
    fn test_logger_initialization() {
        // Test without database
        init(Some("info"));
        info!("Test log message");
    }

    #[test]
    fn test_logger_with_db() {
        // Test with temporary database
        let db_path = PathBuf::from(":memory:");

        // Initialize database schema first
        if let Ok(conn) = Connection::open(&db_path) {
            let _ = conn.execute_batch(
                "CREATE TABLE logs (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    timestamp INTEGER NOT NULL,
                    level TEXT NOT NULL,
                    target TEXT NOT NULL,
                    message TEXT NOT NULL,
                    session_id TEXT
                );",
            );
        }

        init_with_db(Some("info"), Some(db_path), Some(LevelFilter::Info));

        info!("Test info message");
        warn!("Test warning message");
        error!("Test error message");
    }

    #[test]
    fn test_query_logs() {
        let db_path = ":memory:";

        // Create test database with logs
        let conn = Connection::open(db_path).unwrap();
        conn.execute_batch(
            "CREATE TABLE logs (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp INTEGER NOT NULL,
                level TEXT NOT NULL,
                target TEXT NOT NULL,
                message TEXT NOT NULL,
                session_id TEXT
            );",
        ).unwrap();

        let now = chrono::Utc::now().timestamp();
        conn.execute(
            "INSERT INTO logs (timestamp, level, target, message, session_id) VALUES (?1, ?2, ?3, ?4, NULL)",
            params![now, "info", "test", "Test message"],
        ).unwrap();

        // Query logs
        let logs = query_logs(db_path, Some(10), None, None).unwrap();
        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0].message, "Test message");
    }
}
