use once_cell::sync::OnceCell;
use rustler::{Atom, Env, LocalPid, OwnedEnv, Term};
use std::sync::mpsc::{channel, Sender};
use std::sync::Mutex;
use std::thread;

struct LogMessage {
    pid: LocalPid,
    level: Atom,
    message: String,
}

static LOG_SENDER: OnceCell<Sender<LogMessage>> = OnceCell::new();

mod atoms {
    rustler::atoms! { ok, nif_logger }
}

mod log_levels {
    rustler::atoms! { debug, info, warning, error }
}

fn level_to_atom(level: log::Level) -> Atom {
    match level {
        log::Level::Debug => log_levels::debug(),
        log::Level::Trace => log_levels::debug(),
        log::Level::Info => log_levels::info(),
        log::Level::Warn => log_levels::warning(),
        log::Level::Error => log_levels::error(),
    }
}

// Global registry to store registered logger PIDs
static LOGGER_REGISTRY: OnceCell<Mutex<Vec<LocalPid>>> = OnceCell::new();

// Implement the Rust log trait to send to registered BEAM loggers
struct BeamLogger;

impl log::Log for BeamLogger {
    fn enabled(&self, _metadata: &log::Metadata) -> bool {
        LOGGER_REGISTRY.get().is_some()
    }

    fn log(&self, record: &log::Record) {
        let Some(sender) = LOG_SENDER.get() else {
            return;
        };

        let Some(registry) = LOGGER_REGISTRY.get() else {
            return;
        };

        let Ok(pids) = registry.lock() else {
            return;
        };

        let Some(&pid) = pids.first() else {
            return;
        };

        // Pick only the FIRST registered logger
        let level = level_to_atom(record.level());
        let message = format!("{}", record.args());

        // Send to channel - backpressure is handled by the channel
        let _ = sender.send(LogMessage {
            pid,
            level,
            message,
        });
    }

    fn flush(&self) {}
}

static LOGGER: BeamLogger = BeamLogger;

#[rustler::nif]
fn register_logger(pid: LocalPid) -> Atom {
    let registry = LOGGER_REGISTRY.get_or_init(|| Mutex::new(Vec::new()));

    if let Ok(mut pids) = registry.lock() {
        pids.push(pid);
    }

    // Test using standard Rust log macros!
    log::info!("Logger registered");

    atoms::ok()
}

#[rustler::nif]
fn print(message: String) {
    println!("println! {}", message);
}

#[rustler::nif]
fn log(_message: String) -> Atom {
    // Use standard Rust log macros
    log::debug!("Debug: {}", _message);
    log::info!("Info: {}", _message);
    log::warn!("Warning: {}", _message);
    log::error!("Error: {}", _message);
    atoms::ok()
}

fn on_load(_env: Env, _load_data: Term) -> bool {
    // Create the channel for log messages
    let (tx, rx) = channel::<LogMessage>();

    // Spawn the dispatcher thread (unmanaged) that sends messages to PIDs
    thread::spawn(move || {
        for msg in rx {
            let mut env = OwnedEnv::new();

            // Try to send, if it fails the process is dead
            if env
                .send_and_clear(&msg.pid, |_env| (msg.level, msg.message))
                .is_err()
            {
                // Remove dead PID from registry
                if let Some(registry) = LOGGER_REGISTRY.get() {
                    if let Ok(mut pids) = registry.lock() {
                        pids.retain(|pid| *pid != msg.pid);
                    }
                }
            }
        }
    });

    // Store the sender globally
    LOG_SENDER.set(tx).ok();

    // Initialize the Rust log system with our BeamLogger
    log::set_logger(&LOGGER)
        .map(|()| log::set_max_level(log::LevelFilter::Trace))
        .is_ok()
}

rustler::init!("Elixir.NifLogger.NIF", load = on_load);
