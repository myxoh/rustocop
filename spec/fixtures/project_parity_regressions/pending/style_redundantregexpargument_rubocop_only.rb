            uploads
              .map { |item| "![#{item[:prompt].gsub(/\|\'\"/, "")}](#{item[:upload].short_url})" }
              .join(" ")
