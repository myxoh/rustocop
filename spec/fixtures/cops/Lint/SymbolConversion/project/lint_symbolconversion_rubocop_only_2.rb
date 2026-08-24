
      def default_fixture_model_name(fixture_set_name, config = ActiveRecord::Base) # :nodoc:
        config.pluralize_table_names ?
          fixture_set_name.singularize.camelize :
          fixture_set_name.camelize
      end

      def default_fixture_table_name(fixture_set_name, config = ActiveRecord::Base) # :nodoc:
        "#{ config.table_name_prefix }"\
        "#{ fixture_set_name.tr('/', '_') }"\
        "#{ config.table_name_suffix }".to_sym
      end

      def reset_cache
        @@all_cached_fixtures.clear
      end
