use super::*;
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
    fn on_node<'pr>(&self, node: &Node<'pr>, ancestors: &[Node<'pr>], source: &str, context: &mut Context) {
        let mut cop = context.cop_context(self.name(), source, ancestors);
        let Some(mut offense) = nesting_offense(node, &cop) else { return };
        if node.as_block_node().is_some() {
            if let Some(call)=ancestors.iter().rev().find_map(Node::as_call_node){offense=call.location().start_offset()..call.location().end_offset();}
        }
        let depth = ancestors.iter().filter(|ancestor| nesting_offense(ancestor, &cop).is_some()).count() + 1;
        let max = cop.config_usize("Max", 3);
        if depth == max + 1 { cop.report(format!("Avoid more than {max} levels of block nesting."), offense); }
    }
}

fn nesting_offense(node: &Node<'_>, context: &CopContext<'_, '_>) -> Option<std::ops::Range<usize>> {
    if let Some(value) = node.as_if_node() {
        value.if_keyword_loc()?;
        let keyword=value.if_keyword_loc().unwrap();if keyword.as_slice()==b"elsif"{return None;}let modifier=value.end_keyword_loc().is_none();
        if modifier&&!context.config_bool("CountModifierForms",false){return None;}
        return Some(value.location().start_offset()..value.location().end_offset());
    }
    if let Some(value) = node.as_unless_node() {
        let modifier=value.end_keyword_loc().is_none();if modifier&&!context.config_bool("CountModifierForms",false){return None;}
        return Some(value.location().start_offset()..value.location().end_offset());
    }
    macro_rules! location { ($cast:ident, $loc:ident) => { if let Some(value) = node.$cast() { let location = value.$loc(); return Some(location.start_offset()..location.end_offset()); } }; }
    if let Some(value)=node.as_case_node(){return Some(value.location().start_offset()..value.location().end_offset())}
    if let Some(value)=node.as_case_match_node(){return Some(value.location().start_offset()..value.location().end_offset())}
    if let Some(value)=node.as_while_node(){return Some(value.location().start_offset()..value.location().end_offset())}
    if let Some(value)=node.as_until_node(){return Some(value.location().start_offset()..value.location().end_offset())}
    location!(as_for_node, location); if let Some(value)=node.as_rescue_node(){return Some(value.location().start_offset()..value.location().end_offset())}
    if context.config_bool("CountBlocks", false) { if let Some(block) = node.as_block_node() { return Some(block.location().start_offset()..block.location().end_offset()); } }
    None
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
        let method = first_argument(&call).map(|argument| node_source(source, &argument).trim_start_matches(':').to_string()).unwrap_or_default();
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
    fn visit_def_node(&mut self, _node: &ruby_prism::DefNode<'pr>) {}
    fn visit_if_node(&mut self, node: &ruby_prism::IfNode<'pr>) { self.score += 1; ruby_prism::visit_if_node(self,node); }
    fn visit_unless_node(&mut self, node: &ruby_prism::UnlessNode<'pr>) { self.score += 1; ruby_prism::visit_unless_node(self,node); }
    fn visit_while_node(&mut self,node:&ruby_prism::WhileNode<'pr>){self.score+=1;ruby_prism::visit_while_node(self,node)}
    fn visit_until_node(&mut self,node:&ruby_prism::UntilNode<'pr>){self.score+=1;ruby_prism::visit_until_node(self,node)}
    fn visit_for_node(&mut self,node:&ruby_prism::ForNode<'pr>){self.score+=1;ruby_prism::visit_for_node(self,node)}
    fn visit_rescue_node(&mut self,node:&ruby_prism::RescueNode<'pr>){self.score+=1;ruby_prism::visit_rescue_node(self,node)}
    fn visit_case_node(&mut self,node:&ruby_prism::CaseNode<'pr>){if self.perceived{self.score+=if node.conditions().len()>1{2}else{1};}ruby_prism::visit_case_node(self,node)}
    fn visit_when_node(&mut self,node:&ruby_prism::WhenNode<'pr>){if !self.perceived{self.score+=1;}ruby_prism::visit_when_node(self,node)}
    fn visit_in_node(&mut self,node:&ruby_prism::InNode<'pr>){self.score+=1;ruby_prism::visit_in_node(self,node)}
    fn visit_else_node(&mut self,node:&ruby_prism::ElseNode<'pr>){if self.perceived&&node.else_keyword_loc().as_slice()!=b":"{self.score+=1;}ruby_prism::visit_else_node(self,node)}
    fn visit_and_node(&mut self,node:&ruby_prism::AndNode<'pr>){self.score+=1;ruby_prism::visit_and_node(self,node)}
    fn visit_or_node(&mut self,node:&ruby_prism::OrNode<'pr>){self.score+=1;ruby_prism::visit_or_node(self,node)}
    fn visit_local_variable_and_write_node(&mut self,node:&ruby_prism::LocalVariableAndWriteNode<'pr>){self.score+=1;ruby_prism::visit_local_variable_and_write_node(self,node)}
    fn visit_local_variable_or_write_node(&mut self,node:&ruby_prism::LocalVariableOrWriteNode<'pr>){self.score+=1;ruby_prism::visit_local_variable_or_write_node(self,node)}
    fn visit_local_variable_write_node(&mut self,node:&ruby_prism::LocalVariableWriteNode<'pr>){self.safe_navigation.remove(&String::from_utf8_lossy(node.name().as_slice()).into_owned());ruby_prism::visit_local_variable_write_node(self,node)}
    fn visit_call_node(&mut self,node:&CallNode<'pr>){
        if node.call_operator_loc().is_some_and(|location| location.as_slice()==b"&.") {
            let local=node.receiver().and_then(|value|value.as_local_variable_read_node());
            if let Some(local) = local {let receiver=String::from_utf8_lossy(local.name().as_slice()).into_owned();if self.safe_navigation.insert(receiver){self.score+=1;}} else {self.score+=1;}
        }
        if node.block().is_some() && matches!(node.name().as_slice(),b"each"|b"each_with_index"|b"with_index"|b"map"|b"collect"|b"select"|b"find"|b"detect"|b"reduce"|b"inject"|b"times"|b"upto"|b"downto") { self.score+=1; }
        ruby_prism::visit_call_node(self,node)
    }
}

struct ClassLength;
impl Cop for ClassLength {
    fn name(&self)->&'static str{"Metrics/ClassLength"}
    fn on_node<'pr>(&self,node:&Node<'pr>,ancestors:&[Node<'pr>],source:&str,context:&mut Context){
        let (body,offense)=if let Some(class)=node.as_class_node(){(class.body(),class.location())}else if let Some(singleton)=node.as_singleton_class_node(){if ancestors.iter().any(|ancestor|ancestor.as_class_node().is_some()){return}(singleton.body(),singleton.location())}else if let Some(block)=node.as_block_node(){
            let Some(call)=ancestors.iter().rev().find_map(Node::as_call_node).filter(|call| call_name(call)==b"new"&&call.receiver().is_some_and(|receiver| matches!(root_constant_name(&receiver).as_deref(),Some(b"Class"|b"Struct"))))else{return};(block.body(),call.location())
        }else{return};
        let mut cop=context.cop_context(self.name(),source,ancestors);let count_comments=cop.config_bool("CountComments",false);
        let body_source=body.as_ref().map_or_else(||{let full=cop.source_file().at(&offense);let mut lines=full.lines();lines.next();let mut rest=lines.collect::<Vec<_>>();rest.pop();rest.join("\n")},|body|cop.source_file().node(body).to_string());let mut count=metric_code_lines(&body_source,count_comments);
        if let Some(body)=body.as_ref(){for nested in nested_class_ranges(body){let nested_source=&source[nested.clone()];count=count.saturating_sub(metric_code_lines(nested_source,count_comments));}}
        if cop.config_values("CountAsOne").iter().any(|value|value=="array"){count=count.saturating_sub(metric_folded_lines(&body_source,'[',']'));}
        let max=cop.config_usize("Max",100);if count>max{cop.report(format!("Class has too many lines. [{count}/{max}]"),offense);}
    }
}

fn root_constant_name(node:&Node<'_>)->Option<Vec<u8>>{node.as_constant_read_node().map(|constant|constant.name().as_slice().to_vec()).or_else(||node.as_constant_path_node().and_then(|path|path.name().map(|name|name.as_slice().to_vec())))}
fn metric_code_lines(source:&str,count_comments:bool)->usize{source.lines().filter(|line|{let line=line.trim();!line.is_empty()&&(count_comments||!line.starts_with('#'))}).count()}
fn metric_folded_lines(source:&str,open:char,close:char)->usize{source.find(open).and_then(|start|source[start..].find(close).map(|end|source[start..start+end].matches('\n').count())).unwrap_or(0)}
fn nested_class_ranges(body:&Node<'_>)->Vec<std::ops::Range<usize>>{struct Collector(Vec<std::ops::Range<usize>>);impl<'pr> Visit<'pr> for Collector{fn visit_class_node(&mut self,node:&ruby_prism::ClassNode<'pr>){let location=node.location();self.0.push(location.start_offset()..location.end_offset());}fn visit_module_node(&mut self,node:&ruby_prism::ModuleNode<'pr>){let location=node.location();self.0.push(location.start_offset()..location.end_offset());}}let mut collector=Collector(Vec::new());collector.visit(body);collector.0}
