
    assert_equal ["image/jpeg; filename=controller_attachments.jpg",
                  "image/jpeg; filename=attachments.jpg"], mail.attachments.inline.map { |a| a["Content-Type"].to_s }
