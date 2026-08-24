  # connections. The connections are created lazily so slightly over-provisioning a connection pool is not an issue.
  # This also increases the internal redis pool from 10 to concurrency+5.
  config.redis = queues_config_hash.merge({ size: config.concurrency + 5 })

  config.server_middleware(&Gitlab::SidekiqMiddleware::Server.configurator(
    metrics: Settings.monitoring.sidekiq_exporter,
    arguments_logger: SidekiqLogArguments.enabled? && !enable_json_logs,
    skip_jobs: Gitlab::Utils.to_boolean(ENV['SIDEKIQ_SKIP_JOBS'], default: true)
  ))
