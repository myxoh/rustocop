    # Until `merge_request_commits_metadata` records are backfilled, SHAs data may be in found in either table
    metadata_join_sql = <<~SQL.squish
      LEFT JOIN LATERAL (
        SELECT sha
        FROM merge_request_commits_metadata
        WHERE merge_request_commits_metadata.id = merge_request_diff_commits.merge_request_commits_metadata_id
        AND merge_request_commits_metadata.project_id = ?
        LIMIT 1
      ) merge_request_commits_metadata ON TRUE
    SQL

    # raw SQL in pluck() bypass ActiveRecord's type casting, so encode() is needed to convert bytea to hex
    shas_sql = Arel.sql("encode(COALESCE(merge_request_commits_metadata.sha, merge_request_diff_commits.sha), 'hex')")

    relation = self.joins(self.sanitize_sql_array([metadata_join_sql, project_id])).order(:relative_order)

    relation = relation.limit(limit) if limit
