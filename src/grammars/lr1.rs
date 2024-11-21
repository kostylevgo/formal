use super::earley::GrammarSituation;

#[derive(Clone, Debug)]
struct LRSituation<'a> {
    base: GrammarSituation<'a>,
    first: char
}
