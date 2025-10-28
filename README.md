# NifLogger

## Background

Handling logging from a Rust NIF is a complicated topic. There are many issues with using `println!` or even different logging backends like `env_logger`

https://github.com/rusterlium/rustler/issues/335
https://github.com/rusterlium/rustler/issues/72

Lukas Backström from the OTP team shared that ERTS has its own logger for [forwarding log messages from NIFs to Erlang](https://elixirforum.com/t/logging-from-a-nif/60440). Which can be seen in the [logger_proxy.erl](https://github.com/erlang/otp/blob/master/lib/kernel/src/logger_proxy.erl) module.

## Implementation

This repo experiments with this approach in Rustler. The idea is that we use a `mpsc` to forward log messages from Rust to an Elixir process that can call Logger.

The logger process in [NifLogger.Logger](lib/nif_logger/logger.ex) receives the log messages from the NIF, which is dispatched through a separate thread.

In an ideal world, we would be able to simply implement a Rust [Log Backend](https://docs.rs/log/latest/log/trait.Log.html) so that we can just use the standard `log::info!` and friends macros. The challenge is in passing around the `LocalPid` reference that can be used by the backend. 

Currently, I pass a `rustler::Env` to the macro to extract the pid by name (which may have performance issues), then it calls `env.send_and_clear` which I'm not sure has any backpressure at all to the process. 


Another thing to figure out is how we can do this from background threads that don't have a reference to an `Env`. In this case, I think we would need to extract the `LocalPid` from the containing NIF to the background thread, and use that for dispatch.

- [X] log directly from NIF 
- [ ] log in background thread
- [ ] benchmark
