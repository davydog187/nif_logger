use once_cell::sync::OnceCell;
use rustler::{Atom, Env, LocalPid, OwnedEnv, ResourceArc, Term};
use std::sync::mpsc::{channel, Sender};
use std::sync::Mutex;
use std::thread;

struct LogMessage {
    pid: LocalPid,
    level: Atom,
    message: String,
}

#[derive(Clone)]
struct LoggerHandle {
    pid: LocalPid,
}

#[rustler::resource_impl]
impl rustler::Resource for LoggerHandle {}

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

// Global registry to store registered logger handles
static LOGGER_REGISTRY: OnceCell<Mutex<Vec<ResourceArc<LoggerHandle>>>> = OnceCell::new();

// Implement the Rust log trait to send to registered BEAM loggers
struct BeamLogger;

impl log::Log for BeamLogger {
    fn enabled(&self, _metadata: &log::Metadata) -> bool {
        LOG_SENDER.get().is_some()
    }

    fn log(&self, record: &log::Record) {
        if let Some(sender) = LOG_SENDER.get() {
            if let Some(registry) = LOGGER_REGISTRY.get() {
                if let Ok(handles) = registry.lock() {
                    // Send to all registered logger processes
                    for handle in handles.iter() {
                        let _ = sender.send(LogMessage {
                            pid: handle.pid,
                            level: level_to_atom(record.level()),
                            message: format!("{}", record.args()),
                        });
                    }
                }
            }
        }
    }

    fn flush(&self) {}
}

static LOGGER: BeamLogger = BeamLogger;

#[rustler::nif]
fn register_logger(pid: LocalPid) -> Atom {
    // Create handle and add to registry
    let handle = ResourceArc::new(LoggerHandle { pid });
    let registry = LOGGER_REGISTRY.get_or_init(|| Mutex::new(Vec::new()));
    
    if let Ok(mut handles) = registry.lock() {
        handles.push(handle);
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

    // Spawn the dispatcher thread that sends messages to PIDs
    thread::spawn(move || {
        for msg in rx {
            let mut env = OwnedEnv::new();
            let _ = env.send_and_clear(&msg.pid, |_env| (msg.level, msg.message));
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
