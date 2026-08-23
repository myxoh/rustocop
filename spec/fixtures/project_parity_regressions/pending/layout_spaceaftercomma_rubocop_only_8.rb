      assert_equal({'<<' => [1,2,3]}, Psych.unsafe_load(yaml)['development'])
