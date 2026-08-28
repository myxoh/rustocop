use super::NotebookCellKind;

#[test]
fn exposes_every_notebook_cell_kind() {
    assert_eq!(NotebookCellKind::MARKUP, 1);
    assert_eq!(NotebookCellKind::CODE, 2);
}

#[test]
fn exposes_integer_protocol_discriminants() {
    let kinds: [i64; 2] = [NotebookCellKind::MARKUP, NotebookCellKind::CODE];

    assert_eq!(kinds, [1, 2]);
}
