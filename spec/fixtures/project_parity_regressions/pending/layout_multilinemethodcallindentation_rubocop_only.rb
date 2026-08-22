    contacts = Current.account.contacts.where(id: ::OnlineStatusTracker
                  .get_available_contact_ids(Current.account.id))
    @contacts = fetch_contacts(contacts)
