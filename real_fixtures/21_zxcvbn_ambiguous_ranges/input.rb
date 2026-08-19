# frozen_string_literal: true

module Zxcvbn
  module Matchers
    class Date
      def match_with_separator(password)
        result = []
        return result if password.length < 6

        (0..password.length - 6).each do |i|
          (i + 5..[i + 9, password.length - 1].min).each do |j|
            token = password[i..j]
            m = MAYBE_DATE_WITH_SEP.match(token)
            next unless m

            date = map_ints_to_dmy(m[1].to_i, m[3].to_i, m[4].to_i)
            next unless date

            result << Match.new(
              i: i, j: j, token: token,
              pattern: 'date',
              separator: m[2],
              year: date[:year],
              month: date[:month],
              day: date[:day]
            )
          end
        end
        result
      end
    end
  end
end
