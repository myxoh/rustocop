// rubocop-ast 1.49.1
// Source: lib/rubocop/ast/processed_source.rb
// Source SHA-256: dc07b2fc33f68dd854847f0b81dec10b96a356c3f95094edc18e83cf9c0a75ab

use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::ops::{Index, Range};
use std::path::{Path, PathBuf};

use ruby_prism::{parse as prism_parse, CommentType, ParseResult};

use super::node::core::{Ast, NodeId, NodeRef};
use super::prism;
use super::source::SourceBuffer;
use super::source_position::SourcePositionIndex;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ParserEngine {
    Default,
    Whitequark,
    Prism,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ParserEngineError(pub(crate) String);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum DiagnosticLevel {
    Warning,
    Error,
    Fatal,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SourceDiagnostic {
    pub(crate) level: DiagnosticLevel,
    pub(crate) message: String,
    pub(crate) range: Range<usize>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ParserDescriptor {
    pub(crate) builder_class: &'static str,
    pub(crate) parser_class: &'static str,
    pub(crate) reuses_prism_result: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SourceComment {
    pub(crate) text: String,
    pub(crate) range: Range<usize>,
    pub(crate) line: usize,
    pub(crate) embedded_document: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SourceToken {
    pub(crate) kind: &'static str,
    pub(crate) text: String,
    pub(crate) range: Range<usize>,
    pub(crate) line: usize,
    pub(crate) column: usize,
}

impl SourceToken {
    pub(crate) fn begin_pos(&self) -> usize {
        self.range.start
    }
    pub(crate) fn end_pos(&self) -> usize {
        self.range.end
    }
    pub(crate) fn comment(&self) -> bool {
        self.kind == "tCOMMENT"
    }
    pub(crate) fn comma(&self) -> bool {
        self.kind == "tCOMMA"
    }
    pub(crate) fn left_brace(&self) -> bool {
        self.kind == "tLBRACE"
    }
    pub(crate) fn right_bracket(&self) -> bool {
        self.kind == "tRBRACK"
    }
    pub(crate) fn semicolon(&self) -> bool {
        self.kind == "tSEMI"
    }
    pub(crate) fn left_array_bracket(&self) -> bool {
        self.kind == "tLBRACK"
    }
    pub(crate) fn left_ref_bracket(&self) -> bool {
        self.kind == "tLBRACK2"
    }
    pub(crate) fn left_bracket(&self) -> bool {
        matches!(self.kind, "tLBRACK" | "tLBRACK2")
    }
    pub(crate) fn left_curly_brace(&self) -> bool {
        matches!(self.kind, "tLCURLY" | "tLAMBEG")
    }
    pub(crate) fn right_curly_brace(&self) -> bool {
        self.kind == "tRCURLY"
    }
    pub(crate) fn left_parens(&self) -> bool {
        matches!(self.kind, "tLPAREN" | "tLPAREN2")
    }
    pub(crate) fn right_parens(&self) -> bool {
        self.kind == "tRPAREN"
    }
    pub(crate) fn dot(&self) -> bool {
        self.kind == "tDOT"
    }
    pub(crate) fn regexp_dots(&self) -> bool {
        matches!(self.kind, "tDOT2" | "tDOT3")
    }
    pub(crate) fn rescue_modifier(&self) -> bool {
        self.kind == "kRESCUE_MOD"
    }
    pub(crate) fn end_keyword(&self) -> bool {
        self.kind == "kEND"
    }
    pub(crate) fn equal_sign(&self) -> bool {
        matches!(self.kind, "tEQL" | "tOP_ASGN")
    }
    pub(crate) fn new_line(&self) -> bool {
        self.kind == "tNL"
    }
}

#[derive(Clone, Debug)]
pub(crate) struct ProcessedSource<'source> {
    raw_source: &'source str,
    path: Option<PathBuf>,
    ruby_version: f64,
    parser_engine: ParserEngine,
    valid_syntax: bool,
    blank: bool,
    lines: Vec<String>,
    comments: Vec<SourceComment>,
    comment_index: BTreeMap<usize, usize>,
    tokens: Vec<SourceToken>,
    diagnostics: Vec<SourceDiagnostic>,
    parser_error: Option<String>,
    ast: Ast,
    ast_root: Option<NodeId>,
}

impl<'source> ProcessedSource<'source> {
    pub(crate) fn initialize(
        source: &'source str,
        ruby_version: f64,
        path: Option<PathBuf>,
        parser_engine: ParserEngine,
    ) -> Result<Self, ParserEngineError> {
        Self::new(source, ruby_version, path, parser_engine)
    }

    pub(crate) fn builder_class(parser_engine: ParserEngine) -> &'static str {
        match parser_engine {
            ParserEngine::Whitequark => "RuboCop::AST::Builder",
            ParserEngine::Prism | ParserEngine::Default => "RuboCop::AST::BuilderPrism",
        }
    }

    pub(crate) fn parser_class(
        ruby_version: f64,
        parser_engine: ParserEngine,
    ) -> Result<&'static str, ParserEngineError> {
        match parser_engine {
            ParserEngine::Whitequark => match ruby_version {
                1.9 => Ok("Parser::Ruby19"), 2.0 => Ok("Parser::Ruby20"),
                2.1 => Ok("Parser::Ruby21"), 2.2 => Ok("Parser::Ruby22"),
                2.3 => Ok("Parser::Ruby23"), 2.4 => Ok("Parser::Ruby24"),
                2.5 => Ok("Parser::Ruby25"), 2.6 => Ok("Parser::Ruby26"),
                2.7 => Ok("Parser::Ruby27"), 2.8 | 3.0 => Ok("Parser::Ruby30"),
                3.1 => Ok("Parser::Ruby31"), 3.2 => Ok("Parser::Ruby32"),
                3.3 => Ok("Parser::Ruby33"), 3.4 => Ok("Parser::Ruby34"),
                _ => Err(ParserEngineError(format!("RuboCop supports target Ruby versions 3.4 and below with `parser`. Specified target Ruby version: {ruby_version}"))),
            },
            ParserEngine::Prism | ParserEngine::Default => match ruby_version {
                3.3 => Ok("Prism::Translation::Parser33"),
                3.4 => Ok("Prism::Translation::Parser34"),
                3.5 | 4.0 => Ok("Prism::Translation::Parser40"),
                4.1 => Ok("Prism::Translation::Parser41"),
                _ => Err(ParserEngineError(format!("RuboCop supports target Ruby versions 3.3 and above with Prism. Specified target Ruby version: {ruby_version}"))),
            },
        }
    }

    pub(crate) fn create_parser(
        ruby_version: f64,
        parser_engine: ParserEngine,
        prism_result: bool,
    ) -> Result<ParserDescriptor, ParserEngineError> {
        Ok(ParserDescriptor {
            builder_class: Self::builder_class(parser_engine),
            parser_class: Self::parser_class(ruby_version, parser_engine)?,
            reuses_prism_result: parser_engine == ParserEngine::Prism && prism_result,
        })
    }

    pub(crate) fn parse(
        source: &'source str,
        ruby_version: f64,
        parser_engine: ParserEngine,
    ) -> Result<Self, ParserEngineError> {
        Self::new(source, ruby_version, None, parser_engine)
    }

    pub(crate) fn parse_lex(
        source: &'source str,
        ruby_version: f64,
    ) -> Result<Self, ParserEngineError> {
        Self::new(source, ruby_version, None, ParserEngine::Prism)
    }

    pub(crate) fn new(
        source: &'source str,
        ruby_version: f64,
        path: Option<PathBuf>,
        parser_engine: ParserEngine,
    ) -> Result<Self, ParserEngineError> {
        let parser_engine = normalize_parser_engine(parser_engine, ruby_version)?;
        let parsed = prism_parse(source.as_bytes());
        Ok(Self::from_prism_result_unchecked(
            source,
            ruby_version,
            path,
            parser_engine,
            &parsed,
        ))
    }

    pub(crate) fn from_prism_result(
        source: &'source str,
        ruby_version: f64,
        path: Option<PathBuf>,
        parser_engine: ParserEngine,
        parsed: &ParseResult<'_>,
    ) -> Result<Self, ParserEngineError> {
        let parser_engine = normalize_parser_engine(parser_engine, ruby_version)?;
        Ok(Self::from_prism_result_unchecked(
            source,
            ruby_version,
            path,
            parser_engine,
            parsed,
        ))
    }

    fn from_prism_result_unchecked(
        source: &'source str,
        ruby_version: f64,
        path: Option<PathBuf>,
        parser_engine: ParserEngine,
        parsed: &ParseResult<'_>,
    ) -> Self {
        let positions = SourcePositionIndex::new(source);
        let diagnostics: Vec<_> = parsed
            .warnings()
            .map(|diagnostic| {
                source_diagnostic(
                    &positions,
                    DiagnosticLevel::Warning,
                    diagnostic.message(),
                    diagnostic.location().start_offset()..diagnostic.location().end_offset(),
                )
            })
            .chain(parsed.errors().map(|diagnostic| {
                source_diagnostic(
                    &positions,
                    DiagnosticLevel::Error,
                    diagnostic.message(),
                    diagnostic.location().start_offset()..diagnostic.location().end_offset(),
                )
            }))
            .collect();
        let valid_syntax = !diagnostics.iter().any(|diagnostic| {
            matches!(
                diagnostic.level,
                DiagnosticLevel::Error | DiagnosticLevel::Fatal
            )
        });
        let parsed_comments: Vec<_> = parsed
            .comments()
            .map(|comment| {
                let byte_range = comment.location().start_offset()..comment.location().end_offset();
                let range = positions.character_range(byte_range.clone());
                SourceComment {
                    text: String::from_utf8_lossy(comment.text()).into_owned(),
                    line: positions.line_for_byte(byte_range.start),
                    range,
                    embedded_document: comment.type_() == CommentType::EmbDocComment,
                }
            })
            .collect();
        let comments = if valid_syntax {
            parsed_comments
        } else {
            Vec::new()
        };
        let comment_index = comments
            .iter()
            .enumerate()
            .map(|(index, comment)| (comment.line, index))
            .collect();
        let data_start = parsed.data_loc().map(|location| location.start_offset());
        let lines = source_lines(source, data_start);
        let tokens = if valid_syntax {
            let mut tokens = lex(source, data_start, &positions);
            // The lightweight lexer intentionally does not reproduce every
            // regexp/interpolation state. Prism's comment list is
            // authoritative: discard the lexer's guesses and add exactly the
            // comments Prism found. This prevents both false comments inside
            // literals and missed comments after the lightweight lexer has
            // become desynchronized by syntax it does not model.
            tokens.retain(|token| !token.comment());
            tokens.extend(parsed.comments().map(|comment| {
                let byte_range = comment.location().start_offset()..comment.location().end_offset();
                SourceToken {
                    kind: "tCOMMENT",
                    text: String::from_utf8_lossy(comment.text()).into_owned(),
                    range: positions.character_range(byte_range.clone()),
                    line: positions.line_for_byte(byte_range.start),
                    column: positions.column_for_byte(byte_range.start),
                }
            }));
            // Replacing the lightweight lexer's comments with Prism's
            // authoritative comments must preserve Parser's source ordering.
            // Several RuboCop APIs (notably `ProcessedSource#tokens[0]`) rely
            // on this invariant.
            tokens.sort_by_key(|token| (token.range.start, token.range.end));
            tokens
        } else {
            Vec::new()
        };
        let blank = !valid_syntax
            || source.trim().is_empty()
            || parsed
                .node()
                .as_program_node()
                .is_some_and(|program| program.statements().body().is_empty());
        let (ast, ast_root) = if valid_syntax {
            prism::convert(source, &parsed.node())
        } else {
            (Ast::new(source), None)
        };
        Self {
            raw_source: source,
            path,
            ruby_version,
            parser_engine,
            valid_syntax,
            blank,
            lines,
            comments,
            comment_index,
            tokens,
            diagnostics,
            parser_error: None,
            ast,
            ast_root,
        }
    }

    pub(crate) fn buffer(&self) -> SourceBuffer<'source> {
        SourceBuffer::new(self.raw_source)
    }
    pub(crate) fn raw_source(&self) -> &'source str {
        self.raw_source
    }
    pub(crate) fn path(&self) -> Option<&Path> {
        self.path.as_deref()
    }
    pub(crate) fn file_path(&self) -> &str {
        self.path
            .as_deref()
            .and_then(Path::to_str)
            .unwrap_or("(string)")
    }
    pub(crate) fn ruby_version(&self) -> f64 {
        self.ruby_version
    }
    pub(crate) fn parser_engine(&self) -> ParserEngine {
        self.parser_engine
    }
    pub(crate) fn diagnostics(&self) -> &[SourceDiagnostic] {
        &self.diagnostics
    }
    pub(crate) fn parser_error(&self) -> Option<&str> {
        self.parser_error.as_deref()
    }
    pub(crate) fn ast(&self) -> Option<NodeRef<'_>> {
        self.ast_root.map(|root| self.ast.node(root))
    }
    pub(crate) fn valid_syntax(&self) -> bool {
        self.parser_error.is_none() && self.valid_syntax
    }
    pub(crate) fn blank(&self) -> bool {
        self.blank
    }
    pub(crate) fn lines(&self) -> &[String] {
        &self.lines
    }
    pub(crate) fn line(&self, index: usize) -> Option<&str> {
        self.lines.get(index).map(String::as_str)
    }
    pub(crate) fn comments(&self) -> &[SourceComment] {
        &self.comments
    }
    pub(crate) fn comment_index(&self) -> &BTreeMap<usize, usize> {
        &self.comment_index
    }
    pub(crate) fn each_comment(&self) -> impl Iterator<Item = &SourceComment> {
        self.comments.iter()
    }
    pub(crate) fn comments_for(&self, node: NodeRef<'_>) -> Vec<&SourceComment> {
        self.comments
            .iter()
            .filter(|comment| self.associated_node(comment) == Some(node))
            .collect()
    }
    pub(crate) fn ast_with_comments(&self) -> Vec<(NodeRef<'_>, Vec<&SourceComment>)> {
        self.ast().map_or_else(Vec::new, |root| {
            root.each_node(&[])
                .into_iter()
                .filter_map(|node| {
                    let comments = self.comments_for(node);
                    (!comments.is_empty()).then_some((node, comments))
                })
                .collect()
        })
    }
    fn associated_node(&self, comment: &SourceComment) -> Option<NodeRef<'_>> {
        let nodes = self.ast()?.each_node(&[]);
        let preceding_nodes = nodes
            .iter()
            .copied()
            .filter(|node| {
                node.last_line() == comment.line
                    && node
                        .source_range()
                        .is_some_and(|range| range.end <= comment.range.start)
            })
            .collect::<Vec<_>>();
        if let Some(latest_end) = preceding_nodes
            .iter()
            .filter_map(|node| node.source_range().map(|range| range.end))
            .max()
        {
            let latest = preceding_nodes
                .into_iter()
                .filter(|node| {
                    node.source_range()
                        .is_some_and(|range| range.end == latest_end)
                })
                .collect::<Vec<_>>();
            let column = latest.iter().map(|node| node.column()).min()?;
            return latest
                .into_iter()
                .filter(|node| node.column() == column)
                .min_by_key(|node| node.source_length());
        }
        let containing = nodes
            .iter()
            .copied()
            .filter(|node| {
                node.source_range().is_some_and(|range| {
                    comment.range.start >= range.start && comment.range.end <= range.end
                })
            })
            .min_by_key(|node| node.source_length());
        // Parser associates comments in a protected rescue/ensure expression
        // with that structural node. Other enclosing constructs (classes,
        // modules, blocks) do not consume a leading comment that directly
        // precedes a child node.
        if containing.is_some_and(|node| {
            matches!(node.kind(), "ensure" | "rescue")
                || node.kind() == "kwbegin"
                    && node
                        .child_nodes()
                        .iter()
                        .any(|child| matches!(child.kind(), "ensure" | "rescue"))
        }) {
            return containing;
        }
        let mut last_comment_line = comment.line;
        while self.line_with_comment(last_comment_line + 1) {
            last_comment_line += 1;
        }
        let following = nodes
            .into_iter()
            .filter(|node| node.first_line() == last_comment_line + 1)
            .collect::<Vec<_>>();
        let column = following.iter().map(|node| node.column()).min()?;
        let following = following
            .into_iter()
            .filter(|node| node.column() == column)
            .min_by_key(|node| node.source_length());
        following.or(containing)
    }
    pub(crate) fn tokens(&self) -> &[SourceToken] {
        &self.tokens
    }
    pub(crate) fn each_token(&self) -> impl Iterator<Item = &SourceToken> {
        self.tokens.iter()
    }
    pub(crate) fn find_comment(
        &self,
        predicate: impl FnMut(&&SourceComment) -> bool,
    ) -> Option<&SourceComment> {
        self.comments.iter().find(predicate)
    }
    pub(crate) fn find_token(
        &self,
        predicate: impl FnMut(&&SourceToken) -> bool,
    ) -> Option<&SourceToken> {
        self.tokens.iter().find(predicate)
    }
    pub(crate) fn lines_range(&self, range: Range<usize>) -> &[String] {
        &self.lines[range]
    }
    pub(crate) fn lines_slice(&self, start: usize, length: usize) -> &[String] {
        &self.lines[start..self.lines.len().min(start.saturating_add(length))]
    }
    pub(crate) fn sorted_tokens(&self) -> Vec<&SourceToken> {
        let mut tokens: Vec<_> = self.tokens.iter().collect();
        tokens.sort_by_key(|token| token.begin_pos());
        tokens
    }
    pub(crate) fn checksum(&self) -> String {
        sha1_hex(self.raw_source.as_bytes())
    }
    pub(crate) fn comment_at_line(&self, line: usize) -> Option<&SourceComment> {
        self.comment_index
            .get(&line)
            .map(|index| &self.comments[*index])
    }
    pub(crate) fn line_with_comment(&self, line: usize) -> bool {
        self.comment_index.contains_key(&line)
    }
    pub(crate) fn each_comment_in_lines(&self, lines: Range<usize>) -> Vec<&SourceComment> {
        lines
            .filter_map(|line| self.comment_at_line(line))
            .collect()
    }
    pub(crate) fn contains_comment(&self, first_line: usize, last_line: usize) -> bool {
        (first_line..=last_line).any(|line| self.line_with_comment(line))
    }
    pub(crate) fn comments_before_line(&self, line: usize) -> Vec<&SourceComment> {
        (0..=line)
            .filter_map(|line| self.comment_at_line(line))
            .collect()
    }
    pub(crate) fn start_with(&self, string: &str) -> bool {
        self.lines
            .first()
            .is_some_and(|line| line.starts_with(string))
    }
    pub(crate) fn preceding_line(&self, token: &SourceToken) -> Option<&str> {
        token
            .line
            .checked_sub(2)
            .and_then(|index| self.lines.get(index))
            .map(String::as_str)
    }
    pub(crate) fn current_line(&self, token: &SourceToken) -> Option<&str> {
        token
            .line
            .checked_sub(1)
            .and_then(|index| self.lines.get(index))
            .map(String::as_str)
    }
    pub(crate) fn following_line(&self, token: &SourceToken) -> Option<&str> {
        self.lines.get(token.line).map(String::as_str)
    }
    pub(crate) fn line_indentation(&self, line: usize) -> usize {
        self.lines.get(line.saturating_sub(1)).map_or(0, |line| {
            line.chars()
                .take_while(|character| character.is_whitespace())
                .count()
        })
    }
    pub(crate) fn tokens_within(&self, range: Range<usize>) -> Vec<&SourceToken> {
        let sorted = self.sorted_tokens();
        let begin = sorted
            .iter()
            .position(|token| token.begin_pos() >= range.start)
            .unwrap_or(sorted.len());
        let end = sorted
            .iter()
            .position(|token| token.end_pos() >= range.end)
            .unwrap_or(sorted.len().saturating_sub(1));
        if begin > end || begin >= sorted.len() {
            Vec::new()
        } else {
            sorted[begin..=end].to_vec()
        }
    }
    pub(crate) fn first_token_index(&self, range: Range<usize>) -> Option<usize> {
        self.sorted_tokens()
            .iter()
            .position(|token| token.begin_pos() >= range.start)
    }
    pub(crate) fn last_token_index(&self, range: Range<usize>) -> Option<usize> {
        self.sorted_tokens()
            .iter()
            .position(|token| token.end_pos() >= range.end)
    }
    pub(crate) fn source_range(range: Range<usize>) -> Range<usize> {
        range
    }
    pub(crate) fn first_token_of(&self, range: Range<usize>) -> Option<&SourceToken> {
        self.tokens_within(range).first().copied()
    }
    pub(crate) fn last_token_of(&self, range: Range<usize>) -> Option<&SourceToken> {
        self.tokens_within(range).last().copied()
    }
}

