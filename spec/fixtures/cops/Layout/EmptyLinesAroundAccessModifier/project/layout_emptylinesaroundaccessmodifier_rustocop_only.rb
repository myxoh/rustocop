      expose :project_id, documentation: { type: 'Integer', format: 'int64', example: 1 }
      expose :group_id,
        documentation: { type: 'Integer', format: 'int64', example: 1 },
        if: ->(object) { object.is_a?(Gitlab::Search::FoundWikiPage) }
      private

      def group_id
        object.group&.id
      end
