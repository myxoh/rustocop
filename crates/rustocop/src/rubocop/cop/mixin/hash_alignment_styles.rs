// RuboCop 1.87.0
// Source: lib/rubocop/cop/mixin/hash_alignment_styles.rb
// Source SHA-256: 442c96335947a457499dd3c494915eba604953083703fde656d249fb318ab41a

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct Deltas {
    pub(crate) key: Option<isize>,
    pub(crate) separator: Option<isize>,
    pub(crate) value: Option<isize>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PairLayout {
    pub(crate) key_source: String,
    pub(crate) key_column: usize,
    pub(crate) key_end_column: usize,
    pub(crate) pair_column: usize,
    pub(crate) operator_column: usize,
    pub(crate) operator_end_column: usize,
    pub(crate) value_column: usize,
    pub(crate) delimiter: String,
    pub(crate) first_line: usize,
    pub(crate) last_line: usize,
    pub(crate) hash_rocket: bool,
    pub(crate) value_on_new_line: bool,
    pub(crate) value_omission: bool,
    pub(crate) begins_its_line: bool,
}

impl PairLayout {
    fn key_delta(&self, current: &Self) -> isize {
        self.key_column as isize - current.key_column as isize
    }

    fn right_key_delta(&self, current: &Self) -> isize {
        self.key_end_column as isize - current.key_end_column as isize
    }

    fn delimiter_delta(&self, current: &Self) -> isize {
        self.operator_column as isize - current.operator_column as isize
    }

    fn value_delta(&self, current: &Self) -> isize {
        self.value_column as isize - current.value_column as isize
    }
}

pub(crate) struct KeyAlignment;

impl KeyAlignment {
    pub(crate) fn checkable_layout(&self, _pairs: &[PairLayout]) -> bool {
        true
    }

    pub(crate) fn deltas_for_first_pair(&self, first_pair: &PairLayout) -> Deltas {
        Deltas {
            separator: Some(self.separator_delta(first_pair)),
            value: Some(self.value_delta(first_pair)),
            ..Deltas::default()
        }
    }

    pub(crate) fn deltas(&self, first_pair: &PairLayout, current_pair: &PairLayout) -> Deltas {
        if !current_pair.begins_its_line {
            return Deltas::default();
        }
        Deltas {
            key: Some(first_pair.key_delta(current_pair)),
            separator: Some(self.separator_delta(current_pair)),
            value: Some(self.value_delta(current_pair)),
        }
    }

    pub(crate) fn separator_delta(&self, pair: &PairLayout) -> isize {
        if pair.hash_rocket {
            pair.key_end_column as isize + 1 - pair.operator_column as isize
        } else {
            0
        }
    }

    pub(crate) fn value_delta(&self, pair: &PairLayout) -> isize {
        if pair.value_on_new_line || pair.value_omission {
            0
        } else {
            pair.operator_end_column as isize + 1 - pair.value_column as isize
        }
    }
}

pub(crate) struct ValueAlignment;

impl ValueAlignment {
    pub(crate) fn checkable_layout(&self, pairs: &[PairLayout]) -> bool {
        !pairs
            .windows(2)
            .any(|pair| pair[0].last_line >= pair[1].first_line)
            && !Self::mixed_delimiters(pairs)
    }

    fn mixed_delimiters(pairs: &[PairLayout]) -> bool {
        pairs.first().is_some_and(|first| {
            pairs
                .iter()
                .any(|pair| pair.hash_rocket != first.hash_rocket)
        })
    }

    pub(crate) fn deltas(
        &self,
        key_delta: isize,
        separator_delta: isize,
        raw_value_delta: isize,
    ) -> Deltas {
        Deltas {
            key: Some(key_delta),
            separator: Some(separator_delta - key_delta),
            value: Some(raw_value_delta - key_delta - (separator_delta - key_delta)),
        }
    }
}

pub(crate) struct TableAlignment;

impl TableAlignment {
    pub(crate) fn deltas_for_first_pair(&self, pairs: &[PairLayout]) -> Deltas {
        let first_pair = &pairs[0];
        let separator_delta = self.separator_delta(first_pair, first_pair, 0, pairs);
        Deltas {
            separator: Some(separator_delta),
            value: Some(self.value_delta(first_pair, first_pair, pairs) - separator_delta),
            ..Deltas::default()
        }
    }

    pub(crate) fn deltas(
        &self,
        first: &PairLayout,
        current: &PairLayout,
        pairs: &[PairLayout],
    ) -> Deltas {
        let key_delta = self.key_delta(first, current);
        let separator = self.separator_delta(first, current, key_delta, pairs);
        let value = self.value_delta(first, current, pairs) - key_delta - separator;
        Deltas {
            key: Some(key_delta),
            separator: Some(separator),
            value: Some(value),
        }
    }

    pub(crate) fn key_delta(&self, first: &PairLayout, current: &PairLayout) -> isize {
        first.key_delta(current)
    }

    pub(crate) fn separator_delta(
        &self,
        first: &PairLayout,
        current: &PairLayout,
        key_delta: isize,
        pairs: &[PairLayout],
    ) -> isize {
        if current.hash_rocket {
            self.hash_rocket_delta(first, current, pairs) - key_delta
        } else {
            0
        }
    }

    pub(crate) fn hash_rocket_delta(
        &self,
        first: &PairLayout,
        current: &PairLayout,
        pairs: &[PairLayout],
    ) -> isize {
        first.pair_column as isize + self.max_key_width(pairs) as isize + 1
            - current.operator_column as isize
    }

    pub(crate) fn value_delta(
        &self,
        first: &PairLayout,
        current: &PairLayout,
        pairs: &[PairLayout],
    ) -> isize {
        if current.value_omission {
            0
        } else {
            first.key_column as isize
                + self.max_key_width(pairs) as isize
                + self.max_delimiter_width(pairs) as isize
                - current.value_column as isize
        }
    }

    pub(crate) fn max_key_width(&self, pairs: &[PairLayout]) -> usize {
        pairs
            .iter()
            .map(|pair| pair.key_source.len())
            .max()
            .unwrap_or(0)
    }

    pub(crate) fn max_delimiter_width(&self, pairs: &[PairLayout]) -> usize {
        pairs
            .iter()
            .map(|pair| pair.delimiter.len())
            .max()
            .unwrap_or(0)
    }
}

pub(crate) struct SeparatorAlignment;

impl SeparatorAlignment {
    pub(crate) fn deltas_for_first_pair(&self, _first_pair: &PairLayout) -> Deltas {
        Deltas::default()
    }

    pub(crate) fn deltas(&self, first: &PairLayout, current: &PairLayout) -> Deltas {
        let key_delta = self.key_delta(first, current);
        let separator_delta = self.separator_delta(first, current, key_delta);
        let value_delta = self.value_delta(first, current) - key_delta - separator_delta;
        Deltas {
            key: Some(key_delta),
            separator: Some(separator_delta),
            value: Some(value_delta),
        }
    }

    pub(crate) fn key_delta(&self, first: &PairLayout, current: &PairLayout) -> isize {
        first.right_key_delta(current)
    }

    pub(crate) fn separator_delta(
        &self,
        first: &PairLayout,
        current: &PairLayout,
        key_delta: isize,
    ) -> isize {
        if current.hash_rocket {
            self.hash_rocket_delta(first, current) - key_delta
        } else {
            0
        }
    }

    pub(crate) fn hash_rocket_delta(&self, first: &PairLayout, current: &PairLayout) -> isize {
        first.delimiter_delta(current)
    }

    pub(crate) fn value_delta(&self, first: &PairLayout, current: &PairLayout) -> isize {
        if current.value_omission {
            0
        } else {
            first.value_delta(current)
        }
    }
}

pub(crate) struct KeywordSplatAlignment;

impl KeywordSplatAlignment {
    pub(crate) fn deltas(&self, first_pair: &PairLayout, current_pair: &PairLayout) -> Deltas {
        if current_pair.begins_its_line {
            Deltas {
                key: Some(first_pair.key_delta(current_pair)),
                ..Deltas::default()
            }
        } else {
            Deltas::default()
        }
    }
}
