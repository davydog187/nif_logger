defmodule NifLogger.Loop do
  use GenServer, restart: :temporary

  require Logger

  def start_link(opts \\ []) do
    GenServer.start_link(__MODULE__, opts)
  end

  def init(opts) do
    interval = Keyword.get(opts, :interval, 10)
    until = Keyword.get(opts, :until, 5)
    type = Keyword.get(opts, :type, :elixir)

    {:ok, %{interval: interval, type: type, counter: 0, until: until}, {:continue, :loop}}
  end

  def handle_continue(:loop, state) do
    Logger.info("starting loop #{inspect(self())} #{inspect(state)}")
    log(state)
  end

  def handle_info(:log, state) do
    log(state)
  end

  def handle_info({:log, message}, state) do
    Logger.info("HMMM: " <> message)
    {:noreply, state}
  end

  def handle_info(message, state) do
    Logger.warning("unhandled message #{inspect(message)}")

    {:noreply, state}
  end

  def log(state) do
    message = "#{inspect(self())} #{state.counter}"

    case state.type do
      :elixir ->
        Logger.info("Logger #{message}")

      :println ->
        NifLogger.NIF.print(message)

      :log ->
        NifLogger.NIF.log(message)
    end

    Process.send_after(self(), :log, Enum.random(0..state.interval))

    state = update_in(state.counter, &(&1 + 1))

    if state.counter >= state.until do
      {:stop, :finished, state}
    else
      {:noreply, state}
    end
  end
end
