defmodule NifLogger.Logger do
  use GenServer, restart: :temporary

  require Logger

  def start_link(opts) do
    {name, opts} = Keyword.pop(opts, :name, :nif_logger)

    GenServer.start_link(__MODULE__, opts, name: name)
  end

  @impl GenServer
  def init(_opts) do
    # Register this process as a logger in Rust
    :ok = NifLogger.NIF.register_logger(self())

    Logger.debug("#{inspect(self())} registered as logger")

    {:ok, %{}}
  end

  @impl GenServer
  def handle_info({level, message}, state) do
    Logger.log(level, message)
    {:noreply, state}
  end

  def handle_info(log, state) when is_map(log) do
    message =
      if log.kv == %{} do
        log.message
      else
        Map.put(log.kv, :message, log.message)
      end

    Logger.log(log.level, message, file: log.file, line: log.line)
    {:noreply, state}
  end
end
