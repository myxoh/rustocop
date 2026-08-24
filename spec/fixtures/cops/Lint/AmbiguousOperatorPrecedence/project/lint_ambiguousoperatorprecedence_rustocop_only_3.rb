          allow(Spree::Order).to receive(:count).and_return(10**Spree::Order::ORDER_NUMBER_LENGTH / 2 + 1)