pub(crate) fn default_parser_engine(ruby_version: f64) -> ParserEngine {
    if ruby_version >= 3.4 {
        ParserEngine::Prism
    } else {
        ParserEngine::Whitequark
    }
}

impl Index<usize> for ProcessedSource<'_> {
    type Output = str;
    fn index(&self, index: usize) -> &Self::Output {
        &self.lines[index]
    }
}

pub(crate) struct OwnedProcessedSource {
    source: String,
    ruby_version: f64,
    path: PathBuf,
    parser_engine: ParserEngine,
}
impl OwnedProcessedSource {
    pub(crate) fn from_file(
        path: impl Into<PathBuf>,
        ruby_version: f64,
        parser_engine: ParserEngine,
    ) -> io::Result<Self> {
        let path = path.into();
        Ok(Self {
            source: fs::read_to_string(&path)?,
            ruby_version,
            path,
            parser_engine,
        })
    }
    pub(crate) fn processed(&self) -> Result<ProcessedSource<'_>, ParserEngineError> {
        ProcessedSource::new(
            &self.source,
            self.ruby_version,
            Some(self.path.clone()),
            self.parser_engine,
        )
    }
}

fn normalize_parser_engine(
    engine: ParserEngine,
    ruby_version: f64,
) -> Result<ParserEngine, ParserEngineError> {
    let engine = match engine {
        ParserEngine::Default if ruby_version >= 3.4 => ParserEngine::Prism,
        ParserEngine::Default => ParserEngine::Whitequark,
        engine => engine,
    };
    let supported = match engine {
        ParserEngine::Whitequark => [
            1.9, 2.0, 2.1, 2.2, 2.3, 2.4, 2.5, 2.6, 2.7, 2.8, 3.0, 3.1, 3.2, 3.3, 3.4,
        ]
        .contains(&ruby_version),
        ParserEngine::Prism => [3.3, 3.4, 3.5, 4.0, 4.1].contains(&ruby_version),
        ParserEngine::Default => unreachable!(),
    };
    supported.then_some(engine).ok_or_else(|| {
        ParserEngineError(format!(
            "unsupported Ruby version {ruby_version} for {engine:?}"
        ))
    })
}
fn source_diagnostic(
    positions: &SourcePositionIndex,
    level: DiagnosticLevel,
    message: &str,
    range: Range<usize>,
) -> SourceDiagnostic {
    SourceDiagnostic {
        level,
        message: message.to_owned(),
        range: positions.character_range(range),
    }
}
fn source_lines(source: &str, data_start: Option<usize>) -> Vec<String> {
    if source.is_empty() {
        return Vec::new();
    }
    let source = data_start.map_or(source, |position| &source[..position]);
    let mut lines: Vec<_> = source
        .split('\n')
        .map(|line| line.strip_suffix('\r').unwrap_or(line).to_owned())
        .collect();
    if lines.last().is_some_and(|line| line == "__END__") {
        lines.pop();
    }
    lines
}

