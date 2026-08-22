    # See https://gitlab.com/gitlab-org/gitlab/issues/197346 for more details
    initializer :move_initializers, before: :load_config_initializers, after: :setup_main_autoloader do
    end
