    def show_suggest_popover?
      !user_dismissed?(SUGGEST_POPOVER_DISMISSED)
    end

    def show_unfinished_tag_cleanup_callout?
      !user_dismissed?(UNFINISHED_TAG_CLEANUP_CALLOUT)
    end

    def show_registration_enabled_user_callout?
      !Gitlab.com? &&
        current_user&.can_admin_all_resources? &&
        signup_enabled? &&
        REGISTRATION_ENABLED_CALLOUT_ALLOWED_CONTROLLER_PATHS.any? { |path| controller.controller_path.match?(path) }
    end

    def show_openssl_callout?
      return false unless Gitlab.version_info >= Gitlab::VersionInfo.new(17, 1) &&
        Gitlab.version_info < Gitlab::VersionInfo.new(17, 7)

      current_user&.can_admin_all_resources? &&
        !user_dismissed?(OPENSSL_CALLOUT) &&
        controller.controller_path.match?(%r{^admin(/\S*)?$})
    end

    def web_hook_disabled_dismissed?(object)
      return false unless object.is_a?(::WebHooks::HasWebHooks)

      user_dismissed?(WEB_HOOK_DISABLED, object.last_webhook_failure, object: object)
    end

    def show_branch_rules_tip?
      !user_dismissed?(BRANCH_RULES_TIP_CALLOUT)
    end
