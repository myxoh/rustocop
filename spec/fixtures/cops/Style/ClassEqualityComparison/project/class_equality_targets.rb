same = self.class == other.class
variable = self.class == klass
root = key.class == ::String
named = self.class.name == other.class.name
literal_name = self.class.name == "Widget"
dynamic_name = self.class.name == expected_name
