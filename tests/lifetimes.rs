use std::marker::PhantomData;

use pattern_fn::{match_as_opt, FunctionMatcher, Pattern, PatternCtx, WildMatcher};

#[derive(Debug, Clone, PartialEq, Eq)]
enum Expr<'a> {
    If {
        cond: &'a Expr<'a>,
        then: &'a Expr<'a>,
        else_: &'a Expr<'a>,
    },
    LiteralInt(i32),
    LiteralBool(bool),
}

fn expr_if<'hir, D>(
    cond_pat: impl pattern_fn::Pattern<&'hir Expr<'hir>, D>,
    then_pat: impl pattern_fn::Pattern<&'hir Expr<'hir>, D>,
    else_pat: impl pattern_fn::Pattern<&'hir Expr<'hir>, D>,
) -> impl pattern_fn::Pattern<&'hir Expr<'hir>, D> {
    FunctionMatcher(move |cx: &mut PatternCtx<D, ()>, expr: &Expr<'hir>| {
        if let Expr::If { cond, then, else_ } = expr {
            cond_pat.is_match(cx, cond)
                && then_pat.is_match(cx, then)
                && else_pat.is_match(cx, else_)
        } else {
            false
        }
    })
}

#[test]
fn it_works_without_macro() {
    struct Fields<'a> {
        ident: Option<&'a Expr<'a>>,
        ident2: Option<&'a Expr<'a>>,
    }

    let expr_lit = &Expr::LiteralInt(32);
    let expr_bool = &Expr::LiteralBool(false);
    let expr_if_int = &Expr::If {
        cond: expr_bool,
        then: expr_lit,
        else_: expr_lit,
    };

    fn f<'a>(cx: &mut PatternCtx<Fields<'a>, ()>, expr: &'a Expr<'a>) -> bool {
        cx.values.ident = Some(expr);
        true
    }

    let cond = pattern_fn::FunctionMatcher(f);

    fn f2<'a>(cx: &mut PatternCtx<Fields<'a>, ()>, expr: &'a Expr<'a>) -> bool {
        cx.values.ident2 = Some(expr);
        true
    }

    let then = pattern_fn::FunctionMatcher(f2);

    let pat = expr_if(cond, then, WildMatcher);
    let mut cx = PatternCtx::new(
        (),
        Fields {
            ident: None,
            ident2: None,
        },
    );
    pat.is_match(&mut cx, expr_if_int);
    let result = cx.values.ident.unwrap();
    assert_eq!(result, expr_bool);
    assert_eq!(cx.values.ident2.unwrap(), expr_lit);
}

// #[test]
// fn simple() {
//     let expr_lit = &Expr::LiteralInt(32);
//     let expr_bool = &Expr::LiteralBool(false);
//     let expr_if_int = &Expr::If {
//         cond: expr_bool,
//         then: expr_lit,
//         else_: expr_lit,
//     };
//     if let Some(_) = match_as_opt!(expr_if(cond: &Expr<'_>, _, _), expr_if_int) {
//     } else {
//         panic!("Expected match");
//     }
// }
