import Config

if System.get_env("JSON_LOGGING", "false") == "true" do
  config :logger, :default_handler,
    metadata: [:file, :line],
    formatter: {LoggerJSON.Formatters.Basic, metadata: [:request_id, :file, :line]}
end
