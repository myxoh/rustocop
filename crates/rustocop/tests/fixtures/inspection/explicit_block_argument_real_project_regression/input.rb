def self.issue_create(input)
  <<~GRAPHQL
    mutation {
      issueCreate(input: { #{graphql_input(input)} }) {
        success
        issue {
          id
          title
          identifier
        }
      }
    }
  GRAPHQL
end
