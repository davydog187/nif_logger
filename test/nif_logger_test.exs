defmodule NifLoggerTest do
  use ExUnit.Case

  test "logger registers itself" do
    # Start the logger process under test supervision
    pid = start_supervised!({NifLogger.Logger, name: :test_logger})

    # Verify it started and registered
    assert Process.alive?(pid)
    assert Process.whereis(:test_logger) == pid
  end

  test "register_logger works" do
    # Note: The app starts NifLogger.Logger which is the first registered logger
    # So messages go to it, not to test processes
    assert :ok = NifLogger.NIF.register_logger(self())
  end

  test "Rust std log macros send to first registered logger" do
    # The app's NifLogger.Logger is already registered as the first logger
    # So logs will go to it (and forwarded to Elixir Logger)
    # This test just verifies the NIF doesn't crash
    :ok = NifLogger.NIF.log("test")
  end

  test "only first registered logger receives messages" do
    # Demonstrates that only the first logger gets messages
    # The app's NifLogger.Logger is first, so it gets all logs
    :ok = NifLogger.NIF.register_logger(self())
    :ok = NifLogger.NIF.log("test")
    
    # We (second logger) should NOT receive anything
    refute_receive _, 200
  end

  test "concurrent logging from many processes" do
    # Spawn 100 processes that each log once
    # All logs should succeed without panics (backpressure via semaphore)
    num_processes = 100
    
    tasks = 
      for i <- 1..num_processes do
        Task.async(fn ->
          :ok = NifLogger.NIF.log("from_process_#{i}")
        end)
      end

    # Wait for all tasks to complete - should all succeed
    results = Enum.map(tasks, &Task.await/1)
    assert Enum.all?(results, &(&1 == :ok))
  end
end
