    add_foreign_key "active_storage_attachments", "active_storage_blobs", column: "blob_id"
    add_foreign_key "active_storage_variant_records", "active_storage_blobs", column: "blob_id"
    add_foreign_key "inboxes", "portals"
    create_trigger("accounts_after_insert_row_tr", :generated => true, :compatibility => 1).
        on("accounts").
        after(:insert).
        for_each(:row) do
      "execute format('create sequence IF NOT EXISTS conv_dpid_seq_%s', NEW.id);"
    end
