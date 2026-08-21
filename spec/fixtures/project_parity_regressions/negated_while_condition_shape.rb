begin
  create_admin
end while !saved

do_work while !finish || !queue.empty?
work while !ready?
