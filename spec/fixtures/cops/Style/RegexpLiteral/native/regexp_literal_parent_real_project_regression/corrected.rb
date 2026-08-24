match_data = %r{&lt;\s*(?:/\s*)?\w+}.match(content)
return if %r{=\s*["'][^>]*\z}.match?(match_data.pre_match)
encoded_word_regexp = /=[?].*[?]=/
log_regex = /==> Built a resource via #{method} in [\d.\-e]+ seconds+/
