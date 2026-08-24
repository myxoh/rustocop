if tag_user
  return tag_user if tag_user.notification_level == level
  tag_user.save
else
  tag_user = TagUser.create
end

tag_user = TagUser.create unless tag_user
