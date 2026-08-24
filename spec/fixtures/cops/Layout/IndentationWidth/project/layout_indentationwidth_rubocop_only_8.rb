
step 'Call a global function' do
  fail_test 'Failed to call a "global" function' \
    unless  rc.stdout.include?('jenny::mini(1, 2) = 1')
  end
