pub use pattern_fn_proc_macro::match_as_opt;
use std::{any::Any, collections::HashMap, marker::PhantomData};

pub struct PatternCtx<Fields, Aux> {
    pub values: Fields,
    pub aux: Aux,
}

impl<Aux, Fields> PatternCtx<Fields, Aux> {
    pub fn new(aux: Aux, values: Fields) -> Self {
        Self { values, aux }
    }
}

pub trait Pattern<T, Fields, CtxAux = ()> {
    fn is_match(&self, cx: &mut PatternCtx<Fields, CtxAux>, input: T) -> bool;
}

impl<T: PartialEq, D, C> Pattern<T, D, C> for T {
    fn is_match(&self, _: &mut PatternCtx<D, C>, input: T) -> bool {
        self == &input
    }
}

pub struct WildMatcher;

impl<T, D, C> Pattern<T, D, C> for WildMatcher {
    fn is_match(&self, _: &mut PatternCtx<D, C>, _: T) -> bool {
        true
    }
}

// pub struct IdentMatcher<T>(pub &'static str, pub PhantomData<T>);

// impl<T: 'static, C> Pattern<T, C> for IdentMatcher<T> {
//     fn is_match(&self, cx: &mut PatternCtx<C>, input: T) -> bool {
//         cx.values.insert(self.0, Box::new(input));
//         true
//     }
// }

pub struct FunctionMatcher<F>(pub F);

impl<T, D, C, F> Pattern<T, D, C> for FunctionMatcher<F>
where
    F: Fn(&mut PatternCtx<D, C>, T) -> bool,
{
    fn is_match(&self, cx: &mut PatternCtx<D, C>, input: T) -> bool {
        self.0(cx, input)
    }
}

// struct Pattern<T> {}

pub fn add(left: usize, right: usize) -> usize {
    left + right
}
