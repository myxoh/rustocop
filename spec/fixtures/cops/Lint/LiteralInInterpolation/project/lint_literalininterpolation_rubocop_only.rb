    authors = Author.select("authors.name, #{%{(authors.author_address_id || ' ' || authors.author_address_extra_id) as addr_id}}").left_outer_joins(:posts)
