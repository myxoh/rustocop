    assert_equal 3, clients.count

    assert_difference "Client.count", -(clients.count) do
      assert_equal clients.count, companies(:first_firm).dependent_clients_of_firm.delete_all
    end
