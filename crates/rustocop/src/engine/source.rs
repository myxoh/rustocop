use crate::model::SourceLine;

enum RestoreEncoding {
    Utf8,
    Marker { marker: u8, invalid_bytes: Vec<u8> },
    Latin1,
    Binary,
    Encoding(&'static encoding_rs::Encoding),
}

/// A reversible UTF-8 inspection view for Ruby source bytes. Declared source
/// encodings are transcoded for Prism and diagnostic text, while binary and
/// malformed input retain recoverable byte markers for exact rewrites.
pub(crate) struct DecodedSource {
    text: String,
    restore_encoding: RestoreEncoding,
}

impl DecodedSource {
    pub(crate) fn from_bytes(bytes: &[u8]) -> std::io::Result<Self> {
        if let Some(label) = declared_encoding(bytes) {
            if matches!(label.as_str(), "ascii-8bit" | "binary") {
                return Ok(Self {
                    text: binary_inspection_text(bytes),
                    restore_encoding: RestoreEncoding::Binary,
                });
            }
            if matches!(label.as_str(), "iso-8859-1" | "iso8859-1" | "latin1") {
                return Ok(Self {
                    text: bytes.iter().map(|byte| char::from(*byte)).collect(),
                    restore_encoding: RestoreEncoding::Latin1,
                });
            }
            if !matches!(label.as_str(), "utf-8" | "utf8" | "us-ascii") {
                if let Some(encoding) = encoding_rs::Encoding::for_label(label.as_bytes()) {
                    let (text, had_errors) = encoding.decode_without_bom_handling(bytes);
                    if !had_errors {
                        return Ok(Self {
                            text: text.into_owned(),
                            restore_encoding: RestoreEncoding::Encoding(encoding),
                        });
                    }
                }
            }
        }
        if let Ok(text) = std::str::from_utf8(bytes) {
            return Ok(Self {
                text: text.to_string(),
                restore_encoding: RestoreEncoding::Utf8,
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
            restore_encoding: RestoreEncoding::Marker {
                marker,
                invalid_bytes,
            },
        })
    }

    pub(crate) fn as_str(&self) -> &str {
        &self.text
    }

    pub(crate) fn restore(&self, text: &str) -> Vec<u8> {
        match &self.restore_encoding {
            RestoreEncoding::Utf8 => text.as_bytes().to_vec(),
            RestoreEncoding::Marker {
                marker,
                invalid_bytes,
            } => {
                let mut invalid = invalid_bytes.iter();
                text.as_bytes()
                    .iter()
                    .map(|byte| {
                        if byte == marker {
                            invalid.next().copied().unwrap_or(*byte)
                        } else {
                            *byte
                        }
                    })
                    .collect()
            }
            RestoreEncoding::Latin1 => text
                .chars()
                .flat_map(|character| {
                    if u32::from(character) <= 0xff {
                        vec![character as u8]
                    } else {
                        character.to_string().into_bytes()
                    }
                })
                .collect(),
            RestoreEncoding::Binary => restore_binary_text(text),
            RestoreEncoding::Encoding(encoding) => {
                let (encoded, _, _) = encoding.encode(text);
                encoded.into_owned()
            }
        }
    }
}

fn declared_encoding(bytes: &[u8]) -> Option<String> {
    let mut lines = bytes.split(|byte| *byte == b'\n');
    let first = lines.next().unwrap_or_default();
    let candidate = if first.starts_with(b"#!") {
        lines.next().unwrap_or_default()
    } else {
        first
    };
    let line = String::from_utf8_lossy(candidate);
    let lower = line.to_ascii_lowercase();
    for marker in ["coding", "encoding"] {
        let Some(marker_start) = lower.find(marker) else {
            continue;
        };
        let after_marker = &line[marker_start + marker.len()..];
        let separator = after_marker.find([':', '='])?;
        let label = after_marker[separator + 1..]
            .trim_start()
            .split(|character: char| {
                !(character.is_ascii_alphanumeric() || matches!(character, '-' | '_'))
            })
            .next()
            .unwrap_or("");
        if !label.is_empty() {
            return Some(label.to_ascii_lowercase());
        }
    }
    None
}

pub(crate) fn declares_binary_encoding(text: &str) -> bool {
    declared_encoding(text.as_bytes())
        .is_some_and(|encoding| matches!(encoding.as_str(), "ascii-8bit" | "binary"))
}

const BINARY_MARKER_START: u32 = 0xe000;

pub(crate) fn binary_inspection_text(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|byte| {
            if byte.is_ascii() {
                char::from(*byte)
            } else {
                char::from_u32(BINARY_MARKER_START + u32::from(*byte))
                    .expect("binary marker is a valid private-use character")
            }
        })
        .collect()
}

pub(crate) fn restore_binary_text(text: &str) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(text.len());
    for character in text.chars() {
        let value = u32::from(character);
        if (BINARY_MARKER_START + 0x80..=BINARY_MARKER_START + 0xff).contains(&value) {
            bytes.push((value - BINARY_MARKER_START) as u8);
        } else {
            let mut buffer = [0; 4];
            bytes.extend_from_slice(character.encode_utf8(&mut buffer).as_bytes());
        }
    }
    bytes
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

    #[test]
    fn transcodes_latin1_source_for_inspection_and_restores_original_bytes() {
        let source = b"# encoding: ISO-8859-1\ngem 'caf\xE9'\ngem 'caf\xF1'\n";
        let decoded = DecodedSource::from_bytes(source).unwrap();
        assert!(decoded.as_str().contains("café"));
        assert!(decoded.as_str().contains("cafñ"));
        assert_eq!(decoded.restore(decoded.as_str()), source);
    }

    #[test]
    fn binary_source_preserves_each_non_ascii_byte_as_a_distinct_value() {
        let source = b"# encoding: ASCII-8BIT\ngem 'caf\xE9'\ngem 'caf\xF1'\n";
        let decoded = DecodedSource::from_bytes(source).unwrap();
        assert_ne!(
            decoded.as_str().lines().nth(1),
            decoded.as_str().lines().nth(2)
        );
        assert_eq!(decoded.restore(decoded.as_str()), source);
    }

    #[test]
    fn transcodes_supported_multibyte_ruby_encodings_reversibly() {
        let source = b"# encoding: Shift_JIS\ngem '\x82\xA0'\n";
        let decoded = DecodedSource::from_bytes(source).unwrap();
        assert!(decoded.as_str().contains('あ'));
        assert_eq!(decoded.restore(decoded.as_str()), source);
    }
}
