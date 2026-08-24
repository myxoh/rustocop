    let!(:referenced_lfs_object2) { create(:lfs_object, oid: '4' * 64) }
    let!(:lfs_objects_project1_1) do
      create(:lfs_objects_project, project: project1, lfs_object: referenced_lfs_object1
      )
    end
