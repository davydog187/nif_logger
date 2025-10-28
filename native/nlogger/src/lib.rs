use once_cell::sync::OnceCell;
use rustler::{Atom, Env, LocalPid, OwnedEnv, Term};
use std::sync::mpsc::{channel, Sender};
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

// Macro to log with NIF logger process
#[macro_export]
macro_rules! nif_log {
    ($env:expr, $level:expr, $($arg:tt)*) => {{
        if let Some(logger_pid) = $env.whereis_pid(atoms::nif_logger()) {
            if let Some(sender) = LOG_SENDER.get() {
                let _ = sender.send(LogMessage {
                    pid: logger_pid,
                    level: level_to_atom($level),
                    message: format!($($arg)*),
                });
            }
        }
    }};
}

#[macro_export]
macro_rules! nif_info {
    ($env:expr, $($arg:tt)*) => {{
        $crate::nif_log!($env, log::Level::Info, $($arg)*);
    }};
}

#[macro_export]
macro_rules! nif_debug {
    ($env:expr, $($arg:tt)*) => {{
        $crate::nif_log!($env, log::Level::Debug, $($arg)*);
    }};
}

#[macro_export]
macro_rules! nif_warn {
    ($env:expr, $($arg:tt)*) => {{
        $crate::nif_log!($env, log::Level::Warn, $($arg)*);
    }};
}

#[macro_export]
macro_rules! nif_error {
    ($env:expr, $($arg:tt)*) => {{
        $crate::nif_log!($env, log::Level::Error, $($arg)*);
    }};
}

#[rustler::nif]
fn print(message: String) {
    println!("println! {}", message);
}

#[rustler::nif]
fn log(env: Env, message: String) -> Atom {
    nif_debug!(env, "NIF {}", message);
    nif_info!(env, "NIF {}", message);
    nif_warn!(env, "NIF {}", message);
    nif_error!(env, "NIF {}", message);
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

    true
}

rustler::init!("Elixir.NifLogger.NIF", load = on_load);
