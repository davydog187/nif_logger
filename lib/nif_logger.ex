defmodule NifLogger do
  @moduledoc """
  Documentation for `NifLogger`.
  """

  require Logger

  def test do
    Logger.info("Well 2 + 2 = #{NifLogger.NIF.add(2, 2)}")
  end
end
