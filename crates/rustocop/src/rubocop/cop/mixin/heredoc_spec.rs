use super::heredoc::*;

#[derive(Default)]
struct Recorder(usize);
impl Heredoc for Recorder {
    fn on_heredoc(&mut self, _node: &StringNode) {
        self.0 += 1;
    }
}

#[test]
fn callbacks_filter_non_heredocs_and_alias_all_string_kinds() {
    let heredoc = StringNode {
        source: "<<~'SQL'".into(),
        heredoc: true,
    };
    let plain = StringNode {
        source: "'value'".into(),
        heredoc: false,
    };
    let mut recorder = Recorder::default();
    recorder.on_str(&plain);
    recorder.on_str(&heredoc);
    recorder.on_dstr(&heredoc);
    recorder.on_xstr(&heredoc);
    assert_eq!(recorder.0, 3);
    assert_eq!(heredoc_type(&heredoc), "<<~");
    assert_eq!(delimiter_string(&heredoc), "SQL");
}

#[test]
fn indentation_ignores_blank_lines_and_uses_the_smallest_content_indent() {
    assert_eq!(indent_level("  one\n    two\n\n"), 2);
    assert_eq!(indent_level("\n"), 0);
}
