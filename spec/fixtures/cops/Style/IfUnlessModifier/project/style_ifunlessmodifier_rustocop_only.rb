      api_key = create(:api_key, :legacy_broad)
      owner = api_key.owner
      original = owner.method(:record_event!)
      owner.define_singleton_method(:record_event!) do |tag, **kwargs|
        raise ActiveRecord::StatementInvalid, "simulated incident-event failure" if
          tag == Events::UserEvent::CACHE_EXPOSURE_KEY_REVOKED
        original.call(tag, **kwargs)
      end
