# frozen_string_literal: true

RSpec.describe RuboCop::Cop::Style::ImplicitRuntimeError, :config do
  it 'registers an offense for `raise` without error class' do
    expect_offense(<<~RUBY)
      raise 'message'
      ^^^^^^^^^^^^^^^ Use `raise` with an explicit exception class and message, rather than just a message.
    RUBY
  end

  it 'registers an offense for `fail` without error class' do
    expect_offense(<<~RUBY)
      fail 'message'
      ^^^^^^^^^^^^^^ Use `fail` with an explicit exception class and message, rather than just a message.
    RUBY
  end

  it 'registers an offense for `raise` with a multiline string' do
    expect_offense(<<~RUBY)
      raise 'message' \\
      ^^^^^^^^^^^^^^^^^ Use `raise` with an explicit exception class and message, rather than just a message.
            '2nd line'
    RUBY
  end

  it 'registers an offense for `fail` with a multiline string' do
    expect_offense(<<~RUBY)
      fail 'message' \\
      ^^^^^^^^^^^^^^^^ Use `fail` with an explicit exception class and message, rather than just a message.
            '2nd line'
    RUBY
  end

  it 'does not register an offense for `raise` with an error class' do
    expect_no_offenses(<<~RUBY)
      raise StandardError, 'message'
    RUBY
  end

  it 'does not register an offense for `fail` with an error class' do
    expect_no_offenses(<<~RUBY)
      fail StandardError, 'message'
    RUBY
  end

  it 'does not register an offense for `raise` without arguments' do
    expect_no_offenses('raise')
  end

  it 'does not register an offense for `fail` without arguments' do
    expect_no_offenses('fail')
  end

  context 'with adversarial call shapes' do
    it 'registers a parenthesized interpolated message' do
      expect_offense(<<~'RUBY')
        raise("bad #{value}")
        ^^^^^^^^^^^^^^^^^^^^^ Use `raise` with an explicit exception class and message, rather than just a message.
      RUBY
    end

    it 'does not register a receiver call or a dynamic message' do
      expect_no_offenses(<<~RUBY)
        handler.raise 'message'
        raise message
      RUBY
    end
  end
end
