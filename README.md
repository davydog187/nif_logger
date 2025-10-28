# NifLogger

Handling logging from a Rust NIF is a complicated topic. There are many issues with using `println!` or even different logging backends like `env_logger`

https://github.com/rusterlium/rustler/issues/335
https://github.com/rusterlium/rustler/issues/72

Lukas Backström from the OTP team shared that ERTS has its own logger for [forwarding log messages from NIFs to Erlang](https://elixirforum.com/t/logging-from-a-nif/60440). Which can be seen in the [logger_proxy.erl](https://github.com/erlang/otp/blob/master/lib/kernel/src/logger_proxy.erl) module.

This repo experiments with this approach in Rustler. The idea is that we use a `mpsc` to forward log messages from Rust to an Elixir process that can call Logger.

- [X] log directly from NIF 
- [ ] log in background thread
- [ ] benchmark
