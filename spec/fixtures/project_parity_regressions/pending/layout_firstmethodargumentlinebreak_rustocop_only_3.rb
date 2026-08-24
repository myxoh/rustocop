        markdown(CI_CD_TEMPLATE_MESSAGE)

        return unless changes.any?

        markdown(<<~MSG
              The following files require a review from the CI/CD Templates maintainers:
              #{helper.markdown_list(changes)}
        MSG
                )
