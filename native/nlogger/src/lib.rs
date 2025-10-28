use log::{
    kv::{Error as KvError, Key, Value, VisitSource},
    LevelFilter, Metadata, Record,
};

use once_cell::sync::OnceCell;
use rustler::{Env, LocalPid, OwnedEnv, Term};
use std::sync::mpsc::{channel, Sender};
use std::thread;

struct LogMessage {
    pid: LocalPid,
    level: log::Level,
    message: String,
}

static LOG_SENDER: OnceCell<Sender<LogMessage>> = OnceCell::new();

struct NifLogger;

mod atoms {
    rustler::atoms! { log, cool }
}

// Macro to log with NIF PID context
#[macro_export]
macro_rules! nif_log {
    ($env:expr, $level:expr, $($arg:tt)*) => {{
        let pid_raw: u64 = unsafe {
            std::mem::transmute($env.pid())
        };
        log::log!(
            target: module_path!(),
            $level,
            nif_pid = pid_raw;
            $($arg)*
        );
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

impl log::Log for NifLogger {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        // Extract PID from key-value pairs
        struct PidExtractor {
            pid: Option<LocalPid>,
        }

        impl<'kvs> VisitSource<'kvs> for PidExtractor {
            fn visit_pair(&mut self, key: Key<'kvs>, value: Value<'kvs>) -> Result<(), KvError> {
                if key.as_str() == "nif_pid" {
                    if let Some(pid_raw) = value.to_u64() {
                        // Safety: We control the serialization via our macros
                        self.pid = Some(unsafe { std::mem::transmute::<u64, LocalPid>(pid_raw) });
                    }
                }
                Ok(())
            }
        }

        let mut extractor = PidExtractor { pid: None };
        let _ = record.key_values().visit(&mut extractor);

        if let Some(pid) = extractor.pid {
            let msg = LogMessage {
                pid,
                level: record.level(),
                message: format!("{}", record.args()),
            };

            // Send to the dispatcher thread
            if let Some(sender) = LOG_SENDER.get() {
                let _ = sender.send(msg);
            }
        }
    }

    fn flush(&self) {}
}

#[rustler::nif]
fn add(a: i64, b: i64) -> i64 {
    a + b
}

#[rustler::nif]
fn print(pid: String, counter: i64) {
    println!("println! {} {}", pid, counter);
}

#[rustler::nif]
fn log(env: Env, counter: i64) {
    nif_info!(env, "NIF {}", counter);
}

fn on_load(env: Env, _load_data: Term) -> bool {
    let peee = env.pid();

    // Create the channel for log messages
    let (tx, rx) = channel::<LogMessage>();

    // Spawn the dispatcher thread that sends messages to PIDs
    thread::spawn(move || {
        for msg in rx {
            let mut env = OwnedEnv::new();
            let _ = env.send_and_clear(&msg.pid, |_env| {
                (atoms::log(), format!("[{}] {}", msg.level, msg.message))
            });
        }
    });

    // Store the sender globally
    LOG_SENDER.set(tx).ok();

    // Initialize the logger
    log::set_boxed_logger(Box::new(NifLogger))
        .map(|()| log::set_max_level(LevelFilter::Info))
        .is_ok()
}

rustler::init!("Elixir.NifLogger.NIF", load = on_load);
