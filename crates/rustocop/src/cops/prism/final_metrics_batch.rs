use super::*;
use crate::rubocop::ast::node::core::NodeRef as RubocopNodeRef;
use crate::rubocop::ast::prism::convert as convert_rubocop_ast;
use std::collections::HashSet;

pub(super) fn cops() -> Vec<Box<dyn Cop>> {
    vec![
        Box::new(BlockNesting),
        Box::new(PerceivedComplexity),
        Box::new(ClassLength),
        Box::new(CyclomaticComplexity),
    ]
}

struct BlockNesting;
impl Cop for BlockNesting {
    fn name(&self) -> &'static str { "Metrics/BlockNesting" }
    fn phase(&self) -> CopPhase { CopPhase::Source }
    fn on_source(&self, source: &str, context: &mut Context) {
        if source.trim().is_empty() { return; }
        let parsed = ruby_prism::parse(source.as_bytes());
        let (ast, root) = convert_rubocop_ast(source, &parsed.node());
        let Some(root) = root.map(|root| ast.node(root)) else { return };
        let mut cop = context.cop_context(self.name(), source, &[]);
        let max = cop.config_usize("Max", 3);
        let count_blocks = cop.config_bool("CountBlocks", false);
        let count_modifier_forms = cop.config_bool("CountModifierForms", false);
        block_nesting_check(
            root,
            source,
            max,
            count_blocks,
            count_modifier_forms,
            0,
            false,
            &mut cop,
        );
    }
}

#[allow(clippy::too_many_arguments)]
fn block_nesting_check(
    node: RubocopNodeRef<'_>,
    source: &str,
    max: usize,
    count_blocks: bool,
    count_modifier_forms: bool,
    current_level: usize,
    ignored: bool,
    context: &mut CopContext<'_, '_>,
) {
    let considered = block_nesting_considered(node, count_blocks);
    let counted = considered && block_nesting_counted(node, count_modifier_forms);
    let level = current_level + usize::from(counted);
    let mut ignored = ignored;
    if considered && level > max && !ignored {
        if let Some(range) = node.source_range() {
            context.report(
                format!("Avoid more than {max} levels of block nesting."),
                block_nesting_character_range_to_byte(source, range),
            );
            ignored = true;
        }
    }
    for child in node.child_nodes() {
        block_nesting_check(
            child,
            source,
            max,
            count_blocks,
            count_modifier_forms,
            level,
            ignored,
            context,
        );
    }
}

fn block_nesting_considered(node: RubocopNodeRef<'_>, count_blocks: bool) -> bool {
    matches!(
        node.kind(),
        "case" | "case_match" | "if" | "while" | "while_post" | "until" | "until_post"
            | "for" | "resbody"
    ) || count_blocks && matches!(node.kind(), "block" | "numblock" | "itblock")
}

fn block_nesting_counted(node: RubocopNodeRef<'_>, count_modifier_forms: bool) -> bool {
    if node.kind() != "if" {
        return true;
    }
    if node.elsif() {
        return false;
    }
    if node.modifier_form() {
        return count_modifier_forms;
    }
    true
}

fn block_nesting_character_range_to_byte(
    source: &str,
    range: std::ops::Range<usize>,
) -> std::ops::Range<usize> {
    let start = source
        .char_indices()
        .nth(range.start)
        .map_or(source.len(), |(byte, _)| byte);
    let end = source
        .char_indices()
        .nth(range.end)
        .map_or(source.len(), |(byte, _)| byte);
    start..end
}

struct CyclomaticComplexity;
struct PerceivedComplexity;
impl Cop for CyclomaticComplexity { fn name(&self) -> &'static str { "Metrics/CyclomaticComplexity" } fn on_node<'pr>(&self,node:&Node<'pr>,ancestors:&[Node<'pr>],source:&str,context:&mut Context){ complexity_node(self.name(),node,ancestors,source,context,false); } }
impl Cop for PerceivedComplexity { fn name(&self) -> &'static str { "Metrics/PerceivedComplexity" } fn on_node<'pr>(&self,node:&Node<'pr>,ancestors:&[Node<'pr>],source:&str,context:&mut Context){ complexity_node(self.name(),node,ancestors,source,context,true); } }

