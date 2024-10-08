use std::marker::PhantomData;

use pattern_fn::{match_as_opt, FunctionMatcher, Pattern, PatternCtx};

fn some<T, D>(p: impl Pattern<T, D>) -> impl Pattern<Option<T>, D> {
    FunctionMatcher(move |cx: &mut PatternCtx<D, ()>, v| match v {
        Some(v) => p.is_match(cx, v),
        None => false,
    })
}

fn some2<T, D>(p: impl Pattern<T, D>) -> impl Pattern<Option<Option<T>>, D> {
    some(some(p))
}

// #[test]
// fn it_works_without_macro() {
//     let ident = IdentMatcher("ident", PhantomData);
//     let some_1 = some(ident);
//     let some_2 = some(some_1);
//     let mut cx = PatternCtx::new(());
//     some_2.is_match(&mut cx, Some(Some(Some(22))));
//     let result = *cx
//         .values
//         .remove("ident")
//         .unwrap()
//         .downcast::<Option<i32>>()
//         .unwrap();
//     assert_eq!(result, Some(22));
// }

#[test]
fn simple() {
    if let Some(ident) = match_as_opt!(some2(ident: Option<i32>), Some(Some(Some(22)))) {
        assert_eq!(ident, Some(22));
    } else {
        unreachable!();
    }
}
