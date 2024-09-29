pub use pattern_fn_proc_macro::match_as_opt;
use std::{any::Any, collections::HashMap, marker::PhantomData};

pub struct PatternCtx<Aux> {
    values: HashMap<&'static str, Box<dyn Any>>,
    pub aux: Aux,
}

impl<Aux> PatternCtx<Aux> {
    pub fn new(aux: Aux) -> Self {
        Self {
            values: HashMap::new(),
            aux,
        }
    }
}

pub trait Pattern<T, CtxAux = ()> {
    fn is_match(&self, cx: &mut PatternCtx<CtxAux>, input: T) -> bool;
}

impl<T: PartialEq, C> Pattern<T, C> for T {
    fn is_match(&self, _: &mut PatternCtx<C>, input: T) -> bool {
        self == &input
    }
}

pub struct IdentMatcher<T>(&'static str, PhantomData<T>);

impl<T: PartialEq + 'static, C> Pattern<T, C> for IdentMatcher<T> {
    fn is_match(&self, cx: &mut PatternCtx<C>, input: T) -> bool {
        cx.values.insert(self.0, Box::new(input));
        true
    }
}

struct FunctionMatcher<F>(F);

impl<T, C, F> Pattern<T, C> for FunctionMatcher<F>
where
    F: Fn(&mut PatternCtx<C>, T) -> bool,
{
    fn is_match(&self, cx: &mut PatternCtx<C>, input: T) -> bool {
        self.0(cx, input)
    }
}

fn some<T>(p: &impl Pattern<T>) -> impl Pattern<Option<T>> + '_ {
    FunctionMatcher(move |cx: &mut PatternCtx<()>, v| match v {
        Some(v) => p.is_match(cx, v),
        None => false,
    })
}

// struct Pattern<T> {}

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works_without_macro() {
        let ident = IdentMatcher("ident", PhantomData);
        let some_1 = some(&ident);
        let some_2 = some(&some_1);
        let mut cx = PatternCtx::new(());
        some_2.is_match(&mut cx, Some(Some(Some(22))));
        let result = *cx
            .values
            .remove("ident")
            .unwrap()
            .downcast::<Option<i32>>()
            .unwrap();
        assert_eq!(result, Some(22));
    }

    #[test]
    fn simple() {
        // if match_as_opt!(Some(Some(Some(22)))) {}
        if let Some(ident) = match_as_opt!(some(some(ident: Option<i32>)), Some(Some(Some(22)))) {
            assert_eq!(ident, Some(22));
        } else {
            unreachable!();
        }
    }
}
