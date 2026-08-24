def ambiguous_if = true if enabled?
def ambiguous_or(value) = value or fallback
def ambiguous_until = work until finished?

def ordinary(value = "this or that")
  value if enabled?
end

(def explicit = true) if enabled?
def explicit_body = (true if enabled?)
