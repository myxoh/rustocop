# Minimized from rails/rails@ba4f7369aee71f9f38d67bdbf0e8571fb372b535
# railties/test/application/action_controller_test_case_integration_test.rb.

source = <<~RUBY
  def get_current_customer
    render :index
  end
RUBY

def get_current_customer
  current_customer
end
