=begin
UPDATE node
SET owner_id = p.user_id, anonymous_name = NULL
WHERE p.name = lower(node.anonymous_name)
  AND owner_id IS NULL;
=end
