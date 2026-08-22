  scope :order_on_name, lambda { |direction|
    order(
      Arel::Nodes::SqlLiteral.new(
        sanitize_sql_for_order(
          "CASE
           WHEN \"contacts\".\"name\" ~~* '^+\d*' THEN 'z'
           WHEN \"contacts\".\"name\"  ~~*  '^\b*' THEN 'z'
           ELSE LOWER(\"contacts\".\"name\")
           END #{direction}"
        )
      )
    )
  }

  # Find contacts that:
  # 1. Have no identification (email, phone_number, and identifier are NULL or empty string)
  # 2. Have no conversations
