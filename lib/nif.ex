defmodule NifLogger.NIF do
  use Rustler, otp_app: :nif_logger, crate: "nlogger"

  def print(_message), do: :erlang.nif_error(:nif_not_loaded)

  def log(_message), do: :erlang.nif_error(:nif_not_loaded)

  def register_logger(_pid), do: :erlang.nif_error(:nif_not_loaded)
end
