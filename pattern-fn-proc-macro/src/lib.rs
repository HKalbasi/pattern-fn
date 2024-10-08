use proc_macro2::{TokenStream, TokenTree};
use quote::{quote, ToTokens, TokenStreamExt};

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
    Wild,
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
                ParsedPattern::Wild => (),
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
                        let #my_name = #fn_name ( #(#names),* );
                    });
                }
                ParsedPattern::Ident(ident) => {
                    let ty = ident.ty.clone();
                    let ident: TokenStream = ident.name.parse().unwrap();
                    result.extend(quote! {
                        let #my_name = {
                            fn f(cx: &mut PatternCtx<Fields, ()>, value: #ty) -> bool {
                                cx.values.#ident = Some(value);
                                true
                            }

                            pattern_fn::FunctionMatcher(f)
                        }; //::pattern_fn::IdentMatcher::<#ty>(#ident, ::core::marker::PhantomData);
                    });
                }
                ParsedPattern::Wild => {
                    result.extend(quote! {
                        let #my_name = ::pattern_fn::WildMatcher;
                    });
                }
            }
            my_name
        }
        let mut r = quote! {};
        let name = f(self, &mut r);
        (r, name)
    }

    fn parse(pattern: TokenStream) -> Self {
        let p = || pattern.clone().into_iter();
        if p().count() == 1 {
            if pattern.to_string().trim() == "_" {
                return Self::Wild;
            }
        }
        let x = p().nth(1).expect("pattern had single token tree");
        if x.to_string().trim() == ":" {
            Self::Ident(Ident {
                name: p().nth(0).unwrap().to_string(),
                ty: p().skip(2).collect(),
            })
        } else if let TokenTree::Group(g) = x {
            Self::Fn {
                name: p().nth(0).unwrap().to_string(),
                args: split_by_comma(g.stream())
                    .into_iter()
                    .map(Self::parse)
                    .collect(),
            }
        } else {
            panic!("The second element was neither : nor (");
        }
    }
}

fn split_by_comma(item: TokenStream) -> Vec<TokenStream> {
    let mut r = vec![];
    let mut last = TokenStream::new();
    for tok in item {
        if tok.to_string().trim() != "," {
            last.append(tok);
        } else {
            r.push(std::mem::take(&mut last));
        }
    }
    r.push(last);
    r
}

#[proc_macro]
pub fn match_as_opt(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let [pattern, value] = &*split_by_comma(item.clone().into()) else {
        panic!(
            "wrong number of arguments: detected arguments are: {:#?}",
            split_by_comma(item.into())
        );
    };
    let value = value.clone();
    let pp = ParsedPattern::parse(pattern.clone());
    let (initialization, pattern_name) = pp.initialization();
    let idents = pp.idents();
    let ident_names = idents.iter().map(|x| x.name.as_str());
    let ident_types = idents.iter().map(|x| x.ty.clone());
    let ident_tokens = ident_names
        .clone()
        .map(|x| -> TokenStream { x.parse().unwrap() });
    let ident_tokens2 = ident_tokens.clone();
    let ident_tokens3 = ident_tokens.clone();
    let ident_tokens4 = ident_tokens.clone();
    let ident_tokens5 = ident_tokens.clone();
    quote! {{
        use ::pattern_fn::Pattern as _;
        struct Fields {
            #(
                #ident_tokens3: Option<#ident_types>,
            )*
        }
        let mut cx = ::pattern_fn::PatternCtx::new((), Fields { #(
            #ident_tokens4: None,
        )* });
        #initialization
        if !#pattern_name.is_match(&mut cx, #value) {
            None
        } else {
            #(
                let #ident_tokens2 = cx
                    .values
                    .#ident_tokens5
                    .unwrap();
            )*
            Some((#(#ident_tokens),*))
        }
    }}
    .into()
}
