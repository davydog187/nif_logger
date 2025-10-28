defmodule NifLogger.Logger do
  use GenServer

  require Logger

  def start_link(opts) do
    {name, opts} = Keyword.pop(opts, :name, :nif_logger)

    GenServer.start_link(__MODULE__, opts, name: name)
  end

  @impl GenServer
  def init(_opts) do
    {:ok, %{}}
  end

  @impl GenServer
  def handle_info({level, message}, state) do
    Logger.log(level, message)
    {:noreply, state}
  end
end