#[allow(clippy::too_many_lines)] // Token precedence intentionally follows RuboCop's lexer contract.
fn lex(
    source: &str,
    data_start: Option<usize>,
    positions: &SourcePositionIndex,
) -> Vec<SourceToken> {
    let limit = data_start.unwrap_or(source.len());
    let mut tokens = Vec::new();
    let mut byte = 0;
    while byte < limit {
        let character = source[byte..].chars().next().unwrap();
        let start = byte;
        if start == 0 && character == '\u{feff}' {
            byte += character.len_utf8();
            continue;
        }
        if character == ' ' || character == '\t' || character == '\r' {
            byte += character.len_utf8();
            continue;
        }
        if character == '#' {
            let end = source[byte..]
                .find('\n')
                .map_or(limit, |offset| byte + offset);
            push_token(source, positions, &mut tokens, "tCOMMENT", start, end);
            byte = end;
            continue;
        }
        if character == '\n' {
            byte += 1;
            push_token(source, positions, &mut tokens, "tNL", start, byte);
            continue;
        }
        if matches!(character, '\'' | '"' | '`') {
            byte = scan_quoted(source, byte, limit, character);
            push_token(
                source,
                positions,
                &mut tokens,
                if character == '`' {
                    "tXSTRING"
                } else {
                    "tSTRING"
                },
                start,
                byte,
            );
            continue;
        }
        if character.is_ascii_alphabetic() || character == '_' || !character.is_ascii() {
            byte += character.len_utf8();
            while byte < limit
                && source[byte..].chars().next().is_some_and(|c| {
                    c.is_alphanumeric() || c == '_' || !c.is_ascii() || matches!(c, '?' | '!')
                })
            {
                byte += source[byte..].chars().next().unwrap().len_utf8();
            }
            let text = &source[start..byte];
            let line_start = source[..start].rfind('\n').map_or(0, |index| index + 1);
            let has_prefix = !source[line_start..start].trim().is_empty();
            push_token(
                source,
                positions,
                &mut tokens,
                keyword_kind(text, has_prefix),
                start,
                byte,
            );
            continue;
        }
        if character.is_ascii_digit() {
            let prefixed = ["0x", "0X", "0b", "0B", "0o", "0O"]
                .iter()
                .any(|prefix| source[start..limit].starts_with(prefix));
            byte += 1;
            while byte < limit
                && (source.as_bytes()[byte].is_ascii_digit()
                    || source.as_bytes()[byte] == b'_'
                    || prefixed && source.as_bytes()[byte].is_ascii_alphabetic())
            {
                byte += 1;
            }
            if !prefixed
                && byte + 1 < limit
                && source.as_bytes()[byte] == b'.'
                && source.as_bytes()[byte + 1].is_ascii_digit()
            {
                byte += 1;
                while byte < limit
                    && (source.as_bytes()[byte].is_ascii_digit() || source.as_bytes()[byte] == b'_')
                {
                    byte += 1;
                }
            }
            push_token(
                source,
                positions,
                &mut tokens,
                if source[start..byte].contains('.') {
                    "tFLOAT"
                } else {
                    "tINTEGER"
                },
                start,
                byte,
            );
            continue;
        }

        let operator = [
            "...", "..", "&&=", "||=", "+=", "-=", "*=", "/=", "=>", "==", "!=", "<=", ">=", "&.",
            "::", "->", "<<", "**",
        ]
        .into_iter()
        .find(|candidate| source[start..limit].starts_with(candidate));
        if let Some(operator) = operator {
            byte += operator.len();
            let kind = match operator {
                "..." => "tDOT3",
                ".." => "tDOT2",
                "&&=" | "||=" | "+=" | "-=" | "*=" | "/=" => "tOP_ASGN",
                "&." => "tANDDOT",
                "::" => "tCOLON2",
                "->" => "tLAMBDA",
                "=>" => "tASSOC",
                "<<" => "tLSHFT",
                "**" => "tPOW",
                _ => "tOPERATOR",
            };
            push_token(source, positions, &mut tokens, kind, start, byte);
            continue;
        }

        byte += character.len_utf8();
        let previous = tokens
            .last()
            .filter(|token| !token.new_line() && !token.comment());
        let kind = match character {
            '(' => {
                if previous.is_some() {
                    "tLPAREN2"
                } else {
                    "tLPAREN"
                }
            }
            ')' => "tRPAREN",
            '[' => {
                if previous.is_some_and(|token| {
                    matches!(
                        token.kind,
                        "tIDENTIFIER" | "tCONSTANT" | "tRBRACK" | "tRPAREN"
                    )
                }) {
                    "tLBRACK2"
                } else {
                    "tLBRACK"
                }
            }
            ']' => "tRBRACK",
            '{' => {
                if previous.is_some_and(|token| token.kind == "tLAMBDA") {
                    "tLAMBEG"
                } else if previous.is_some_and(|token| {
                    matches!(
                        token.kind,
                        "tIDENTIFIER" | "tCONSTANT" | "tRPAREN" | "tRBRACK"
                    )
                }) {
                    "tLCURLY"
                } else {
                    "tLBRACE"
                }
            }
            '}' => "tRCURLY",
            ',' => "tCOMMA",
            ';' => "tSEMI",
            '.' => "tDOT",
            '=' => "tEQL",
            ':' => "tCOLON",
            '|' => "tPIPE",
            '*' => "tSTAR",
            '&' => "tAMPER",
            _ => "tCHAR",
        };
        push_token(source, positions, &mut tokens, kind, start, byte);
    }
    tokens
}

