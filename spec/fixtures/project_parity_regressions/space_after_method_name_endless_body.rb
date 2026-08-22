def filter_version(filter_id) = (@filter_versions ||= {})[filter_id] ||= store.filter_version(account_id: account.id, filter_id: filter_id)
