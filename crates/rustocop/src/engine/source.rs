use crate::model::SourceLine;

pub(crate) fn split(content: &str) -> Vec<SourceLine> {
    if content.is_empty() {
        return Vec::new();
    }
    content
        .split_inclusive('\n')
        .map(|raw_line| {
            if let Some(body) = raw_line.strip_suffix("\r\n") {
                SourceLine {
                    body: body.to_string(),
                    ending: "\r\n".to_string(),
                }
            } else if let Some(body) = raw_line.strip_suffix('\n') {
                SourceLine {
                    body: body.to_string(),
                    ending: "\n".to_string(),
                }
            } else {
                SourceLine {
                    body: raw_line.to_string(),
                    ending: String::new(),
                }
            }
        })
        .collect()
}

pub(crate) fn join(lines: &[SourceLine]) -> String {
    let capacity = lines
        .iter()
        .map(|line| line.body.len() + line.ending.len())
        .sum();
    let mut content = String::with_capacity(capacity);
    for line in lines {
        content.push_str(&line.body);
        content.push_str(&line.ending);
    }
    content
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_mixed_line_endings() {
        let source = "first\r\nsecond\nlast";
        assert_eq!(join(&split(source)), source);
    }
}
