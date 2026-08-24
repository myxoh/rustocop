foo(**attributes.merge(presenter_class: ::LabelPresenter))
foo(existing: true, **attributes.merge(presenter_class: ::LabelPresenter))
foo(**attributes.merge(presenter_class: ::LabelPresenter), &block)
    super(**attributes.merge(presenter_class: ::LabelPresenter))
