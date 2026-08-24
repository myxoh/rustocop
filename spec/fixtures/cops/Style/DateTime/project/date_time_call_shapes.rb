converted = to_datetime
explicit = value.to_datetime
modern = DateTime.iso8601("2016-06-29")
historic = DateTime.iso8601("1751-04-23", Date::ENGLAND)

DateTime.stub(:current, DateTime.civil(2005, 2, 10)) do
  DateTime.now
end