fn scan_quoted(source: &str, mut byte: usize, limit: usize, quote: char) -> usize {
    byte += quote.len_utf8();
    let mut escaped = false;
    while byte < limit {
        let character = source[byte..].chars().next().unwrap();
        byte += character.len_utf8();
        if escaped {
            escaped = false;
        } else if character == '\\' {
            escaped = true;
        } else if character == quote {
            break;
        }
    }
    byte
}

fn keyword_kind(text: &str, has_prefix: bool) -> &'static str {
    match text {
        "def" => "kDEF",
        "class" => "kCLASS",
        "module" => "kMODULE",
        "end" => "kEND",
        "if" => "kIF",
        "unless" => "kUNLESS",
        "while" => "kWHILE",
        "until" => "kUNTIL",
        "do" => "kDO",
        "then" => "kTHEN",
        "else" => "kELSE",
        "elsif" => "kELSIF",
        "ensure" => "kENSURE",
        "rescue" if has_prefix => "kRESCUE_MOD",
        "rescue" => "kRESCUE",
        "return" => "kRETURN",
        "yield" => "kYIELD",
        "super" => "kSUPER",
        "self" => "kSELF",
        "nil" => "kNIL",
        "true" => "kTRUE",
        "false" => "kFALSE",
        value if value.chars().next().is_some_and(char::is_uppercase) => "tCONSTANT",
        _ => "tIDENTIFIER",
    }
}
fn push_token(
    source: &str,
    positions: &SourcePositionIndex,
    tokens: &mut Vec<SourceToken>,
    kind: &'static str,
    start: usize,
    end: usize,
) {
    let range = positions.character_range(start..end);
    let line = positions.line_for_byte(start);
    let column = positions.column_for_byte(start);
    tokens.push(SourceToken {
        kind,
        text: source[start..end].to_owned(),
        range,
        line,
        column,
    });
}

