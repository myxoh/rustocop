use crate::model::SourceLine;

/// A byte-preserving inspection view for Ruby source that is not valid UTF-8.
/// Prism accepts byte source, while most of Rustocop's authoring APIs use
/// `str`; an unused one-byte control marker keeps all parser offsets stable.
pub(crate) struct DecodedSource {
    text: String,
    marker: Option<u8>,
    invalid_bytes: Vec<u8>,
}

impl DecodedSource {
    pub(crate) fn from_bytes(bytes: &[u8]) -> std::io::Result<Self> {
        if let Ok(text) = std::str::from_utf8(bytes) {
            return Ok(Self {
                text: text.to_string(),
                marker: None,
                invalid_bytes: Vec::new(),
            });
        }
        let marker = (1..=31)
            .filter(|byte| !matches!(byte, 9 | 10 | 13))
            .find(|byte| !bytes.contains(byte))
            .ok_or_else(|| {
                std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "source uses every available byte-preserving marker",
                )
            })?;
        let mut text = Vec::with_capacity(bytes.len());
        let mut invalid_bytes = Vec::new();
        let mut remaining = bytes;
        while !remaining.is_empty() {
            match std::str::from_utf8(remaining) {
                Ok(_) => {
                    text.extend_from_slice(remaining);
                    break;
                }
                Err(error) => {
                    let valid = error.valid_up_to();
                    text.extend_from_slice(&remaining[..valid]);
                    let invalid_length = error.error_len().unwrap_or(remaining.len() - valid);
                    for byte in &remaining[valid..valid + invalid_length] {
                        text.push(marker);
                        invalid_bytes.push(*byte);
                    }
                    remaining = &remaining[valid + invalid_length..];
                }
            }
        }
        Ok(Self {
            text: String::from_utf8(text).expect("invalid bytes were replaced with ASCII markers"),
            marker: Some(marker),
            invalid_bytes,
        })
    }

    pub(crate) fn as_str(&self) -> &str {
        &self.text
    }

    pub(crate) fn restore(&self, text: &str) -> Vec<u8> {
        let Some(marker) = self.marker else {
            return text.as_bytes().to_vec();
        };
        let mut invalid = self.invalid_bytes.iter();
        text.as_bytes()
            .iter()
            .map(|byte| {
                if *byte == marker {
                    invalid.next().copied().unwrap_or(*byte)
                } else {
                    *byte
                }
            })
            .collect()
    }
}

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

    #[test]
    fn preserves_invalid_bytes_around_source_edits() {
        let decoded = DecodedSource::from_bytes(b"%i[\xC0 :foo]\n").unwrap();
        assert_eq!(decoded.as_str().len(), b"%i[\xC0 :foo]\n".len());
        let corrected = decoded.as_str().replace(":foo", "foo");
        assert_eq!(decoded.restore(&corrected), b"%i[\xC0 foo]\n");
    }
}
