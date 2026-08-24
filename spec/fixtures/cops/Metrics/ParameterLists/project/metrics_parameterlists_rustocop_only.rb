    include ActiveSupport::Callbacks

    define_callbacks :handle, terminator: lambda { |target, result_lambda|
      result_lambda.call
      target.errored?
    }
    define_callbacks :handle_record, terminator: lambda { |target, result_lambda|
      result_lambda.call
      target.errored?
    }
    define_callbacks :handle_standalone, terminator: lambda { |target, result_lambda|
      result_lambda.call
      target.errored?
    }

    def initialize( # rubocop:disable Metrics/ParameterLists
      fields:,
      current_user:,
      arguments:,
      resource:,
      action:,
      query:,
      records: nil
    )
      @records = records
      @fields = fields
      @current_user = current_user
      @arguments = arguments
      @resource = resource
      @query = query

      @action = action
    end
