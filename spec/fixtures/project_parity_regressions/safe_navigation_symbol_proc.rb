safe_ids = records.map { |record| record&.id }
strict_ids = records.map { |record| record.id }

[safe_ids, strict_ids]
