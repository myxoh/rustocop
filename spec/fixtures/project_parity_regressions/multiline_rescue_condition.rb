begin
  work
rescue FirstError,
       SecondError => error
  handle(error)
rescue ThirdError
  retry_work
end
