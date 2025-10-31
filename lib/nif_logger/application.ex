defmodule NifLogger.Application do
  # See https://hexdocs.pm/elixir/Application.html
  # for more information on OTP Applications
  @moduledoc false

  use Application

  @impl true
  def start(_type, _args) do
    children = [
      NifLogger.Logger,
      Supervisor.child_spec({NifLogger.Loop, type: :elixir}, id: :elixir),
      Supervisor.child_spec({NifLogger.Loop, type: :println}, id: :println1),
      Supervisor.child_spec({NifLogger.Loop, type: :println}, id: :println2),
      Supervisor.child_spec({NifLogger.Loop, type: :log}, id: :log1),
      Supervisor.child_spec({NifLogger.Loop, type: :log}, id: :log2)
    ]

    # See https://hexdocs.pm/elixir/Supervisor.html
    # for other strategies and supported options
    opts = [strategy: :one_for_one, name: NifLogger.Supervisor]
    Supervisor.start_link(children, opts)
  end
end