fn complexity_node(name: &'static str, node: &Node<'_>, ancestors: &[Node<'_>], source: &str, context: &mut Context, perceived: bool) {
    let (method, body, offense) = if let Some(definition) = node.as_def_node() {
        (String::from_utf8_lossy(definition.name().as_slice()).into_owned(), definition.body(), definition.location())
    } else if let Some(block) = node.as_block_node() {
        let Some(call) = ancestors.iter().rev().find_map(Node::as_call_node).filter(|call| call_name(call) == b"define_method") else { return };
        // MethodComplexity's NodePattern accepts only literal `sym`/`str`
        // names. Dynamic strings and other expressions are intentionally not
        // inspected by RuboCop.
        let Some(argument) = first_argument(&call).filter(|argument| {
            argument.as_symbol_node().is_some() || argument.as_string_node().is_some()
        }) else { return };
        let method = node_source(source, &argument)
            .trim_start_matches(':')
            .trim_matches(['\'', '"'])
            .to_string();
        (method, block.body(), call.location())
    } else { return };
    let mut cop = context.cop_context(name, source, ancestors);
    if cop.policy().allows_method(method.as_bytes()) { return; }
    let mut counter = ComplexityCounter { score: 1, perceived, safe_navigation: HashSet::new() };
    if let Some(body) = body { counter.visit(&body); }
    let max = cop.config_usize("Max", if perceived { 8 } else { 7 });
    if counter.score > max {
        let metric = if perceived { "Perceived complexity" } else { "Cyclomatic complexity" };
        cop.report(format!("{metric} for `{method}` is too high. [{}/{max}]", counter.score), offense);
    }
}

struct ComplexityCounter { score: usize, perceived: bool, safe_navigation: HashSet<String> }
impl<'pr> Visit<'pr> for ComplexityCounter {
    fn visit_if_node(&mut self, node: &ruby_prism::IfNode<'pr>) { self.score += if self.perceived&&node.if_keyword_loc().is_some_and(|keyword|keyword.as_slice()!=b"elsif")&&node.subsequent().is_some(){2}else{1}; ruby_prism::visit_if_node(self,node); }
    fn visit_unless_node(&mut self, node: &ruby_prism::UnlessNode<'pr>) { self.score += if self.perceived&&node.else_clause().is_some(){2}else{1}; ruby_prism::visit_unless_node(self,node); }
    fn visit_while_node(&mut self,node:&ruby_prism::WhileNode<'pr>){if !node.is_begin_modifier(){self.score+=1}ruby_prism::visit_while_node(self,node)}
    fn visit_until_node(&mut self,node:&ruby_prism::UntilNode<'pr>){if !node.is_begin_modifier(){self.score+=1}ruby_prism::visit_until_node(self,node)}
    fn visit_for_node(&mut self,node:&ruby_prism::ForNode<'pr>){self.score+=1;ruby_prism::visit_for_node(self,node)}
    fn visit_rescue_node(&mut self,node:&ruby_prism::RescueNode<'pr>){self.visit_rescue_clause(node,true)}
    fn visit_rescue_modifier_node(&mut self,node:&ruby_prism::RescueModifierNode<'pr>){self.score+=1;ruby_prism::visit_rescue_modifier_node(self,node)}
    fn visit_case_node(&mut self,node:&ruby_prism::CaseNode<'pr>){if self.perceived{let branches=node.conditions().len()+usize::from(node.else_clause().is_some());self.score+=if node.predicate().is_none(){branches}else{((branches as f64*0.2)+0.8).round() as usize};}ruby_prism::visit_case_node(self,node)}
    fn visit_when_node(&mut self,node:&ruby_prism::WhenNode<'pr>){if !self.perceived{self.score+=1;}ruby_prism::visit_when_node(self,node)}
    fn visit_in_node(&mut self,node:&ruby_prism::InNode<'pr>){
        self.score+=1;
        let pattern=node.pattern();
        // Prism wraps an `in pattern if guard` pattern in an IfNode, while
        // rubocop-ast stores the guard inside the single in_pattern node. The
        // guard's descendants still count, but the wrapper itself does not.
        if let Some(guard)=pattern.as_if_node(){
            self.visit(&guard.predicate());
            if let Some(statements)=guard.statements(){self.visit_statements_node(&statements)}
        }else if let Some(guard)=pattern.as_unless_node(){
            self.visit(&guard.predicate());
            if let Some(statements)=guard.statements(){self.visit_statements_node(&statements)}
        }else{
            self.visit(&pattern);
        }
        if let Some(statements)=node.statements(){self.visit_statements_node(&statements)}
    }
    fn visit_else_node(&mut self,node:&ruby_prism::ElseNode<'pr>){ruby_prism::visit_else_node(self,node)}
    fn visit_and_node(&mut self,node:&ruby_prism::AndNode<'pr>){self.score+=1;ruby_prism::visit_and_node(self,node)}
    fn visit_or_node(&mut self,node:&ruby_prism::OrNode<'pr>){self.score+=1;ruby_prism::visit_or_node(self,node)}
    fn visit_local_variable_and_write_node(&mut self,node:&ruby_prism::LocalVariableAndWriteNode<'pr>){self.score+=1;ruby_prism::visit_local_variable_and_write_node(self,node)}
    fn visit_local_variable_or_write_node(&mut self,node:&ruby_prism::LocalVariableOrWriteNode<'pr>){self.score+=1;ruby_prism::visit_local_variable_or_write_node(self,node)}
    fn visit_call_and_write_node(&mut self,node:&ruby_prism::CallAndWriteNode<'pr>){self.score+=1;ruby_prism::visit_call_and_write_node(self,node)}
    fn visit_call_or_write_node(&mut self,node:&ruby_prism::CallOrWriteNode<'pr>){self.score+=1;ruby_prism::visit_call_or_write_node(self,node)}
    fn visit_index_and_write_node(&mut self,node:&ruby_prism::IndexAndWriteNode<'pr>){self.score+=1;ruby_prism::visit_index_and_write_node(self,node)}
    fn visit_index_or_write_node(&mut self,node:&ruby_prism::IndexOrWriteNode<'pr>){self.score+=1;ruby_prism::visit_index_or_write_node(self,node)}
    fn visit_class_variable_and_write_node(&mut self,node:&ruby_prism::ClassVariableAndWriteNode<'pr>){self.score+=1;ruby_prism::visit_class_variable_and_write_node(self,node)}
    fn visit_class_variable_or_write_node(&mut self,node:&ruby_prism::ClassVariableOrWriteNode<'pr>){self.score+=1;ruby_prism::visit_class_variable_or_write_node(self,node)}
    fn visit_global_variable_and_write_node(&mut self,node:&ruby_prism::GlobalVariableAndWriteNode<'pr>){self.score+=1;ruby_prism::visit_global_variable_and_write_node(self,node)}
    fn visit_global_variable_or_write_node(&mut self,node:&ruby_prism::GlobalVariableOrWriteNode<'pr>){self.score+=1;ruby_prism::visit_global_variable_or_write_node(self,node)}
    fn visit_instance_variable_and_write_node(&mut self,node:&ruby_prism::InstanceVariableAndWriteNode<'pr>){self.score+=1;ruby_prism::visit_instance_variable_and_write_node(self,node)}
    fn visit_instance_variable_or_write_node(&mut self,node:&ruby_prism::InstanceVariableOrWriteNode<'pr>){self.score+=1;ruby_prism::visit_instance_variable_or_write_node(self,node)}
    fn visit_constant_and_write_node(&mut self,node:&ruby_prism::ConstantAndWriteNode<'pr>){self.score+=1;ruby_prism::visit_constant_and_write_node(self,node)}
    fn visit_constant_or_write_node(&mut self,node:&ruby_prism::ConstantOrWriteNode<'pr>){self.score+=1;ruby_prism::visit_constant_or_write_node(self,node)}
    fn visit_constant_path_and_write_node(&mut self,node:&ruby_prism::ConstantPathAndWriteNode<'pr>){self.score+=1;ruby_prism::visit_constant_path_and_write_node(self,node)}
    fn visit_constant_path_or_write_node(&mut self,node:&ruby_prism::ConstantPathOrWriteNode<'pr>){self.score+=1;ruby_prism::visit_constant_path_or_write_node(self,node)}
    fn visit_local_variable_write_node(&mut self,node:&ruby_prism::LocalVariableWriteNode<'pr>){self.safe_navigation.remove(&String::from_utf8_lossy(node.name().as_slice()).into_owned());ruby_prism::visit_local_variable_write_node(self,node)}
    fn visit_call_node(&mut self,node:&CallNode<'pr>){
        if node.call_operator_loc().is_some_and(|location| location.as_slice()==b"&.") {
            let local=node.receiver().and_then(|value|value.as_local_variable_read_node());
            if let Some(local) = local {let receiver=String::from_utf8_lossy(local.name().as_slice()).into_owned();if self.safe_navigation.insert(receiver){self.score+=1;}} else {self.score+=1;}
        }
        if node.block().is_some_and(|block| rubocop_counted_block(&block))
            && iterating_method(node.name().as_slice())
        {
            self.score+=1;
        }
        ruby_prism::visit_call_node(self,node)
    }
}

// rubocop-ast deliberately represents Ruby 3 numbered-parameter and `it`
// blocks as `numblock`/`itblock`. Neither node kind is in the metrics cops'
// COUNTED_NODES list; a block pass (`&:name`) and an ordinary block are.
fn rubocop_counted_block(node: &Node<'_>) -> bool {
    let Some(block) = node.as_block_node() else {
        return node.as_block_argument_node().is_some();
    };
    block.parameters().is_none_or(|parameters| {
        parameters.as_numbered_parameters_node().is_none()
            && parameters.as_it_parameters_node().is_none()
    })
}

impl ComplexityCounter {
    fn visit_rescue_clause<'pr>(&mut self,node:&ruby_prism::RescueNode<'pr>,count:bool){
        if count{self.score+=1}
        for exception in &node.exceptions(){self.visit(&exception)}
        if let Some(reference)=node.reference(){self.visit(&reference)}
        if let Some(statements)=node.statements(){self.visit_statements_node(&statements)}
        if let Some(subsequent)=node.subsequent(){self.visit_rescue_clause(&subsequent,false)}
    }
}

fn iterating_method(name:&[u8])->bool{matches!(name,
    b"all?"|b"any?"|b"chain"|b"chunk"|b"chunk_while"|b"collect"|b"collect_concat"|b"count"|b"cycle"|
    b"detect"|b"drop"|b"drop_while"|b"each"|b"each_cons"|b"each_entry"|b"each_slice"|b"each_with_index"|
    b"each_with_object"|b"entries"|b"filter"|b"filter_map"|b"find"|b"find_all"|b"find_index"|b"flat_map"|
    b"grep"|b"grep_v"|b"group_by"|b"inject"|b"lazy"|b"map"|b"max"|b"max_by"|b"min"|b"min_by"|b"minmax"|
    b"minmax_by"|b"none?"|b"one?"|b"partition"|b"reduce"|b"reject"|b"reverse_each"|b"select"|b"slice_after"|
    b"slice_before"|b"slice_when"|b"sort"|b"sort_by"|b"sum"|b"take"|b"take_while"|b"tally"|b"to_h"|b"uniq"|
    b"zip"|b"with_index"|b"with_object"|b"bsearch"|b"bsearch_index"|b"collect!"|b"combination"|b"d_permutation"|
    b"delete_if"|b"each_index"|b"keep_if"|b"map!"|b"permutation"|b"product"|b"reject!"|b"repeat"|
    b"repeated_combination"|b"select!"|b"sort!"|b"each_key"|b"each_pair"|b"each_value"|b"fetch"|
    b"fetch_values"|b"has_key?"|b"merge"|b"merge!"|b"transform_keys"|b"transform_keys!"|
    b"transform_values"|b"transform_values!")}

struct ClassLength;
impl Cop for ClassLength {
    fn name(&self)->&'static str{"Metrics/ClassLength"}
    fn on_node<'pr>(&self,node:&Node<'pr>,ancestors:&[Node<'pr>],source:&str,context:&mut Context){
        let (body,offense,classlike)=if let Some(class)=node.as_class_node(){(class.body(),class.location(),true)}else if let Some(singleton)=node.as_singleton_class_node(){if ancestors.iter().any(|ancestor|ancestor.as_class_node().is_some()){return}(singleton.body(),singleton.location(),false)}else if let Some(block)=node.as_block_node(){
            let Some(call)=ancestors.iter().rev().find_map(Node::as_call_node).filter(|call| call_name(call)==b"new"&&call.receiver().is_some_and(|receiver| matches!(root_constant_name(&receiver).as_deref(),Some(b"Class"|b"Struct"))))else{return};
            if ancestors.iter().any(|ancestor|ancestor.as_local_variable_write_node().is_some()){return}
            (block.body(),call.location(),false)
        }else{return};
        let mut cop=context.cop_context(self.name(),source,ancestors);let count_comments=cop.config_bool("CountComments",false);
        if classlike && body.as_ref().is_some_and(namespace_class_body) { return; }
        let body_source=body.as_ref().map_or_else(||{let full=cop.source_file().at(&offense);let mut lines=full.lines();lines.next();let mut rest=lines.collect::<Vec<_>>();rest.pop();rest.join("\n")},|body|cop.source_file().node(body).to_string());
        let mut count=if classlike { classlike_code_lines(source,&offense,body.as_ref(),count_comments) } else { metric_code_lines(&body_source,count_comments) };
        if cop.config_values("CountAsOne").iter().any(|value|value=="array"){count=count.saturating_sub(metric_folded_lines(&body_source,'[',']'));}
        let max=cop.config_usize("Max",100);if count>max{cop.report(format!("Class has too many lines. [{count}/{max}]"),offense);}
    }
}

fn root_constant_name(node:&Node<'_>)->Option<Vec<u8>>{node.as_constant_read_node().map(|constant|constant.name().as_slice().to_vec()).or_else(||node.as_constant_path_node().and_then(|path|path.name().map(|name|name.as_slice().to_vec())))}
fn metric_code_lines(source:&str,count_comments:bool)->usize{source.lines().filter(|line|{let line=line.trim();!line.is_empty()&&(count_comments||!line.starts_with('#'))}).count()}
fn metric_folded_lines(source:&str,open:char,close:char)->usize{source.find(open).and_then(|start|source[start..].find(close).map(|end|source[start..start+end].matches('\n').count())).unwrap_or(0)}
fn namespace_class_body(body:&Node<'_>)->bool{body.as_statements_node().and_then(|statements|(statements.body().len()==1).then(||statements.body().first()).flatten()).is_some_and(|only|only.as_class_node().is_some()||only.as_module_node().is_some())}
fn classlike_code_lines(source:&str,location:&ruby_prism::Location<'_>,body:Option<&Node<'_>>,count_comments:bool)->usize{
    let first=source[..location.start_offset()].bytes().filter(|byte|*byte==b'\n').count();
    let last=source[..location.end_offset()].bytes().filter(|byte|*byte==b'\n').count();
    let nested=body.map(nested_class_ranges).unwrap_or_default().into_iter().map(|range|{
        let start=source[..range.start].bytes().filter(|byte|*byte==b'\n').count();
        let end=source[..range.end].bytes().filter(|byte|*byte==b'\n').count();
        start.saturating_add(1)..=end.saturating_add(1)
    }).collect::<Vec<_>>();
    source.lines().enumerate().filter(|(index,line)|{
        // RuboCop's class-like calculator feeds one-based AST line numbers
        // directly to ProcessedSource's zero-based indexer. Preserve that
        // observable offset (including its closing-line behavior) exactly.
        *index>first.saturating_add(1)&&*index<=last&&!nested.iter().any(|range|range.contains(index))&&{
            let line=line.trim();!line.is_empty()&&(count_comments||!line.starts_with('#'))
        }
    }).count()
}
fn nested_class_ranges(body:&Node<'_>)->Vec<std::ops::Range<usize>>{struct Collector(Vec<std::ops::Range<usize>>);impl<'pr> Visit<'pr> for Collector{fn visit_class_node(&mut self,node:&ruby_prism::ClassNode<'pr>){let location=node.location();self.0.push(location.start_offset()..location.end_offset());}fn visit_module_node(&mut self,node:&ruby_prism::ModuleNode<'pr>){let location=node.location();self.0.push(location.start_offset()..location.end_offset());}}let mut collector=Collector(Vec::new());collector.visit(body);collector.0}
