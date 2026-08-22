      expect(data_import.import_types).to eq(%w[contacts conversations])
      expect(data_import.stats).to include(
        'contacts' => include('total' => 12),
        'conversations' => include('total' => 8)
      )
