defmodule NifLoggerTest do
  use ExUnit.Case

  test "logger registers itself" do
    # Start the logger process under test supervision
    pid = start_supervised!({NifLogger.Logger, name: :test_logger})

    # Verify it started and registered
    assert Process.alive?(pid)
    assert Process.whereis(:test_logger) == pid
  end

  test "register_logger sends message to process" do
    assert :ok = NifLogger.NIF.register_logger(self())

    # Should receive the registration message with format {level_atom, message_string}
    assert_receive {:info, "Logger registered"}, 500
  end

  test "Rust std log macros send to registered logger" do
    # Register self() as a logger
    :ok = NifLogger.NIF.register_logger(self())

    # Clear the registration message
    assert_receive {:info, "Logger registered"}, 500

    # Call the NIF which uses Rust's log::info!, log::warn!, etc.
    :ok = NifLogger.NIF.log("test")

    # Should receive all 4 log levels
    assert_receive {:debug, "Debug: test"}, 500
    assert_receive {:info, "Info: test"}, 500
    assert_receive {:warning, "Warning: test"}, 500
    assert_receive {:error, "Error: test"}, 500
  end

  test "multiple loggers receive messages" do
    # Register self() as first logger
    :ok = NifLogger.NIF.register_logger(self())

    # Clear first registration message
    assert_receive {:info, "Logger registered"}, 500

    # Spawn a second receiver that forwards ALL messages
    test_pid = self()
    receiver = spawn(fn ->
      Stream.repeatedly(fn ->
        receive do
          msg -> send(test_pid, {:received, msg})
        end
      end)
      |> Enum.take(5)  # Registration + 4 log levels
    end)

    :ok = NifLogger.NIF.register_logger(receiver)

    # Second logger should receive registration message
    assert_receive {:received, {:info, "Logger registered"}}, 500

    # Trigger a log - sends 4 messages (debug, info, warning, error)
    :ok = NifLogger.NIF.log("broadcast")

    # First logger (self) gets all 4 directly
    assert_receive {:debug, "Debug: broadcast"}, 500
    assert_receive {:info, "Info: broadcast"}, 500
    assert_receive {:warning, "Warning: broadcast"}, 500
    assert_receive {:error, "Error: broadcast"}, 500

    # Second logger forwards all 4 to us
    assert_receive {:received, {:debug, "Debug: broadcast"}}, 500
    assert_receive {:received, {:info, "Info: broadcast"}}, 500
    assert_receive {:received, {:warning, "Warning: broadcast"}}, 500
    assert_receive {:received, {:error, "Error: broadcast"}}, 500
  end

  test "concurrent logging from many processes" do
    # Register self() as the logger
    :ok = NifLogger.NIF.register_logger(self())
    assert_receive {:info, "Logger registered"}, 500

    # Spawn 100 processes that each log once
    num_processes = 100
    
    tasks = 
      for i <- 1..num_processes do
        Task.async(fn ->
          :ok = NifLogger.NIF.log("from_process_#{i}")
        end)
      end

    # Wait for all tasks to complete
    Enum.each(tasks, &Task.await/1)

    # Collect all messages (each process sends 4 log levels)
    messages = 
      for _ <- 1..(num_processes * 4) do
        receive do
          msg -> msg
        after
          2000 -> nil
        end
      end
      |> Enum.reject(&is_nil/1)

    # Verify we got all expected messages
    assert length(messages) == num_processes * 4
    
    # Verify each process's logs are present
    for i <- 1..num_processes do
      assert Enum.any?(messages, fn 
        {_level, msg} -> String.contains?(msg, "from_process_#{i}")
      end), "Missing logs from process #{i}"
    end
  end
end
