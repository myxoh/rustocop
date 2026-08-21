stdout = output.select { |type,| type == :stdout }
               .map { |_, line| line }
stderr = output.select { |type,| type == :stderr }
               .map { |_, line| line }
