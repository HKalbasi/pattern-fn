use proc_macro2::TokenStream;
use quote::quote;

#[derive(Debug, Clone)]
struct Ident {
    name: String,
    ty: TokenStream,
}

enum ParsedPattern {
    Fn {
        name: String,
        args: Vec<ParsedPattern>,
    },
    Ident(Ident),
}

impl ParsedPattern {
    fn idents(&self) -> Vec<Ident> {
        fn f(this: &ParsedPattern, result: &mut Vec<Ident>) {
            match this {
                ParsedPattern::Fn { name, args } => {
                    for arg in args {
                        f(arg, result);
                    }
                }
                ParsedPattern::Ident(i) => {
                    result.push(i.clone());
                }
            }
        }
        let mut r = vec![];
        f(self, &mut r);
        r
    }

    fn initialization(&self) -> (TokenStream, TokenStream) {
        fn f(this: &ParsedPattern, result: &mut TokenStream) -> TokenStream {
            let my_name = format!("v{}", result.clone().into_iter().count())
                .parse()
                .unwrap();
            match this {
                ParsedPattern::Fn { name, args } => {
                    let names = args.iter().map(|x| f(x, result)).collect::<Vec<_>>();
                    let fn_name: TokenStream = name.parse().unwrap();
                    result.extend(quote! {
                        let #my_name = #fn_name ( #(&#names),* );
                    });
                }
                ParsedPattern::Ident(ident) => {
                    let ty = ident.ty.clone();
                    let ident = ident.name.as_str();
                    result.extend(quote! {
                        let #my_name = IdentMatcher::<#ty>(#ident, ::core::marker::PhantomData);
                    });
                }
            }
            my_name
        }
        let mut r = quote! {};
        let name = f(self, &mut r);
        (r, name)
    }
}

#[proc_macro]
pub fn match_as_opt(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let pp = ParsedPattern::Fn {
        name: "some".to_owned(),
        args: vec![ParsedPattern::Fn {
            name: "some".to_owned(),
            args: vec![ParsedPattern::Ident(Ident {
                name: "ident".to_owned(),
                ty: "Option<i32>".parse().unwrap(),
            })],
        }],
    };
    let (initialization, pattern_name) = pp.initialization();
    let idents = pp.idents();
    let ident_names = idents.iter().map(|x| x.name.as_str());
    let ident_types = idents.iter().map(|x| x.ty.clone());
    let ident_tokens = ident_names
        .clone()
        .map(|x| -> TokenStream { x.parse().unwrap() });
    let ident_tokens2 = ident_tokens.clone();
    quote! {{
        let mut cx = PatternCtx::new(());
        #initialization
        #pattern_name.is_match(&mut cx, Some(Some(Some(22))));
        #(
            let #ident_tokens2 = *cx
                .values
                .remove(#ident_names)
                .unwrap()
                .downcast::<#ident_types>()
                .unwrap();
        )*
        Some(#(#ident_tokens),*)
    }}
    .into()
}
