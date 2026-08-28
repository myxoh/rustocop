use super::super::catalog_cop::compatibility_custom;
use super::*;

pub(super) fn cops() -> Vec<Box<dyn Cop>> {
    vec![
        Box::new(EmptyLineAfterGuardClause),
        Box::new(LineEndStringConcatenationIndentation),
        Box::new(SpaceInsideBlockBraces),
        compatibility_custom(
            "Layout/SpaceInsideHashLiteralBraces",
            space_inside_hash_literal_braces,
        ),
        Box::new(LineContinuationLeadingSpace),
    ]
}

#[derive(Clone)]
struct ContinuedString {
    start: usize,
    end: usize,
    opening_end: usize,
    closing_start: usize,
    depth: usize,
}

fn continued_strings(source: &str) -> Vec<Vec<ContinuedString>> {
    struct Collector { strings: Vec<ContinuedString>, depth: usize }
    impl<'pr> Visit<'pr> for Collector {
        fn visit_string_node(&mut self,node:&ruby_prism::StringNode<'pr>){self.strings.push(ContinuedString{start:node.location().start_offset(),end:node.location().end_offset(),opening_end:node.opening_loc().map_or(node.content_loc().start_offset(),|location|location.end_offset()),closing_start:node.closing_loc().map_or(node.content_loc().end_offset(),|location|location.start_offset()),depth:self.depth});}
        fn visit_interpolated_string_node(&mut self,node:&ruby_prism::InterpolatedStringNode<'pr>){if node.opening_loc().is_none(){for part in node.parts().iter(){self.visit(&part);}return}self.strings.push(ContinuedString{start:node.location().start_offset(),end:node.location().end_offset(),opening_end:node.opening_loc().map_or(node.location().start_offset(),|location|location.end_offset()),closing_start:node.closing_loc().map_or(node.location().end_offset(),|location|location.start_offset()),depth:self.depth});self.depth+=1;for part in node.parts().iter(){if part.as_string_node().is_none(){self.visit(&part);}}self.depth-=1;}
    }
    let parsed=parse(source.as_bytes());let mut collector=Collector{strings:Vec::new(),depth:0};collector.visit(&parsed.node());collector.strings.retain(|string|string.start<=string.opening_end&&string.opening_end<=string.closing_start&&string.closing_start<=string.end&&string.end<=source.len());collector.strings.sort_by_key(|string|(string.depth,string.start));
    let mut groups=Vec::new();let mut group=Vec::new();
    for string in collector.strings {
        let joined=group.last().is_some_and(|previous:&ContinuedString|previous.depth==string.depth&&previous.end<=string.start&&{let gap=&source[previous.end..string.start];gap.contains("\\\n")&&gap.bytes().all(|byte|matches!(byte,b' '|b'\t'|b'\r'|b'\n'|b'\\'))});
        if !joined&&!group.is_empty(){groups.push(std::mem::take(&mut group));}group.push(string);
    }
    if group.len()>1{groups.push(group);}groups.into_iter().filter(|group|group.len()>1).collect()
}

struct LineContinuationLeadingSpace;
impl Cop for LineContinuationLeadingSpace {
    fn name(&self)->&'static str{"Layout/LineContinuationLeadingSpace"}
    fn phase(&self)->CopPhase{CopPhase::Source}
    fn on_source(&self,source:&str,context:&mut Context){
        let mut cop=context.cop_context(self.name(),source,&[]);let trailing=cop.policy().enforced_style("trailing")=="trailing";
        for group in continued_strings(source){for pair in group.windows(2){let previous=&pair[0];let next=&pair[1];
            if trailing {let spaces=source[next.opening_end..next.closing_start].bytes().take_while(|byte|*byte==b' ').count();if spaces==0{continue}let offense=next.opening_end..next.opening_end+spaces;cop.add_offense(offense.clone(),"Move leading spaces to the end of the previous line.",|corrector|{corrector.remove(offense);corrector.replace(previous.closing_start..previous.closing_start," ".repeat(spaces));});}
            else {let spaces=source[previous.opening_end..previous.closing_start].bytes().rev().take_while(|byte|*byte==b' ').count();if spaces==0{continue}let offense=previous.closing_start-spaces..previous.closing_start;cop.add_offense(offense.clone(),"Move trailing spaces to the start of the next line.",|corrector|{corrector.remove(offense);corrector.replace(next.opening_end..next.opening_end," ".repeat(spaces));});}
        }}
    }
}

struct LineEndStringConcatenationIndentation;
impl Cop for LineEndStringConcatenationIndentation {
    fn name(&self)->&'static str{"Layout/LineEndStringConcatenationIndentation"}
    fn phase(&self)->CopPhase{CopPhase::Source}
    fn on_source(&self,source:&str,context:&mut Context){
        let mut cop=context.cop_context(self.name(),source,&[]);let style=cop.policy().enforced_style("aligned").to_string();let related=cop.related_config_value("Layout/IndentationWidth","Width").and_then(|value|value.parse().ok()).unwrap_or(2);let width=cop.config_usize("IndentationWidth",related);
        let parsed = ruby_prism::parse(source.as_bytes());
        let (ast, root) = crate::rubocop::ast::prism::convert(source, &parsed.node(), None);
        let mut dstr_rules = std::collections::HashMap::new();
        if let Some(root) = root.map(|root| ast.node(root)) {
            for dstr in root.each_node(&["dstr"]) {
                let children = dstr.child_nodes();
                if !dstr.multiline()
                    || children.is_empty()
                    || children.iter().any(|child| !matches!(child.kind(), "str" | "dstr") || child.multiline())
                {
                    continue;
                }
                let Some(first_range) = children[0].source_range() else { continue };
                let first_start = character_offset_to_byte(source, first_range.start);
                let always_indented = dstr.parent().is_none_or(|parent| {
                    matches!(parent.kind(), "block" | "begin" | "def" | "defs" | "if")
                });
                let source_line = cop.source_file().line(first_start);
                let base_column = if dstr.parent().is_some_and(|parent| parent.kind() == "pair") {
                    dstr.parent().map_or(0, |parent| parent.column())
                } else {
                    source_line.find(|character: char| !character.is_whitespace()).unwrap_or(0)
                };
                dstr_rules.insert(first_start, (always_indented, base_column));
            }
        }
        let mut findings=Vec::<(&'static str,std::ops::Range<usize>,std::ops::Range<usize>,String)>::new();
        let mut correction_edits=Vec::<(std::ops::Range<usize>,String)>::new();
        for group in continued_strings(source){let first=&group[0];let line_start=cop.source_file().line_start(first.start);let first_column=first.start-line_start;let Some((always_indented,base_column))=dstr_rules.get(&first.start).copied() else { continue };let indented=style!="aligned"||always_indented;let expected=if indented{base_column+width}else{first_column};let before=findings.len();
            for (index,string) in group.iter().enumerate().skip(1){let current_start=cop.source_file().line_start(string.start);let actual=string.start-current_start;let pair_expected=if index==1{expected}else{let previous=&group[index-1];previous.start-cop.source_file().line_start(previous.start)};if actual==pair_expected{continue}let message=if indented&&index==1{"Indent the first part of a string concatenated with backslash."}else{"Align parts of a string concatenated with backslash."};findings.push((message,string.start..string.end,current_start..string.start," ".repeat(pair_expected)));}
            if findings.len()>before{for (index,string) in group.iter().enumerate().skip(1){let current_start=cop.source_file().line_start(string.start);let target=if index==1{expected}else{let previous=&group[index-1];previous.start-cop.source_file().line_start(previous.start)};if string.start-current_start!=target{correction_edits.push((current_start..string.start," ".repeat(target)));}}}
        }
        for (message,offense,_,_) in findings{let edits=correction_edits.clone();cop.add_offense(offense,message,|corrector|{for (range,replacement) in edits{corrector.replace(range,replacement);}});}
    }
}

fn character_offset_to_byte(source: &str, character: usize) -> usize {
    source.char_indices().nth(character).map_or(source.len(), |(byte, _)| byte)
}

fn space_inside_hash_literal_braces(context: &mut CompatibilityCopContext<'_, '_, '_>) {
    #[derive(Default)]
    struct HashBraces(Vec<(usize, usize)>);

    impl<'pr> Visit<'pr> for HashBraces {
        fn visit_hash_node(&mut self, node: &ruby_prism::HashNode<'pr>) {
            self.0.push((
                node.opening_loc().start_offset(),
                node.closing_loc().start_offset(),
            ));
            ruby_prism::visit_hash_node(self, node);
        }

        fn visit_hash_pattern_node(&mut self, node: &ruby_prism::HashPatternNode<'pr>) {
            if let (Some(opening), Some(closing)) = (node.opening_loc(), node.closing_loc()) {
                if opening.as_slice() == b"{" {
                    self.0
                        .push((opening.start_offset(), closing.start_offset()));
                }
            }
            ruby_prism::visit_hash_pattern_node(self, node);
        }
    }

    let mut braces = HashBraces::default();
    braces.visit(&context.prism_result().node());
    for (opening, closing) in braces.0 {
        enforce_hash_brace_spacing(context, opening, closing);
    }
}

fn enforce_hash_brace_spacing(
    context: &mut CompatibilityCopContext<'_, '_, '_>,
    opening: usize,
    closing: usize,
) {
    let source = context.source();
    if opening >= closing || closing >= source.len() {
        return;
    }
    let inside = &source[opening + 1..closing];
    if inside.trim().is_empty() {
        let style = context
            .config_value("EnforcedStyleForEmptyBraces")
            .unwrap_or("no_space");
        if style == "no_space" && !inside.is_empty() {
            context.remove(
                "Space inside empty hash literal braces detected.",
                opening + 1..closing,
                opening + 1..closing,
            );
        } else if style == "space" && inside.is_empty() {
            context.insert(
                "Space inside empty hash literal braces missing.",
                opening..opening + 1,
                opening + 1,
                " ",
            );
        }
        return;
    }

    let style = context.policy().enforced_style("space");
    let left_end = opening
        + 1
        + inside
            .bytes()
            .take_while(|byte| matches!(byte, b' ' | b'\t'))
            .count();
    let right_start = closing
        - inside
            .bytes()
            .rev()
            .take_while(|byte| matches!(byte, b' ' | b'\t'))
            .count();
    let left_newline = source[opening + 1..left_end].contains('\n')
        || source.as_bytes().get(opening + 1) == Some(&b'\n');
    let right_newline = source[right_start..closing].contains('\n')
        || source.as_bytes().get(closing.wrapping_sub(1)) == Some(&b'\n');
    let compact_left = style == "compact" && source.as_bytes().get(left_end) == Some(&b'{');
    let compact_right = style == "compact"
        && right_start > 0
        && source.as_bytes().get(right_start - 1) == Some(&b'}');
    let want_left = style != "no_space" && !compact_left;
    let want_right = style != "no_space" && !compact_right;

    if !left_newline && source.as_bytes().get(left_end) != Some(&b'#') {
        report_hash_side(context, opening, left_end, want_left, true);
    }
    if !right_newline {
        report_hash_side(context, right_start, closing, want_right, false);
    }
}

fn report_hash_side(
    context: &mut CompatibilityCopContext<'_, '_, '_>,
    whitespace_start: usize,
    brace: usize,
    want_space: bool,
    opening: bool,
) {
    let has_space = if opening {
        brace > whitespace_start + 1
    } else {
        brace > whitespace_start
    };
    let symbol = if opening { "{" } else { "}" };
    if want_space && !has_space {
        let offense = if opening {
            whitespace_start..whitespace_start + 1
        } else {
            brace..brace + 1
        };
        context.insert(
            format!("Space inside {symbol} missing."),
            offense,
            if opening { whitespace_start + 1 } else { brace },
            " ",
        );
    } else if !want_space && has_space {
        let range = if opening {
            whitespace_start + 1..brace
        } else {
            whitespace_start..brace
        };
        context.remove(
            format!("Space inside {symbol} detected."),
            range.clone(),
            range,
        );
    }
}
