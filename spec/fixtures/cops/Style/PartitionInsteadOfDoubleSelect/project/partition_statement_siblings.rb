positives = values.select { |value| value.positive? }
negatives = values.reject { |value| value.positive? }

sorted = values.select { |value| value.positive? }.sort
reversed = values.reject { |value| value.positive? }.reverse

def positive_values = values.select { |value| value.positive? }
def negative_values = values.reject { |value| value.positive? }

[positives, negatives, sorted, reversed, positive_values, negative_values]
