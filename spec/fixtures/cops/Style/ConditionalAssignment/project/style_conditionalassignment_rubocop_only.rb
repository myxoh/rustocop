      # Removes the foreign key on +accounts.owner_id+.
      #
      #   remove_foreign_key :accounts, column: :owner_id
      #
      # Removes the foreign key on +accounts.owner_id+.
      #
      #   remove_foreign_key :accounts, to_table: :owners
      #
      # Removes the foreign key named +special_fk_name+ on the +accounts+ table.
      #
      #   remove_foreign_key :accounts, name: :special_fk_name
      #
      # Checks if the foreign key exists before trying to remove it. Will silently ignore indexes that
      # don't exist.
      #
      #   remove_foreign_key :accounts, :branches, if_exists: true
      #
      # The +options+ hash accepts the same keys as SchemaStatements#add_foreign_key
      # with an addition of
      # [<tt>:to_table</tt>]
      #   The name of the table that contains the referenced primary key.
      def remove_foreign_key(from_table, to_table = nil, **options)
        return unless use_foreign_keys?
        to_table ||= options[:to_table]
        return if options.delete(:if_exists) == true && !foreign_key_exists?(from_table, to_table, **options.slice(:column, :name))

        fk_name_to_delete = foreign_key_for!(from_table, to_table: to_table, **options).name

        at = build_alter_table_definition from_table
        at.drop_foreign_key fk_name_to_delete

        execute_alter_table(at)
      end

      # Changes an existing foreign key on a table. Currently only the PostgreSQL
      # adapter (version 18.4+) implements this; see
      # PostgreSQL::SchemaStatements#change_foreign_key for details.
      def change_foreign_key(from_table, to_table = nil, **options)
        raise NotImplementedError, "change_foreign_key is not implemented"
      end

      # Checks to see if a foreign key exists on a table for a given foreign key definition.
      #
      #   # Checks to see if a foreign key exists.
      #   foreign_key_exists?(:accounts, :branches)
      #
      #   # Checks to see if a foreign key on a specified column exists.
      #   foreign_key_exists?(:accounts, column: :owner_id)
      #
      #   # Checks to see if a foreign key with a custom name exists.
      #   foreign_key_exists?(:accounts, name: "special_fk_name")
      #
      def foreign_key_exists?(from_table, to_table = nil, **options)
        foreign_key_for(from_table, to_table: to_table, **options).present?
      end

      def foreign_key_column_for(table_name, column_name) # :nodoc:
        name = strip_table_name_prefix_and_suffix(table_name)
        "#{name.singularize}_#{column_name}"
      end

      def foreign_key_options(from_table, to_table, options) # :nodoc:
        options = options.dup

        if options[:primary_key].is_a?(Array)
          options[:column] ||= options[:primary_key].map do |pk_column|
            foreign_key_column_for(to_table, pk_column)
          end
        else
          options[:column] ||= foreign_key_column_for(to_table, "id")
        end

        options[:name]   ||= foreign_key_name(from_table, options)

        if options[:column].is_a?(Array) || options[:primary_key].is_a?(Array)
          if Array(options[:primary_key]).size != Array(options[:column]).size
            raise ArgumentError, <<~MSG.squish
              For composite primary keys, specify :column and :primary_key, where
              :column must reference all the :primary_key columns from #{to_table.inspect}
            MSG
          end
        end

        options
      end

      # Returns an array of check constraints for the given table, or a Hash of them
      # keyed by table name when given an Array of tables.
      # The check constraints are represented as CheckConstraintDefinition objects.
      def check_constraints(table_name)
        raise NotImplementedError
      end

      # Adds a new check constraint to the table. +expression+ is a String
      # representation of verifiable boolean condition.
      #
      #   add_check_constraint :products, "price > 0", name: "price_check"
      #
      # generates:
      #
      #   ALTER TABLE "products" ADD CONSTRAINT price_check CHECK (price > 0)
      #
      # The +options+ hash can include the following keys:
      # [<tt>:name</tt>]
      #   The constraint name. Defaults to <tt>chk_rails_<identifier></tt>.
      # [<tt>:if_not_exists</tt>]
      #   Silently ignore if the constraint already exists, rather than raise an error.
      # [<tt>:validate</tt>]
      #   (PostgreSQL only) Specify whether or not the constraint should be validated. Defaults to +true+.
      def add_check_constraint(table_name, expression, if_not_exists: false, **options)
        return unless supports_check_constraints?

        options = check_constraint_options(table_name, expression, options)
        return if if_not_exists && check_constraint_exists?(table_name, **options)

        at = build_alter_table_definition(table_name)
        at.add_check_constraint(expression, options)

        execute_alter_table(at)
      end

      def check_constraint_options(table_name, expression, options) # :nodoc:
        options = options.dup
        options[:name] ||= check_constraint_name(table_name, expression: expression, **options)
        options
      end

      # Removes the given check constraint from the table. Removing a check constraint
      # that does not exist will raise an error.
