def first_name
  read_attribute(:first_name) || begin
    name.split(' ').first unless name.blank?
  end
end

def assign_type
  self.type = begin
    :video if video?
  end
end

def redundant
  begin
    perform_work
  end
end
