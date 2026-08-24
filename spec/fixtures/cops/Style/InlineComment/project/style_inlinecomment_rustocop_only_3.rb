      expect { base_service.send(:event_name) }.to raise_error(NotImplementedError, /must implement #event_name/)
