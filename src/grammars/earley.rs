use super::context_free::*;

#[derive(Clone, Debug)]
pub struct GrammarSituation<'a> {
    rule: &'a GrammarRule,
    point: usize
}

#[derive(Clone, Debug)]
struct EarleySituation<'a, 'b> {
    base: GrammarSituation<'a>,
    word: &'b String,
    index: usize
}
