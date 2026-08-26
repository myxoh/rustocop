# frozen_string_literal: true

RSpec.describe RuboCop::Cop::Lint::InterpolationCheck, :config do
  it 'registers an offense and corrects for interpolation in single quoted string' do
    expect_offense(<<~'RUBY')
      'foo #{bar}'
      ^^^^^^^^^^^^ Interpolation in single quoted string detected. Use double quoted strings if you need interpolation.
    RUBY

    expect_correction(<<~'RUBY')
      "foo #{bar}"
    RUBY
  end

  it 'registers an offense and corrects when containing a closing brace without double quotes' do
    expect_offense(<<~'RUBY')
      'foo #{bar} }'
      ^^^^^^^^^^^^^^ Interpolation in single quoted string detected. Use double quoted strings if you need interpolation.
    RUBY

    expect_correction(<<~'RUBY')
      "foo #{bar} }"
    RUBY
  end

  it 'registers an offense and corrects when including interpolation and double quoted string in single quoted string' do
    expect_offense(<<~'RUBY')
      'foo "#{bar}"'
      ^^^^^^^^^^^^^^ Interpolation in single quoted string detected. Use double quoted strings if you need interpolation.
    RUBY

    expect_correction(<<~'RUBY')
      %{foo "#{bar}"}
    RUBY
  end

  it 'registers an offense for interpolation in single quoted split string' do
    expect_offense(<<~'RUBY')
      'x' \
        'foo #{bar}'
        ^^^^^^^^^^^^ Interpolation in single quoted string detected. Use double quoted strings if you need interpolation.
    RUBY
  end

  it 'registers an offense for interpolation in double + single quoted split string' do
    expect_offense(<<~'RUBY')
      "x" \
        'foo #{bar}'
        ^^^^^^^^^^^^ Interpolation in single quoted string detected. Use double quoted strings if you need interpolation.
    RUBY
  end

  it 'does not register an offense for properly interpolation strings' do
    expect_no_offenses(<<~'RUBY')
      hello = "foo #{bar}"
    RUBY
  end

  it 'does not register an offense for interpolation in nested strings' do
    expect_no_offenses(<<~'RUBY')
      foo = "bar '#{baz}' qux"
    RUBY
  end

  it 'does not register an offense for interpolation in a regexp' do
    expect_no_offenses(<<~'RUBY')
      /\#{20}/
    RUBY
  end

  it 'does not register an offense for an escaped interpolation' do
    expect_no_offenses(<<~'RUBY')
      "\#{msg}"
    RUBY
  end

  it 'does not crash for \xff' do
    expect_no_offenses(<<~'RUBY')
      foo = "\xff"
    RUBY
  end

  it 'does not register an offense for escaped crab claws in dstr' do
    expect_no_offenses(<<~'RUBY')
      foo = "alpha #{variable} beta \#{gamma}\" delta"
    RUBY
  end

  it 'does not register offense for strings in %w()' do
    expect_no_offenses(<<~'RUBY')
      %w("#{a}-foo")
    RUBY
  end

  it 'does not register an offense when using invalid syntax in interpolation' do
    expect_no_offenses(<<~'RUBY')
      '#{%<expression>s}'
    RUBY
  end

  it 'does not register an offense when using invalid syntax in interpolation with double quotes' do
    expect_no_offenses(<<~'RUBY')
      'Text `A("#{%<base>s}/%<path>s")` and `B` with C.'
    RUBY
  end

  it 'does not register an offense when double quotes and unbalanced braces would break percent literal' do
    expect_no_offenses(<<~'RUBY')
      'a "b" } #{c}'
    RUBY
  end

  context 'with adversarial literal boundaries' do
    it 'does not inspect interpolation-looking text in a single quoted heredoc' do
      expect_no_offenses(<<~'RUBY')
        <<~'TEXT'
          #{not_interpolated}
        TEXT
      RUBY
    end

    it 'registers an offense for an unescaped interpolation after an escaped hash' do
      expect_offense(<<~'RUBY')
        '\#{literal} then #{dynamic}'
        ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ Interpolation in single quoted string detected. Use double quoted strings if you need interpolation.
      RUBY
    end


    it 'does not register when the interpolation marker is immediately escaped' do
      expect_no_offenses(<<~'RUBY')
        '"\\#{test}"'
      RUBY
    end

    it 'does not register when changing multiline documentation to double quotes is not one string' do
      expect_no_offenses(<<~'RUBY')
        'command(
          value: "#{example}",
        )'
      RUBY
    end

    it 'registers an interpolation containing `yield` in a block' do
      expect_offense(<<~'RUBY')
        layout { 'THIS. IS. #{yield.upcase}!' }
                 ^^^^^^^^^^^^^^^^^^^^^^^^^^^^ Interpolation in single quoted string detected. Use double quoted strings if you need interpolation.
      RUBY

      expect_correction(<<~'RUBY')
        layout { "THIS. IS. #{yield.upcase}!" }
      RUBY
    end
  end
end
