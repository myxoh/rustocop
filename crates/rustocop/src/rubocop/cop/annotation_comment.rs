// RuboCop 1.87.0
// Source: lib/rubocop/cop/mixin/annotation_comment.rb
// Source SHA-256: 2d6dd6e97ebc587bcf38ba351f5f9da7ffb62a91937a1a10115f20e4158e043f

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct AnnotationComment {
    comment_begin: usize,
    comment: String,
    keywords: Vec<String>,
    margin: Option<String>,
    keyword: Option<String>,
    colon: Option<String>,
    space: Option<String>,
    note: Option<String>,
}

impl AnnotationComment {
    pub(crate) fn new(comment_text: &str, comment_begin: usize, keywords: &[String]) -> Self {
        let mut annotation = Self {
            comment_begin,
            comment: comment_text.to_owned(),
            keywords: keywords.to_vec(),
            margin: None,
            keyword: None,
            colon: None,
            space: None,
            note: None,
        };
        annotation.split_comment(comment_text, keywords);
        annotation
    }

    pub(crate) fn comment(&self) -> &str {
        &self.comment
    }

    pub(crate) fn keywords(&self) -> &[String] {
        &self.keywords
    }

    pub(crate) fn annotation(&self) -> Option<bool> {
        self.keyword.as_ref()?;
        let appearance = self.colon.is_some() || self.space.is_some();
        Some(appearance && !self.just_keyword_of_sentence())
    }

    pub(crate) fn keyword_appearance(&self) -> bool {
        self.keyword.is_some() && (self.colon.is_some() || self.space.is_some())
    }

    pub(crate) fn correct(&self, colon: bool) -> bool {
        let (Some(keyword), Some(_space), Some(_note)) = (&self.keyword, &self.space, &self.note)
        else {
            return false;
        };
        if keyword != &keyword.to_uppercase() {
            return false;
        }
        self.colon.is_none() != colon
    }

    pub(crate) fn bounds(&self) -> Option<(usize, usize)> {
        let margin = self.margin.as_ref()?;
        let start = self.comment_begin + margin.chars().count();
        let length = [&self.keyword, &self.colon, &self.space]
            .into_iter()
            .flatten()
            .map(|part| part.chars().count())
            .sum::<usize>();
        Some((start, start + length))
    }

    pub(crate) fn margin(&self) -> Option<&str> {
        self.margin.as_deref()
    }
    pub(crate) fn keyword(&self) -> Option<&str> {
        self.keyword.as_deref()
    }
    pub(crate) fn colon(&self) -> Option<&str> {
        self.colon.as_deref()
    }
    pub(crate) fn space(&self) -> Option<&str> {
        self.space.as_deref()
    }
    pub(crate) fn note(&self) -> Option<&str> {
        self.note.as_deref()
    }

    fn split_comment(&mut self, text: &str, keywords: &[String]) {
        let Some(after_hash) = text.strip_prefix('#') else {
            return;
        };
        let (margin, body) = after_hash
            .strip_prefix(' ')
            .map_or(("#", after_hash), |body| ("# ", body));
        let mut sorted = keywords.to_vec();
        sorted.sort_by_key(|keyword| std::cmp::Reverse(keyword.chars().count()));
        let Some((keyword, remainder)) = sorted.iter().find_map(|keyword| {
            let length = keyword.len();
            let prefix = body.get(..length)?;
            if !prefix.eq_ignore_ascii_case(keyword) {
                return None;
            }
            let remainder = &body[length..];
            let boundary = remainder
                .chars()
                .next()
                .is_none_or(|character| !is_word(character));
            boundary.then_some((prefix, remainder))
        }) else {
            return;
        };

        let colon_offset = remainder
            .char_indices()
            .find(|(_, character)| !character.is_whitespace())
            .map_or(remainder.len(), |(index, _)| index);
        let (colon, after_colon) = if remainder[colon_offset..].starts_with(':') {
            (
                Some(&remainder[..colon_offset + 1]),
                &remainder[colon_offset + 1..],
            )
        } else {
            (None, remainder)
        };
        let space_end = after_colon
            .char_indices()
            .find(|(_, character)| !character.is_whitespace())
            .map_or(after_colon.len(), |(index, _)| index);
        let space = (space_end > 0).then_some(&after_colon[..space_end]);
        let note = after_colon[space_end..]
            .split_whitespace()
            .next()
            .filter(|note| !note.is_empty());

        self.margin = Some(margin.to_owned());
        self.keyword = Some(keyword.to_owned());
        self.colon = colon.map(str::to_owned);
        self.space = space.map(str::to_owned);
        self.note = note.map(str::to_owned);
    }

    fn just_keyword_of_sentence(&self) -> bool {
        let Some(keyword) = self.keyword.as_deref() else {
            return false;
        };
        let mut characters = keyword.chars();
        let capitalized = characters.next().map_or_else(String::new, |first| {
            first
                .to_uppercase()
                .chain(characters.flat_map(char::to_lowercase))
                .collect()
        });
        keyword == capitalized
            && self.colon.is_none()
            && self.space.is_some()
            && self.note.is_some()
    }
}

fn is_word(character: char) -> bool {
    character.is_alphanumeric() || character == '_'
}
