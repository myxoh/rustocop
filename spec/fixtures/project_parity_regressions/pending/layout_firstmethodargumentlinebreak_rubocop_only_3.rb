  private
    def mail_with_defaults(&block)
      mail(to: "test@localhost", from: "tester@example.com",
            subject: "using helpers", &block)
    end
