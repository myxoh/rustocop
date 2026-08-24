          plucked_runner_and_namespace_ids =
            ::Ci::RunnerNamespace
              .for_runner(runner_ids)
              .select(:runner_id, :namespace_id)
              .pluck(:runner_id, :namespace_id) # rubocop: disable CodeReuse/ActiveRecord)

          namespace_ids = plucked_runner_and_namespace_ids.collect(&:last).uniq
          groups = apply_lookahead(::Group.id_in(namespace_ids))
          Preloaders::GroupPolicyPreloader.new(groups, current_user).execute
