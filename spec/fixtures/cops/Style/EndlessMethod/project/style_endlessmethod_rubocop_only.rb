    enum :status, { open: 0, read_only: 1, closed: 2, archived: 3 }, scopes: false

    validates :name,
              length: {
                maximum: Proc.new { SiteSetting.max_topic_title_length },
              },
              presence: true,
              allow_nil: true
    validates :description, length: { maximum: 500 }
    validates :chatable_type, length: { maximum: 100 }
    validates :type, length: { maximum: 100 }
    validates :slug, length: { maximum: 100 }
    validates :emoji, length: { maximum: 100 }
    validate :ensure_slug_ok, if: :slug_changed?
    before_validation :generate_auto_slug

    scope :with_categories,
          -> do
            joins(
              "LEFT JOIN categories ON categories.id = chat_channels.chatable_id AND chat_channels.chatable_type = 'Category'",
            )
          end
    scope :public_channels,
          -> do
            with_categories
              .where(chatable_type: public_channel_chatable_types)
              .where.not(categories: { id: nil })
          end

    delegate :empty?, to: :chat_messages, prefix: true

    class << self
      def sti_class_mapping =
        {
          "CategoryChannel" => Chat::CategoryChannel,
          "DirectMessageChannel" => Chat::DirectMessageChannel,
        }

      def polymorphic_class_mapping = { "DirectMessage" => Chat::DirectMessage }

      def editable_statuses
        statuses.filter { |k, _| !%w[read_only archived].include?(k) }
      end

      def public_channel_chatable_types
        %w[Category]
      end

      def direct_channel_chatable_types
        %w[DirectMessage]
      end

      def chatable_types
        public_channel_chatable_types + direct_channel_chatable_types
      end

      def find_by_id_or_slug(id)
        with_categories.find_by(
          "chat_channels.id = :id OR categories.slug = :slug OR chat_channels.slug = :slug",
          id: Integer(id, exception: false),
          slug: id.to_s.downcase,
        )
      end
    end
