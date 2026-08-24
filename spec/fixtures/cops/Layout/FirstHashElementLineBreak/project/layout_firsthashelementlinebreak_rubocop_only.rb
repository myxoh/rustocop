    order2 = Cpk::Order.create!(id: [9999, 10002], status: "open")

    book = nil
    assert_difference -> { order1.reload.books_count }, 1 do
      book = Cpk::Book.create!(id: [9999, 10001], title: "Book", order: order1)
    end

    assert_difference(
      { -> { order1.reload.books_count } => -1,
        -> { order2.reload.books_count } => 1 }
    ) do
      book.update!(order: order2)
    end

    assert_difference -> { order2.reload.books_count }, -1 do
      book.destroy!
    end
