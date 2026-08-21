markdown_reference = /(\[([^\]]+)\]:([ ]+)(\S+))/
user_tag = %r{\[USER=\"\d+\"\]([\S]+)\[/USER\]}

extended_space = /[ ]/x
multiple = /[ ab]/

[markdown_reference, user_tag, extended_space, multiple]