pub(crate) fn sha1_hex(bytes: &[u8]) -> String {
    let mut h = [
        0x67452301u32,
        0xefcdab89,
        0x98badcfe,
        0x10325476,
        0xc3d2e1f0,
    ];
    let bit_len = (bytes.len() as u64) * 8;
    let mut data = bytes.to_vec();
    data.push(0x80);
    while data.len() % 64 != 56 {
        data.push(0);
    }
    data.extend_from_slice(&bit_len.to_be_bytes());
    for chunk in data.chunks(64) {
        let mut w = [0u32; 80];
        for (i, word) in w[..16].iter_mut().enumerate() {
            *word = u32::from_be_bytes(chunk[i * 4..i * 4 + 4].try_into().unwrap());
        }
        for i in 16..80 {
            w[i] = (w[i - 3] ^ w[i - 8] ^ w[i - 14] ^ w[i - 16]).rotate_left(1);
        }
        let (mut a, mut b, mut c, mut d, mut e) = (h[0], h[1], h[2], h[3], h[4]);
        for (i, word) in w.iter().enumerate() {
            let (f, k) = match i {
                0..=19 => ((b & c) | (!b & d), 0x5a827999),
                20..=39 => (b ^ c ^ d, 0x6ed9eba1),
                40..=59 => ((b & c) | (b & d) | (c & d), 0x8f1bbcdc),
                _ => (b ^ c ^ d, 0xca62c1d6),
            };
            let temp = a
                .rotate_left(5)
                .wrapping_add(f)
                .wrapping_add(e)
                .wrapping_add(k)
                .wrapping_add(*word);
            e = d;
            d = c;
            c = b.rotate_left(30);
            b = a;
            a = temp;
        }
        h[0] = h[0].wrapping_add(a);
        h[1] = h[1].wrapping_add(b);
        h[2] = h[2].wrapping_add(c);
        h[3] = h[3].wrapping_add(d);
        h[4] = h[4].wrapping_add(e);
    }
    h.iter().map(|word| format!("{word:08x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sorted_tokens_use_prism_as_the_authority_for_comments() {
        // The deliberately small lexer can mistake the apostrophe in this
        // regexp for the start of a quoted string. A later real comment must
        // still be present, while the `#` inside the regexp must not be.
        let source = "/don't#comment/\ncall# real comment\n";
        let processed = ProcessedSource::new(source, 3.3, None, ParserEngine::Prism).unwrap();
        let comments: Vec<_> = processed
            .sorted_tokens()
            .into_iter()
            .filter(|token| token.comment())
            .map(|token| token.text.as_str())
            .collect();

        assert_eq!(comments, ["# real comment"]);
    }
}
